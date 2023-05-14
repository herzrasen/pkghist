use std::println;
use std::str::FromStr;

use crate::error::Error;
use crate::error::ErrorDetail;

use chrono::NaiveDateTime;
use clap::ArgMatches;

use regex::Regex;

pub mod cli;

pub fn parse_args<'a>(argv: &[String]) -> ArgMatches {
    cli::build_cli().get_matches_from(argv)
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Format {
    Plain {
        with_colors: bool,
        without_details: bool,
    },
    Json {
        without_details: bool,
    },
    Compact {
        with_colors: bool,
        without_details: bool,
    },
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format_str = s.to_lowercase();
        if format_str == "json" {
            Ok(Format::Json {
                without_details: false,
            })
        } else if format_str == "plain" {
            Ok(Format::Plain {
                with_colors: true,
                without_details: false,
            })
        } else if format_str == "compact" {
            Ok(Format::Compact {
                with_colors: true,
                without_details: false,
            })
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

#[derive(Debug)]
pub struct Config {
    pub exclude: bool,
    pub removed_only: bool,
    pub with_removed: bool,
    pub logfile: String,
    pub filters: Vec<Regex>,
    pub format: Format,
    pub limit: Option<u32>,
    pub direction: Option<Direction>,
    pub after: Option<NaiveDateTime>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exclude: false,
            removed_only: false,
            with_removed: false,
            logfile: String::from("/var/log/pacman.log"),
            format: Format::Plain {
                with_colors: true,
                without_details: false,
            },
            limit: None,
            direction: None,
            after: None,
            filters: Vec::new(),
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Default::default()
    }

    pub fn from_arg_matches(matches: &ArgMatches) -> Config {
        let filters = match matches.get_many::<String>("filter") {
            Some(filters) => filters.fold(Vec::new(), |mut current, f| {
                println!("{}", f);
                let r = Regex::new(f).unwrap();
                current.push(r);
                current
            }),
            None => Vec::new(),
        };

        let with_colors = !matches.get_flag("no-colors");

        let without_details = matches.get_flag("no-details");

        let format = match matches
            .get_one::<String>("output-format")
            .unwrap()
            .parse()
            .unwrap()
        {
            Format::Plain { .. } => Format::Plain {
                with_colors,
                without_details,
            },
            Format::Compact { .. } => Format::Compact {
                with_colors,
                without_details,
            },
            Format::Json { .. } => Format::Json { without_details },
        };

        let limit = match matches.get_one::<String>("limit") {
            Some(all) if all == "all" => None,
            Some(v) => Some(v.parse::<u32>().unwrap()),
            None => None,
        };

        let direction = if matches.contains_id("first") {
            Some(Direction::from_first(
                matches.get_one::<String>("first").unwrap().parse().unwrap(),
            ))
        } else if matches.contains_id("last") {
            Some(Direction::from_last(
                matches.get_one::<String>("last").unwrap().parse().unwrap(),
            ))
        } else {
            None
        };

        let after = match matches.get_one::<String>("after") {
            Some(date_str) => {
                Some(NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M").unwrap())
            }
            None => None,
        };

        Config {
            exclude: matches.get_flag("exclude"),
            removed_only: matches.get_flag("removed-only"),
            with_removed: matches.get_flag("with-removed"),
            logfile: matches.get_one::<String>("logfile").unwrap().to_owned(),
            limit,
            filters,
            format,
            direction,
            after,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn should_parse_format_plain() {
        let format: Result<Format, Error> = "plain".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Plain {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_parse_format_plain_ignore_case() {
        let format: Result<Format, Error> = "PlAiN".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Plain {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_parse_format_json() {
        let format: Result<Format, Error> = "json".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Json {
                without_details: false
            }
        )
    }

    #[test]
    fn should_parse_format_json_ignore_case() {
        let format: Result<Format, Error> = "JsOn".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Json {
                without_details: false
            }
        )
    }

    #[test]
    fn should_parse_format_compact() {
        let format: Result<Format, Error> = "compact".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Compact {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_parse_format_compact_ignore_case() {
        let format: Result<Format, Error> = "CoMpAcT".parse();
        assert!(format.is_ok());
        assert_eq!(
            format.unwrap(),
            Format::Compact {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_not_parse_format() {
        let format: Result<Format, Error> = "foo".parse();
        assert!(format.is_err());
        assert_eq!(
            format.err().unwrap(),
            Error::new(ErrorDetail::InvalidFormat)
        )
    }

    #[test]
    fn should_create_config_from_args() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.exclude, false);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, false);
        assert_eq!(
            config.format,
            Format::Plain {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_create_config_from_args_exclude() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--exclude"),
            String::from("^lib"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.filters.is_empty(), false);
        assert_eq!(config.exclude, true)
    }

    #[test]
    fn should_create_config_from_args_no_colors() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--no-colors")]);
        let config = Config::from_arg_matches(&matches);
        println!("{:?}", config);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, false);
        assert_eq!(
            config.format,
            Format::Plain {
                with_colors: false,
                without_details: false
            }
        )
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
        assert_eq!(
            config.format,
            Format::Json {
                without_details: false
            }
        )
    }

    #[test]
    fn should_create_config_from_args_format_json_no_details() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--output-format"),
            String::from("json"),
            String::from("--no-details"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(
            config.format,
            Format::Json {
                without_details: true
            }
        )
    }

    #[test]
    fn should_create_config_from_args_format_compact() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--output-format"),
            String::from("compact"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(
            config.format,
            Format::Compact {
                with_colors: true,
                without_details: false
            }
        )
    }

    #[test]
    fn should_create_config_from_args_format_compact_no_details() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--output-format"),
            String::from("compact"),
            String::from("--no-details"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(
            config.format,
            Format::Compact {
                with_colors: true,
                without_details: true
            }
        )
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

    #[test]
    fn should_create_config_from_args_after() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--after"),
            String::from("2019-01-01 12:00"),
        ]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(
            config.after,
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            ))
        )
    }

    #[test]
    fn should_create_config_from_args_after_none() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.after, None)
    }
}
