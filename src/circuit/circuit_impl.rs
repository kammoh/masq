use std::collections::HashMap;
use std::convert::identity;

use crate::circuit::node::GateType;

use super::node::{Node, NodePortId, NodeType};
use super::{Circuit, NodeIndex};
use boolinator::Boolinator;
use itertools::Itertools;
use petgraph::data::Build;
use petgraph::{
    visit::{Dfs, EdgeRef, Reversed},
    Direction,
};

impl Circuit {
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let node_type = node.node_type.clone();
        let id = self.graph.add_node(node);
        match node_type {
            NodeType::Input => {
                self.inputs.insert(id).expect("duplicate input");
            }
            NodeType::Clock => {
                self.clocks.insert(id).expect("duplicate clock");
            }
            NodeType::Register => {
                self.registers.insert(id).expect("duplicate register");
            }
            NodeType::Output => {
                self.outputs.insert(id).expect("duplicate output");
            }
            NodeType::Constant(v) => {
                self.consts[usize::from(v)]
                    .is_none()
                    .expect("duplicate constant");
                self.consts[usize::from(v)] = Some(id);
            }
            _ => (),
        };
        id
    }

    pub fn connect(
        &mut self,
        src: NodeIndex,
        src_port: NodePortId,
        dst: NodeIndex,
        dst_port: NodePortId,
    ) {
        self.graph.add_edge(src, dst, (src_port, dst_port));
    }

    pub fn const_node(&mut self, value: bool) -> NodeIndex {
        if let Some(idx) = self.consts[usize::from(value)] {
            idx
        } else {
            let idx = self.add_node(Node::constant(value));
            self.consts[usize::from(value)] = Some(idx);
            idx
        }
    }

    pub fn secure_inputs(&self) -> Vec<NodeIndex> {
        self.inputs
            .iter()
            .filter_map(|id| {
                if self.graph[*id].secure {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn secure_outputs(&self) -> Vec<NodeIndex> {
        self.outputs
            .iter()
            .filter_map(|&id| {
                if self.graph[id].secure {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn propagate_secure(&mut self) {
        for start in self.secure_inputs() {
            let mut dfs = Dfs::new(&self.graph, start);
            while let Some(nx) = dfs.next(&self.graph) {
                self.graph[nx].secure = true;
            }
        }
    }

    pub fn replicate_node(
        &mut self,
        num_shares: u8,
        nx: &NodeIndex,
    ) -> Vec<(NodeIndex, NodePortId, NodePortId)> {
        let node = &self.graph[*nx];
        let node_name = node.name.clone();
        let lr = node_name
            .as_ref()
            .and_then(|name| name.rfind("[").map(|i| name.split_at(i)));

        fn share_name_lr(l: &str, r: &str, share: u8) -> String {
            format!("{}_s{}{}", l, share, r)
        }
        let share_name = |share: u8| lr.map(|(l, r)| share_name_lr(l, r, share));
        let mut replicas = Vec::new();
        let node_type = node.node_type.clone();
        for share in 1..num_shares {
            let node_type = match node_type {
                NodeType::Gate(gt, _) => NodeType::Gate(gt, false),
                _ => node_type.clone(),
            };
            let replica = Node {
                name: share_name(share),
                node_type: node_type.clone(),
                secure: true,
            };
            let duplicate_node = self.add_node(replica);
            replicas.push((duplicate_node, 0, 0));
        }
        if let Some((l, r)) = lr {
            self.graph[*nx].name = Some(share_name_lr(l, r, 0));
        }
        replicas
    }

    pub fn node_inputs_map(&self, nx: &NodeIndex) -> HashMap<u8, (NodeIndex, NodePortId)> {
        self.graph
            .edges_directed(*nx, Direction::Incoming)
            .map(|e| (e.weight().1, (e.source(), e.weight().0)))
            .collect()
    }

    pub fn node_inputs(&self, nx: &NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .edges_directed(*nx, Direction::Incoming)
            .sorted_by(|a, b| Ord::cmp(&a.weight().1, &b.weight().1))
            .map(|e| e.source())
    }

    // pub fn node_input_ports(
    //     &self,
    //     nx: &NodeIndex,
    // ) -> Option<HashMap<String, (NodeIndex, NodePortId)>> {
    //     let node = &self.graph[*nx];
    //     let port_names = self.input_ordering_map.get(&node.node_type)?;
    //     let inputs = self.node_inputs_map(nx);
    //     let r: HashMap<_, _> = port_names
    //         .iter()
    //         .zip(0..)
    //         .map(|(port_name, idx)| (port_name.clone(), inputs.get(&idx).unwrap().clone()))
    //         .collect();

    //     Some(r)
    // }

    pub fn node_input(&self, nx: &NodeIndex, port_name: &str) -> Option<(NodeIndex, NodePortId)> {
        let node = &self.graph[*nx];
        let port_names = self.input_ordering_map.get(&node.node_type)?;
        let port_idx = port_names.iter().position(|n| n == port_name);
        let inputs = self.node_inputs_map(nx);
        port_idx.and_then(|idx| inputs.get(&(idx as u8))).copied()
    }

    fn convert_or(&mut self, nx: &NodeIndex) -> bool {
        let node = &mut self.graph[*nx];
        if !node.secure {
            return false;
        }
        match node.node_type {
            NodeType::Gate(GateType::Or(_), ref mut inv) => {
                *inv = !*inv;
            }
            _ => {
                return false;
            }
        }
        let incomings = self
            .graph
            .edges_directed(*nx, Direction::Incoming)
            .map(|e| (e.source(), e.weight().clone(), e.id()))
            .collect_vec();

        for (si, (sp, dp), e) in incomings {
            let src_is_single_output =
                self.graph.edges_directed(si, Direction::Outgoing).count() == 1;

            let src_node = &mut self.graph[si];
            let secure = src_node.secure.clone();
            match src_node.node_type {
                NodeType::Gate(_, ref mut inv) if src_is_single_output => {
                    *inv = !*inv;
                }
                _ => {
                    let not_gate = self.add_node(Node {
                        secure: secure,
                        node_type: NodeType::Gate(GateType::Buf, true),
                        name: None,
                    });
                    self.graph.remove_edge(e);
                    self.connect(si, sp, not_gate, 0);
                    self.connect(not_gate, 0, *nx, dp);
                }
            }
        }
        true
    }

    pub fn convert_secure_ors(&mut self) {
        let current_nodes = self.graph.node_indices().collect_vec();
        for nx in current_nodes.iter() {
            self.convert_or(nx);
        }
    }

    pub fn replace_gate_with_gadget(
        &mut self,
        num_shares: u8,
        nx: &NodeIndex,
    ) -> Vec<(NodeIndex, NodePortId, NodePortId)> {
        let input_secure = self
            .node_inputs(nx)
            .map(|si| self.graph[si].secure.clone())
            .collect_vec();

        let has_insecure_input = !(input_secure.iter().cloned().all(identity));

        let num_in_ports = self.graph.edges_directed(*nx, Direction::Incoming).count() as NodePortId;
        let num_out_ports = 1 as NodePortId; // TODO
        let node = &mut self.graph[*nx];

        match node.node_type {
            NodeType::Gate(gt, inv) => match gt {
                GateType::And(_) if has_insecure_input => self.replicate_node(num_shares, nx),
                GateType::Mux if !input_secure[0] => self.replicate_node(num_shares, nx),
                _ => {
                    node.node_type = NodeType::Gadget {
                        base_type: gt,
                        invert: inv,
                        num_shares,
                    };
                    (1..num_shares)
                        .map(|s| (nx.clone(), s * num_in_ports, s * num_out_ports))
                        .collect_vec()
                }
            },
            _ => panic!("???"),
        }
    }
}
