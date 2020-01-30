use anyhow::Result;
use edif::netlist;
use std::fs;

#[test]
fn parse() -> Result<()> {
    let s = fs::read_to_string(format!("{}/tests/test.edf", env!("CARGO_MANIFEST_DIR")))?;
    let n = netlist::from_str(&s)?;
    n.verify_references()?;

    Ok(())
}
