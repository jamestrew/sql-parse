use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let dir: PathBuf = ["tree-sitter-python", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .warnings(false)
        .compile("tree-sitter-python");
}
