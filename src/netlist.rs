use crate::ast;
use crate::atom::Atom;
use fxhash::{FxHashMap, FxHashSet};
use std::mem;

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

    fn push(&mut self, component: Atom) {
        self.0.push(component);
    }

    fn as_slice(&self) -> &[Atom] {
        self.0.as_slice()
    }

    fn to_flattened_name(&self) -> Atom {
        self.0
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("/")
            .into()
    }
}

#[derive(Debug)]
pub struct Instance {
    pub path: Path,
    pub instances: FxHashMap<Atom, Instance>,
    pub nets: FxHashMap<Atom, Net>,
    pub interface: FxHashMap<Atom, ast::Port>,
}

impl Instance {
    fn from_ast(
        ast: &crate::ast::Edif,
        parent_path: &[Atom],
        inst_name: &Atom,
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
            .map(|p| (Atom::from(format!("{}/{}", inst_name, p.name.name)), p))
            .collect();

        Instance {
            path,
            instances,
            nets,
            interface,
        }
    }

    fn flatten(&mut self) {
        let mut if_ports = FxHashSet::default();

        for (_, mut inst) in mem::take(&mut self.instances) {
            inst.flatten();

            if inst.instances.is_empty() && inst.nets.is_empty() {
                self.instances.insert(inst.path.to_flattened_name(), inst);
                continue;
            } else {
                debug_assert!(inst
                    .instances
                    .values()
                    .all(|inst| inst.instances.is_empty()));
                self.instances.extend(inst.instances);
            }

            let inst_path = inst.path;

            if_ports.clear();
            for (_, net) in &inst.nets {
                if_ports.extend(
                    net.ports
                        .iter()
                        .filter(|p| p.instance == inst_path)
                        .cloned(),
                );
            }

            let mut merger = NetMerger::new(if_ports.iter().cloned(), inst_path.clone());

            for (name, net) in &mut self.nets {
                if net.ports.intersection(&if_ports).next().is_some() {
                    if !merger.merge(name, net) {
                        unreachable!()
                    }
                }
            }

            self.nets.retain(|_, n| !n.ports.is_empty());

            for (name, mut net) in inst.nets {
                if !merger.merge(&name, &mut net) {
                    let mut p = inst_path.clone();
                    p.push(name);
                    assert!(self.nets.insert(p.to_flattened_name(), net).is_none());
                }
            }

            self.nets.extend(merger.build());
        }

        // for (net_name, n) in &self.nets {
        //     for p in &n.ports {
        //         if p.instance == self.name {
        //             continue;
        //         }

        //         let port_path = Atom::from(format!("{}/{}", p.instance, p.port));
        //         assert!(
        //             self.instances.contains_key(&p.instance),
        //             "instance \"{}\" not found, referenced by port {:?} in net {}",
        //             p.instance,
        //             p,
        //             net_name,
        //         );
        //         assert!(self.instances[&p.instance]
        //             .interface
        //             .contains_key(&port_path));
        //     }
        // }
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

    fn merge(&mut self, net_name: &Atom, net: &mut Net) -> bool {
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
            .get_or_insert_with(|| (net_name.clone(), FxHashSet::default()))
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
}

/// Instantiated netlist.
#[derive(Debug)]
pub struct Netlist {
    pub top: Instance,
}

impl Netlist {
    pub fn from_ast(ast: &crate::ast::Edif) -> Self {
        let top = Instance::from_ast(
            ast,
            &[],
            &ast.design.inst_name,
            &ast.design.libraryref,
            &ast.design.cellref,
        );
        Netlist { top }
    }

    /// Flatten the nested instance hierarchy.
    pub fn flatten(&mut self) {
        self.top.flatten();
    }
}
