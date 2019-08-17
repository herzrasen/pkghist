use crate::error::Error;
use std::env;

pub mod config;
pub mod error;
pub mod log;
pub mod opt;
pub mod pacman;
pub mod pkghist;

fn main() -> Result<(), Error> {
    let argv: Vec<String> = env::args().collect();
    let matches = opt::parse_args(&argv);

    log::setup_logging(matches.occurrences_of("verbose")).unwrap();

    let config = config::Config::from_arg_matches(&matches);
    pkghist::run(config)
}
