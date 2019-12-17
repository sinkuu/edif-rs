use crate::ast;
use crate::atom::Atom;
use fxhash::FxHashMap;

#[derive(Debug)]
pub struct Instance {
    pub name: Atom,
    pub instances: FxHashMap<Atom, Instance>,
    pub nets: FxHashMap<Atom, Net>,
    pub interface: FxHashMap<Atom, ast::Port>,
}

impl Instance {
    fn from_ast(ast: &crate::ast::Edif, inst_name: &Atom, lib: &Atom, cell: &Atom) -> Self {
        let view = &ast.libs[lib].cells[cell].view;

        let mut instances = FxHashMap::default();
        let mut nets = FxHashMap::default();

        for c in &view.contents {
            match c {
                ast::Content::Instance(inst) => {
                    let name = &inst.name.name;
                    instances.insert(
                        name.clone(),
                        Instance::from_ast(
                            ast,
                            name,
                            inst.libraryref.as_ref().unwrap(),
                            &inst.cellref,
                        ),
                    );
                }
                ast::Content::Net(net) => {
                    nets.insert(net.name.name.clone(), Net::from_ast(&net));
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

        for n in nets.values() {
            for p in n.ports.values() {
                if let Some(instance_ref) = &p.instance_ref {
                    assert!(
                        instances.contains_key(instance_ref),
                        "instance `{}` not found in instance `{}`",
                        instance_ref,
                        inst_name,
                    );

                    assert!(
                        instances[instance_ref].interface.contains_key(&p.port),
                        "instance `{}` does not have port `{}`",
                        instance_ref,
                        p.port,
                    );
                }
            }
        }

        Instance {
            name: inst_name.clone(),
            instances,
            nets,
            interface,
        }
    }

    fn into_hierarchy_named(self, parent_path: &[Atom]) -> (Atom, Instance) {
        let Instance {
            name,
            nets,
            instances,
            interface,
        } = self;

        let mut path = parent_path.to_vec();
        path.push(name.clone());
        let new_inst_name = path
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("/");

        let instances = instances
            .into_iter()
            .map(|(_, inst)| inst.into_hierarchy_named(&path))
            .collect();

        let nets = nets
            .into_iter()
            .map(|(_, net)| {
                let new_name = Atom::from(format!("{}/{}", new_inst_name, net.name));
                let ports = net
                    .ports
                    .into_iter()
                    .map(|(_, mut portref)| {
                        if let Some(inst_ref) = portref.instance_ref {
                            let inst_ref = format!("{}/{}", new_inst_name, inst_ref);
                            portref.port = Atom::from(format!("{}/{}", inst_ref, portref.port));
                            portref.instance_ref = Some(Atom::from(inst_ref));
                        } else {
                            portref.port =
                                Atom::from(format!("{}/{}", new_inst_name, portref.port));
                        };

                        (portref.port.clone(), portref)
                    })
                    .collect();

                (
                    new_name.clone(),
                    Net {
                        name: new_name,
                        ports,
                    },
                )
            })
            .collect();

        let interface = interface
            .into_iter()
            .map(|(_, mut port)| {
                port.name = ast::Name {
                    name: Atom::from(format!("{}/{}", new_inst_name, port.name.name)),
                    rename_from: port.name.rename_from,
                };
                (port.name.name.clone(), port)
            })
            .collect();

        let finst = Instance {
            name: Atom::from(new_inst_name),
            instances,
            nets,
            interface,
        };

        (finst.name.clone(), finst)
    }
}

#[derive(Debug)]
pub struct Net {
    pub name: Atom,
    pub ports: FxHashMap<Atom, ast::PortRef>,
}

impl Net {
    fn from_ast(ast: &ast::Net) -> Net {
        Net {
            name: ast.name.name.clone(),
            ports: ast
                .portrefs
                .iter()
                .cloned()
                .map(|pr| (pr.port.clone(), pr))
                .collect(),
        }
    }
}

/// Instantiated netlist.
#[derive(Debug)]
pub struct Netlist {
    top: Instance,
}

impl Netlist {
    pub fn from_ast(ast: &crate::ast::Edif) -> Self {
        let top = Instance::from_ast(
            ast,
            &ast.design.inst_name,
            &ast.design.libraryref,
            &ast.design.cellref,
        );
        Netlist { top }
    }

    /// Rename netlist elements to include its parents, e.g. `port` -> `inst/inner_inst/port`.
    pub fn into_hierarchy_named(self) -> Self {
        Netlist {
            top: self.top.into_hierarchy_named(&[]).1,
        }
    }
}
