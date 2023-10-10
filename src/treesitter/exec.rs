use tree_sitter::{Node, Parser, Query, QueryCursor};

use super::*;

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
        for node in str_node.children(&mut tc) {
            match node.kind() {
                "string_start" => string_start = node.into(),
                "string_end" => string_end = node.into(),
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
