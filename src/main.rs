use anyhow::Result;
use fxhash::FxHashMap;
use petgraph::{
    graph::{Graph, NodeIndex},
    Directed,
};
use std::io::{self, Read};

#[macro_use]
#[allow(dead_code)]
mod atom {
    include!(concat!(env!("OUT_DIR"), "/edif_atom.rs"));
}

use atom::Atom;

mod ast;
mod netlist;
mod parser;
mod sexpr;

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut s = String::new();
    stdin.read_to_string(&mut s)?;

    let ast = parser::EdifParser::parse_from_str(&s)?;

    netlist::Netlist::from_ast(&ast);

    Ok(())
}
