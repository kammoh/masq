pub mod circuit;
pub mod netlist;
pub mod utils;

use crate::circuit::Dot;
use crate::circuit::Masking;
use crate::circuit::NetlistAndLibrary;

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    CircuitError(#[from] circuit::Error),
}

fn main() -> Result<(), AppError> {
    let netlist_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "tests/hdl/simple/simple_1.json"
    );
    println!("reading netlist: {}", netlist_path);

    let netlist = NetlistAndLibrary::from_path(netlist_path)?;

    println!("Constructing circuit");
    let mut circuit = circuit::Circuit::try_from(&netlist)?;

    let dot_file = format!("{}_orig.dot", circuit.name);
    println!("Writing DOT to {}", dot_file);
    circuit.dump_to_file(&dot_file).expect("Writing dot failed");

    println!("Propagating secure");
    circuit.mask(1);

    let dot_file = format!("{}.dot", circuit.name);
    println!("Writing DOT to {}", dot_file);
    circuit.dump_to_file(&dot_file).expect("Writing dot failed");

    // circuit.

    Ok(())
}

#[cfg(test)]
mod tests{
    use crate::*;

    #[test]
    fn samples()  -> Result<(), AppError> {
        let netlist_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "tests/hdl/simple/simple_1.json"
        );
        println!("reading netlist: {}", netlist_path);
    
        let netlist = NetlistAndLibrary::from_path(netlist_path)?;
    
        println!("Constructing circuit");
        let mut circuit = circuit::Circuit::try_from(&netlist)?;
    
        let dot_file = format!("{}_orig.dot", circuit.name);
        println!("Writing DOT to {}", dot_file);
        circuit.dump_to_file(&dot_file).expect("Writing dot failed");
    
        println!("Propagating secure");
        circuit.mask(1);
    
        let dot_file = format!("{}.dot", circuit.name);
        println!("Writing DOT to {}", dot_file);
        circuit.dump_to_file(&dot_file).expect("Writing dot failed");
    
        // circuit.
    
        Ok(())
    }
}

