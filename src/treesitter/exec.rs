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

    fn weird_concat_str(&self, cat_node: Node<'_>, sql_blocks: &mut Vec<SqlBlock>) {
        let mut tc = cat_node.walk();

        for node in cat_node.children(&mut tc) {
            if node.kind() == "string" {
                self.basic_string_sql(node, sql_blocks);
            }
        }
    }

    fn binary_operator_str(&self, infix_node: Node<'_>, sql_blocks: &mut Vec<SqlBlock>) {
        let mut tc = infix_node.walk();

        for node in infix_node.children(&mut tc) {
            match node.kind() {
                "string" => self.basic_string_sql(node, sql_blocks),
                "binary_operator" => self.binary_operator_str(node, sql_blocks),
                _ => (),
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
                        "concatenated_string" => self.weird_concat_str(arg_node, &mut sql_blocks),
                        "binary_operator" => self.binary_operator_str(arg_node, &mut sql_blocks),
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
    fn multi_match_file() {
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
crs.execute('foo ' 'bar')
crs.execute('eggs' + 'spam')
crs.execute('green' + 'eggs' + 'spam')
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
            "foo ",
            "bar",
            "eggs",
            "spam",
            "green",
            "eggs",
            "spam",
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

/*

(binary_operator) ; [4:13 - 25]
left: (string) ; [4:13 - 17]
 (string_start) ; [4:13 - 13]
 (string_content) ; [4:14 - 16]
 (string_end) ; [4:17 - 17]
right: (string) ; [4:21 - 25]
 (string_start) ; [4:21 - 21]
 (string_content) ; [4:22 - 24]
 (string_end) ; [4:25 - 25]



(binary_operator) ; [5:13 - 31]
left: (binary_operator) ; [5:13 - 23]
 left: (string) ; [5:13 - 17]
  (string_start) ; [5:13 - 13]
  (string_content) ; [5:14 - 16]
  (string_end) ; [5:17 - 17]
 right: (string) ; [5:21 - 23]
  (string_start) ; [5:21 - 21]
  (string_content) ; [5:22 - 22]
  (string_end) ; [5:23 - 23]
right: (string) ; [5:27 - 31]
 (string_start) ; [5:27 - 27]
 (string_content) ; [5:28 - 30]
 (string_end) ; [5:31 - 31]

*/
