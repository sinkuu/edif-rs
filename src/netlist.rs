use crate::atom::Atom;
use fxhash::FxHashMap;

#[derive(Debug)]
pub struct Netlist {
    pub instances: FxHashMap<Atom, Instance>,
    pub nets: FxHashMap<Atom, Net>,
}

#[derive(Debug)]
pub struct Instance {
    pub name: Atom,
    pub ports: FxHashMap<Atom, crate::ast::Direction>,
}

impl Instance {
    fn from_ast_cell(ast: &crate::ast::Cell, name: Atom) -> Instance {
        let ports = ast
            .view
            .interface
            .ports
            .iter()
            .map(|p| (p.name.name.clone(), p.dir))
            .collect();
        Instance {
            name,
            ports,
        }
    }
}

#[derive(Debug)]
pub struct Net {
    pub name: Atom,
    pub ports: Vec<(Atom, Atom)>,
}

impl Net {
    fn from_ast(ast: &crate::ast::Net, current_cell: &Atom) -> Self {
        Net {
            name: ast.name.name.clone(),
            ports: ast
                .portrefs
                .iter()
                .map(|p| {
                    (
                        p.instance_ref
                            .clone()
                            .unwrap_or_else(|| current_cell.clone()),
                        p.port.clone(),
                    )
                })
                .collect(),
        }
    }
}

impl Netlist {
    pub fn from_ast(ast: crate::ast::Edif) -> Netlist {
        let work_lib = ast.libs.iter().find(|lib| &*lib.name == "work").unwrap();
        let hdl_lib = ast
            .libs
            .iter()
            .find(|lib| &*lib.name == "hdi_primitives")
            .unwrap();

        let mut instances = FxHashMap::default();
        let mut nets = FxHashMap::default();

        let mut stack = vec![(
            atom!("work"),
            atom!("main"),
            &work_lib.cells[&atom!("main")],
        )];

        while !stack.is_empty() {
            let mut new_stack = vec![];
            for (lib, inst_name, cell) in stack {
                for c in &cell.view.contents {
                    use crate::ast::Content;
                    match c {
                        Content::Net(net) => {
                            nets.insert(net.name.name.clone(), Net::from_ast(net, &inst_name));
                        }
                        Content::Instance(inst) => {
                            let lref = inst.libraryref.as_ref().unwrap_or(&lib);
                            let l = match lref {
                                &atom!("work") => &work_lib,
                                &atom!("hdi_primitives") => &hdl_lib,
                                _ => panic!("unknown library: {:?}", inst.libraryref),
                            };

                            let cell = &l.cells[&inst.cellref];
                            assert!(cell.view.name == inst.viewref);

                            new_stack.push((lref.clone(), inst.name.name.clone(), cell));
                        }
                    }
                }

                instances.insert(inst_name.clone(), Instance::from_ast_cell(cell, inst_name));
            }

            stack = new_stack;
        }

        for n in nets.values() {
            for (inst, port) in &n.ports {
                assert!(instances.contains_key(inst), "{} not found", inst);
                assert!(instances[inst].ports.contains_key(port));
            }
        }

        Netlist { instances, nets }
    }
}
