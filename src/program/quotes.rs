use std::path::PathBuf;

use crate::cli::Cli;
use crate::treesitter::Treesitter as TS;
use crate::utils::*;

use super::Program;

pub(crate) struct Quotes {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
}

impl Program for Quotes {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(cli);
        Self {
            treesitter,
            search_paths,
        }
    }

    fn run(&mut self) {
        todo!()
    }
}
