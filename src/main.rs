use anyhow::Result;
use combine::Parser;

#[macro_use]
#[allow(dead_code)]
mod atom {
    include!(concat!(env!("OUT_DIR"), "/edif_atom.rs"));
}

mod ast;
mod parser;
mod sexpr;

fn main() -> Result<()> {
    let s = std::fs::read_to_string("./mips.edf").unwrap();

    let e = sexpr::sexpr_parser()
        .easy_parse(combine::stream::state::State::new(s.as_str()))
        .unwrap()
        .0;
    let ep = parser::EdifParser::new();
    dbg!(ep.parse(&e)?);

    Ok(())
}
