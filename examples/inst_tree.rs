use std::collections::VecDeque;
use std::env;

fn main() -> anyhow::Result<()> {
    let s = std::fs::read_to_string(env::args().nth(1).unwrap())?;
    let mut netlist = edif::netlist::from_str(&s)?;
    let flatten = env::args().nth(2).map(|a| a == "--flatten").unwrap_or(false);
    if flatten {
        netlist.top.verify_references().unwrap();
        netlist.flatten();
    }
    netlist.top.verify_references().unwrap();

    let mut stack = VecDeque::new();
    stack.push_back((0, netlist.top));

    while let Some((level, inst)) = stack.pop_back() {
        let path = if flatten {
            inst.path.to_string()
        } else {
            inst.path.name().to_string()
        };
        println!("{}{}", "  ".repeat(level), path);

        let mut children = inst.instances.into_iter().collect::<Vec<_>>();
        children.sort_by(|(a, _), (b, _)| a.cmp(b));

        stack.extend(children.into_iter().rev().map(|(_, inst)| (level + 1, inst)));
    }

    Ok(())
}
