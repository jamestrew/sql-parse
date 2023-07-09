use clap::Parser;
use sql_parse::cli::Cli;

fn main() {
    let args = Cli::parse();
    // let mut program = Program::from(args);
    // program.run();

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
}

// 1. fetch files of interest with `rg --files-with-matches crs\.execute <PATH>`
// 2. use treesitter to find all nodes of interest
// this depends on the ts query but eg. crs.execute[many]
// 3. use regex to match for issue keywords
// 4. return the file and line num of the issue spot
