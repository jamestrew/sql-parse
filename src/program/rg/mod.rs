mod finder;

use std::{ops::Range, path::PathBuf, rc::Rc};

use finder::*;
use regex::{Match, Regex, RegexBuilder};
use tree_sitter::Point;

use super::Program;
use crate::{
    cli::{Cli, RegexOptions},
    error_exit,
    treesitter::{SqlBlock, Treesitter},
    utils::*,
};

#[derive(Debug, Default)]
pub struct FileState {
    path: String,
    lines: Vec<usize>,
    code: String,
}

pub struct Rg {
    treesitter: Treesitter,
    search_paths: Rc<Vec<PathBuf>>,
    re: Regex,
    file: FileState,
    finder: Box<dyn Finder>,
}

fn make_regex(rg_opts: &RegexOptions) -> Regex {
    let mut regex = if let Some(pattern) = rg_opts.regex.regex.clone() {
        RegexBuilder::new(&pattern)
    } else if let Some(file_path) = rg_opts.regex.regex_file.clone() {
        let pattern = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| error_exit!("Failed to read provided regex file: {:?}", file_path));
        RegexBuilder::new(&pattern)
    } else {
        unreachable!("invalid rg option {:?}", rg_opts.regex)
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

impl Program for Rg {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        let rg_opts: RegexOptions = cli.command.into();

        let re_pattern = make_regex(&rg_opts);
        let finder: Box<dyn Finder> = match (rg_opts.invert_match, rg_opts.replace.as_ref()) {
            (true, _) => Box::new(InverseSearch::new_finder(&rg_opts)),
            (false, Some(_)) => Box::new(Replace::new_finder(&rg_opts)),
            (false, None) => Box::new(PlainSearch::new_finder(&rg_opts)),
        };

        Self {
            treesitter,
            search_paths: Rc::new(search_paths),
            re: re_pattern,
            file: FileState::default(),
            finder,
        }
    }

    fn run(&mut self) {
        for (code, path) in iter_valid_files(&self.search_paths.clone()) {
            self.file.lines = block_lines(&code);
            self.file.path = path.as_path().to_str().unwrap().to_string();
            self.file.code = code;

            self.finder
                .find(&mut self.treesitter, &mut self.file, &self.re);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MatchRange {
    abs_match_range: Range<usize>,
    start_point: Point,
    line_range: Range<usize>,
    line_match_range: Range<usize>,
}

impl MatchRange {
    pub fn from_regex_match(
        ts_block: &SqlBlock,
        regex_match: &Match,
        lines: &[usize],
        code: &str,
    ) -> Self {
        let start_byte = ts_block.string_start.byte_range.end + regex_match.start();
        let end_byte = ts_block.string_start.byte_range.end + regex_match.end();

        let row = lines
            .iter()
            .rposition(|&line_byte| {
                line_byte <= regex_match.start() + ts_block.string_start.byte_range.end
            })
            .unwrap_or(0);

        let line_start = lines[row];
        let line_end = code[start_byte..]
            .as_bytes()
            .iter()
            .position(|&byte| byte == b'\n')
            .unwrap_or(0)
            + start_byte;
        let column = start_byte - line_start;

        Self {
            abs_match_range: start_byte..end_byte,
            start_point: Point { row, column },
            line_range: line_start..line_end,
            line_match_range: column..(column + regex_match.end() - regex_match.start()),
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
        assert_eq!(block_lines(input), expected);
    }
}
