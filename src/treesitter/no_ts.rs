use tree_sitter::Point;

use super::{Position, SourceCode, SqlBlock, TreesitterQuery};

pub struct NoTS {}

impl NoTS {
    pub fn new() -> Self {
        Self {}
    }

    fn zero_or_minus_one(value: usize) -> usize {
        if value > 0 {
            value - 1
        } else {
            0
        }
    }

    fn last_pos(code: SourceCode) -> Position {
        let last_byte = code.as_bytes().len();

        let lines = code.split('\n').collect::<Vec<&str>>();
        let row = Self::zero_or_minus_one(lines.len());
        let column = lines
            .last()
            .map_or(0, |line| Self::zero_or_minus_one(line.chars().count()));

        Position {
            byte_range: last_byte..last_byte,
            point: Point { row, column },
        }
    }

    fn first_pos() -> Position {
        Position {
            byte_range: 0..0,
            point: Point { row: 0, column: 0 },
        }
    }
}

impl TreesitterQuery for NoTS {
    fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock> {
        let string_start = Self::first_pos();
        let string_end = Self::last_pos(code);
        vec![SqlBlock {
            string_start,
            string_end,
        }]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_inner_text {
        ($name:tt, $code:expr) => {
            #[test]
            fn $name() {
                let blocks = NoTS::new().sql_blocks($code);

                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].inner_text($code), $code);
            }
        };
    }

    assert_inner_text!(blank_file, "");
    assert_inner_text!(one_char, "a");
    assert_inner_text!(one_liner, "foobar");
    assert_inner_text!(one_ish_liner, "foobar\n");
    assert_inner_text!(multi_liner, "foo\nbar\nbaz");
}
