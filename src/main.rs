pub mod opt;
use std::env;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let matches = opt::parse_args(&argv);
}
