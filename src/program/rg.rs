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
        let rg_opts: Ripgrep = cli.command.into();
        Self {
            treesitter,
            search_paths,
            regex: Self::get_regex(rg_opts.regexp),
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

fn rg_args(opts: &Ripgrep) -> Vec<String> {
    let mut args = Vec::with_capacity(4);
    if opts.ignore_case {
        args.push(String::from("-i"));
    }
    if opts.invert_matching {
        args.push(String::from("-v"))
    }
    if opts.multiline {
        args.push(String::from("-U"))
    }
    if let Some(replacement) = &opts.replace {
        args.push(format!("-r={}", replacement).to_string());
    }

    args
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_rg(i: bool, v: bool, u: bool, r: Option<String>) -> Ripgrep {
        Ripgrep {
            treesitter_query: PathBuf::from("foo.md"),
            regexp: RegexpOption {
                regexp: Some(String::from("hello")),
                regexp_file: None,
            },
            search_paths: Vec::new(),
            ignore_case: i,
            invert_matching: v,
            multiline: u,
            replace: r,
        }
    }

    #[test]
    fn default_rg_args() {
        let opts = make_rg(false, false, false, None);
        let opts = rg_args(&opts);
        assert_eq!(opts.len(), 0);
    }

    #[test]
    fn rg_args_all_on() {
        let opts = make_rg(true, true, true, Some(String::from("potato")));
        let opts = rg_args(&opts);
        assert_eq!(opts.len(), 4);

        assert_eq!(opts, vec!["-i", "-v", "-U", "-r=potato"])
    }
}
