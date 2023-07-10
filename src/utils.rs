use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {{
        eprintln!("ERROR: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}

pub(crate) fn is_python_file(path: &Path) -> bool {
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
