use clap::Parser;
use sql_parse::{cli::Cli, program::new_program};

fn main() {
    let args = Cli::parse();
    let mut program = new_program(args);
    program.run()
}
