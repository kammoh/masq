use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::bool_from_int;

pub type SignalId = u32;

/// Represents one module in the Yosys hierarchy
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct Module {
    /// Module attributes (Verilog `(* attr *)`)
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
    /// Module parameter (Verilog `parameter`) default values
    #[serde(default)]
    pub parameter_default_values: HashMap<String, AttributeVal>,
    /// Module ports (interfaces to other modules)
    #[serde(default)]
    pub ports: HashMap<String, Port>,
    /// Module cells (objects inside this module)
    #[serde(default)]
    pub cells: HashMap<String, Cell>,
    /// Module memories
    #[serde(default)]
    pub memories: HashMap<String, Memory>,
    /// Module netnames (names of wires in this module)
    #[serde(default)]
    pub netnames: HashMap<String, Netname>,
}

/// Legal values for the direction of a port on a module
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum PortDirection {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
    #[serde(rename = "inout")]
    InOut,
}

/// Constant bit values
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum ConstBit {
    /// Constant 0
    #[serde(rename = "0")]
    _0,
    /// Constant 1
    #[serde(rename = "1")]
    _1,
    /// Constant X (invalid)
    #[serde(rename = "x")]
    X,
    /// Constant Z (tri-state)
    #[serde(rename = "z")]
    Z,
}


/// A number representing a single bit of a wire
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum BitVal {
    /// An actual signal number
    Signal(SignalId),
    /// A special constant value
    Constant(ConstBit),
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum AttributeVal {
    Number(usize),
    String(String),
}

/// Represents an entire .json file used by Yosys
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct Netlist {
    /// The program that created this file.
    #[serde(default)]
    pub creator: String,
    /// A map from module names to module objects contained in this .json file
    #[serde(default)]
    pub modules: HashMap<String, Module>,
}


/// Represents a port on a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Port {
    /// Port direction
    pub direction: PortDirection,
    /// Bit value(s) representing the wire(s) connected to this port
    pub bits: Vec<BitVal>,
    /// Bit offset for mapping to HDL bit numbering
    #[serde(default)]
    pub offset: usize,
    // /// Whether or not HDL bit numbering is MSB-first
    // #[serde(default)]
    // pub upto: usize,
    // /// Whether or not HDL considers value signed
    // #[serde(default)]
    // pub signed: usize,
}

/// Represents a cell in a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Cell {
    /// Indicates an internal/auto-generated name that starts with `$`
    #[serde(default, deserialize_with = "bool_from_int")]
    pub hide_name: bool,
    /// Name of the type of this cell
    #[serde(rename = "type")]
    pub cell_type: String,
    /// Parameters specified on this cell
    #[serde(default)]
    pub parameters: HashMap<String, AttributeVal>,
    /// Attributes specified on this cell
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
    /// The direction of the ports on this cell
    #[serde(default)]
    pub port_directions: HashMap<String, PortDirection>,
    /// Bit value(s) representing the wire(s) connected to the inputs/outputs of this cell
    pub connections: HashMap<String, Vec<BitVal>>,
}

/// Represents a memory in a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Memory {
    /// Indicates an internal/auto-generated name that starts with `$`
    #[serde(default)]
    pub hide_name: bool,
    /// Attributes for this memory
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
    /// Memory width
    pub width: usize,
    /// Memory size
    pub size: usize,
    /// Lowest valid memory address
    #[serde(default)]
    pub start_offset: usize,
}

/// Represents the name of a net in a module
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Netname {
    /// Indicates an internal/auto-generated name that starts with `$`
    #[serde(default, deserialize_with = "bool_from_int")]
    pub hide_name: bool,
    /// Bit value(s) that should be given this name
    pub bits: Vec<BitVal>,
    /// Bit offset for mapping to HDL bit numbering
    // #[serde(default)]
    // pub offset: usize,
    // /// Whether or not HDL bit numbering is MSB-first
    // #[serde(default)]
    // pub upto: usize,
    // /// Whether or not HDL considers value signed
    // #[serde(default)]
    // pub signed: usize,
    /// Attributes for this netname
    #[serde(default)]
    pub attributes: HashMap<String, AttributeVal>,
}
