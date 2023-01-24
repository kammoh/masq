use itertools::Itertools;

use super::json_netlist::{
    AttributeVal, BitVal, Cell, Module, Netlist, Port, PortDirection, SpecialBit,
};
use std::{
    collections::{hash_map, HashMap},
    io::{Read, Write},
};

impl SpecialBit {
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            &SpecialBit::_0 => Some(false),
            &SpecialBit::_1 => Some(true),
            _ => None,
        }
    }
}

impl AttributeVal {
    pub fn to_number(&self) -> Option<usize> {
        match self {
            &AttributeVal::Num(n) => Some(n),
            &AttributeVal::Str(ref s) => {
                // If it's an empty string, the value was zero
                if s.len() == 0 {
                    Some(0)
                } else {
                    usize::from_str_radix(s, 2).ok()
                }
            }
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        self.to_number().map(|n| n != 0)
    }

    pub fn to_string_if_string(&self) -> Option<&str> {
        match self {
            &AttributeVal::Num(_) => None,
            &AttributeVal::Str(ref s) => {
                if s.len() == 0 {
                    // If it's an empty string then it wasn't originally a string
                    None
                } else if s
                    .find(|c| !(c == '0' || c == '1' || c == 'x' || c == 'z'))
                    .is_none()
                {
                    // If it only contains 01xz, then it wasn't originally a string
                    None
                } else {
                    if *s.as_bytes().last().unwrap() == b' ' {
                        // If the last character is a space, drop it
                        Some(s.split_at(s.len() - 1).0)
                    } else {
                        Some(s)
                    }
                }
            }
        }
    }
}

impl Cell {
    pub fn input_ports(&self) -> impl Iterator<Item = (&String, &BitVal)> + '_ {
        self.connections
            .iter()
            .filter_map(|(name, bits)| match self.port_directions.get(name) {
                Some(PortDirection::Input) if bits.len() == 1 => Some((name, &bits[0])),
                Some(PortDirection::Input) => panic!("{name}: Input bits.len() == {}", bits.len()),
                _ => None,
            })
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
    }
    pub fn output_ports(&self) -> impl Iterator<Item = (&String, &Vec<BitVal>)> + '_ {
        self.connections
            .iter()
            .filter_map(|(name, bits)| match self.port_directions.get(name) {
                Some(PortDirection::Output) => Some((name, bits)),
                _ => None,
            })
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
    }
}

impl Module {
    pub fn is_top(&self) -> bool {
        self.attributes
            .get("top")
            .and_then(|v| v.to_bool())
            .unwrap_or(false)
    }
    pub fn is_blackbox(&self) -> bool {
        self.attributes
            .get("blackbox")
            .and_then(|v| v.to_bool())
            .unwrap_or(false)
    }

    pub fn filter_ports(&self, dir: PortDirection) -> impl Iterator<Item = &String> + '_ {
        self.ports
            .iter()
            .filter_map(move |(n, p)| (p.direction == dir).then_some(n))
    }

    pub fn inputs(&self) -> Vec<String> {
        self.filter_ports(PortDirection::Input)
            .map(|s| s.clone())
            .collect()
    }
    pub fn outputs(&self) -> Vec<String> {
        self.filter_ports(PortDirection::Output)
            .map(|s| s.clone())
            .collect()
    }
}

impl Netlist {
    /// Create a new netlist
    pub fn new(creator: &str) -> Self {
        Self {
            creator: creator.to_owned(),
            modules: HashMap::new(),
        }
    }

    /// Read netlist data from a reader
    pub fn from_reader<R: Read>(reader: R) -> Result<Netlist, serde_json::Error> {
        serde_json::from_reader(reader)
    }

    /// Read netlist data from a slice containing the bytes from a Yosys .json file
    pub fn from_slice(input: &[u8]) -> Result<Netlist, serde_json::Error> {
        serde_json::from_slice(input)
    }

    /// Serialize to a String
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to a writer
    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), serde_json::Error> {
        serde_json::to_writer(writer, self)
    }

    pub fn get_top(&self) -> Option<(&String, &Module)> {
        let mut first_none_blackbox = None;
        for (name, module) in self.modules.iter() {
            if module.is_top() {
                return Some((name, module));
            }
            if first_none_blackbox.is_none() && module.is_blackbox() {
                first_none_blackbox = Some((name, module));
            }
        }
        first_none_blackbox
    }
}
