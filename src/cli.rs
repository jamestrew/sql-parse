use std::io::BufRead;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand};

use crate::error_exit;
use crate::utils::expand_paths;

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn missing_paths_error() {
        let mut cli = Cli::command();
        cli.error(ErrorKind::MissingRequiredArgument, "Missing search path(s)")
            .exit();
    }

    pub fn tree_sitter(&self) -> (Option<&PathBuf>, bool) {
        let (path, no_ts) = match &self.command {
            Commands::TS(Basics {
                treesitter_query, ..
            }) => (treesitter_query, false),
            Commands::Quotes(Basics {
                treesitter_query, ..
            }) => (treesitter_query, false),
            Commands::Regex(RegexOptions {
                treesitter_query,
                no_ts,
                ..
            }) => (treesitter_query, *no_ts),
        };

        (path.as_ref(), no_ts)
    }

    pub fn search_paths(&self) -> Vec<PathBuf> {
        let paths = match &self.command {
            Commands::TS(Basics { search_paths, .. }) => search_paths,
            Commands::Quotes(Basics { search_paths, .. }) => search_paths,
            Commands::Regex(RegexOptions { search_paths, .. }) => search_paths,
        };

        if atty::is(atty::Stream::Stdin) && paths.is_empty() {
            Cli::missing_paths_error();
        }

        let paths = if paths.is_empty() {
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
            paths.to_owned()
        };
        expand_paths(paths)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find all matching tree-sitter nodes
    TS(Basics),

    /// Convert all SQL strings matched by tree-sitter in `"""` quotes
    Quotes(Basics),

    /// Pipe tree-sitter matched nodes to regex pattern matching
    Regex(RegexOptions),
}

#[derive(Args)]
pub struct Basics {
    /// Path for treesitter query file
    #[arg(short, long, value_name = "FILE")]
    pub treesitter_query: Option<PathBuf>,

    /// Files to search through
    pub search_paths: Vec<PathBuf>,
}

#[derive(Args)]
pub struct RegexOptions {
    /// Path for treesitter query file.
    #[arg(short, long, value_name = "FILE")]
    pub treesitter_query: Option<PathBuf>,

    #[command(flatten)]
    pub regex: RegexPattern,

    /// Files to search through.
    pub search_paths: Vec<PathBuf>,

    /// Set regex search to be case insensitive.
    #[arg(short = 'i', long, default_value_t = false)]
    pub ignore_case: bool,

    /// Invert matching.
    #[arg(short = 'v', long, default_value_t = false, conflicts_with = "replace")]
    pub invert_match: bool,

    /// Enable matching across multiple lines.
    #[arg(short = 'U', long, default_value_t = false)]
    pub multiline: bool,

    /// Replace every match with the text given when printing results.
    #[arg(
        short,
        long,
        value_name = "REPLACEMENT_TEXT",
        conflicts_with = "invert_match"
    )]
    pub replace: Option<String>,

    /// Confirm each replace. Requires --replace to be used.
    #[arg(short, long, default_value_t = false, requires = "replace")]
    pub confirm: bool,

    /// Don't use tree-sitter. AKA raw regex over the entire file(s).
    #[arg(long, default_value_t = false)]
    pub no_ts: bool,
}

impl From<Commands> for RegexOptions {
    fn from(value: Commands) -> Self {
        match value {
            Commands::Regex(rg) => rg,
            _ => unreachable!("can't get RegexOption from non-rg commands"),
        }
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RegexPattern {
    /// Regex pattern
    #[arg(value_name = "PATTERN", conflicts_with = "regex_file")]
    pub regex: Option<String>,

    /// Regex pattern as a file
    #[arg(long, value_name = "FILE", conflicts_with = "regex")]
    pub regex_file: Option<PathBuf>,
}
