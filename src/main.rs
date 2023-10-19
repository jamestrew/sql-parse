use clap::Parser;
use sql_parse::cli::Cli;
use sql_parse::program::new_program;

fn main() {
    #[cfg(debug_assertions)]
    {
        use tracing_subscriber::fmt;
        let log_file = std::fs::File::create("/tmp/sql-parse.log").unwrap();

        let subscriber = fmt::Subscriber::builder().with_writer(log_file).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        tracing::info!("Starting sql-parse");
    }

    let args = Cli::parse();
    let mut program = new_program(args);
    program.run();
}
