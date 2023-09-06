use std::borrow::Cow;
use std::ops::Range;

use regex::{self, Match, Regex, RegexBuilder, Replacer};
use tree_sitter::Point;

use crate::cli::RegexOptions;
use crate::error_exit;
use crate::treesitter::SqlBlock;

pub fn make_regex(rg_opts: &RegexOptions) -> Regex {
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

#[derive(Debug, Default)]
pub struct FileState {
    pub path: String,
    pub lines: Vec<usize>,
    pub code: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct MatchRange {
    pub abs_match_range: Range<usize>,
    pub block_match_range: Range<usize>,
    pub start_point: Point,
    pub abs_line_range: Range<usize>,
    pub line_match_range: Range<usize>,
}

impl std::fmt::Debug for MatchRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MatchRange {{")?;
        writeln!(f, "  abs_match_range: {:?},", self.abs_match_range)?;
        writeln!(f, "  block_match_range: {:?},", self.block_match_range)?;
        writeln!(f, "  start_point: {:?},", self.start_point)?;
        writeln!(f, "  abs_line_range: {:?},", self.abs_line_range)?;
        writeln!(f, "  line_match_range: {:?},", self.line_match_range)?;
        writeln!(f, "}}")
    }
}

impl MatchRange {
    pub fn from_regex_match(
        ts_block: &SqlBlock,
        regex_match: &Match,
        lines: &[usize],
        code: &str,
    ) -> Self {
        let block_match_start = regex_match.start();
        let block_match_end = regex_match.end();

        let abs_start = ts_block.string_start.byte_range.end + block_match_start;
        let abs_end = ts_block.string_start.byte_range.end + block_match_end;

        let row = lines
            .iter()
            .rposition(|&line_byte| {
                line_byte <= block_match_start + ts_block.string_start.byte_range.end
            })
            .unwrap_or(0);

        let abs_line_start = lines[row];
        let match_last_line_idx = lines
            .iter()
            .rposition(|&line_byte| line_byte <= abs_end)
            .unwrap_or(0);
        let match_last_line_abs_pos = lines[match_last_line_idx];
        let abs_line_end = code[match_last_line_abs_pos..]
            .find('\n')
            .map(|line_end_pos| {
                if row != match_last_line_idx {
                    line_end_pos + match_last_line_abs_pos
                } else {
                    line_end_pos + abs_line_start
                }
            })
            .unwrap_or(code.len());
        let column = abs_start - abs_line_start;

        Self {
            abs_match_range: abs_start..abs_end,
            block_match_range: block_match_start..block_match_end,
            start_point: Point { row, column },
            abs_line_range: abs_line_start..abs_line_end,
            line_match_range: column..(column + block_match_end - block_match_start),
        }
    }

    pub fn shifted_ranged(&self, chars: isize) -> MatchRange {
        fn adjust_range(range: Range<usize>, chars: isize) -> Range<usize> {
            let shifted_start = (range.start as isize + chars) as usize;
            let shifted_end = (range.end as isize + chars) as usize;
            shifted_start..shifted_end
        }

        Self {
            abs_match_range: adjust_range(self.abs_match_range(), chars),
            block_match_range: adjust_range(self.block_match_range(), chars),
            start_point: self.start_point, // bad bad bad but w/e
            abs_line_range: adjust_range(self.abs_line_range(), chars),
            line_match_range: adjust_range(self.line_match_range(), chars),
        }
    }

    #[allow(unused)]
    pub fn match_length(&self) -> usize {
        self.line_match_range.len()
    }

    pub fn abs_match_range(&self) -> Range<usize> {
        self.abs_match_range.start..self.abs_match_range.end
    }

    pub fn block_match_range(&self) -> Range<usize> {
        self.block_match_range.start..self.block_match_range.end
    }

    pub fn abs_line_range(&self) -> Range<usize> {
        self.abs_line_range.start..self.abs_line_range.end
    }

    pub fn line_match_range(&self) -> Range<usize> {
        self.line_match_range.start..self.line_match_range.end
    }
}

pub struct CodeDiff<'a> {
    before: &'a str,
    diff: &'a str,
    after: &'a str,
}

impl<'a> CodeDiff<'a> {
    pub fn new_raw(before: &'a str, diff: &'a str, after: &'a str) -> Self {
        Self {
            before,
            diff,
            after,
        }
    }
    pub fn new_line(source_code: &'a str, rng: &'a MatchRange) -> Self {
        let line = &source_code[rng.abs_line_range()];
        Self {
            before: &line[..rng.line_match_range.start],
            diff: &line[rng.line_match_range()],
            after: &line[rng.line_match_range.end..],
        }
    }

    pub fn new_block(block: &'a str, rng: &'a MatchRange) -> Self {
        Self {
            before: &block[..rng.block_match_range.start],
            diff: &block[rng.block_match_range()],
            after: &block[rng.block_match_range.end..],
        }
    }

    pub fn with_diff_color(self, color: console::Color) -> String {
        format!(
            "{}{}{}",
            self.before,
            console::Style::new()
                .fg(color)
                .force_styling(true)
                .apply_to(self.diff),
            self.after
        )
    }
}

pub fn block_lines(code: &str) -> Vec<usize> {
    let mut lines = vec![0];
    code.chars()
        .enumerate()
        .filter(|(_, ch)| *ch == '\n')
        .for_each(|(idx, _)| lines.push(idx + 1));
    lines
}

pub fn replace_in_range<'h, R: Replacer>(
    regex: &Regex,
    haystack: &'h str,
    rng: Range<usize>,
    rep: R,
) -> Cow<'h, str> {
    if rng.start >= rng.end || rng.end > haystack.len() {
        return Cow::Borrowed(haystack);
    }

    let before = &haystack[0..rng.start];
    let target = &haystack[rng.start..rng.end];
    let after = &haystack[rng.end..];

    let replaced_target = regex.replace(target, rep);
    Cow::Owned(format!("{}{}{}", before, replaced_target, after))
}

pub fn replace_in_range_partitioned<'h, R: Replacer>(
    regex: &Regex,
    haystack: &'h str,
    rng: Range<usize>,
    rep: R,
) -> (Cow<'h, str>, Cow<'h, str>, Cow<'h, str>) {
    if rng.start >= rng.end || rng.end > haystack.len() {
        return (
            Cow::Borrowed(haystack),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
        );
    }

    let before = Cow::Borrowed(&haystack[0..rng.start]);
    let target = &haystack[rng.start..rng.end];
    let after = Cow::Borrowed(&haystack[rng.end..]);

    let replaced_target = regex.replace(target, rep);
    (before, replaced_target, after)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::treesitter::Treesitter;

    fn ts_block(input: &str) -> SqlBlock {
        let mut ts = Treesitter::try_from(None).unwrap();
        ts.sql_blocks(input).pop().unwrap()
    }

    fn get_first_rng(input: &str, re_str: &str) -> MatchRange {
        let mut ts = Treesitter::try_from(None).unwrap();
        let block = ts.sql_blocks(input).pop().unwrap();
        let sql = &input[block.inner_text_range()];
        let re = regex::Regex::new(re_str).unwrap();
        let m = re.find(sql).expect("not testing the lack of matches");
        let lines = block_lines(input);
        MatchRange::from_regex_match(&block, &m, &lines, input)
    }

    mod match_range {
        use super::*;

        macro_rules! assert_rng {
            ($actual:expr, $expected:expr, $sql:expr) => {
                if $actual != $expected {
                    let mut bad_text = String::new();
                    if $actual.abs_match_range != $expected.abs_match_range {
                        bad_text.push_str("abs_match_range: ");
                        bad_text.push_str(&$sql[$actual.abs_match_range()]);
                        bad_text.push_str("\n");
                    }
                    if $actual.abs_line_range != $expected.abs_line_range {
                        bad_text.push_str("line_range: ");
                        bad_text.push_str(&$sql[$actual.abs_line_range()]);
                        bad_text.push_str("\n");
                    }
                    panic!(
                        "assertion failed\nexpected: {:?}\ngot: {:?}\n{}\n",
                        $expected, $actual, bad_text,
                    )
                }
            };
        }

        #[test]
        fn one_liner() {
            let input = r#"crs.execute("SELECT 'yo'; SELECT 'hi'";)"#;
            let rng = get_first_rng(input, "SELECT");
            let expected = MatchRange {
                abs_match_range: 13..19,
                block_match_range: 0..6,
                start_point: Point { row: 0, column: 13 },
                abs_line_range: 0..40,
                line_match_range: 13..19,
            };
            assert_rng!(rng, expected, input);
            assert_eq!(rng.match_length(), 6);
        }

        #[test]
        fn multi_liner_first_line() {
            let input = r#"crs.execute("""SELECT 'yo';
SELECT 'hi'""";)"#;
            let expected = MatchRange {
                abs_match_range: 15..21,
                block_match_range: 0..6,
                start_point: Point { row: 0, column: 15 },
                abs_line_range: 0..27,
                line_match_range: 15..21,
            };
            let rng = get_first_rng(input, "SELECT");
            assert_rng!(rng, expected, input);
            assert_eq!(rng.match_length(), 6);
        }

        #[test]
        fn multi_liner_second_line() {
            let input = r#"crs.execute("""
SELECT 'yo';
SELECT 'hi'""";)"#;
            let rng = get_first_rng(input, "SELECT");
            let expected = MatchRange {
                abs_match_range: 16..22,
                block_match_range: 1..7,
                start_point: Point { row: 1, column: 0 },
                abs_line_range: 16..28,
                line_match_range: 0..6,
            };
            assert_rng!(rng, expected, input);
            assert_eq!(rng.match_length(), 6);
        }

        #[test]
        fn multi_line_pattern() {
            let input = r#"crs.execute("""foo
SELECT 'yo';
SELECT 'hi'""";)"#;
            let rng = get_first_rng(input, "foo\nSELECT");
            let expected = MatchRange {
                abs_match_range: 15..25,
                block_match_range: 0..10,
                start_point: Point { row: 0, column: 15 },
                abs_line_range: 0..31,
                line_match_range: 15..25,
            };
            assert_rng!(rng, expected, input);
            assert_eq!(rng.match_length(), 10);
        }

        #[test]
        fn multi_line_pattern_second_line() {
            let input = r#"crs.execute("""
foo
SELECT 'yo';
SELECT 'hi'""";)"#;
            let rng = get_first_rng(input, "foo\nSELECT");
            let expected = MatchRange {
                abs_match_range: 16..26,
                block_match_range: 1..11,
                start_point: Point { row: 1, column: 0 },
                abs_line_range: 16..32,
                line_match_range: 0..10,
            };
            assert_rng!(rng, expected, input);
            assert_eq!(rng.match_length(), 10);
        }
    }

    mod code_diff {
        use super::*;

        #[test]
        fn new_line() {
            let input = r#"crs.execute("SELECT 'yo'; SELECT 'hi';")"#;
            let rng = get_first_rng(input, "SELECT");
            let actual = CodeDiff::new_line(input, &rng).with_diff_color(console::Color::Green);
            let expect = "crs.execute(\"\u{1b}[32mSELECT\u{1b}[0m 'yo'; SELECT 'hi';\")";

            assert_eq!(actual, expect);
        }

        #[test]
        fn multi_line() {
            let input = r#"crs.execute("""foo
SELECT 'yo';
SELECT 'hi';""")"#;
            let rng = get_first_rng(input, "foo\nSELECT");
            let actual = CodeDiff::new_line(input, &rng).with_diff_color(console::Color::Green);
            assert_eq!(
                actual,
                "crs.execute(\"\"\"\u{1b}[32mfoo\nSELECT\u{1b}[0m 'yo';"
            );
        }

        #[test]
        fn new_block() {
            let input = r#"crs.execute("""
SELECT 'yo';
SELECT 'hi';""")"#;
            let block = ts_block(input);
            let rng = get_first_rng(input, "SELECT");

            let actual = CodeDiff::new_block(block.inner_text(input), &rng)
                .with_diff_color(console::Color::Green);
            let expect = "\n\u{1b}[32mSELECT\u{1b}[0m 'yo';\nSELECT 'hi';";
            assert_eq!(actual, expect);
        }
    }

    #[test]
    fn test_block_lines() {
        let input = "hello\nworld";
        let expected = vec![0, 6];
        assert_eq!(block_lines(input), expected);
    }

    #[test]
    fn test_replace_in_range_1() {
        let regex = Regex::new(r"\d+").unwrap();
        let text = "The numbers 1234 are here, and 5678 are there.";
        let range = 16..42;
        let result = replace_in_range(&regex, text, range, "****");
        assert_eq!(result, "The numbers 1234 are here, and **** are there.");
    }

    #[test]
    fn test_replace_in_range_2() {
        let regex = Regex::new(r"\d+").unwrap();
        let text = "The numbers 1234 are here, and 5678 are there.";
        let range = 16..42;
        let result = replace_in_range(&regex, text, range, "<REDACTED>");
        assert_eq!(
            result,
            "The numbers 1234 are here, and <REDACTED> are there."
        );
    }

    #[test]
    fn test_replace_in_range_partitioned_1() {
        let regex = Regex::new(r"\d+").unwrap();
        let text = "The numbers 1234 are here, and 5678 are there.";
        let range = 16..42;
        let result = replace_in_range_partitioned(&regex, text, range, "****");
        assert_eq!(result.0, "The numbers 1234");
        assert_eq!(result.1, " are here, and **** are th");
        assert_eq!(result.2, "ere.");
    }

    #[test]
    fn test_replace_in_range_partitioned_2() {
        let regex = Regex::new(r"\d+").unwrap();
        let text = "The numbers 1234 are here, and 5678 are there.";
        let range = 16..42;
        let result = replace_in_range_partitioned(&regex, text, range, "<REDACTED>");
        assert_eq!(result.0, "The numbers 1234");
        assert_eq!(result.1, " are here, and <REDACTED> are th");
        assert_eq!(result.2, "ere.");
    }
}
