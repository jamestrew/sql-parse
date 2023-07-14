use std::path::PathBuf;

use regex::{Regex, RegexBuilder};

use crate::cli::{Cli, RegexpOptions};
use crate::error_exit;
use crate::treesitter::{SqlBlock, Treesitter as TS};
use crate::utils::*;

use super::Program;

pub(super) struct Rg {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
    regexp: Regex,
    invert_match: bool,
}

impl Rg {
    fn get_regex(rg_opts: &RegexpOptions) -> Regex {
        let mut regex = if let Some(pattern) = rg_opts.regexp.regexp.clone() {
            RegexBuilder::new(&pattern)
        } else if let Some(file_path) = rg_opts.regexp.regexp_file.clone() {
            let pattern = std::fs::read_to_string(&file_path).unwrap_or_else(|_| {
                error_exit!("Failed to read provided regexp file: {:?}", file_path)
            });
            RegexBuilder::new(&pattern)
        } else {
            unreachable!("invalid rg option {:?}", rg_opts.regexp)
        };

        if rg_opts.ignore_case {
            regex.case_insensitive(true);
        }
        if rg_opts.multiline {
            regex.multi_line(true);
        }

        regex
            .build()
            .unwrap_or_else(|_| error_exit!("Failed to build regex"))
    }
}

impl Program for Rg {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        let rg_opts: RegexpOptions = cli.command.into();

        Self {
            treesitter,
            search_paths,
            regexp: Rg::get_regex(&rg_opts),
            invert_match: rg_opts.invert_match,
        }
    }

    fn run(&mut self) {
        for (code, _path) in iter_valid_files(&self.search_paths) {
            for block in self.treesitter.sql_blocks(&code) {
                let sql = block.inner_text(&code);
                // let output = self.pipe_to_rg(sql);
                // print_output(&output, &code, &block);
            }
        }
    }
}

fn print_output(rg_output: &str, code: &str, block_info: &SqlBlock) {
    // loop lines
    // split rg output by `:`
    // get correct line and col(?) numbers
    // prin
}

#[cfg(test)]
mod test {}
