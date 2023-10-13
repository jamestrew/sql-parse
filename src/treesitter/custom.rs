use std::path::PathBuf;

use tree_sitter::{Parser, Query, QueryCursor};

use super::*;
use crate::error_exit;

pub struct CustomQuery {
    parser: Parser,
    query: Query,
}

impl TreesitterQuery for CustomQuery {
    fn sql_blocks(&mut self, code: SourceCode) -> Vec<SqlBlock> {
        let tree = parser_tree(&mut self.parser, code);
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());
        let capture_groups = self.query.capture_names();

        let mut sql_blocks = Vec::new();
        for mtch in matches {
            let mut string_start = Position::default();
            let mut string_end = Position::default();

            for capture in mtch.captures {
                let node = capture.node;
                let grp = &capture_groups[capture.index as usize];
                let grp = CaptureGroup::from(grp.as_str());
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

impl From<&str> for CustomQuery {
    fn from(query: &str) -> Self {
        let parser = new_parser();
        let query = new_query(query);

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

impl From<&PathBuf> for CustomQuery {
    fn from(ts_path: &PathBuf) -> Self {
        let file = std::fs::read_to_string(ts_path).unwrap_or_else(|_| {
            error_exit!("Failed to read provided regex file: {}", ts_path.display())
        });
        Self::from(file.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SQL_STRING: &str = r#"
(string
    (string_start) @ss
    (string_content) @str (#match? @str "SELECT|FROM|WHERE|UPDATE|INSERT")
    (string_end) @se)"#;

    macro_rules! ss_display_stmt {
        ($name:tt, $code:expr) => {
            #[test]
            fn $name() {
                let mut ts = CustomQuery::from(SQL_STRING);
                let blocks = ts.sql_blocks($code);

                insta::with_settings!({
                    description => $code,
                }, {
                    for blk in &blocks {
                        let snippet = &$code[blk.inner_text_range()];
                        insta::assert_display_snapshot!(snippet);
                    }
                })
            }
        };
    }

    ss_display_stmt!(no_match, "'foo'");
    ss_display_stmt!(basic_match, "'SELECT'");

    ss_display_stmt!(basic_strings_1, "crs.execute('SELECT 1 FROM foo')");
    ss_display_stmt!(basic_strings_2, r#"crs.execute("SELECT 1 FROM foo")"#);
    ss_display_stmt!(basic_fstring_1, "crs.execute(f'SELECT 1 FROM {foo}')");
    ss_display_stmt!(basic_fstring_2, r#"crs.execute(f"SELECT 1 FROM {bar}")"#);

    ss_display_stmt!(
        multiline_f_string,
        r#"
    crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
    """)"#
    );

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

        let mut ts = CustomQuery::from(SQL_STRING);
        let blocks = ts.sql_blocks(code);

        assert_eq!(blocks.len(), expect.len());
        for (blk, &exp) in blocks.iter().zip(expect.iter()) {
            let snippet = &code[blk.inner_text_range()];
            assert_eq!(snippet, exp);
        }
    }
}
