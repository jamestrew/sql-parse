use std::path::PathBuf;

use super::Program;
use crate::{
    cli::Cli,
    treesitter::{SqlBlock, Treesitter as TS},
    utils::*,
};

const TRIPLE_QUOTES: &str = "\"\"\"";

pub(crate) struct Quotes {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
}

impl Program for Quotes {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(&cli);
        Self {
            treesitter,
            search_paths,
        }
    }

    fn run(&mut self) {
        for (mut code, path) in iter_valid_files(&self.search_paths) {
            let mut change_count = 0;
            for block in self.treesitter.sql_blocks(&code).iter().rev() {
                if replace_quotes(&mut code, block) {
                    change_count += 1;
                }
            }

            if write_file(path, code.as_bytes()).is_err() {
                eprintln!("Failed to write to path: {}", path.display());
            }
            println!("{change_count} changes made to {}", path.display());
        }
    }
}

fn replace_quotes(code: &mut String, block: &SqlBlock) -> bool {
    let mut start = block.string_start.byte_range.clone();
    let end = block.string_end.byte_range.clone();

    if is_f_string(&code[start.start..start.end]) {
        start = start.start + 1..start.end;
    }

    if &code[start.start..start.end] == TRIPLE_QUOTES {
        return false;
    }

    code.replace_range(end, TRIPLE_QUOTES);
    code.replace_range(start, TRIPLE_QUOTES);
    true
}

fn is_f_string(code: &str) -> bool {
    if let Some(first_letter) = code.chars().next() {
        first_letter == 'f'
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::treesitter::Treesitter;

    const QUERY: &str = include_str!("../../queries/execute.scm");
    fn get_ts(query: &str) -> Treesitter {
        Treesitter::try_from(query.to_string()).unwrap()
    }

    #[test]
    fn single_quotes_replace() {
        let mut code = String::from(r"crs.execute('SELECT 1 FROM foo')");
        let expect = r#"crs.execute("""SELECT 1 FROM foo""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        let block = &blocks[0];
        assert_eq!(blocks.len(), 1);
        replace_quotes(&mut code, block);

        assert_eq!(code, expect);
    }

    #[test]
    fn single_quote_f_replace() {
        let mut code = String::from(r"crs.execute(f'SELECT 1 FROM foo where x = {x}')");
        let expect = r#"crs.execute(f"""SELECT 1 FROM foo where x = {x}""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        let block = &blocks[0];
        assert_eq!(blocks.len(), 1);
        replace_quotes(&mut code, block);

        assert_eq!(code, expect);
    }

    #[test]
    fn triple_quotes_replace() {
        let mut code = String::from(r#"crs.execute("""SELECT 1 FROM foo""")"#);
        let expect = r#"crs.execute("""SELECT 1 FROM foo""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        let block = &blocks[0];
        assert_eq!(blocks.len(), 1);
        replace_quotes(&mut code, block);

        assert_eq!(code, expect);
    }

    #[test]
    fn triple_quotes_f_replace() {
        let mut code = String::from(r#"crs.execute(f"""SELECT 1 FROM foo where x = {x}""")"#);
        let expect = r#"crs.execute(f"""SELECT 1 FROM foo where x = {x}""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        let block = &blocks[0];
        assert_eq!(blocks.len(), 1);
        replace_quotes(&mut code, block);

        assert_eq!(code, expect);
    }

    #[test]
    fn multi_line_replace() {
        let mut code = String::from(
            r#"
crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
""")"#,
        );

        let expect = r#"
crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        let block = &blocks[0];
        assert_eq!(blocks.len(), 1);
        replace_quotes(&mut code, block);

        assert_eq!(code, expect);
    }

    #[test]
    fn multiple_blocks() {
        let mut code = String::from(
            r#"
crs.execute('SELECT 1 FROM foo')
crs.execute(f'SELECT 2 FROM foo WHERE x = {x}')
crs.execute(f"""
    SELECT 5 FROM foo
""")

crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
""")"#,
        );

        let expect = r#"
crs.execute("""SELECT 1 FROM foo""")
crs.execute(f"""SELECT 2 FROM foo WHERE x = {x}""")
crs.execute(f"""
    SELECT 5 FROM foo
""")

crs.execute(f"""
    SELECT 6 FROM foo where x = {x} AND y = {y}
""")"#;

        let mut ts = get_ts(QUERY);
        let blocks = ts.sql_blocks(&code);
        for block in blocks.iter().rev() {
            replace_quotes(&mut code, block);
        }

        assert_eq!(code, expect);
    }
}
