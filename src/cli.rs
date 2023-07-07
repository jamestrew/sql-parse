use regex::Regex;
use std::path::PathBuf;

use clap::{error::ErrorKind, Args, CommandFactory, Parser};

use crate::error_exit;

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
pub struct Cli {
    /// Path for treesitter query file
    #[arg(short, long, value_name = "FILE")]
    pub treesitter_query: PathBuf,

    #[command(flatten)]
    pub regexp: RegexpOption,

    /// Set regexp search to be case insensitive. Default: false.
    // #[arg(short = 'i', long, default_value_t = false)]
    // ignore_case: bool,

    /// Files to search through
    pub search_paths: Vec<PathBuf>,
}

impl Cli {
    pub fn missing_paths_error() {
        let mut cli = Cli::command();
        cli.error(ErrorKind::MissingRequiredArgument, "Missing search paths")
            .exit();
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RegexpOption {
    /// Regexp pattern
    #[arg(
        short = 'e',
        long,
        value_name = "PATTERN",
        conflicts_with = "regexp_file"
    )]
    pub regexp: Option<String>,

    /// Regexp pattern as a file
    #[arg(short = 'E', long, value_name = "FILE", conflicts_with = "regexp")]
    pub regexp_file: Option<PathBuf>,
}

impl From<&RegexpOption> for Regex {
    fn from(value: &RegexpOption) -> Self {
        if let Some(pattern) = &value.regexp {
            Self::new(pattern)
                .unwrap_or_else(|_| error_exit!("Invalid regexp expression: {}", pattern))
        } else if let Some(file_path) = &value.regexp_file {
            let pattern = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
                error_exit!("Failed to read provided regexp file: {:?}", file_path)
            });
            Self::new(&pattern)
                .unwrap_or_else(|_| error_exit!("Invalid regexp expression: {}", pattern))
        } else {
            unreachable!()
        }
    }
}
