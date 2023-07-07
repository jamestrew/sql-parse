use std::path::PathBuf;

use clap::{Args, Parser};

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
struct Cli {
    /// Path for treesitter query file
    #[arg(short, long, value_name = "FILE")]
    treesitter_query: PathBuf,

    #[command(flatten)]
    regexp: RegexpOption,

    /// Set regexp search to be case insensitive. Default: false.
    #[arg(short = 'i', long, default_value_t = false)]
    ignore_case: bool,

    /// Files to search through
    search_path: Vec<PathBuf>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct RegexpOption {
    /// Regexp pattern
    #[arg(
        short = 'e',
        long,
        value_name = "PATTERN",
        conflicts_with = "regexp_file"
    )]
    regexp: Option<String>,

    /// Regexp pattern as a file
    #[arg(short = 'E', long, value_name = "FILE", conflicts_with = "regexp")]
    regexp_file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    println!("{:?}", args.treesitter_query);
    println!("{:?}", args.regexp);
    println!("{:?}", args.search_path);
    Ok(())
}

// 1. fetch files of interest with `rg --files-with-matches crs\.execute <PATH>`
// 2. use treesitter to find all nodes of interest
// this depends on the ts query but eg. crs.execute[many]
// 3. use regex to match for issue keywords
// 4. return the file and line num of the issue spot
