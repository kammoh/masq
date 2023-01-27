use std::collections::HashMap;

use crate::netlist::json_netlist::{AttributeVal, Module, Netlist, Port, PortDirection};

use super::{Circuit, Error};

impl TryFrom<&Circuit> for Netlist {
    type Error = Error;

    fn try_from(circuit: &Circuit) -> Result<Self, Self::Error> {
        let mut modules = HashMap::new();

        let mut top = Module::default();

        top.attributes
            .insert("top".to_string(), AttributeVal::Number(1));

        for nx in circuit.inputs.iter() {
            let node = &circuit.graph[*nx];
            let port_name = node.name.as_ref().map(|s| s.to_string()).unwrap();
            top.ports.insert(
                port_name,
                Port {
                    direction: PortDirection::Input,
                    bits: Vec::new(),
                    offset: Default::default(),
                },
            );
        }

        modules.insert(circuit.name.clone(), top);

        Ok(Netlist {
            creator: "masquerade".to_string(),
            modules,
        })
    }
}
