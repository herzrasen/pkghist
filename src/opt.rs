use clap::*;

pub fn parse_args<'a>(argv: &[String]) -> ArgMatches<'a> {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Trace package versions from pacman's logfile")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Set the level of verbosity"),
        )
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
            Arg::with_name("depth")
                .help("How many versions to go back in report")
                .short("d")
                .long("depth")
                .takes_value(true)
                .default_value("all")
                .validator(|v| {
                    if v.to_lowercase() == String::from("all") {
                        return Ok(());
                    }
                    match v.parse::<u32>() {
                        Ok(_) => Ok(()),
                        Err(_) => Err(String::from("Please provide either 'all' or a number")),
                    }
                }),
        )
        .arg(
            Arg::with_name("no-colors")
                .help("Disable colored output")
                .long("no-colors")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("filter")
                .help("Filter the packages that should be searched for.")
                .multiple(true),
        );
    app.get_matches_from(argv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_verbosity3() {
        let matches = parse_args(&[String::from("pkghist"), String::from("-vvv")]);
        assert_eq!(matches.occurrences_of("verbose"), 3)
    }

    #[test]
    fn should_parse_default_logfile() {
        let matches = parse_args(&[String::from("pkghist")]);
        assert_eq!(matches.value_of("logfile"), Some("/var/log/pacman.log"))
    }

    #[test]
    fn should_parse_provided_logfile_short() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("-l"),
            String::from("/my/very/special/pacman.log"),
        ]);
        assert_eq!(
            matches.value_of("logfile"),
            Some("/my/very/special/pacman.log")
        )
    }

    #[test]
    fn should_parse_provided_logfile_long() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--logfile"),
            String::from("/my/very/special/pacman.log"),
        ]);
        assert_eq!(
            matches.value_of("logfile"),
            Some("/my/very/special/pacman.log")
        )
    }

    #[test]
    fn should_parse_with_removed() {
        let matches = parse_args(&[String::from("pkghist"), String::from("-r")]);
        assert_eq!(matches.is_present("with-removed"), true);
        assert_eq!(matches.is_present("removed-only"), false)
    }

    #[test]
    fn should_parse_with_installed_long() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--with-removed")]);
        assert_eq!(matches.is_present("with-removed"), true);
        assert_eq!(matches.is_present("removed-only"), false)
    }

    #[test]
    fn should_parse_removed_only_short() {
        let matches = parse_args(&[String::from("pkghist"), String::from("-R")]);
        assert_eq!(matches.is_present("with-removed"), false);
        assert_eq!(matches.is_present("removed-only"), true)
    }

    #[test]
    fn should_parse_removed_only_long() {
        let matches = parse_args(&[String::from("pkghist"), String::from("--removed-only")]);
        assert_eq!(matches.is_present("with-removed"), false);
        assert_eq!(matches.is_present("removed-only"), true)
    }

    #[test]
    fn should_parse_depth_short() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("-d"),
            String::from("3"),
        ]);
        assert_eq!(matches.value_of("depth"), Some("3"))
    }

    #[test]
    fn should_parse_depth_long() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("--depth"),
            String::from("3"),
        ]);
        assert_eq!(matches.value_of("depth"), Some("3"))
    }

    #[test]
    fn should_parse_packages() {
        let matches = parse_args(&[
            String::from("pkghist"),
            String::from("bash"),
            String::from("linux"),
        ]);
        assert_eq!(matches.occurrences_of("filter"), 2)
    }

}
