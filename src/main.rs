use anyhow::Result;
use std::io::{self, Read};

#[macro_use]
#[allow(dead_code)]
mod atom {
    include!(concat!(env!("OUT_DIR"), "/edif_atom.rs"));
}

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

    let mut netlist = netlist::Netlist::from_ast(&ast);
    netlist.flatten();
    println!("{:#?}", netlist);

    Ok(())
}
