use crate::error::Error;
use std::env;

pub mod error;
pub mod opt;
pub mod pacman;
pub mod pkghist;

fn main() -> Result<(), Error> {
    let argv: Vec<String> = env::args().collect();
    let matches = opt::parse_args(&argv);

    let config = opt::Config::from_arg_matches(&matches);
    pkghist::run(config)
}
