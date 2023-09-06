use std::path::PathBuf;

use super::Program;
use crate::cli::Cli;
use crate::treesitter::Treesitter as TS;
use crate::utils::*;

pub(crate) struct Treesitter {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
}

impl Program for Treesitter {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        Self {
            treesitter,
            search_paths,
        }
    }

    fn run(&mut self) {
        for (code, path) in iter_valid_files(&self.search_paths) {
            let path = path.as_path().to_str().unwrap();
            for block in self.treesitter.sql_blocks(&code) {
                print(path, block.start_line_num(), None, block.inner_text(&code));
            }
        }
    }
}
