mod quotes;
mod rg;
mod treesitter;

use quotes::Quotes;
use rg::Rg;
use treesitter::Treesitter;

use crate::cli::{Cli, Commands};

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
