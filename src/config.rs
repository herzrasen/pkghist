use std::str::FromStr;

use clap::ArgMatches;

use crate::error::Error;
use crate::error::ErrorDetail;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Format {
    Plain { with_colors: bool },
    Json,
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format_str = s.to_lowercase();
        if format_str == String::from("json") {
            Ok(Format::Json)
        } else if format_str == String::from("plain") {
            Ok(Format::Plain { with_colors: true })
        } else {
            Err(Error::new(ErrorDetail::InvalidFormat))
        }
    }
}

pub struct Config {
    pub removed_only: bool,
    pub with_removed: bool,
    pub logfile: String,
    pub filters: Vec<String>,
    pub format: Format,
    pub no_colors: bool,
}

impl Config {
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
        Config {
            removed_only: matches.is_present("removed-only"),
            with_removed: matches.is_present("with-removed"),
            logfile: String::from(matches.value_of("logfile").unwrap()),
            filters,
            format,
            no_colors: matches.is_present("no-colors"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::opt::parse_args;

    use super::*;

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
    fn should_create_config_from_args_no_colors() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--no-colors")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.no_colors, true);
        assert_eq!(config.format, Format::Plain { with_colors: false })
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
}
