use std::collections::HashMap;
use std::path::Path;

use crate::circuit::node::Blackbox;
use crate::netlist::json_netlist::{
    AttributeVal, BitVal, Netlist, PortDirection, SignalId,
};
use crate::utils::MapToVec;

use super::cell_library::CellLibrary;
use super::{Circuit, Error, NodeBuilder, NodeIndex, NodePortId, NodeType};

use std::{fs::File, io::BufReader};

pub struct NetlistAndLibrary {
    netlist: Netlist,
    cell_library: CellLibrary,
}

impl NetlistAndLibrary {
    pub fn new(netlist: Netlist) -> Self {
        Self {
            netlist,
            cell_library: CellLibrary::new(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(netlist_path: P) -> Result<Self, Error> {
        let file = File::open(netlist_path)?;
        let reader = BufReader::new(file);
        let netlist = Netlist::from_reader(reader)?;
        Ok(NetlistAndLibrary {
            netlist,
            cell_library: CellLibrary::new(),
        })
    }
}

impl TryFrom<&NetlistAndLibrary> for Circuit {
    type Error = Error;

    fn try_from(nl_cl: &NetlistAndLibrary) -> Result<Self, Self::Error> {
        fn io_node_name(port_name: &str, w: usize, idx: usize) -> Option<String> {
            if w > 1 {
                Some(format!("{}[{}]", port_name, idx))
            } else {
                Some(port_name.to_owned())
            }
        }
        let mut circuit = Circuit::default();

        let cl = &nl_cl.cell_library;
        let netlist = &nl_cl.netlist;

        for (bb_name, module) in netlist.modules.iter().filter(|(_, m)| m.is_blackbox()) {
            let bb = Blackbox {
                inputs: module.inputs(),
                outputs: module.outputs(),
            };
            circuit.blackboxes.insert(bb_name.to_owned(), bb);
        }

        let (top_name, module) = netlist.get_top().expect("No top module was found!");
        println!("Top module: {}", top_name);
        circuit.name = top_name.clone();
        // 1-to-1 mapping of signal -> (src_node_id, src_node_port)
        let mut sig_driver = HashMap::<SignalId, (NodeIndex, NodePortId)>::default();
        // mapping of node_id -> Vec of input bits in the order of node's input ports
        let mut node_to_inbits = MapToVec::<NodeIndex, BitVal>::default();
        // FIXME for all modules?
        // add IO nodes:
        for (port_name, port) in module.ports.iter() {
            let net = module
                .netnames
                .get(port_name)
                .ok_or_else(|| Error::NetnameNotFound(port_name.to_owned()))?;
            let w = net.bits.len();
            match port.direction {
                PortDirection::Input => {
                    for (idx, bit) in net.bits.iter().enumerate() {
                        let (node_type, is_secure) = match net.attributes.get("MASQ") {
                            Some(AttributeVal::String(s)) => match s.to_lowercase().as_str() {
                                "secure" => (NodeType::Input, true),
                                "clock" => (NodeType::Clock, false),
                                "reset" => (NodeType::Reset, false),
                                _ => (NodeType::Input, false),
                            },
                            _ => (NodeType::Input, false),
                        };
                        let node = NodeBuilder::default()
                            .node_type(node_type)
                            .secure(is_secure)
                            .name(io_node_name(port_name, w, idx))
                            .build()
                            .unwrap();
                        let node_id = circuit.add_node(node);

                        match bit {
                            BitVal::Signal(sig) => {
                                sig_driver.insert(*sig, (node_id, 0)).map(|old_value| {
                                    panic!(
                                        "driver node {:?} already present for signal {}",
                                        old_value, sig
                                    )
                                });
                            }
                            BitVal::Constant(_) => {
                                panic!("Input ports can't be connected to constants")
                            }
                        };
                    }
                }
                PortDirection::Output => {
                    for (idx, &bit) in net.bits.iter().enumerate() {
                        let node = NodeBuilder::default()
                            .node_type(NodeType::Output)
                            .name(io_node_name(port_name, w, idx))
                            .build()
                            .unwrap();
                        let node_id = circuit.add_node(node);
                        node_to_inbits.append(node_id, bit);
                    }
                }
                t => panic!("{:?} is not supported", t),
            };
        }
        // add gates and registers:
        for (cell_name, cell) in module.cells.iter() {
            let node_type = NodeType::try_from((cl, &cell.cell_type))?;
            let node = NodeBuilder::default()
                .node_type(node_type.clone())
                .name((!cell_name.is_empty() && !cell.hide_name).then_some(cell_name.clone()))
                .build()
                .unwrap();
            let node_id = circuit.add_node(node);
            //
            for ((_, bits), out_port_id) in cell.output_ports().zip(0..) {
                for bit in bits {
                    if let BitVal::Signal(sig) = bit {
                        sig_driver.insert(*sig, (node_id, out_port_id));
                    }
                }
            }

            // if we have an ordering in CellLibrary use that
            if let Some(order) = cl.get_input_port_order(&cell.cell_type) {
                for port_name in order {
                    let bits = cell.connections.get(port_name).unwrap();
                    assert!(bits.len() == 1);
                    node_to_inbits.append(node_id, bits[0]);
                }
                circuit.input_ordering_map.insert(node_type, order.clone());
            } else {
                let mut names = Vec::new();
                // otherwise use alphabetical ordering
                for (name, bit) in cell.input_ports() {
                    // Signal or Constants
                    node_to_inbits.append(node_id, *bit);
                    names.push(name.clone());
                }
                if names.len() > 1 {
                    circuit.input_ordering_map.insert(node_type, names);
                }
            }
        }
        // should include all Gate, Register, and Output nodes
        for (dst_node, in_bits) in node_to_inbits.iter() {
            for (bit, dst_port) in in_bits.iter().zip(0..) {
                // TODO ordered or edge: (src.out_port -> dst.in_port)
                let (src_node, src_port) = match bit {
                    BitVal::Signal(s) => *sig_driver.get(s).unwrap(),
                    BitVal::Constant(v) => (circuit.const_node(v.to_bool().unwrap()), 0),
                };
                circuit.connect(src_node, src_port, *dst_node, dst_port);
            }
        }
        Ok(circuit)
    }
}
