use crate::cli::Cli;
use crate::error_exit;

use regex::Regex;
use std::{io::BufRead, path::PathBuf};

#[derive(Debug)]
pub struct Program {
    pub(crate) treesitter_query: String,
    pub(crate) regexp: Regex,
    pub(crate) search_paths: Vec<PathBuf>,
}

impl Program {
    fn get_search_path(args: Cli) -> Vec<PathBuf> {
        if atty::is(atty::Stream::Stdin) && args.search_paths.is_empty() {
            Cli::missing_paths_error();
        }

        if args.search_paths.is_empty() {
            let stdin = std::io::stdin();
            stdin
                .lock()
                .lines()
                .map(|line| {
                    if let Ok(line) = line {
                        PathBuf::from(&line)
                    } else {
                        error_exit!("Failed to read stdin line")
                    }
                })
                .collect::<Vec<_>>()
        } else {
            args.search_paths
        }
    }

    pub fn run(&mut self) {
        println!("running");
        todo!()
    }
}

impl From<Cli> for Program {
    fn from(args: Cli) -> Self {
        let ts_query = std::fs::read_to_string(&args.treesitter_query).unwrap_or_else(|_| {
            error_exit!(
                "Failed to read provided regexp file: {:?}",
                args.treesitter_query
            )
        });

        Self {
            treesitter_query: ts_query,
            regexp: Regex::from(&args.regexp),
            search_paths: Self::get_search_path(args),
        }
    }
}
