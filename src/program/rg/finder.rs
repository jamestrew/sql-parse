use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use console::{pad_str, style, Term};
use regex::{self, Regex};
use textwrap::wrap;

use super::utils::*;
use crate::cli::RegexOptions;
use crate::treesitter::Treesitter;
use crate::utils::*;

pub enum FindChoice {
    Continue,
    Exit,
}

pub trait Finder {
    fn new_finder(rg_opts: &RegexOptions) -> Self
    where
        Self: Sized;

    fn find(&mut self, ts: &mut Treesitter, search_paths: Rc<Vec<PathBuf>>) {
        for (code, path) in iter_valid_files(&search_paths) {
            let fs = FileState {
                path: path.as_path().to_str().unwrap().to_string(),
                lines: block_lines(&code),
                code,
            };

            let _ = self.find_in_file(ts, fs);
        }
    }
    fn find_in_file(&mut self, ts: &mut Treesitter, file: FileState) -> FindChoice;
}

pub struct PlainSearch {
    re: Regex,
}

impl Finder for PlainSearch {
    fn new_finder(rg_opts: &RegexOptions) -> Self {
        Self {
            re: make_regex(rg_opts),
        }
    }

    fn find_in_file(&mut self, ts: &mut Treesitter, file: FileState) -> FindChoice {
        for block in ts.sql_blocks(&file.code) {
            let sql = block.inner_text(&file.code);
            self.re
                .find_iter(sql)
                .map(|m| MatchRange::from_regex_match(&block, &m, &file.lines, &file.code))
                .for_each(|rng| {
                    print(
                        &file.path,
                        rng.start_point.row + 1,
                        Some(rng.start_point.column + 1),
                        &file.code[rng.abs_line_range()],
                        Some(rng.line_match_range()),
                    )
                });
        }
        FindChoice::Continue
    }
}

pub struct InverseSearch {
    re: Regex,
}
impl Finder for InverseSearch {
    fn new_finder(rg_opts: &RegexOptions) -> Self {
        Self {
            re: make_regex(rg_opts),
        }
    }

    fn find_in_file(&mut self, ts: &mut Treesitter, file: FileState) -> FindChoice {
        for block in ts.sql_blocks(&file.code) {
            let sql = block.inner_text(&file.code);

            if !self.re.is_match(sql) {
                print(
                    &file.path,
                    block.start_line_num(),
                    None,
                    block.inner_text(&file.code),
                    None,
                );
            }
        }
        FindChoice::Continue
    }
}

pub struct Replace {
    re: Regex,
    replace_text: String,
}

impl Finder for Replace {
    fn new_finder(rg_opts: &RegexOptions) -> Self {
        Self {
            re: make_regex(rg_opts),
            replace_text: rg_opts.replace.as_ref().unwrap().to_owned(),
        }
    }

    fn find_in_file(&mut self, ts: &mut Treesitter, mut file: FileState) -> FindChoice {
        let mut change_count = 0;

        ts.sql_blocks(&file.code).iter().rev().for_each(|block| {
            let inner_text_range = block.inner_text_range();
            let sql = &file.code[inner_text_range.clone()];
            let new_inner_text = self.re.replace_all(sql, &self.replace_text).into_owned();
            file.code.replace_range(inner_text_range, &new_inner_text);
            change_count += 1;
        });

        if write_file(&file.path, file.code.as_bytes()).is_err() {
            eprintln!("Failed to write to path: {}", file.path);
        }
        println!("{change_count} changes made to {}", file.path);
        FindChoice::Continue
    }
}

#[derive(Debug, Clone, Copy)]
enum ConfirmAns {
    Yes,
    No,
    All,
    Quit,
    Help,
}

impl FromStr for ConfirmAns {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "y" | "yes" => Ok(ConfirmAns::Yes),
            "n" | "no" => Ok(ConfirmAns::No),
            "a" | "all" => Ok(ConfirmAns::All),
            "q" | "quit" => Ok(ConfirmAns::Quit),
            "h" | "help" => Ok(ConfirmAns::Help),
            _ => Err(()),
        }
    }
}

impl Display for ConfirmAns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmAns::Yes => write!(f, "[y]es"),
            ConfirmAns::No => write!(f, "[n]o"),
            ConfirmAns::All => write!(f, "[a]ll"),
            ConfirmAns::Quit => write!(f, "[q]uit"),
            ConfirmAns::Help => write!(f, "[h]elp"),
        }
    }
}

impl ConfirmAns {
    fn options() -> String {
        format!(
            "{}, {}, {}, {} - {}",
            ConfirmAns::Yes,
            ConfirmAns::No,
            ConfirmAns::All,
            ConfirmAns::Quit,
            ConfirmAns::Help,
        )
    }

    fn help() -> &'static str {
        r#"y    yes; make this change.
n    no; skip this match.
a    all; make this change and all remaining ones without further confirmation.
q    quit; don't make any more changes."#
    }
}

pub struct ReplaceConfirm {
    re: Regex,
    replace_text: String,
    last_ans: Option<ConfirmAns>,
}

impl ReplaceConfirm {
    fn get_answer(&mut self) -> ConfirmAns {
        if let Some(ans) = &self.last_ans {
            if matches!(ans, ConfirmAns::All) {
                return ConfirmAns::All;
            }
        }

        let mut ans_text = String::default();
        loop {
            ans_text.clear();
            print!("{}: ", ConfirmAns::options());
            io::stdout().lock().flush().unwrap();

            io::stdin()
                .read_line(&mut ans_text)
                .expect("failed to read input");

            if let Ok(ans) = ans_text.trim().parse::<ConfirmAns>() {
                if matches!(ans, ConfirmAns::Help) {
                    println!("{}", ConfirmAns::help());
                } else {
                    self.last_ans = Some(ans);
                    return ans;
                }
            } else {
                eprintln!(
                    "bad input {:?} - expected one of {}",
                    ans_text,
                    ConfirmAns::options()
                );
            }
        }
    }

    fn process_replacements(&mut self, replacements: Vec<Range<usize>>, mut file: FileState) {
        for rng in replacements.iter().rev() {
            let rng = rng.to_owned();
            let mut _match = &file.code[rng.clone()];
            let new = self.re.replace_all(_match, &self.replace_text).into_owned();
            file.code.replace_range(rng, &new);
        }
        if write_file(&file.path, file.code.as_bytes()).is_err() {
            eprintln!("Failed to write to path: {}", file.path);
        }
    }
}

impl Finder for ReplaceConfirm {
    fn new_finder(rg_opts: &RegexOptions) -> Self {
        Self {
            re: make_regex(rg_opts),
            replace_text: rg_opts.replace.as_ref().unwrap().to_owned(),
            last_ans: None,
        }
    }

    fn find(&mut self, ts: &mut Treesitter, search_paths: Rc<Vec<PathBuf>>) {
        for (code, path) in iter_valid_files(&search_paths) {
            let fs = FileState {
                path: path.as_path().to_str().unwrap().to_string(),
                lines: block_lines(&code),
                code,
            };

            if matches!(self.find_in_file(ts, fs), FindChoice::Exit) {
                return;
            }
        }
    }

    fn find_in_file(&mut self, ts: &mut Treesitter, file: FileState) -> FindChoice {
        let mut replacements = Vec::new();
        'outer: for block in ts.sql_blocks(&file.code) {
            let inner_text_range = block.inner_text_range();
            let sql = &file.code[inner_text_range.clone()];
            let matches: Vec<_> = self.re.find_iter(sql).collect();

            for m in matches {
                let rng = MatchRange::from_regex_match(&block, &m, &file.lines, &file.code);
                print(
                    &file.path,
                    rng.start_point.row + 1,
                    Some(rng.start_point.column + 1),
                    &file.code[rng.line_range],
                    Some(rng.line_match_range),
                );

                match self.get_answer() {
                    ConfirmAns::Yes | ConfirmAns::All => replacements.push(rng.abs_match_range),
                    ConfirmAns::No => {}
                    ConfirmAns::Quit => break 'outer,
                    ConfirmAns::Help => unreachable!("help not processed in find_in_file"),
                }
            }
        }

        self.process_replacements(replacements, file);
        if let Some(ans) = self.last_ans.as_ref() {
            if matches!(ans, ConfirmAns::Quit) {
                return FindChoice::Exit;
            }
        }
        FindChoice::Continue
    }
}
