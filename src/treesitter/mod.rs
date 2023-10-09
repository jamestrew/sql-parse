mod utils;

use std::path::PathBuf;

use tree_sitter::{Node, Parser, Query, QueryCursor};
pub use utils::SqlBlock;
use utils::*;

use crate::cli::Cli;
use crate::error_exit;

pub trait TreesitterQuery {
    fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock>;
}

pub fn ts_query_factory(cli: &Cli) -> Box<dyn TreesitterQuery> {
    match cli.tree_sitter() {
        Some(path) => Box::new(CustomQuery::from(path)),
        None => Box::new(Exec::new()),
    }
}

pub struct CustomQuery {
    parser: Parser,
    query: Query,
}

impl TreesitterQuery for CustomQuery {
    fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock> {
        let tree = parser_tree(&mut self.parser, code);
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        let mut sql_blocks = Vec::new();
        for mtch in matches {
            let mut string_start = Position::default();
            let mut string_end = Position::default();

            for capture in mtch.captures {
                let node = capture.node;
                let grp = CaptureGroup::from(node.kind());
                match grp {
                    CaptureGroup::StringStart => string_start = node.into(),
                    CaptureGroup::StringEnd => string_end = node.into(),
                    _ => {}
                }
            }

            sql_blocks.push(SqlBlock {
                string_start,
                string_end,
            });
        }

        sql_blocks
    }
}

impl From<&PathBuf> for CustomQuery {
    fn from(ts_path: &PathBuf) -> Self {
        let file = std::fs::read_to_string(ts_path).unwrap_or_else(|_| {
            error_exit!("Failed to read provided regex file: {}", ts_path.display())
        });

        let parser = new_parser();
        let query = new_query(&file);

        let capture_groups = query
            .capture_names()
            .iter()
            .map(|name| CaptureGroup::from(name.as_str()))
            .collect::<Vec<_>>();

        let ss_count = capture_groups
            .iter()
            .filter(|grp| matches!(grp, CaptureGroup::StringStart))
            .count();

        let se_count = capture_groups
            .iter()
            .filter(|grp| matches!(grp, CaptureGroup::StringEnd))
            .count();

        if ss_count != 1 || se_count != 1 {
            error_exit!(
                "tree-sitter query must contain exactly one of '@ss' and '@se' capture groups."
            )
        }

        Self { parser, query }
    }
}

pub struct Exec {
    parser: Parser,
    query: Query,
}

impl Exec {
    pub fn new() -> Self {
        let parser = new_parser();
        let query = new_query(include_str!("../../queries/execute.scm"));
        Self { parser, query }
    }

    fn basic_string_sql(&self, str_node: Node<'_>, sql_blocks: &mut Vec<SqlBlock>) {
        let mut tc = str_node.walk();

        let mut string_start: Position = Default::default();
        let mut string_end: Position = Default::default();
        for n in str_node.children(&mut tc) {
            match n.kind() {
                "string_start" => string_start = n.into(),
                "string_end" => string_end = n.into(),
                _ => (),
            }
        }

        sql_blocks.push(SqlBlock {
            string_start,
            string_end,
        });
    }

    fn format_string_sql(&self, call_node: Node<'_>, sql_blocks: &mut Vec<SqlBlock>) {
        if let Some(attr_node) = call_node.child_by_field_name("function") {
            if let Some(str_node) = attr_node.child_by_field_name("object") {
                self.basic_string_sql(str_node, sql_blocks);
            }
        }
    }
}

impl TreesitterQuery for Exec {
    fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock> {
        let tree = parser_tree(&mut self.parser, code);

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        let mut sql_blocks = Vec::new();
        for m in matches {
            'outer: for cap in m.captures {
                if cap.node.kind() != "argument_list" {
                    continue;
                }

                let mut tree_cursor = cap.node.walk();
                for arg_node in cap.node.children(&mut tree_cursor) {
                    match arg_node.kind() {
                        "string" => self.basic_string_sql(arg_node, &mut sql_blocks),
                        "call" => self.format_string_sql(arg_node, &mut sql_blocks),
                        _ => continue,
                    };
                    break 'outer;
                }
            }
        }
        sql_blocks.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exec_matching_node_ranges() {
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

crs.execute('SELECT 1 FROM foo', "foo")
crs.execute('SELECT 1 FROM {foo}'.format(foo="foo"))
"#;

        let expect = [
            "SELECT 1 FROM foo",
            "SELECT 2 FROM foo WHERE x = {x}",
            "SELECT 3 FROM bar",
            "SELECT 4 FROM foo WHERE x = {x}",
            "\n    SELECT 5 FROM foo\n",
            "\n    SELECT 6 FROM foo where x = {x} AND y = {y}\n",
            "SELECT 1 FROM foo",
            "SELECT 1 FROM {foo}",
        ];

        let mut ts = Exec::new();

        let blocks = ts.sql_blocks(code);

        assert_eq!(blocks.len(), expect.len());
        for (idx, &sql) in expect.iter().enumerate() {
            let snippet = &code[blocks[idx].inner_text_range()];
            assert_eq!(sql, snippet);
        }
    }
}
