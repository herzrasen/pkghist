use std::str::FromStr;

use clap::ArgMatches;

use crate::error::Error;
use crate::error::ErrorDetail;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Format {
    Plain,
    Json,
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format_str = s.to_lowercase();
        if format_str == String::from("json") {
            Ok(Format::Json)
        } else if format_str == String::from("plain") {
            Ok(Format::Plain)
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
}

impl Config {
    pub fn from_arg_matches(matches: &ArgMatches) -> Config {
        let filters = match matches.values_of("filter") {
            Some(packages) => packages.map(String::from).collect(),
            None => Vec::new(),
        };
        Config {
            removed_only: matches.is_present("removed-only"),
            with_removed: matches.is_present("with-removed"),
            logfile: String::from(matches.value_of("logfile").unwrap()),
            filters,
            format: matches.value_of("output-format").unwrap().parse().unwrap(),
        }
    }

    pub fn is_relevant_package(&self, package: &str) -> bool {
        self.filters.is_empty() || self.filters.contains(&String::from(package))
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
        assert_eq!(format.unwrap(), Format::Plain)
    }

    #[test]
    fn should_parse_format_plain_ignore_case() {
        let format: Result<Format, Error> = "PlAiN".parse();
        assert!(format.is_ok());
        assert_eq!(format.unwrap(), Format::Plain)
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
    fn should_create_config_from_args_1() {
        let matches = parse_args(&[String::from("pkghist")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, false)
    }

    #[test]
    fn should_create_config_from_args_2() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--removed-only")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, true)
    }

    #[test]
    fn should_create_config_from_args_3() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--with-removed")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), true);
        assert_eq!(config.with_removed, true);
        assert_eq!(config.removed_only, false)
    }

    #[test]
    fn should_create_config_from_args_4() {
        let matches = parse_args(&[String::from("pkghist"), String::from("linux")]);
        let config = Config::from_arg_matches(&matches);
        assert_eq!(config.logfile, "/var/log/pacman.log");
        assert_eq!(config.filters.is_empty(), false);
        assert_eq!(config.filters.len(), 1);
        assert_eq!(config.with_removed, false);
        assert_eq!(config.removed_only, false)
    }

    #[test]
    fn should_be_relevant_when_filters_are_empty() {
        let config = Config {
            removed_only: false,
            with_removed: false,
            format: Format::Plain,
            filters: Vec::new(),
            logfile: "/var/log/pacman.log".to_string(),
        };
        assert_eq!(config.is_relevant_package("linux"), true)
    }

    #[test]
    fn should_not_be_relevant_with_filters() {
        let mut filters: Vec<String> = Vec::new();
        filters.push(String::from("vim"));
        let config = Config {
            logfile: String::from("/not/relevant"),
            with_removed: false,
            removed_only: false,
            filters,
            format: Format::Plain,
        };
        assert_eq!(config.is_relevant_package("linux"), false)
    }

}
