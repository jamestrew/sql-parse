pub mod quotes;
pub mod treesitter;

use crate::cli::{Cli, Commands};
use crate::program::quotes::Quotes;
use crate::program::treesitter::Treesitter;

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
        Commands::Rg(_) => todo!(),
    }
}
