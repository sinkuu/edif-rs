use std::env;
use std::path::Path;

fn main() {
    println!("rerun-if-changed=build.rs");
    string_cache_codegen::AtomType::new("atom::Atom", "atom!")
        .atoms(&[
            "direction",
            "INPUT",
            "OUTPUT",
            "INOUT",
            "net",
            "instance",
            "edif",
            "edifversion",
            "edifLevel",
            "keywordmap",
            "status",
            "comment",
            "design",
            "Library",
            "edifLevel",
            "technology",
            "cell",
            "celltype",
            "viewtype",
            "contents",
            "property",
            "interface",
            "port",
            "array",
            "rename",
            "view",
            "joined",
            "member",
            "portref",
            "instanceref",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("edif_atom.rs"))
        .unwrap()
}
