use chrono::NaiveDateTime;
use clap::{command, Arg, Command};

pub fn build_cli() -> Command {
    command!(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Trace package versions from pacman's logfile")
        .arg(
            Arg::new("output-format")
                .short('o')
                .long("output-format")
                .num_args(1)
                .value_parser(["json", "plain", "compact"])
                .default_value("plain")
                .help("Select the output format"),
        )
        .arg(
            Arg::new("logfile")
                .short('l')
                .long("logfile")
                .value_name("FILE")
                .help("Specify a logfile")
                .default_value("/var/log/pacman.log")
                .num_args(1),
        )
        .arg(
            Arg::new("with-removed")
                .short('r')
                .long("with-removed")
                .num_args(0)
                .conflicts_with("removed-only")
                .help("Include packages that are currently uninstalled"),
        )
        .arg(
            Arg::new("removed-only")
                .short('R')
                .long("removed-only")
                .num_args(0)
                .conflicts_with("with-removed")
                .help("Only output packages that are currently uninstalled"),
        )
        .arg(
            Arg::new("limit")
                .help("How many versions to go back in report. [limit > 0]")
                .short('L')
                .long("limit")
                .num_args(1)
                .value_parser(validate_gt_0),
        )
        .arg(
            Arg::new("no-colors")
                .num_args(0)
                .help("Disable colored output")
                .long("no-colors"),
        )
        .arg(
            Arg::new("no-details")
                .num_args(0)
                .long("no-details")
                .help("Only output the package names")
        )
        .arg(
            Arg::new("first")
                .long("first")
                .value_name("n")
                .num_args(1)
                .conflicts_with_all(&["filter", "last"])
                .help("Output the first 'n' pacman events")
                .value_parser(validate_gt_0),
        )
        .arg(
            Arg::new("last")
                .long("last")
                .value_name("n")
                .num_args(1)
                .conflicts_with("filter")
                .help("Output the last 'n' pacman events")
                .value_parser(validate_gt_0),
        )
        .arg(
            Arg::new("after")
                .long("after")
                .short('a')
                .value_name("date")
                .help(
                    "Only consider events that occurred after 'date' [Format: \"YYYY-MM-DD HH:MM\"]",
                )
                .value_parser(validate_date)
                .num_args(1),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('x')
                .num_args(0)
                .help("If set, every filter result will be excluded.")
        )
        .arg(
            Arg::new("filter")
                .help("Filter the packages that should be searched for. \
                Use regular expressions to specify the exact pattern to match \
                (e.g. '^linux$' only matches the package 'linux')")
                .num_args(0..),
        )
}

fn validate_gt_0(str: &str) -> Result<String, String> {
    match str.parse::<u32>() {
        Ok(l) => {
            if l > 0 {
                Ok(str.to_owned())
            } else {
                Err(String::from("limit must be greater than 0"))
            }
        }
        Err(_) => Err(String::from("Please provide a positive number")),
    }
}

fn validate_date(str: &str) -> Result<String, String> {
    match NaiveDateTime::parse_from_str(str, "%Y-%m-%d %H:%M") {
        Ok(_) => Ok(str.to_owned()),
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
        let r = validate_gt_0("123");
        assert_eq!(r.is_ok(), true)
    }

    #[test]
    fn should_not_validate_gt_0_no_number() {
        let r = validate_gt_0("notanumber");
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_not_validate_gt_0() {
        let r = validate_gt_0("0");
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_validate_date() {
        let d = validate_date("2019-10-02 12:30");
        assert_eq!(d.is_ok(), true)
    }

    #[test]
    fn should_not_validate_date() {
        let d = validate_date("20191002 1230");
        assert_eq!(d.is_err(), true)
    }
}
