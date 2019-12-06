use crate::atom::Atom;

#[derive(Debug)]
pub struct Edif {
    pub libs: Vec<Library>,
}

#[derive(Debug)]
pub struct Library {
    pub name: Atom,
    pub cells: Vec<Cell>,
}

#[derive(Debug)]
pub struct Cell {
    pub name: Atom,
    pub view: View,
}

#[derive(Debug)]
pub struct View {
    pub name: Atom,
    pub interface: Interface,
    pub contents: Vec<Content>,
}

#[derive(Debug)]
pub struct Interface {
    pub ports: Vec<Port>,
}

#[derive(Debug)]
pub struct Name {
    pub name: Atom,
    pub rename_from: Option<String>,
}

#[derive(Debug)]
pub struct Port {
    pub kind: PortKind,
    pub dir: Direction,
    pub name: Name,
    pub is_array: bool,
}

#[derive(Debug)]
pub enum PortKind {
    Single,
    Array(i32),
}

#[derive(Debug)]
pub enum Direction {
    Input,
    Output,
    InOut,
}

#[derive(Debug)]
pub enum Content {
    Net(Net),
    Instance(Instance),
}

#[derive(Debug)]
pub struct Net {
    pub name: Name,
    pub portrefs: Vec<PortRef>,
}

#[derive(Debug)]
pub struct Instance {
    pub name: Name,
}

#[derive(Debug)]
pub struct PortRef {
    pub port: Atom,
    pub member: Option<i32>,
    pub instance_ref: Option<Atom>,
}
