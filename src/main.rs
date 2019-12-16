use anyhow::Result;
use combine::Parser;
use fxhash::FxHashMap;
use petgraph::{
    graph::{Graph, NodeIndex},
    Directed,
};

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
    let s = std::fs::read_to_string("./mips.edf").unwrap();

    let e = sexpr::sexpr_parser()
        .easy_parse(combine::stream::state::State::new(s.as_str()))
        .unwrap()
        .0;
    let ep = parser::EdifParser::new();
    let ast = ep.parse(&e)?;

    let nl = netlist::Netlist::from_ast(ast);

    Ok(())
}
