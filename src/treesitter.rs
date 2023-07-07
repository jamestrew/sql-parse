use anyhow::anyhow;
use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_python::language as Python;

use crate::error_exit;

type SourceCode<'a> = &'a str;

const SQL_CAPTURE: &str = "sql";

pub struct Treesitter {
    parser: Parser,
    query: Query,
    capture_names: Vec<String>,
}

impl Treesitter {
    pub fn matching_node(&mut self, code: SourceCode) -> () {
        let tree = self
            .parser
            .parse(code, None)
            .unwrap_or_else(|| error_exit!("Tree-sitter failed to parse code:\n{}", code));

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                if self.capture_names[capture.index as usize] == SQL_CAPTURE {
                    todo!()
                }
            }
        }
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
        let capture_names: Vec<String> = query.capture_names().iter().cloned().collect();

        if !capture_names.contains(&SQL_CAPTURE.to_owned()) {
            return Err(anyhow!(
                "tree-sitter query must contain 'sql' capture group."
            ));
        }

        Ok(Self {
            parser,
            query,
            capture_names,
        })
    }
}
