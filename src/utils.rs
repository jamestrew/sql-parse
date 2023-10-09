use std::path::{Path, PathBuf};

use console::style;
#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {{
        eprintln!("ERROR: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}

pub fn expand_paths(search_paths: Vec<PathBuf>) -> Vec<PathBuf> {
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
