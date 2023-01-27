use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::{collections::HashMap, env, fmt};
use either::Either;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Number {
    pub size: Option<usize>,
    pub value: Either<i64, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    String(String),
    Number(Number),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Primary {
    Literal(Literal),
    Net(String), // 1-bit wire with name
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDeclaration {
    pub attributes: HashMap<String, Literal>,
    pub name: String,
    pub ports: HashMap<String, Port>,
    pub cells: HashMap<String, Instance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Input,
    Output,
    InOut,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Port {
    pub attributes: HashMap<String, Literal>,
    pub direction: Direction,
    pub size: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instance {
    pub attributes: HashMap<String, Literal>,
    pub module: String,
    pub parameters: HashMap<String, Literal>,
    pub connections: HashMap<String, Primary>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() -> Result<(), Error> {
        let netlist_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "tests/Xoodyak/mkPerm_netlist.v"
        );
        let src = std::fs::read_to_string(netlist_path)?;
        println!("{}", src);
        Ok(())
    }
}