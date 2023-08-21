use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use console::{self, pad_str, style, Term};
use regex::{self, Regex};
use textwrap::wrap;

use super::utils::*;
use crate::cli::RegexOptions;
use crate::treesitter::Treesitter;
use crate::{utils::*, error_exit};

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
                    let line = CodeDiff::new_line(&file.code, &rng)
                        .with_diff_color(console::Color::Green, true);
                    print(
                        &file.path,
                        rng.start_point.row + 1,
                        Some(rng.start_point.column + 1),
                        &line,
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

            // maybe check line by line?
            if !self.re.is_match(sql) {
                print(
                    &file.path,
                    block.start_line_num(),
                    None,
                    block.inner_text(&file.code),
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
}

impl FromStr for ConfirmAns {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "y" | "yes" => Ok(ConfirmAns::Yes),
            "n" | "no" => Ok(ConfirmAns::No),
            "a" | "all" => Ok(ConfirmAns::All),
            "q" | "quit" => Ok(ConfirmAns::Quit),
            _ => Err(anyhow::anyhow!("failed to parse confirmation input")),
        }
    }
}

pub struct ReplaceConfirm {
    re: Regex,
    replace_text: String,
    last_ans: Option<ConfirmAns>,
    term: Term,
}

impl ReplaceConfirm {
    fn left_side_diff(&self, sql_code: &str, rng: &MatchRange) -> String {
        CodeDiff::new_block(sql_code, rng).with_diff_color(console::Color::Red, false)
    }

    fn right_side_diff(&self, sql_code: &str, rng: &MatchRange) -> String {
        let (before, diff, after) = replace_in_range_partitioned(
            &self.re,
            sql_code,
            rng.block_match_range(),
            &self.replace_text,
        );
        CodeDiff::new_raw(&before, &diff, &after).with_diff_color(console::Color::Red, false)
    }

    fn print_confirm(
        &mut self,
        path: &str,
        sql_code: &str,
        rng: &MatchRange,
    ) -> anyhow::Result<ConfirmAns> {
        self.term.clear_screen()?;

        println!(
            "{}:{}:{}",
            style(path).magenta(),
            style((rng.start_point.row + 1).to_string()).green(),
            rng.start_point.column + 1,
        );

        self.print_sep("SQL BLOCK");

        // let left = format!(
        //     "{}{}{}",
        //     &sql_code[..rng.block_match_range.start],
        //     style(&sql_code[rng.block_match_range()]).red(),
        //     &sql_code[rng.block_match_range.end..]
        // );
        // let match_line = &sql_code[rng.block_match_range()];
        // let replaced = self
        //     .re
        //     .replace_all(match_line, &self.replace_text)
        //     .into_owned();
        // let right = format!(
        //     "{}{}{}",
        //     &sql_code[..rng.block_match_range.start],
        //     style(&replaced).green(),
        //     &sql_code[rng.block_match_range.end..]
        // );
        let left = self.left_side_diff(sql_code, rng);
        let right = self.right_side_diff(sql_code, rng);

        let max_length = (self.term.size().1 as usize / 2) - 2;

        let lines_left = wrap(&left, max_length)
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let lines_right = wrap(&right, max_length)
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let max_lines = std::cmp::max(lines_left.len(), lines_right.len());

        for i in 0..max_lines {
            let default = String::from("");
            let left_line = lines_left.get(i).unwrap_or(&default);
            let right_line = lines_right.get(i).unwrap_or(&default);
            let left_line = pad_str(left_line, max_length, console::Alignment::Left, None);
            let right_line = pad_str(right_line, max_length, console::Alignment::Left, None);
            println!("{} │ {}", left_line, right_line);
        }

        self.print_sep("");
        println!(
            "
  {} yes; make this change.
  {} no; skip this match.
  {} all; make this change and all remaining ones without further confirmation.
  {} quit; don't make any more changes.",
            style("y").green(),
            style("n").red(),
            style("a").yellow(),
            style("q").cyan()
        );

        self.get_answer()
    }

    fn print_sep(&self, title: &str) {
        let title = if title.is_empty() {
            "".into()
        } else {
            format!(" {} ", title)
        };
        println!(
            "{title:━^width$}",
            title = style(title).bold(),
            width = self.term.size().1 as usize
        );
    }

    fn get_answer(&mut self) -> anyhow::Result<ConfirmAns> {
        if let Some(ans) = &self.last_ans {
            if matches!(ans, ConfirmAns::All) {
                return Ok(ConfirmAns::All);
            }
        }

        loop {
            let ans = self.term.read_char()?.to_string().parse::<ConfirmAns>();
            if let Ok(ans) = ans {
                return Ok(ans);
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
        error_exit!("--confirm still under construction");
        let term = Term::stdout();
        term.hide_cursor().unwrap();

        Self {
            re: make_regex(rg_opts),
            replace_text: rg_opts.replace.as_ref().unwrap().to_owned(),
            last_ans: None,
            term,
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
        self.term.show_cursor().unwrap();
    }

    fn find_in_file(&mut self, ts: &mut Treesitter, file: FileState) -> FindChoice {
        let mut replacements = Vec::new();
        'outer: for block in ts.sql_blocks(&file.code) {
            let inner_text_range = block.inner_text_range();
            let sql = &file.code[inner_text_range.clone()];
            let mut display_sql = sql.to_string();
            #[allow(unused)]
            let mut display_rng = MatchRange::default();
            let mut shift = 0;

            let matches: Vec<_> = self.re.find_iter(sql).collect();
            for mtch in matches {
                let rng = MatchRange::from_regex_match(&block, &mtch, &file.lines, &file.code);
                display_rng = rng.shifted_ranged(shift);

                if matches!(self.last_ans, Some(ConfirmAns::All)) {
                    replacements.push(rng.abs_match_range);
                    continue;
                }

                // TODO: need to send both display_sql/rng AND actual sql/rng
                // for right and left side respectively
                let ans = self
                    .print_confirm(&file.path, &display_sql, &display_rng)
                    .expect("failed to print confirmation message");

                match ans {
                    ConfirmAns::Yes | ConfirmAns::All => {
                        replacements.push(rng.abs_match_range());
                        let pre_replace_len = display_sql.len();
                        display_sql = replace_in_range(
                            &self.re,
                            &display_sql,
                            rng.block_match_range(),
                            &self.replace_text,
                        )
                        .to_string();
                        shift = (display_sql.len() as i32 - pre_replace_len as i32).abs() as usize;
                    }
                    ConfirmAns::No => {}
                    ConfirmAns::Quit => break 'outer,
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
