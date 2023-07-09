use std::path::PathBuf;

use crate::treesitter::Treesitter as TS;
use crate::cli::Cli;

use super::Program;

pub(crate) struct Treesitter {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
}

impl Treesitter {}

impl Program for Treesitter {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = Self::basic_cli_options(cli);
        Self {
            treesitter,
            search_paths
        }
    }

    fn run(&mut self) {
        todo!()
    }
}
