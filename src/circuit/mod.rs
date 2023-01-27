mod cell_library;
mod circuit_impl;
mod dot;
mod from_netlist;
mod into_netlist;
mod masking;
mod node;

use petgraph::stable_graph::{self, StableDiGraph};
use simple_error::SimpleError;

pub use dot::Dot;
pub use from_netlist::NetlistAndLibrary;
pub use masking::Masking;

use node::Blackbox;
use node::{Node, NodeBuilder, NodePortId, NodeType};

use std::collections::{HashMap, HashSet};

type NodeIndex = stable_graph::NodeIndex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Netname {0} not found")]
    NetnameNotFound(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    SimpleError(#[from] SimpleError),
}

#[derive(Debug, Clone, Default)]
pub struct Circuit {
    pub name: String,
    graph: StableDiGraph<Node, (NodePortId, NodePortId)>, // edges store src port and dst port
    input_ordering_map: HashMap<NodeType, Vec<String>>,
    blackboxes: HashMap<String, Blackbox>,
    // blackbox_impls: HashMap<String, Circuit>,
    inputs: HashSet<NodeIndex>,
    clocks: HashSet<NodeIndex>,
    outputs: HashSet<NodeIndex>,
    registers: HashSet<NodeIndex>,
    consts: [Option<NodeIndex>; 2],
}
