use crate::{
    netlist::json_netlist,
    utils::{MapToSet, MapToVec},
};

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeIndex(u32);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Netname {0} not found")]
    NetnameNotFound(String),
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum CellType {
    #[default]
    Input,
    Clock,
    Reset,
    Gate {
        gate_type: String,
    },
    Register {
        clk: NodeIndex,
    },
    Output,
    Constant(bool),
}

impl CellType {
    pub fn has_input(&self) -> bool {
        match self {
            CellType::Gate { .. } | CellType::Register { .. } | CellType::Output => true,
            _ => false,
        }
    }
    pub fn has_output(&self) -> bool {
        match self {
            CellType::Input
            | CellType::Clock
            | CellType::Reset
            | CellType::Gate { .. }
            | CellType::Register { .. }
            | CellType::Constant { .. } => true,
            _ => false,
        }
    }
}

#[derive(Builder, Debug)]
pub struct Node {
    #[builder(default = "false")]
    secure: bool,
    #[builder(default)]
    inputs: Vec<NodeIndex>,
    #[builder(default)]
    fanouts: HashSet<NodeIndex>, // unordered, but really don't need it to be a set
    cell_type: CellType,
    #[builder(default)]
    name: Option<String>,
}

impl Node {
    // pub fn new(cell_type: CellType) -> Node {
    //     Node {
    //         secure: false,
    //         inputs: Vec::new(),
    //         fanouts: HashSet::new(),
    //         cell_type: cell_type,
    //         name: None,
    //     }
    // }
}

#[derive(Debug)]
pub struct Circuit {
    inputs: HashSet<NodeIndex>,
    clocks: HashSet<NodeIndex>,
    resets: HashSet<NodeIndex>,
    outputs: HashSet<NodeIndex>,
    registers: HashSet<NodeIndex>,
    nodes: HashMap<NodeIndex, Node>,
    next_id: NodeIndex,
}

impl Default for Circuit {
    fn default() -> Self {
        let mut circuit = Circuit {
            inputs: Default::default(),
            clocks: Default::default(),
            resets: Default::default(),
            outputs: Default::default(),
            registers: Default::default(),
            nodes: Default::default(),
            next_id: Default::default(),
        };
        circuit.add_node(
            NodeBuilder::default()
                .cell_type(CellType::Constant(false))
                .build()
                .unwrap(),
        );
        circuit.add_node(
            NodeBuilder::default()
                .cell_type(CellType::Constant(true))
                .build()
                .unwrap(),
        );
        assert_eq!(circuit.nodes.len(), 2);
        circuit
    }
}

impl Circuit {
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        while self.nodes.contains_key(&self.next_id) {
            // TODO wraparound once, handle out of indices
            self.next_id.0 += 1;
        }
        let id = self.next_id;
        let mut requirement = id.0 > 1;

        let r = match node.cell_type {
            CellType::Input => self.inputs.insert(id),
            CellType::Clock => self.clocks.insert(id),
            CellType::Reset => self.resets.insert(id),
            CellType::Output => self.outputs.insert(id),
            CellType::Register { clk: _ } => self.registers.insert(id),
            CellType::Constant(v) => {
                requirement = true;
                if v {
                    id.0 == 1
                } else {
                    id.0 == 0
                }
            }
            _ => true,
        };
        assert!(requirement && r);
        let r = self.nodes.insert(id, node);
        assert!(r.is_none());
        id
    }

    pub fn remove_node(&mut self, node_id: NodeIndex) -> bool {
        self.nodes
            .remove(&node_id)
            .and_then(|node| match node.cell_type {
                CellType::Input => self.inputs.remove(&node_id).then_some(()),
                CellType::Output => self.outputs.remove(&node_id).then_some(()),
                CellType::Register { clk: _ } => self.registers.remove(&node_id).then_some(()),
                CellType::Constant(_) => None, // never remove constants
                _ => Some(()),
            })
            .map(|_| {
                self.next_id.0 = self.next_id.0.min(node_id.0);
            })
            .is_some()
    }
    pub fn const_id(value: bool) -> NodeIndex {
        NodeIndex(if value { 1 } else { 0 })
    }

    pub fn from_netlist(netlist: json_netlist::Netlist) -> Result<Circuit, Error> {
        let mut circuit = Circuit::default();

        let mut signal_driver = HashMap::<json_netlist::SignalId, NodeIndex>::default();
        let mut signal_fanouts = MapToSet::<json_netlist::SignalId, NodeIndex>::default();
        let mut input_bitvals = MapToVec::<NodeIndex, json_netlist::BitVal>::default();
        let mut output_signal = HashMap::<NodeIndex, json_netlist::SignalId>::default();

        for (mname, module) in netlist.modules.iter() {
            if module.is_top() {
                // FIXME for all modules?
                println!("Top module: {}", mname);
                // add IO nodes:
                for (port_name, port) in module.ports.iter() {
                    let net = module
                        .netnames
                        .get(port_name)
                        .ok_or_else(|| Error::NetnameNotFound(port_name.to_owned()))?;
                    let secure = match net.attributes.get("AGEMA") {
                        Some(json_netlist::AttributeVal::Str(s)) => s == "secure",
                        _ => false,
                    };
                    match port.direction {
                        json_netlist::PortDirection::Input => {
                            for (idx, b) in net.bits.iter().enumerate() {
                                let node = NodeBuilder::default()
                                    .cell_type(CellType::Input)
                                    .name(Some(format!("{}[{}]", port_name, idx)))
                                    .secure(secure)
                                    .build()
                                    .unwrap();
                                let node_id = circuit.add_node(node);

                                match b {
                                    json_netlist::BitVal::Signal(sig) => {
                                        signal_driver.insert(*sig, node_id); // 1-1 mapping
                                        output_signal.insert(node_id, *sig);
                                    }
                                    json_netlist::BitVal::Constant(_) => {
                                        panic!("Input ports can't be driven by constants")
                                    }
                                };
                            }
                        }
                        json_netlist::PortDirection::Output => {
                            for (idx, bit) in net.bits.iter().enumerate() {
                                let const_inputs = if let json_netlist::BitVal::Constant(sp) = bit {
                                    Vec::from([Self::const_id(sp.to_bool().unwrap())])
                                } else {
                                    Vec::new()
                                };
                                let node = NodeBuilder::default()
                                    .cell_type(CellType::Output)
                                    .name(Some(format!("{}[{}]", port_name, idx)))
                                    .secure(secure)
                                    .inputs(const_inputs)
                                    .build()
                                    .unwrap();
                                let node_id = circuit.add_node(node);

                                input_bitvals.append(node_id, *bit);
                                if let json_netlist::BitVal::Signal(sig) = bit {
                                    signal_fanouts.append(*sig, node_id);
                                }
                            }
                        }
                        _ => panic!("InOut ports are not supported"),
                    };
                }
                // add gates and registers:
                for (cell_name, cell) in module.cells.iter() {
                    assert!(cell.output_ports().count() <= 1);
                    let node = NodeBuilder::default()
                        .cell_type(CellType::Gate {
                            gate_type: cell.cell_type.clone(),
                        })
                        .name(Some(cell_name.clone()))
                        .build()
                        .unwrap();
                    let node_id = circuit.add_node(node);
                    for (_, out_bits) in cell.output_ports() {
                        for bit in out_bits {
                            if let json_netlist::BitVal::Signal(sig) = bit {
                                output_signal.insert(node_id, *sig);
                                signal_driver.insert(*sig, node_id);
                            }
                        }
                    }
                    for (_, bit) in cell.input_ports() {
                        // Signal or Constants
                        input_bitvals.append(node_id, bit);
                        if let json_netlist::BitVal::Signal(sig) = bit {
                            signal_driver.insert(sig, node_id);
                            signal_fanouts.append(sig, node_id);
                        }
                    }
                }

                for (node_id, node) in circuit.nodes.iter_mut() {
                    if node.cell_type.has_input() {
                        for bv in input_bitvals.get(node_id).unwrap() {
                            let in_node = match bv {
                                json_netlist::BitVal::Signal(s) => *signal_driver.get(s).unwrap(),
                                json_netlist::BitVal::Constant(v) => {
                                    Circuit::const_id(v.to_bool().unwrap())
                                }
                            };
                            node.inputs.push(in_node);
                        }
                    }
                    if node.cell_type.has_output() {
                        println!("output_signals: {}", output_signal.len());
                        let sig = output_signal.get(node_id).unwrap();
                        for out_node_id in signal_fanouts.get(sig).unwrap() {
                            node.fanouts.insert(*out_node_id);
                        }
                    }
                }
            }
        }
        Ok(circuit)
    }
}
