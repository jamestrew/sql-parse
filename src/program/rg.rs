use std::{ops::Range, path::PathBuf};

use regex::{Match, Regex, RegexBuilder};
use tree_sitter::Point;

use super::Program;
use crate::{
    cli::{Cli, RegexpOptions},
    error_exit,
    treesitter::{SqlBlock, Treesitter},
    utils::*,
};

pub(super) struct Rg {
    treesitter: Treesitter,
    search_paths: Vec<PathBuf>,
    re: Regex,
    invert_match: bool,
    replace_text: Option<String>,
}

impl Rg {
    fn make_regex(rg_opts: &RegexpOptions) -> Regex {
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
            .unwrap_or_else(|err| error_exit!("Failed to build regex:\n{}", err))
    }
}

impl Program for Rg {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        let rg_opts: RegexpOptions = cli.command.into();

        Self {
            treesitter,
            search_paths,
            re: Rg::make_regex(&rg_opts),
            invert_match: rg_opts.invert_match,
            replace_text: rg_opts.replace,
        }
    }

    fn run(&mut self) {
        for (code, _path) in iter_valid_files(&self.search_paths) {
            for block in self.treesitter.sql_blocks(&code) {
                let sql = block.inner_text(&code);
                if self.invert_match {
                    todo!("handle invert match")
                }
                if self.replace_text.is_some() {
                    todo!("handle replace text")
                }

                let lines = block_lines(&sql);
                self.re
                    .find_iter(&sql)
                    .map(|m| MatchRange::from_regex_match(&block, &m, &lines))
                    .for_each(|mr| {
                        println!(
                            "{}:{}:{}",
                            mr.start_point.row + 1,
                            mr.start_point.column + 1,
                            &code[mr.line_range]
                        )
                    });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct MatchRange {
    match_range: Range<usize>,
    start_point: Point,
    line_range: Range<usize>,
}

impl MatchRange {
    fn from_regex_match(ts_block: &SqlBlock, regex_match: &Match, lines: &[usize]) -> Self {
        let start_byte = ts_block.string_start.byte_range.end + regex_match.start();
        let end_byte = ts_block.string_start.byte_range.end + regex_match.end();

        let row = lines
            .iter()
            .rposition(|&line_byte| line_byte < regex_match.start())
            .unwrap_or(0);

        let column = if row == 0 {
            ts_block.inner_text_start_column()
        } else {
            regex_match.start() - lines[row]
        };

        let line_start = lines[row] + ts_block.string_start.byte_range.end;
        let line_end = lines
            .get(row + 1)
            .map(|&x| ts_block.string_start.byte_range.end + x - 1)
            .unwrap_or(ts_block.string_end.byte_range.start);

        Self {
            match_range: start_byte..end_byte,
            start_point: Point {
                row: row + ts_block.string_start.point.row,
                column,
            },
            line_range: line_start..line_end,
        }
    }
}

fn block_lines(code: &str) -> Vec<usize> {
    let mut lines = vec![0];
    code.as_bytes()
        .iter()
        .enumerate()
        .filter(|(_, &ch)| ch == b'\n')
        .for_each(|(idx, _)| lines.push(idx + 1));
    lines
}

#[cfg(test)]
mod test {
    use crate::program::rg::block_lines;

    #[test]
    fn test_block_lines() {
        let input = "hello\nworld";
        let expected = vec![0, 6];
        assert_eq!(block_lines(&input), expected);
    }
}
