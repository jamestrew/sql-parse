use std::path::PathBuf;

use crate::cli::Cli;
use crate::treesitter::Treesitter as TS;
use crate::utils::iter_valid_files;

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
            search_paths,
        }
    }

    fn run(&mut self) {
        for (code, path) in iter_valid_files(&self.search_paths) {
            for block in self.treesitter.sql_blocks(&code) {
                println!(
                    "{}:{}:{}",
                    path.display(),
                    block.start_line_num(),
                    block.inner_text(&code)
                );
            }
        }
    }
}
