use crate::error::Error;
use std::env;

pub mod error;
pub mod opt;
pub mod pacman;
pub mod pkghist;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    run(args)
}

fn run(args: Vec<String>) -> Result<(), Error> {
    let matches = opt::parse_args(&args);

    let config = opt::Config::from_arg_matches(&matches);
    pkghist::run(config)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    use super::*;
    use filepath::FilePath;

    #[test]
    fn should_run() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-07-14 21:33] [PACMAN] synchronizing package lists\n\
             [2019-07-14 21:33] [PACMAN] starting full system upgrade\n\
             [2019-07-14 21:33] [ALPM] transaction started\n\
             [2019-07-14 21:33] [ALPM] installed feh (3.1.3-1)\n\
             [2019-07-14 21:33] [ALPM] upgraded libev (4.25-1 -> 4.27-1)\n\
             [2019-07-14 21:33] [ALPM] upgraded iso-codes (4.2-1 -> 4.3-1)"
        )
        .unwrap();

        let args = vec![String::from("pkghist"), String::from("-l"), file_name];
        let r = run(args);

        assert_eq!(r.is_ok(), true);

        fs::remove_file(file.path().unwrap()).unwrap()
    }
}
