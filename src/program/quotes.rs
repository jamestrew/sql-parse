use std::path::PathBuf;

use crate::cli::Cli;
use crate::treesitter::{SqlBlock, Treesitter as TS};
use crate::utils::*;

use super::Program;

const TRIPLE_QUOTES: &str = r#"""""#;

pub(crate) struct Quotes {
    treesitter: TS,
    search_paths: Vec<PathBuf>,
}

impl Program for Quotes {
    fn new(cli: Cli) -> Self {
        let (treesitter, search_paths) = basic_cli_options(cli);
        Self {
            treesitter,
            search_paths,
        }
    }

    fn run(&mut self) {
        for (mut code, path) in iter_valid_files(&self.search_paths) {
            for block in self.treesitter.sql_blocks(&code).iter().rev() {
                replace_quotes(&mut code, block);
            }

            if let Err(_) = write_file(path, code.as_bytes()) {
                eprintln!("Failed to write to path: {}", path.display());
            }
        }
    }
}

fn replace_quotes(code: &mut String, block: &SqlBlock) {
    let mut start = block.string_start.byte_range.clone();
    let end = block.string_end.byte_range.clone();

    if is_f_string(&code[start.start..start.end]) {
        start = start.start + 1..start.end;
    }

    code.replace_range(end, TRIPLE_QUOTES);
    code.replace_range(start, TRIPLE_QUOTES);
}

fn is_f_string(code: &str) -> bool {
    if let Some(first_letter) = code.chars().next() {
        first_letter == 'f'
    } else {
        false
    }
}

fn write_file(path: &PathBuf, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
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
