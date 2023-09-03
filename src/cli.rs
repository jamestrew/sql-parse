use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand};

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

impl Commands {
    pub fn basics(&self) -> (&Vec<PathBuf>, Option<&PathBuf>) {
        let ret = match self {
            Commands::TS(Basics {
                search_paths,
                treesitter_query,
            }) => (search_paths, treesitter_query),
            Commands::Quotes(Basics {
                search_paths,
                treesitter_query,
            }) => (search_paths, treesitter_query),
            Commands::Regex(RegexOptions {
                search_paths,
                treesitter_query,
                ..
            }) => (search_paths, treesitter_query),
        };
        (ret.0, ret.1.as_ref())
    }
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
