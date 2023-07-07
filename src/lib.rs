pub mod cli;
pub mod program;
mod utils;
mod treesitter;
mod oldshit;

use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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

    todo!()
}

#[allow(dead_code)]
fn find_pattern(sql_text: &str) {
    use regex::Regex;

    let re = Regex::new(r"(DECLARE @[\w_]+)").unwrap();
    let mat = re.find(sql_text).unwrap();
    println!("{:?}", mat);
}

#[cfg(test)]
mod test {
    mod test_regex {
        use crate::*;

        const SQL: &str = "
            DECLARE @my_table (
                foo INT,
                bar VARCHAR(20)
            );
            SELECT
                42,
                'hello'
            INTO @my_table;
        ";

        #[test]
        fn f() {
            find_pattern(SQL);
            assert!(false);
        }
    }
}
