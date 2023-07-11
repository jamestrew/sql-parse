use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::cli::{Cli, RegexpOption};
use crate::error_exit;
use crate::treesitter::Treesitter as TS;
use crate::utils::*;

use super::Program;

pub(super) struct Rg {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
    regex: String,
}

impl Rg {
    fn get_regex(value: RegexpOption) -> String {
        if let Some(pattern) = value.regexp {
            pattern
        } else if let Some(file_path) = &value.regexp_file {
            let pattern = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
                error_exit!("Failed to read provided regexp file: {:?}", file_path)
            });
            pattern
        } else {
            unreachable!("invalid rg option {:?}", value)
        }
    }

    fn pipe_to_rg(&self, code: &str) -> String {
        let echo_child = Command::new("echo")
            .arg(code)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let rg_child = Command::new("rg")
            .arg("--line-number")
            .arg("--column")
            .arg("--color=never")
            .arg("--engine=auto")
            .arg(&self.regex)
            .stdin(Stdio::from(echo_child.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut output = String::new();
        rg_child
            .stdout
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();

        println!("{}", output);
        output
    }
}

impl Program for Rg {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        Self {
            treesitter,
            search_paths,
            regex: Self::get_regex(cli.command.into()),
        }
    }

    fn run(&mut self) {
        for (code, _path) in iter_valid_files(&self.search_paths) {
            for block in self.treesitter.sql_blocks(&code) {
                let sql = block.inner_text(&code);
                self.pipe_to_rg(sql);
            }
        }
    }
}
