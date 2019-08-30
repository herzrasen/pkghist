use std::str::FromStr;

use crate::error::Error;
use crate::error::ErrorDetail;

use clap::{App, Arg, ArgMatches};

pub fn parse_args<'a>(argv: &[String]) -> ArgMatches<'a> {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Trace package versions from pacman's logfile")
        .arg(
            Arg::with_name("output-format")
                .short("o")
                .long("output-format")
                .takes_value(true)
                .possible_values(&[&"json", &"plain"])
                .default_value("plain")
                .help("Select the output format"),
        )
        .arg(
            Arg::with_name("logfile")
                .short("l")
                .long("logfile")
                .value_name("FILE")
                .help("Specify a logfile")
                .default_value("/var/log/pacman.log")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("with-removed")
                .short("r")
                .long("with-removed")
                .takes_value(false)
                .conflicts_with("removed-only")
                .help("Include packages that are currently uninstalled"),
        )
        .arg(
            Arg::with_name("removed-only")
                .short("R")
                .long("removed-only")
                .takes_value(false)
                .conflicts_with("with-removed")
                .help("Only output packages that are currently uninstalled"),
        )
        .arg(
            Arg::with_name("limit")
                .help("How many versions to go back in report")
                .short("L")
                .long("limit")
                .takes_value(true)
                .validator(|v| match v.parse::<u32>() {
                    Ok(_) => Ok(()),
                    Err(_) => Err(String::from("Please provide a positive number")),
                }),
        )
        .arg(
            Arg::with_name("no-colors")
                .help("Disable colored output")
                .long("no-colors")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("first")
                .long("first")
                .value_name("n")
                .takes_value(true)
                .conflicts_with_all(&["filter", "last"])
                .help("Output the first 'n' pacman events")
                .validator(validate_number),
        )
        .arg(
            Arg::with_name("last")
                .long("last")
                .value_name("n")
                .takes_value(true)
                .conflicts_with("filter")
                .help("Output the last 'n' pacman events")
                .validator(validate_number),
        )
        .arg(
            Arg::with_name("filter")
                .help("Filter the packages that should be searched for")
                .multiple(true),
        );
    app.get_matches_from(argv)
}

fn validate_number(str: String) -> Result<(), String> {
    match str.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Please provide a positive number")),
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Format {
    Plain { with_colors: bool },
    Json,
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format_str = s.to_lowercase();
        if format_str == "json" {
            Ok(Format::Json)
        } else if format_str == "plain" {
            Ok(Format::Plain { with_colors: true })
        } else {
            Err(Error::new(ErrorDetail::InvalidFormat))
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Direction {
    Forwards { n: usize },
    Backwards { n: usize },
}

impl Direction {
    fn from_first(n: u32) -> Direction {
        Direction::Forwards { n: n as usize }
    }

    fn from_last(n: u32) -> Direction {
        Direction::Backwards { n: n as usize }
    }
}

pub struct Config {
    pub removed_only: bool,
    pub with_removed: bool,
    pub logfile: String,
    pub filters: Vec<String>,
    pub format: Format,
    pub limit: Option<u32>,
    pub direction: Option<Direction>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            removed_only: false,
            with_removed: false,
            logfile: String::from("/var/log/pacman.log"),
            format: Format::Plain { with_colors: true },
            limit: None,
            direction: None,
            filters: Vec::new(),
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Default::default()
    }

    pub fn from_arg_matches(matches: &ArgMatches) -> Config {
        let filters = match matches.values_of("filter") {
            Some(packages) => packages.map(String::from).collect(),
            None => Vec::new(),
        };
        let format_from_matches: Format =
            matches.value_of("output-format").unwrap().parse().unwrap();
        let format = if format_from_matches == (Format::Plain { with_colors: true }) {
            if matches.is_present("no-colors") {
                Format::Plain { with_colors: false }
            } else {
                Format::Plain { with_colors: true }
            }
        } else {
            Format::Json
        };

        let limit = match matches.value_of("limit") {
            Some("all") => None,
            Some(v) => Some(v.parse::<u32>().unwrap()),
            None => None,
        };

        let direction = if matches.is_present("first") {
            Some(Direction::from_first(
                matches.value_of("first").unwrap().parse().unwrap(),
            ))
        } else if matches.is_present("last") {
            Some(Direction::from_last(
                matches.value_of("last").unwrap().parse().unwrap(),
            ))
        } else {
            None
        };

        Config {
            removed_only: matches.is_present("removed-only"),
            with_removed: matches.is_present("with-removed"),
            logfile: String::from(matches.value_of("logfile").unwrap()),
            limit,
            filters,
            format,
            direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_number() {
        let r = validate_number(String::from("123"));
        assert_eq!(r.is_ok(), true)
    }

    #[test]
    fn should_not_validate_number() {
        let r = validate_number(String::from("notanumber"));
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_parse_format_plain() {
        let format: Result<Format, Error> = "plain".parse();
        assert!(format.is_ok());
        assert_eq!(format.unwrap(), Format::Plain { with_colors: true })
    }

    #[test]
    fn should_parse_format_plain_ignore_case() {
        let format: Result<Format, Error> = "PlAiN".parse();
        assert!(format.is_ok());
        assert_eq!(format.unwrap(), Format::Plain { with_colors: true })
    }

    #[test]
    fn should_parse_format_json() {
        let format: Result<Format, Error> = "json".parse();
        assert!(format.is_ok());
        assert_eq!(format.unwrap(), Format::Json)
    }

    #[test]
    fn should_parse_format_json_ignore_case() {
        let format: Result<Format, Error> = "JsOn".parse();
        assert!(format.is_ok());
        assert_eq!(format.unwrap(), Format::Json)
    }

    #[test]
    fn should_not_parse_format() {
        let format: Result<Format, Error> = "foo".parse();
        assert!(format.is_err());
        assert_eq!(
            format.err().unwrap(),
            Error::new(ErrorDetail::InvalidFormat)
        );
    }

    #[test]
    fn should_create_config_from_args() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, false);
        assert_eq!(config.format, Format::Plain { with_colors: true })
    }

    #[test]
    fn should_create_config_from_args_removed_only() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--removed-only")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, true)
    }

    #[test]
    fn should_create_config_from_args_with_removed() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--with-removed")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, true);
        assert_eq!(config.removed_only, false)
    }

    #[test]
    fn should_create_config_from_args_filters() {
        let matches = parse_args(&[String::from("pkghist"), String::from("linux")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.filters.is_empty(), false);
        assert_eq!(config.filters.len(), 1);
    }

    #[test]
    fn should_create_config_from_args_format_json() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--output-format"),
            String::from("json"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.format, Format::Json)
    }

    #[test]
    fn should_create_config_from_args_limit_some() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--limit"),
            String::from("3"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.limit, Some(3))
    }

    #[test]
    fn should_create_config_from_args_limit_none() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.limit, None)
    }

    #[test]
    fn should_create_config_from_args_first_some() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--first"),
            String::from("50"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.direction, Some(Direction::Forwards { n: 50 }))
    }

    #[test]
    fn should_create_config_from_args_first_none() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.direction, None)
    }

    #[test]
    fn should_create_config_from_args_last_some() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--last"),
            String::from("50"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.direction, Some(Direction::Backwards { n: 50 }))
    }

}
