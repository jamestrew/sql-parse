use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tree_sitter::{Parser, Query, QueryCursor};

fn main() -> anyhow::Result<()> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_python::language())
        .expect("Error loading Python grammar");

    let source_code = include_str!("../test/b.py");
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();

    let query = include_str!("../queries/execute.scm");

    let q = Query::new(tree_sitter_python::language(), query).unwrap();
    let mut query_cursor = QueryCursor::new();

    let capture_names = q.capture_names();

    let matches = query_cursor.matches(&q, root_node, source_code.as_bytes());
    for mat in matches {
        for cap in mat.captures {
            let capture_name = &capture_names[cap.index as usize];
            if capture_name == "sql" {
                let start_pos = cap.node.start_position();
                let text = &source_code[cap.node.byte_range()];
                println!("{}: {}", start_pos.row + 1, text);
            }
        }
    }

    println!("hello world");
    Ok(())
}

// 1. fetch files of interest with `rg --files-with-matches crs\.execute <PATH>`
// 2. use treesitter to find all nodes of interest
// this depends on the ts query but eg. crs.execute[many]
// 3. use regex to match for issue keywords
// 4. return the file and line num of the issue spot

#[allow(dead_code)]
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
