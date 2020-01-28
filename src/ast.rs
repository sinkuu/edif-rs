use crate::atom::Atom;
use fxhash::FxHashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Edif {
    pub libs: FxHashMap<Atom, Library>,
    pub design: Design,
}

#[derive(Debug)]
pub struct Design {
    pub inst_name: Atom,
    pub cellref: Atom,
    pub libraryref: Atom,
}

#[derive(Debug)]
pub struct Library {
    pub name: Atom,
    pub cells: FxHashMap<Atom, Cell>,
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name {
    pub name: Atom,
    pub rename_from: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub kind: PortKind,
    pub dir: Direction,
    pub name: Name,
}

#[derive(Clone, Copy, Debug)]
pub enum PortKind {
    Single,
    Array(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub cellref: Atom,
    pub viewref: Atom,
    pub libraryref: Option<Atom>,
    pub properties: Arc<FxHashMap<Name, Property>>,
}

#[derive(Debug, Clone)]
pub enum Property {
    String(String),
    Integer(i32),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub struct PortRef {
    pub port: Atom,
    pub member: Option<i32>,
    pub instance_ref: Option<Atom>,
}
