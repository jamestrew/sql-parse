use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_python() -> Language;
}

fn main() -> anyhow::Result<()> {
    let language = unsafe { tree_sitter_python() };
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let source_code = "def foo():\n\tprint('hello world')";
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();

    assert_eq!(root_node.kind(), "source_file");
    assert_eq!(root_node.start_position().column, 0);
    assert_eq!(root_node.end_position().column, 0);

    println!("hello world");
    Ok(())
}

// 1. fetch files of interest with `rg --files-with-matches crs\.execute <PATH>`
// 2. use treesitter to find all nodes of interest
// this depends on the ts query but eg. crs.execute[many]
// 3. use regex to match for issue keywords
// 4. return the file and line num of the issue spot

fn files_with_matches() -> anyhow::Result<&'static [PathBuf]> {
    let _output = Command::new("rg")
        .arg("--files-with-matches")
        .arg("--color=never")
        .arg("crs\\.execute")
        .arg("test")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to capture rg output"))?;

    todo!()
}
