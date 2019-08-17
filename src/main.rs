use std::env;

pub mod config;
pub mod error;
pub mod log;
pub mod opt;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let matches = opt::parse_args(&argv);

    log::setup_logging(matches.occurrences_of("verbose")).unwrap();
}
