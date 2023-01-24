use derive_builder::Builder;
use simple_error::SimpleError;
use std::{fmt::Display, str::FromStr};
use strum::EnumProperty;

use super::cell_library::CellLibrary;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Debug,
    Hash,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::EnumProperty,
)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum GateType {
    And(u8),
    #[strum(props(inverted_alias = "NOT"))]
    Buf,
    Or(u8),
    Mux,
    #[strum(props(inverted_alias = "XNOR"))]
    Xor(u8),
}

impl GateType {
    pub fn inverted_alias(&self) -> String {
        self.get_str("inverted_alias")
            .map_or(format!("N{}", self), |s| s.to_owned())
    }
}

// trait InputOrder {
//     fn input_order(&self) -> Option<Vec<&str>>;
//     fn num_inputs(&self) -> usize {
//         self.input_order().map_or(0, |v| v.len())
//     }
//     fn input_index(&self, port: &str) -> Option<usize> {
//         self.input_order().map_or(None, |v| {
//             v.iter().position(|p| port.eq_ignore_ascii_case(p))
//         })
//     }
// }

// impl InputOrder for GateType {
//     fn input_order(&self) -> Option<Vec<&str>> {
//         match self {
//             Self::Mux => Some(vec!["s", "i0", "i1"]),
//             _ => None,
//         }
//     }
// }

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Blackbox {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NodeType {
    Input,
    Clock,
    Reset,
    Gate(GateType, bool),
    Gadget {
        base_type: GateType,
        invert: bool,
        num_shares: u8,
    },
    Blackbox(String),
    Register,
    Output,
    Constant(bool),
}

impl FromStr for NodeType {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn non_inverted_name(s: &str) -> Option<&str> {
            match s {
                "NOT" => Some("BUF"),
                "XNOR" => Some("XOR"),
                "NAND" => Some("AND"),
                "NOR" => Some("OR"),
                _ => None,
            }
        }
        fn gate_type_from_str(s: &str, inverted: bool) -> Option<(GateType, bool)> {
            GateType::from_str(s).ok().map(|gt| (gt, inverted))
        }
        match s {
            "DFF" => Ok(Self::Register),
            _ => {
                let node_type = gate_type_from_str(s, false)
                    .or_else(|| {
                        non_inverted_name(s).and_then(|inv_name| gate_type_from_str(inv_name, true))
                    })
                    .map(|(gate_type, invert)| Self::Gate(gate_type, invert))
                    .or_else(|| s.parse::<usize>().ok().map(|v| Self::Constant(v != 0)))
                    .unwrap_or(Self::Blackbox(s.to_owned()));

                Ok(node_type)
            }
        }
    }
}

// impl InputOrder for NodeType {
//     fn input_order(&self) -> Option<Vec<&str>> {
//         match self {
//             Self::Gate(gate_type, _) => gate_type.input_order(),
//             _ => None,
//         }
//     }
// }

fn bool_to_int<T>(v: bool) -> T
where
    T: From<u8>,
{
    (if v { 1 } else { 0 }).into()
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Input => f.write_str("IN"),
            NodeType::Clock => f.write_str("CLOCK"),
            NodeType::Reset => f.write_str("Reset"),
            NodeType::Gate(gate_type, inv) => {
                if *inv {
                    gate_type.inverted_alias().fmt(f)
                } else {
                    gate_type.fmt(f)
                }
            }
            NodeType::Blackbox(bb) => bb.fmt(f),
            NodeType::Register => f.write_str("FF"),
            NodeType::Output => f.write_str("OUT"),
            NodeType::Constant(v) => bool_to_int::<u8>(*v).fmt(f),
            NodeType::Gadget {
                base_type,
                invert,
                num_shares,
            } => write!(f, "{} Gadget", NodeType::Gate(*base_type, *invert)),
        }
    }
}

impl NodeType {
    pub fn has_input(&self) -> bool {
        match self {
            NodeType::Gate { .. } | NodeType::Register | NodeType::Output => true,
            _ => false,
        }
    }
    pub fn has_output(&self) -> bool {
        match self {
            NodeType::Input
            | NodeType::Clock
            | NodeType::Gate { .. }
            | NodeType::Register
            | NodeType::Constant { .. } => true,
            _ => false,
        }
    }
}

impl TryFrom<(&CellLibrary, &str)> for NodeType {
    type Error = SimpleError;

    fn try_from(value: (&CellLibrary, &str)) -> Result<Self, Self::Error> {
        value
            .0
            .node_type_from_cell_type(value.1)
            .ok_or(SimpleError::new("Cell type not found!"))
    }
}

impl TryFrom<(&CellLibrary, &String)> for NodeType {
    type Error = SimpleError;

    fn try_from(value: (&CellLibrary, &String)) -> Result<Self, Self::Error> {
        Self::try_from((value.0, value.1.as_str()))
    }
}

#[derive(Builder, Clone, Debug)]
#[builder(setter(into))]
pub struct Node {
    #[builder(default = "false")]
    pub secure: bool,
    pub node_type: NodeType,
    #[builder(default)]
    pub name: Option<String>,
}

impl Node {
    pub fn constant(value: bool) -> Node {
        NodeBuilder::default()
            .node_type(NodeType::Constant(value))
            .build()
            .unwrap()
    }
}

pub type NodePortId = u8;
