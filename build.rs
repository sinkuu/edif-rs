use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    string_cache_codegen::AtomType::new("atom::Atom", "atom!")
        .with_atom_doc("Interned symbols in EDIF AST.")
        .atoms(vec![
            "array",
            "boolean",
            "cell",
            "cellref",
            "celltype",
            "comment",
            "contents",
            "design",
            "direction",
            "edif",
            "edifLevel",
            "edifLevel",
            "edifversion",
            "false",
            "hdi_primitives",
            "INOUT",
            "INPUT",
            "instance",
            "instanceref",
            "integer",
            "interface",
            "joined",
            "keywordmap",
            "Library",
            "libraryref",
            "main",
            "member",
            "net",
            "NETLIST",
            "OUTPUT",
            "port",
            "portref",
            "property",
            "rename",
            "status",
            "string",
            "technology",
            "true",
            "view",
            "viewref",
            "viewtype",
            "work",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("edif_atom.rs"))
        .unwrap()
}
