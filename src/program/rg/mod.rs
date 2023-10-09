mod finder;
mod utils;

use std::path::PathBuf;
use std::rc::Rc;

use finder::*;

use super::Program;
use crate::cli::{Cli, RegexOptions};
use crate::treesitter::{ts_query_factory, TreesitterQuery};

pub struct Rg {
    treesitter: Box<dyn TreesitterQuery>,
    search_paths: Rc<Vec<PathBuf>>,
    finder: Box<dyn Finder>,
}

impl Program for Rg {
    fn new(cli: Cli) -> Self {
        let treesitter = ts_query_factory(&cli);
        let search_paths = Rc::new(cli.search_paths());

        let rg_opts: RegexOptions = cli.command.into();

        let finder: Box<dyn Finder> = match (
            rg_opts.invert_match,
            rg_opts.replace.as_ref(),
            rg_opts.confirm,
        ) {
            (false, None, _) => Box::new(PlainSearch::new_finder(&rg_opts)),
            (false, Some(_), false) => Box::new(Replace::new_finder(&rg_opts)),
            (false, Some(_), true) => Box::new(ReplaceConfirm::new_finder(&rg_opts)),
            (true, _, _) => Box::new(InverseSearch::new_finder(&rg_opts)),
        };

        Self {
            treesitter,
            search_paths,
            finder,
        }
    }

    fn run(&mut self) {
        self.finder
            .find(&mut self.treesitter, self.search_paths.clone());
    }
}
