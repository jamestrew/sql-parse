use std::ops::Range;

use tree_sitter::{Node, Parser, Point, Query, Tree};
use tree_sitter_python::language as Python;

use crate::error_exit;

pub type SourceCode<'a> = &'a str;

pub fn new_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(Python())
        .unwrap_or_else(|_| error_exit!("Failed to set up Python tree-sitter parser."));
    parser
}

pub fn new_query(query_str: &str) -> Query {
    Query::new(Python(), query_str)
        .unwrap_or_else(|_| error_exit!("Failed to create tree-sitter query. Verify query syntax."))
}

pub fn parser_tree(parser: &mut Parser, code: SourceCode) -> Tree {
    parser
        .parse(code, None)
        .unwrap_or_else(|| error_exit!("Tree-sitter failed to parse code:\n{}", code))
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CaptureGroup {
    Sql,
    StringStart,
    StringEnd,
    Other,
}

impl From<&str> for CaptureGroup {
    fn from(value: &str) -> Self {
        match value {
            "sql" => Self::Sql,
            "ss" => Self::StringStart,
            "se" => Self::StringEnd,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub byte_range: Range<usize>,
    pub point: Point,
}

impl From<Node<'_>> for Position {
    fn from(node: Node<'_>) -> Self {
        Self {
            byte_range: node.byte_range(),
            point: node.start_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqlBlock {
    pub string_start: Position,
    pub string_end: Position,
}

impl SqlBlock {
    pub fn inner_text_range(&self) -> Range<usize> {
        self.string_start.byte_range.end..self.string_end.byte_range.start
    }

    pub fn inner_text<'a>(&'a self, code: &'a str) -> &'a str {
        &code[self.inner_text_range()]
    }

    pub fn start_line_num(&self) -> usize {
        self.string_start.point.row + 1
    }
}
