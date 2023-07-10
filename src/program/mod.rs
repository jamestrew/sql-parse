pub mod treesitter;

use crate::cli::{Cli, Commands};
use crate::error_exit;
use crate::program::treesitter::Treesitter;
use crate::treesitter::Treesitter as TS;

use std::{io::BufRead, path::PathBuf};

pub trait Program {
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

    fn basic_cli_options(cli: Cli) -> (TS, Vec<PathBuf>) {
        let (search_path, ts_file) = cli.command.basics();
        let ts_query = std::fs::read_to_string(ts_file).unwrap_or_else(|_| {
            error_exit!("Failed to read provided regexp file: {}", ts_file.display())
        });
        let treesitter = TS::try_from(ts_query).unwrap_or_else(|err| {
            error_exit!("{}", err);
        });

        (treesitter, Self::get_search_path(search_path))
    }

    fn new(cli: Cli) -> Self;

    fn run(&mut self);
}

pub fn new_program(cli: Cli) -> impl Program {
    match cli.command {
        Commands::TS(_) => Treesitter::new(cli),
        Commands::Quotes(_) => todo!(),
        Commands::Rg(_) => todo!(),
    }
}

// pub struct Treesitter {
//     pub(crate) treesitter: Treesitter,
//     pub(crate) search_paths: Vec<PathBuf>,
// }

// impl Program {
//     fn get_search_path(args: Cli) -> Vec<PathBuf> {
//         if atty::is(atty::Stream::Stdin) && args.search_paths.is_empty() {
//             Cli::missing_paths_error();
//         }
//
//         if args.search_paths.is_empty() {
//             let stdin = std::io::stdin();
//             stdin
//                 .lock()
//                 .lines()
//                 .map(|line| {
//                     if let Ok(line) = line {
//                         PathBuf::from(&line)
//                     } else {
//                         error_exit!("Failed to read stdin line")
//                     }
//                 })
//                 .collect::<Vec<_>>()
//         } else {
//             args.search_paths
//         }
//     }
//
//     pub fn run(&mut self) {
//         self.search_paths.iter().for_each(|path| {
//             if !path.exists() {
//                 eprintln!("Path doesn't exist: {:?}", path);
//                 return;
//             }
//             if !is_python_file(path) {
//                 eprintln!("Non Python files unsupported. Skipping {:?}", path);
//                 return;
//             }
//             todo!("parse file with tree-sitter and then regexp search")
//         })
//     }
// }

// impl From<Cli> for Program {
//     fn from(args: Cli) -> Self {
//         let ts_query = std::fs::read_to_string(&args.treesitter_query).unwrap_or_else(|_| {
//             error_exit!(
//                 "Failed to read provided regexp file: {:?}",
//                 args.treesitter_query
//             )
//         });
//
//         let treesitter = Treesitter::try_from(ts_query).unwrap_or_else(|err| {
//             error_exit!("{}", err);
//         });
//
//         Self {
//             treesitter,
//             regexp: Regex::from(&args.regexp),
//             search_paths: Self::get_search_path(args),
//         }
//     }
// }
