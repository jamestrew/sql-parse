#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {{
        eprintln!("ERROR: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}

