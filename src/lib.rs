pub mod cli;
mod oldshit;
pub mod program;
mod treesitter;
pub mod utils;

use std::{
    io,
    path::PathBuf,
    process::{Command, Stdio},
};

use tree_sitter::{Parser, Query, QueryCursor};

#[allow(dead_code)]
fn treesitter_stuff() {
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
}

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

    //     use std::io::Read;
    //     use std::process::{Command, Stdio};
    //
    //     let text = "\
    //     SELECT *
    //     FROM {schema}.foo
    //     WHERE x = ?;
    //
    //
    //     DECLARE @my_table (
    //         foo INT,
    //         bar VARCHAR(20)
    //     );
    //     SELECT
    //         42,
    //         'hello'
    //     INTO @my_table;
    // ";
    //     let echo_child = Command::new("echo")
    //         .arg(text)
    //         .stdout(Stdio::piped())
    //         .spawn()
    //         .unwrap();
    //
    //     let rg_child = Command::new("rg")
    //         .arg("--line-number")
    //         .arg("--column")
    //         .arg("--color=never")
    //         .arg("SELECT")
    //         .stdin(Stdio::from(echo_child.stdout.unwrap()))
    //         .stdout(Stdio::piped())
    //         .spawn()
    //         .unwrap();
    //
    //     let mut output = String::new();
    //     rg_child
    //         .stdout
    //         .unwrap()
    //         .read_to_string(&mut output)
    //         .unwrap();
    //
    //     println!("{}", output);

    todo!()
}
