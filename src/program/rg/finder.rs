use regex::Regex;

use super::{FileState, MatchRange};
use crate::{cli::RegexOptions, treesitter::Treesitter, utils::*};

pub trait Finder {
    fn new_finder(rg_opts: &RegexOptions) -> Self
    where
        Self: Sized;

    fn find(&mut self, ts: &mut Treesitter, file: &mut FileState, re: &Regex);
}

pub struct PlainSearch {}

impl Finder for PlainSearch {
    fn new_finder(_rg_opts: &RegexOptions) -> Self {
        Self {}
    }

    fn find(&mut self, ts: &mut Treesitter, file: &mut FileState, re: &Regex) {
        for block in ts.sql_blocks(&file.code) {
            let sql = block.inner_text(&file.code);
            re.find_iter(sql)
                .map(|m| MatchRange::from_regex_match(&block, &m, &file.lines, &file.code))
                .for_each(|rng| {
                    print(
                        &file.path,
                        rng.start_point.row + 1,
                        Some(rng.start_point.column + 1),
                        &file.code[rng.line_range],
                        Some(rng.line_match_range),
                    )
                });
        }
    }
}

pub struct InverseSearch {}
impl Finder for InverseSearch {
    fn new_finder(_rg_opts: &RegexOptions) -> Self {
        Self {}
    }

    fn find(&mut self, ts: &mut Treesitter, file: &mut FileState, re: &Regex) {
        for block in ts.sql_blocks(&file.code) {
            let sql = block.inner_text(&file.code);

            if !re.is_match(sql) {
                print(
                    &file.path,
                    block.start_line_num(),
                    None,
                    block.inner_text(&file.code),
                    None,
                );
            }
        }
    }
}

pub struct Replace {
    replace_text: String,
}
impl Finder for Replace {
    fn new_finder(rg_opts: &RegexOptions) -> Self {
        Self {
            replace_text: rg_opts.replace.as_ref().unwrap().to_owned(),
        }
    }

    fn find(&mut self, ts: &mut Treesitter, file: &mut FileState, re: &Regex) {
        let mut change_count = 0;

        ts.sql_blocks(&file.code).iter().rev().for_each(|block| {
            let inner_text_range = block.inner_text_range();
            let sql = &file.code[inner_text_range.clone()];
            let new_inner_text = re.replace_all(sql, &self.replace_text).into_owned();
            file.code.replace_range(inner_text_range, &new_inner_text);
            change_count += 1;
        });

        if write_file(&file.path, file.code.as_bytes()).is_err() {
            eprintln!("Failed to write to path: {}", file.path);
        }
        println!("{change_count} changes made to {}", file.path);
    }
}
