use std::io::BufRead;
use std::path::{Path, PathBuf};

use console::style;

use crate::cli::Cli;
use crate::treesitter::Treesitter;

#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {{
        eprintln!("ERROR: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}

fn get_search_path(search_paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    if atty::is(atty::Stream::Stdin) && search_paths.is_empty() {
        Cli::missing_paths_error();
    }

    if search_paths.is_empty() {
        let stdin = std::io::stdin();
        stdin
            .lock()
            .lines()
            .map(|line| {
                if let Ok(line) = line {
                    PathBuf::from(&line)
                } else {
                    error_exit!("Failed to read stdin line")
                }
            })
            .collect::<Vec<_>>()
    } else {
        search_paths.to_owned()
    }
}

fn expand_paths(search_paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut expanded = Vec::new();

    for path in search_paths {
        if path.is_dir() {
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.is_file() && is_python_file(&path) {
                        expanded.push(path);
                    }
                }
            }
        } else {
            expanded.push(path);
        }
    }
    expanded
}

pub(crate) fn basic_cli_options(cli: &Cli) -> (Treesitter, Vec<PathBuf>) {
    let (search_path, ts_file) = cli.command.basics();
    let search_path = get_search_path(search_path);

    let treesitter = Treesitter::try_from(ts_file).unwrap_or_else(|err| {
        error_exit!("{}", err);
    });

    (treesitter, expand_paths(search_path))
}

fn is_python_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext) = ext.to_str() {
            return ext.eq_ignore_ascii_case("py");
        }
    }
    false
}

pub(crate) fn iter_valid_files(paths: &[PathBuf]) -> impl Iterator<Item = (String, &PathBuf)> {
    paths.iter().filter_map(|path| {
        if !path.exists() {
            eprintln!("Path doesn't exist: {} -- skipping", path.display());
            return None;
        }

        if !is_python_file(path) {
            eprintln!(
                "Non Python files unsupported: {} -- skipping",
                path.display()
            );
            return None;
        }

        match std::fs::read_to_string(path) {
            Ok(code) => Some((code, path)),
            Err(_) => {
                eprintln!("Failed to read file: {} -- skipping", path.display());
                None
            }
        }
    })
}

pub(crate) fn write_file<P>(path: &P, bytes: &[u8]) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}

pub(crate) fn print(path: &str, lnum: usize, col: Option<usize>, text: &str) {
    let column = if let Some(col) = col {
        format!(":{col}")
    } else {
        "".to_string()
    };

    println!(
        "{}:{}{}:{}",
        style(path).magenta(),
        style(lnum.to_string()).green(),
        column,
        text
    );
}
