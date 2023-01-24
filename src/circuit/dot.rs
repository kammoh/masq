use petgraph::stable_graph::{self, EdgeReference, StableDiGraph};
use petgraph::visit::EdgeRef;

use super::{Circuit, Error, Node, NodeIndex, NodePortId, NodeType};
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub trait Dot {
    fn dump_to_file(&self, outfile: &str) -> Result<(), Error>;
}

impl Dot for Circuit {
    fn dump_to_file(&self, outfile: &str) -> Result<(), Error> {
        let mut of = File::create(outfile).unwrap();
        fn fmt_node(node_index: NodeIndex, node: &Node) -> String {
            let shape = match &node.node_type {
                NodeType::Gate { .. } => Some("record"),
                NodeType::Register => Some("record"),
                NodeType::Constant(_) => Some("octagon"),
                _ => None,
            };
            let mut node_attrs = HashMap::new();
            let node_type = node.node_type.to_string();
            node_attrs.insert("label", node.name.as_deref().unwrap_or(&node_type));
            if let Some(shape) = shape {
                node_attrs.insert("shape", shape);
            }
            if node.secure {
                node_attrs.insert("color", "red");
            }
            node_attrs
                .iter()
                .map(|(&k, &v)| format!("{k}=\"{v}\""))
                .join(", ")
        }
        fn fmt_edge(
            g: &StableDiGraph<Node, (NodePortId, NodePortId)>,
            e: EdgeReference<(NodePortId, NodePortId), stable_graph::DefaultIx>,
        ) -> String {
            let mut edge_attrs = Vec::new();
            if g[e.source()].secure {
                edge_attrs.push(("color", "red".to_string()));
            }
            let label = true;
            if label {
                edge_attrs.push(("label", format!("{},{}", e.weight().0, e.weight().1)));
            }
            edge_attrs
                .iter()
                .map(|(k, v)| format!("{k}=\"{v}\""))
                .join(", ")
        }
        let name = "netlist";
        of.write_all(
            format!(
                "digraph \"{name}\" {{\n  rankdir=\"LR\";\n  remincross=true;\n{:?}\n  {{rank=\"source\";{};}}\n  {{rank=\"sink\";{};}}\n}}",
                petgraph::dot::Dot::with_attr_getters(
                    &self.graph,
                    &[
                        petgraph::dot::Config::NodeNoLabel,
                        petgraph::dot::Config::EdgeNoLabel,
                        petgraph::dot::Config::GraphContentOnly
                    ],
                    &|g, e| fmt_edge(g, e),
                    &|_, (idx, node)| fmt_node(idx, node)
                ),
                self.inputs.iter().map(|id| id.index().to_string()).join(";"),
                self.outputs.iter().map(|id| id.index().to_string()).join(";"),
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}
