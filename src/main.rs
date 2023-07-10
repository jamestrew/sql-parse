use clap::Parser;
use sql_parse::cli::Cli;
use sql_parse::program::{new_program, Program};

fn main() {
    let args = Cli::parse();
    let mut program = new_program(args);
    program.run()
}

// 1. fetch files of interest with `rg --files-with-matches crs\.execute <PATH>`
// 2. use treesitter to find all nodes of interest
// this depends on the ts query but eg. crs.execute[many]
// 3. use regex to match for issue keywords
// 4. return the file and line num of the issue spot
