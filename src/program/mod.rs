mod quotes;
mod treesitter;
mod rg;


use crate::cli::{Cli, Commands};
use quotes::Quotes;
use treesitter::Treesitter;
use rg::Rg;

pub trait Program {
    fn new(cli: Cli) -> Self
    where
        Self: Sized;

    fn run(&mut self);
}

pub fn new_program(cli: Cli) -> Box<dyn Program> {
    match cli.command {
        Commands::TS(_) => Box::new(Treesitter::new(cli)),
        Commands::Quotes(_) => Box::new(Quotes::new(cli)),
        Commands::Regexp(_) => Box::new(Rg::new(cli)),
    }
}
