use crate::error::{Error, ErrorDetail};
use crate::opt::cli;
use std::env;
use std::io;
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

    if matches.is_present("completions") {
        generate_completions(matches.value_of("completions").unwrap(), &mut io::stdout())
    } else {
        let config = opt::Config::from_arg_matches(&matches);
        pkghist::run(config)
    }
}

fn generate_completions<W: std::io::Write>(shell: &str, stdout: &mut W) -> Result<(), Error> {
    match shell.parse() {
        Ok(s) => {
            cli::build_cli().gen_completions_to(env!("CARGO_PKG_NAME"), s, stdout);
            Ok(())
        }
        Err(e) => Err(Error::new(ErrorDetail::IOError {
            msg: format!("{:?}", e),
        })),
    }
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

    #[test]
    fn should_run_completions() {
        let args = vec![
            String::from("pkghist"),
            String::from("--completions"),
            String::from("zsh"),
        ];
        let r = run(args);

        assert_eq!(r.is_ok(), true)
    }

    #[test]
    fn should_create_completions_for_bash() {
        let mut stdout = Vec::new();
        let r = generate_completions("bash", &mut stdout);

        assert_eq!(r.is_ok(), true);

        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str.starts_with("_pkghist() {\n"), true);
        assert_eq!(str.contains("esac"), true)
    }

    #[test]
    fn should_create_completions_for_zsh() {
        let mut stdout = Vec::new();
        let r = generate_completions("zsh", &mut stdout);

        assert_eq!(r.is_ok(), true);

        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str.starts_with("#compdef pkghist\n"), true)
    }

    #[test]
    fn should_create_completions_for_fish() {
        let mut stdout = Vec::new();
        let r = generate_completions("fish", &mut stdout);

        assert_eq!(r.is_ok(), true);

        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str.contains("__fish_use_subcommand"), true)
    }

    #[test]
    fn should_fail_to_create_completions() {
        let mut stdout = Vec::new();
        let r = generate_completions("nosh", &mut stdout);

        assert_eq!(r.is_err(), true)
    }
}
