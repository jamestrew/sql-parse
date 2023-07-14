use std::path::PathBuf;

use clap::{error::ErrorKind, Args, CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn missing_paths_error() {
        let mut cli = Cli::command();
        cli.error(ErrorKind::MissingRequiredArgument, "Missing search paths")
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
    Regexp(RegexpOptions),
}

impl Commands {
    pub fn basics(&self) -> (&Vec<PathBuf>, &PathBuf) {
        match self {
            Commands::TS(Basics {
                search_paths,
                treesitter_query,
            }) => (search_paths, treesitter_query),
            Commands::Quotes(Basics {
                search_paths,
                treesitter_query,
            }) => (search_paths, treesitter_query),
            Commands::Regexp(RegexpOptions {
                search_paths,
                treesitter_query,
                ..
            }) => (search_paths, treesitter_query),
        }
    }
}

#[derive(Args)]
pub struct Basics {
    /// Path for treesitter query file
    #[arg(short, long, value_name = "FILE")]
    pub treesitter_query: PathBuf,

    /// Files to search through
    pub search_paths: Vec<PathBuf>,
}

#[derive(Args)]
pub struct RegexpOptions {
    /// Path for treesitter query file
    #[arg(short, long, value_name = "FILE")]
    pub treesitter_query: PathBuf,

    #[command(flatten)]
    pub regexp: RegexpPattern,

    /// Files to search through
    pub search_paths: Vec<PathBuf>,

    /// Set regexp search to be case insensitive.
    #[arg(short = 'i', long, default_value_t = false)]
    pub ignore_case: bool,

    /// Invert matching.
    #[arg(short = 'v', long, default_value_t = false, conflicts_with = "replace")]
    pub invert_match: bool,

    /// Enable matching across multiple lines.
    #[arg(short = 'U', long, default_value_t = false)]
    pub multiline: bool,

    /// Replace every match with the text given when printing results.
    #[arg(short, long, value_name = "REPLACEMENT_TEXT", conflicts_with = "invert_match")]
    pub replace: Option<String>,
}

impl From<Commands> for RegexpOptions {
    fn from(value: Commands) -> Self {
        match value {
            Commands::Regexp(rg) => rg,
            _ => unreachable!("can't get RegexpOption from non-rg commands"),
        }
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RegexpPattern {
    /// Regexp pattern
    #[arg(
        short = 'e',
        long,
        value_name = "PATTERN",
        conflicts_with = "regexp_file"
    )]
    pub regexp: Option<String>,

    /// Regexp pattern as a file
    #[arg(long, value_name = "FILE", conflicts_with = "regexp")]
    pub regexp_file: Option<PathBuf>,
}
