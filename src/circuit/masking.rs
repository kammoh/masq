use std::collections::HashMap;

use boolinator::Boolinator;
use itertools::Itertools;
use petgraph::{
    visit::{Dfs, EdgeRef, Reversed},
    Direction,
};
use crate::circuit::node::NodeType;

use super::{
    node::{GateType},
    Circuit,
};

pub trait Masking {
    fn mask(&mut self, order: u8);
}

impl Masking for Circuit {
    fn mask(&mut self, order: u8) {
        let num_shares = order + 1;

        self.propagate_secure();
        self.convert_secure_ors();

        let mut replica_map = HashMap::new();

        // FIXME do we need to traverse?
        for start in self.secure_outputs() {
            let mut dfs = Dfs::new(&self.graph, start);
            while let Some(nx) = dfs.next(Reversed(&self.graph)) {
                let node = &self.graph[nx];
                if !node.secure || replica_map.contains_key(&nx) {
                    continue;
                }
                let replicas = match node.node_type {
                    NodeType::Input
                    | NodeType::Output
                    | NodeType::Register
                    | NodeType::Gate(GateType::Xor(_) | GateType::Buf, _) => {
                        self.replicate_node(num_shares, &nx)
                    }
                    NodeType::Gate(GateType::And(_) | GateType::Or(_) | GateType::Mux, _) => {
                        self.replace_gate_with_gadget(num_shares, &nx)
                    }
                    _ => panic!("??? nx={:?}", nx),
                };

                replica_map
                    .insert(nx, replicas)
                    .is_none()
                    .expect("nx already existed!");
            }
        }

        for (nx, replicas) in replica_map.iter() {
            let incoming_edges = &self
                .graph
                .edges_directed(*nx, Direction::Incoming)
                .map(|e| (e.source(), e.weight().clone()))
                .collect_vec();

            for (sx, (src_port, dst_port)) in incoming_edges {
                let src_node = &self.graph[*sx];
                if !src_node.secure {
                    for (rx, _, dst_offset) in replicas.iter() {
                        self.connect(*sx, *src_port, *rx, dst_port + dst_offset);
                    }
                } else {
                    println!("secure source: [{:?}] {:?}", sx, src_node);
                    if let Some(src_replicas) = replica_map.get(&sx) {
                        for ((rx, _, src_offset), (tx, dst_offset, _)) in
                            src_replicas.iter().zip(replicas)
                        {
                            self.connect(*rx, src_port + src_offset, *tx, dst_port + dst_offset)
                        }
                    } else {
                        panic!("... I have no idea why");
                    }
                }
            }
        }

        // match node_type {
        //     NodeType::Gate(gate_type, invert) => match gate_type {
        //         GateType::Buf | GateType::Xor(_) => todo!(),
        //         GateType::And(_) => todo!(),
        //         GateType::Or(_) => todo!(),
        //         GateType::Mux => todo!(),
        //     },
        //     NodeType::Gate(ref gt, _) => {
        //         replica_map.insert(
        //             nx,
        //             self.replicate_node(
        //                 num_shares,
        //                 node_name_lr,
        //                 &NodeType::Gate(gt.clone(), false),
        //             ),
        //         );
        //         node_type
        //     }
        //     NodeType::Input | NodeType::Output | NodeType::Register => {
        //         replica_map.insert(
        //             nx,
        //             self.replicate_node(num_shares, node_name_lr, &node_type),
        //         );
        //         node_type
        //     }
        //     NodeType::Blackbox(_)
        //     | NodeType::Gadget { .. }
        //     | NodeType::Clock
        //     | NodeType::Reset
        //     | NodeType::Constant(_) => {
        //         panic!("Should not happen")
        //     }
        // };
        //     }
        // }

        // let mut replica_map = HashMap::new();
        // for start in self.secure_inputs() {
        //     let mut dfs = Dfs::new(&self.graph, start);
        //     while let Some(nx) = dfs.next(&self.graph) {
        //         let node = &self.graph[nx];
        //         let node_name = node.name.clone();
        //         let node_name_lr = node_name
        //             .as_ref()
        //             .and_then(|name| name.rfind("[").map(|i| name.split_at(i)));

        //         let node_type = node.node_type.clone();
        //         let node_type = match node_type {
        //             NodeType::Gate(gt, invert)
        //                 if matches!(gt, GateType::And(_) | GateType::Or(_)) =>
        //             {
        //                 NodeType::Gadget {
        //                     base_type: gt,
        //                     invert,
        //                     num_shares,
        //                 }
        //             }
        //             NodeType::Gate(ref gt, _) => {
        //                 replica_map.insert(
        //                     nx,
        //                     self.replicate_node(
        //                         num_shares,
        //                         node_name_lr,
        //                         &NodeType::Gate(gt.clone(), false),
        //                     ),
        //                 );
        //                 node_type
        //             }
        //             NodeType::Input | NodeType::Output | NodeType::Register => {
        //                 replica_map.insert(
        //                     nx,
        //                     self.replicate_node(num_shares, node_name_lr, &node_type),
        //                 );
        //                 node_type
        //             }
        //             NodeType::Blackbox(_) | NodeType::Gadget { .. } => panic!("Should not happen"),
        //             NodeType::Clock | NodeType::Reset | NodeType::Constant(_) => {
        //                 panic!("Should not happen")
        //             }
        //         };
        //         let new_node = Node {
        //             name: node_share_name(node_name_lr, 0),
        //             secure: true,
        //             node_type: node_type,
        //         };
        //         self.graph[nx] = new_node;
        //     }
        // }

        // for start in self.secure_outputs() {
        //     let mut dfs = Dfs::new(&self.graph, start);

        //     while let Some(nx) = dfs.next(Reversed(&self.graph)) {
        //         let node = &self.graph[nx];
        //         if !node.secure {
        //             continue;
        //         }
        //         let incoming_edges: Vec<_> = self
        //             .graph
        //             .edges_directed(nx, Direction::Incoming)
        //             .map(|e| (e.source().clone(), e.weight().clone()))
        //             .collect();

        //         if incoming_edges.is_empty() {
        //             continue;
        //         }

        //         let gadget_shares = match &node {
        //             Node {
        //                 node_type: NodeType::Gadget { num_shares, .. },
        //                 ..
        //             } => Some(num_shares.clone()),
        //             _ => None,
        //         };

        //         match (gadget_shares, replica_map.get(&nx)) {
        //             (Some(num_shares), _) => {
        //                 for (si, w) in incoming_edges.iter() {
        //                     match (&self.graph[*si], replica_map.get(&si)) {
        //                         (Node { secure: false, .. }, _) => {
        //                             let zero = self.const_node(false);
        //                             for share in 1..num_shares.clone() {
        //                                 self.connect(zero, w.0, nx, w.1 + share * num_shares);
        //                             }
        //                         }
        //                         (
        //                             Node {
        //                                 node_type: NodeType::Gadget { .. },
        //                                 ..
        //                             },
        //                             _,
        //                         ) => {
        //                             for share in 1..num_shares {
        //                                 self.connect(
        //                                     *si,
        //                                     w.0 + share * num_shares,
        //                                     nx,
        //                                     w.1 + share * num_shares,
        //                                 );
        //                             }
        //                         }
        //                         (_, Some(src_replicas)) => {
        //                             for (replica, share) in src_replicas.iter().zip(1..) {
        //                                 self.connect(*replica, w.0, nx, w.1 + share * num_shares);
        //                             }
        //                         }
        //                         _ => {}
        //                     };
        //                 }
        //             }

        //             (_, Some(replicas)) => {
        //                 for (replica, share) in replicas.iter().zip(1..) {
        //                     for (si, w) in incoming_edges.iter() {
        //                         let src_node = &self.graph[*si];
        //                         match (src_node, replica_map.get(&si)) {
        //                             (Node { secure: false, .. }, src_replicas) => {
        //                                 assert_eq!(src_replicas, None);
        //                                 self.connect(*si, w.0, *replica, w.1);
        //                             }
        //                             (
        //                                 Node {
        //                                     node_type: NodeType::Gadget { num_shares, .. },
        //                                     ..
        //                                 },
        //                                 src_replicas,
        //                             ) => {
        //                                 assert_eq!(src_replicas, None);
        //                                 self.connect(*si, w.0 + share * num_shares, *replica, w.1);
        //                             }
        //                             (_, Some(src_replicas)) => {
        //                                 for dup_src in src_replicas {
        //                                     self.connect(*dup_src, w.0, *replica, w.1);
        //                                 }
        //                             }
        //                             _ => {}
        //                         }
        //                     }
        //                 }
        //             }
        //             _ => {}
        //         }
        //     }
        // }
    }
}
