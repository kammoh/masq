use std::{collections::HashMap, str::FromStr};

use itertools::Itertools;

use super::node::NodeType;

#[derive(Debug)]
pub struct CellLibrary {
    cell_name_map: HashMap<String, String>,
    cell_map: HashMap<String, NodeType>,
    input_port_order: HashMap<String, Vec<String>>,
    output_port_order: HashMap<String, Vec<String>>,
}

impl CellLibrary {
    pub fn new() -> CellLibrary {
        let m = [
            //
            ("MUX", vec!["S", "A", "B"]),
            ("DFF", vec!["C", "D"]),
        ];

        let m = m.map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect_vec()));
        CellLibrary {
            cell_name_map: HashMap::new(),
            cell_map: HashMap::new(),
            input_port_order: HashMap::from(m),
            output_port_order: HashMap::new(),
        }
    }

    pub fn get_input_port_order(&self, cell: &str) -> Option<&Vec<String>> {
        self.input_port_order.get(cell)
    }

    pub fn get_output_port_order(&self, cell: &str) -> Option<&Vec<String>> {
        self.output_port_order.get(cell)
    }

    pub fn node_type_from_cell_type(&self, cell_type: &str) -> Option<NodeType> {
        let direct_node = self.cell_map.get(cell_type);
        if direct_node.is_some() {
            return direct_node.cloned();
        }
        let cell_type = self
            .cell_name_map
            .get(cell_type)
            .map(|s| s.as_str())
            .unwrap_or(cell_type);
        NodeType::from_str(cell_type).ok()
    }
}
