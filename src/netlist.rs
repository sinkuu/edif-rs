use crate::ast;
use crate::atom::Atom;
use fxhash::{FxHashMap, FxHashSet};
use std::fmt::{self, Debug};
use std::mem;

/// Create a [`Netlist`](Netlist) from a string of an EDIF netlist.
pub fn from_str(s: &str) -> anyhow::Result<Netlist> {
    let ast = crate::parser::EdifParser::parse_from_str(s)?;
    Ok(Netlist::from_ast(&ast))
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct Path(Vec<Atom>);

impl Path {
    fn from_path_and_name(path: &[Atom], name: Atom) -> Self {
        let mut v = path.to_vec();
        v.push(name);
        Path(v)
    }

    pub fn name(&self) -> Atom {
        self.0.last().unwrap().clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, component: Atom) {
        self.0.push(component);
    }

    fn as_slice(&self) -> &[Atom] {
        self.0.as_slice()
    }

    pub fn to_flattened_path(&self) -> Path {
        let len = self.0.len();
        if len == 1 {
            return self.clone();
        }

        let c = self.0[1..]
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("/");
        Path(vec![self.0[0].clone(), c.into()])
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, c) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Instance {
    pub path: Path,
    pub instances: FxHashMap<Atom, Instance>,
    pub nets: FxHashMap<Atom, Net>,
    pub interface: FxHashMap<Atom, ast::Port>,
    pub lib: Atom,
    pub cell: Atom,
    pub properties: FxHashMap<Atom, ast::Property>,
}

impl Instance {
    fn from_ast(
        ast: &crate::ast::Edif,
        parent_path: &[Atom],
        inst_name: &Atom,
        properties: &FxHashMap<ast::Name, ast::Property>,
        lib: &Atom,
        cell: &Atom,
    ) -> Self {
        let view = &ast.libs[lib].cells[cell].view;

        let path = Path::from_path_and_name(parent_path, inst_name.clone());

        let mut instances = FxHashMap::default();
        let mut nets = FxHashMap::default();

        for c in &view.contents {
            match c {
                ast::Content::Instance(inst) => {
                    let name = inst.name.name.clone();
                    let inst = Instance::from_ast(
                        ast,
                        path.as_slice(),
                        &name,
                        &inst.properties,
                        inst.libraryref.as_ref().unwrap(),
                        &inst.cellref,
                    );
                    instances.insert(name, inst);
                }
                ast::Content::Net(net) => {
                    nets.insert(Atom::from(&net.name.name), Net::from_ast(&net, &path));
                }
            }
        }

        let interface = view
            .interface
            .ports
            .iter()
            .cloned()
            .map(|p| (p.name.name.clone(), p))
            .collect();

        Instance {
            path,
            instances,
            nets,
            interface,
            properties: properties
                .iter()
                .map(|(k, v)| (k.name.clone(), v.clone()))
                .collect(),
            cell: cell.clone(),
            lib: lib.clone(),
        }
    }

    fn flatten(&mut self) {
        let mut if_ports = FxHashSet::default();

        for net in self.nets.values_mut() {
            net.flatten();
        }

        self.path = self.path.to_flattened_path();

        for (_, mut inst) in mem::take(&mut self.instances) {
            inst.flatten();

            if inst.instances.is_empty() && inst.nets.is_empty() {
                self.instances.insert(inst.path.name(), inst);
                continue;
            } else {
                assert!(inst
                    .instances
                    .values()
                    .all(|inst| inst.instances.is_empty()));
                self.instances.extend(inst.instances);
            }

            let inst_path = inst.path;

            if_ports.clear();

            // List up the ports that possibly interconnect between `inst` and `self`.
            for net in inst.nets.values() {
                if_ports.extend(
                    net.ports
                        .iter()
                        .filter(|p| p.instance == inst_path)
                        .cloned(),
                );
            }

            let mut merger = NetMerger::new(if_ports.iter().cloned(), inst_path.clone());

            let self_path = &self.path;
            for (name, net) in &mut self.nets {
                if net.ports.intersection(&if_ports).next().is_some() {
                    assert!(merger.merge(|| format!("{}/{}", self_path, name).into(), net));
                }
            }

            self.nets.retain(|_, n| !n.ports.is_empty());

            for (name, mut net) in inst.nets {
                if !merger.merge(|| name.clone(), &mut net) {
                    // An internal connection within `inst`.
                    net.flatten();
                    assert!(self
                        .nets
                        .insert(format!("{}/{}", inst_path, name).into(), net)
                        .is_none());
                }
            }

            for (name, mut net) in merger.build() {
                net.flatten();
                assert!(self.nets.insert(name, net).is_none());
            }
        }

        self.path = self.path.to_flattened_path();
    }

    fn verify_references_inner(
        &self,
        port_refs: &mut FxHashMap<Path, Vec<PortRef>>,
    ) -> anyhow::Result<()> {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(occupied) = port_refs.entry(self.path.clone()) {
            fn check_array(member: Option<i32>, kind: ast::PortKind) -> bool {
                use ast::PortKind::*;
                match (member, kind) {
                    (None, Single) => true,
                    (Some(x), Array(y)) => x < y,
                    _ => false,
                }
            }
            if let Some(rp) = occupied.get().iter().find(|rp| {
                rp.instance != self.path || !check_array(rp.member, self.interface[&rp.port].kind)
            }) {
                anyhow::bail!("Invalid reference {:?}", rp);
            }
            occupied.remove_entry();
        }

        for n in self.nets.values() {
            for p in &n.ports {
                if p.instance == self.path {
                    if !self.interface.contains_key(&p.port) {
                        anyhow::bail!("Instance '{}' does not have port '{}'.", self.path, p.port);
                    }
                    continue;
                }

                port_refs
                    .entry(p.instance.clone())
                    .or_default()
                    .push(p.clone());
            }
        }

        for inst in self.instances.values() {
            inst.verify_references_inner(port_refs)?;
        }

        Ok(())
    }

    pub fn verify_references(&self) -> anyhow::Result<()> {
        let mut refs = FxHashMap::default();
        self.verify_references_inner(&mut refs)?;
        if refs.values().all(|v| v.is_empty()) {
            Ok(())
        } else {
            let missing_ports = refs
                .into_iter()
                .flat_map(|(_, v)| v)
                .map(|p| {
                    let mut inst = p.instance;
                    inst.push(p.port);
                    inst.to_string()
                })
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("Missing ports: {}", missing_ports)
        }
    }
}

struct NetMerger {
    idx: FxHashMap<PortRef, usize>,
    nets: Vec<Option<(Atom, FxHashSet<PortRef>)>>,
    instance: Path,
}

impl NetMerger {
    fn new(ports: impl Iterator<Item = PortRef>, instance: Path) -> Self {
        let idx = ports
            .enumerate()
            .map(|(i, p)| (p, i))
            .collect::<FxHashMap<PortRef, usize>>();
        let len = idx.len();
        NetMerger {
            idx,
            nets: vec![None; len],
            instance,
        }
    }

    fn merge(&mut self, net_name: impl FnOnce() -> Atom, net: &mut Net) -> bool {
        let indices = net
            .ports
            .iter()
            .filter(|p| p.instance == self.instance)
            .map(|p| (p, self.idx[p]))
            .collect::<Vec<_>>();

        if indices.is_empty() {
            return false;
        }

        let i = *indices.iter().map(|(_, i)| i).min().unwrap();
        for (p, j) in indices {
            if i == j {
                continue;
            }

            if let Some((name_j, nets_j)) = self.nets[j].take() {
                match &mut self.nets[i] {
                    Some((name_i, nets_i)) => {
                        if name_j < *name_i {
                            *name_i = name_j;
                        }
                        nets_i.extend(nets_j);
                    }
                    None => {
                        self.nets[i] = Some((name_j, nets_j));
                    }
                }
            }

            *self.idx.get_mut(p).unwrap() = i;
        }

        self.nets[i]
            .get_or_insert_with(|| (net_name(), FxHashSet::default()))
            .1
            .extend(net.ports.drain());

        true
    }

    fn build(self) -> impl Iterator<Item = (Atom, Net)> {
        let instance = self.instance;
        self.nets
            .into_iter()
            .flatten()
            .map(move |(name, mut ports)| {
                ports.retain(|p| p.instance != instance);
                (name, Net { ports })
            })
    }
}

#[derive(Debug)]
pub struct Net {
    pub ports: FxHashSet<PortRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortRef {
    pub instance: Path,
    pub port: Atom,
    pub member: Option<i32>,
}

impl Net {
    fn from_ast(ast: &ast::Net, parent_path: &Path) -> Net {
        Net {
            ports: ast
                .portrefs
                .iter()
                .map(|pr| {
                    let instance = if let Some(inst_ref) = &pr.instance_ref {
                        let mut p = parent_path.clone();
                        p.push(inst_ref.clone());
                        p
                    } else {
                        parent_path.clone()
                    };

                    PortRef {
                        instance,
                        port: pr.port.clone(),
                        member: pr.member,
                    }
                })
                .collect(),
        }
    }

    fn flatten(&mut self) {
        self.ports = mem::take(&mut self.ports)
            .into_iter()
            .map(|mut p| {
                p.instance = p.instance.to_flattened_path();
                p
            })
            .collect();
    }
}

/// Instantiated netlist.
#[derive(Debug)]
pub struct Netlist {
    pub top: Box<Instance>,
}

impl Netlist {
    pub fn from_ast(ast: &crate::ast::Edif) -> Self {
        let top = Instance::from_ast(
            ast,
            &[],
            &ast.design.inst_name,
            &Default::default(),
            &ast.design.libraryref,
            &ast.design.cellref,
        );
        Netlist { top: Box::new(top) }
    }

    /// Flatten the nested instance hierarchy.
    pub fn flatten(&mut self) {
        self.top.flatten();
    }

    pub fn verify_references(&self) -> anyhow::Result<()> {
        self.top.verify_references()
    }
}
