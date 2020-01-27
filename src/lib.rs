#[macro_use]
#[allow(dead_code, clippy::all)]
mod atom {
    include!(concat!(env!("OUT_DIR"), "/edif_atom.rs"));
}

pub use crate::atom::Atom;

pub mod ast;
pub mod netlist;
pub mod parser;
mod sexpr;
