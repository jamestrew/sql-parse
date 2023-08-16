use std::fmt::Display;
use std::ops::Range;

use anyhow::anyhow;
use tree_sitter::{Parser, Point, Query, QueryCursor};
use tree_sitter_python::language as Python;

use crate::error_exit;

type SourceCode<'a> = &'a str;

#[derive(Debug, PartialEq, Clone, Copy)]
enum CaptureGroup {
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

impl Display for CaptureGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureGroup::Sql => f.write_str("sql"),
            CaptureGroup::StringStart => f.write_str("ss"),
            CaptureGroup::StringEnd => f.write_str("se"),
            CaptureGroup::Other => f.write_str("other"),
        }
    }
}

const MANDATORY_CAPTURE_GROUPS: &[CaptureGroup] = &[
    // CaptureGroup::Sql,
    CaptureGroup::StringStart,
    CaptureGroup::StringEnd,
];

fn check_capture_groups(capture_names: &[CaptureGroup]) -> Option<CaptureGroup> {
    MANDATORY_CAPTURE_GROUPS
        .iter()
        .find(|&&capture_group| !capture_names.contains(&capture_group))
        .copied()
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub byte_range: Range<usize>,
    pub point: Point,
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

pub struct Treesitter {
    parser: Parser,
    query: Query,
    capture_groups: Vec<CaptureGroup>,
}

impl Treesitter {
    pub fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock> {
        let tree = self
            .parser
            .parse(code, None)
            .unwrap_or_else(|| error_exit!("Tree-sitter failed to parse code:\n{}", code));

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        let mut sql_blocks = Vec::new();
        for m in matches {
            let mut string_start: Position = Default::default();
            let mut string_end: Position = Default::default();

            m.captures
                .iter()
                .map(|&cap| (cap, &self.capture_groups[cap.index as usize]))
                .filter(|(_cap, grp)| MANDATORY_CAPTURE_GROUPS.contains(grp))
                .for_each(|(cap, grp)| match grp {
                    CaptureGroup::StringStart => {
                        string_start.byte_range = cap.node.byte_range();
                        string_start.point = cap.node.start_position();
                    }
                    CaptureGroup::StringEnd => {
                        string_end.byte_range = cap.node.byte_range();
                        string_end.point = cap.node.start_position();
                    }
                    _ => {}
                });

            sql_blocks.push(SqlBlock {
                string_start,
                string_end,
            });
        }
        sql_blocks
    }
}

impl TryFrom<String> for Treesitter {
    type Error = anyhow::Error;

    fn try_from(query: String) -> Result<Self, Self::Error> {
        let mut parser = Parser::new();
        parser
            .set_language(Python())
            .map_err(|_| anyhow!("Failed to set up Python tree-sitter parser"))?;

        let query = Query::new(Python(), &query)
            .map_err(|_| anyhow!("Failed to create tree-sitter query. Verify query syntax."))?;

        let capture_names: Vec<CaptureGroup> = query
            .capture_names()
            .iter()
            .map(|name| CaptureGroup::from(name.as_str()))
            .collect();

        if let Some(missing) = check_capture_groups(&capture_names) {
            return Err(anyhow!(
                "tree-sitter query must contain '{}' capture group.",
                missing
            ));
        }

        Ok(Self {
            parser,
            query,
            capture_groups: capture_names,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const QUERY: &str = include_str!("../queries/execute.scm");

    fn get_ts(query: &str) -> Treesitter {
        Treesitter::try_from(query.to_string()).unwrap()
    }

    #[test]
    fn from_string() {
        let _ = Treesitter::try_from(QUERY.to_string()).unwrap();
    }

    #[test]
    fn matching_node_ranges() {
        let code = r#"
crs.execute('SELECT 1 FROM foo')
crs.execute(f'SELECT 2 FROM foo WHERE x = {x}')
crs.execute("SELECT 3 FROM bar")
crs.execute(f"SELECT 4 FROM foo WHERE x = {x}")
crs.execute(f"""
    SELECT 5 FROM foo
""")

crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
""")
"#;

        let expect = [
            "SELECT 1 FROM foo",
            "SELECT 2 FROM foo WHERE x = {x}",
            "SELECT 3 FROM bar",
            "SELECT 4 FROM foo WHERE x = {x}",
            "\n    SELECT 5 FROM foo\n",
            "\n    SELECT 6 FROM foo where x = {x} AND y = {y}\n",
        ];

        let mut ts = get_ts(QUERY);

        let blocks = ts.sql_blocks(code);

        assert_eq!(blocks.len(), expect.len());
        for (idx, &sql) in expect.iter().enumerate() {
            let snippet = &code[blocks[idx].inner_text_range()];
            assert_eq!(sql, snippet);
        }
    }
}
