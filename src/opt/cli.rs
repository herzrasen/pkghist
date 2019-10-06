use chrono::NaiveDateTime;
use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Trace package versions from pacman's logfile")
        .arg(
            Arg::with_name("output-format")
                .short("o")
                .long("output-format")
                .takes_value(true)
                .possible_values(&[&"json", &"plain", &"compact"])
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
                .help("How many versions to go back in report. [limit > 0]")
                .short("L")
                .long("limit")
                .takes_value(true)
                .validator(validate_gt_0),
        )
        .arg(
            Arg::with_name("no-colors")
                .help("Disable colored output")
                .long("no-colors")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("no-details")
            .long("no-details")
            .takes_value(false)
            .help("Only output the package names")
        )
        .arg(
            Arg::with_name("first")
                .long("first")
                .value_name("n")
                .takes_value(true)
                .conflicts_with_all(&["filter", "last"])
                .help("Output the first 'n' pacman events")
                .validator(validate_gt_0),
        )
        .arg(
            Arg::with_name("last")
                .long("last")
                .value_name("n")
                .takes_value(true)
                .conflicts_with("filter")
                .help("Output the last 'n' pacman events")
                .validator(validate_gt_0),
        )
        .arg(
            Arg::with_name("after")
                .long("after")
                .short("a")
                .value_name("date")
                .help(
                    "Only consider events that occurred after 'date' [Format: \"YYYY-MM-DD HH:MM\"]",
                )
                .validator(validate_date)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .short("x")
                .takes_value(false)
                .help("If set, every filter result will be excluded.")
        )
        .arg(
            Arg::with_name("filter")
                .help("Filter the packages that should be searched for. \
                Use regular expressions to specify the exact pattern to match \
                (e.g. '^linux$' only matches the package 'linux'")
                .multiple(true),
        )
}

fn validate_gt_0(str: String) -> Result<(), String> {
    match str.parse::<u32>() {
        Ok(l) => {
            if l > 0 {
                Ok(())
            } else {
                Err(String::from("limit must be greater than 0"))
            }
        }
        Err(_) => Err(String::from("Please provide a positive number")),
    }
}

fn validate_date(str: String) -> Result<(), String> {
    match NaiveDateTime::parse_from_str(str.as_str(), "%Y-%m-%d %H:%M") {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from(
            "Please provide a date in the format \"YYYY-MM-DD HH:MM\"",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_validate_gt_0() {
        let r = validate_gt_0(String::from("123"));
        assert_eq!(r.is_ok(), true)
    }

    #[test]
    fn should_not_validate_gt_0_no_number() {
        let r = validate_gt_0(String::from("notanumber"));
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_not_validate_gt_0() {
        let r = validate_gt_0(String::from("0"));
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_validate_date() {
        let d = validate_date(String::from("2019-10-02 12:30"));
        assert_eq!(d.is_ok(), true)
    }

    #[test]
    fn should_not_validate_date() {
        let d = validate_date(String::from("20191002 1230"));
        assert_eq!(d.is_err(), true)
    }
}
