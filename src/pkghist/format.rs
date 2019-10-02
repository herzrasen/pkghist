use serde_json;

use crate::error::Error;
use crate::opt::Format;
use crate::pacman::action::Action;
use crate::pkghist::{Event, PackageHistory};
use termion::color;

fn format_json<W: std::io::Write>(
    stdout: &mut W,
    packages_with_version: &[PackageHistory],
    without_details: bool,
) -> Result<(), Error> {
    let json = if without_details {
        let packages: Vec<String> = packages_with_version.iter().map(|p| p.p.clone()).collect();
        serde_json::to_string_pretty(&packages).unwrap()
    } else {
        serde_json::to_string_pretty(packages_with_version).unwrap()
    };
    writeln!(stdout, "{}", json)?;
    Ok(())
}

fn format_plain<W: std::io::Write>(
    stdout: &mut W,
    package_histories: &[PackageHistory],
    with_colors: bool,
    without_details: bool,
) -> Result<(), Error> {
    for package_history in package_histories {
        if with_colors {
            // check if last event is a removal
            match last_action(&package_history) {
                Action::Removed => write!(stdout, "{red}", red = color::Fg(color::Red))?,
                _ => write!(stdout, "{green}", green = color::Fg(color::Green))?,
            }
            writeln!(
                stdout,
                "{package}{reset}",
                package = package_history.p,
                reset = color::Fg(color::Reset)
            )?
        } else {
            writeln!(stdout, "{}", package_history.p)?
        }
        if !without_details {
            for event in &package_history.e {
                if with_colors {
                    match event.a.parse().unwrap() {
                        Action::Removed => write!(stdout, "{red}", red = color::Fg(color::Red))?,
                        Action::Downgraded => {
                            write!(stdout, "{yellow}", yellow = color::Fg(color::Yellow))?
                        }
                        _ => {}
                    }
                    writeln!(
                        stdout,
                        "  [{date}] {action}",
                        date = event.d,
                        action = event.a,
                    )?;
                    writeln!(
                        stdout,
                        "    {version}{reset}",
                        version = event.v,
                        reset = color::Fg(color::Reset)
                    )?
                } else {
                    writeln!(
                        stdout,
                        "  [{date}] {action}",
                        date = event.d,
                        action = event.a
                    )?;
                    writeln!(stdout, "    {version}", version = event.v)?
                }
            }
        }
    }
    Ok(())
}

fn format_compact<W: std::io::Write>(
    stdout: &mut W,
    package_histories: &[PackageHistory],
    with_colors: bool,
    without_details: bool,
) -> Result<(), Error> {
    let (p_max, d_max, a_max, v_max) = max_lens(&package_histories);
    for package_history in package_histories {
        for event in &package_history.e {
            match (with_colors, without_details) {
                (with_colors, true) => {
                    if with_colors {
                        match event.a.parse().unwrap() {
                            Action::Removed => {
                                write!(stdout, "{red}", red = color::Fg(color::Red))?
                            }
                            Action::Downgraded => {
                                write!(stdout, "{yellow}", yellow = color::Fg(color::Yellow))?
                            }
                            _ => write!(stdout, "{green}", green = color::Fg(color::Green))?,
                        }
                    }
                    write!(
                        stdout,
                        "|{package: <p_max$}|",
                        package = package_history.p,
                        p_max = p_max
                    )?;
                    if with_colors {
                        writeln!(stdout, "{reset}", reset = color::Fg(color::Reset))?
                    } else {
                        writeln!(stdout, "")?
                    }
                }
                (with_colors, false) => {
                    if with_colors {
                        match event.a.parse().unwrap() {
                            Action::Removed => {
                                write!(stdout, "{red}", red = color::Fg(color::Red))?
                            }
                            Action::Downgraded => {
                                write!(stdout, "{yellow}", yellow = color::Fg(color::Yellow))?
                            }
                            _ => write!(stdout, "{green}", green = color::Fg(color::Green))?,
                        }
                    }
                    write!(
                        stdout,
                        "|{package: <p_max$}|{date: <d_max$}|{action: <a_max$}|{version: <v_max$}|",
                        package = package_history.p,
                        p_max = p_max,
                        date = event.d,
                        d_max = d_max,
                        action = event.a,
                        a_max = a_max,
                        version = event.v,
                        v_max = v_max
                    )?;
                    if with_colors {
                        writeln!(stdout, "{reset}", reset = color::Fg(color::Reset))?
                    } else {
                        writeln!(stdout)?
                    }
                }
            }
        }
    }
    Ok(())
}

fn max_lens(package_histories: &[PackageHistory]) -> (usize, usize, usize, usize) {
    let p_max = package_histories.iter().map(|p| p.p.len()).max().unwrap();
    let events: Vec<Event> = package_histories.iter().flat_map(|p| p.e.clone()).collect();
    let d_max = events.iter().map(|e| e.d.len()).max().unwrap();
    let a_max = events.iter().map(|e| e.a.len()).max().unwrap();
    let v_max = events.iter().map(|e| e.v.len()).max().unwrap();
    (p_max, d_max, a_max, v_max)
}

fn last_action(package_history: &PackageHistory) -> Action {
    let last_event = package_history.e.last().unwrap();
    last_event.a.parse().unwrap()
}

pub trait Printer {
    fn print<W: std::io::Write>(
        &self,
        stdout: &mut W,
        package_histories: &[PackageHistory],
    ) -> Result<(), Error>;
}

impl Printer for Format {
    fn print<W: std::io::Write>(
        &self,
        stdout: &mut W,
        package_histories: &[PackageHistory],
    ) -> Result<(), Error> {
        match *self {
            Format::Plain {
                with_colors,
                without_details,
            } => format_plain(stdout, package_histories, with_colors, without_details),
            Format::Json { without_details } => {
                format_json(stdout, package_histories, without_details)
            }
            Format::Compact {
                with_colors,
                without_details,
            } => format_compact(stdout, package_histories, with_colors, without_details),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Printer;

    #[test]
    fn should_print_json_to_stdout() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Json {
            without_details: false,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(
            str,
            "[\n  {\n    \"p\": \"foo\",\n    \"e\": [\n      {\n        \"v\": \"0.0.1\",\n        \"d\": \"2019-08-26 12:00:00\",\n        \"a\": \"Installed\"\n      }\n    ]\n  }\n]\n"
        )
    }

    #[test]
    fn should_print_json_to_stdout_no_details() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Json {
            without_details: true,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "[\n  \"foo\"\n]\n")
    }

    #[test]
    fn should_print_to_stdout_colored() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![
                Event {
                    a: String::from("Upgraded"),
                    v: String::from("0.0.2"),
                    d: String::from("2019-08-26 12:00:00"),
                },
                Event {
                    a: String::from("Downgraded"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 13:00:00"),
                },
                Event {
                    a: String::from("Removed"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 14:00:00"),
                },
            ],
        }];
        let mut stdout = Vec::new();
        Format::Plain {
            with_colors: true,
            without_details: false,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(
            str,
            "\u{1b}[38;5;1mfoo\u{1b}[39m\n  [2019-08-26 12:00:00] Upgraded\n    0.0.2\u{1b}[39m\n\u{1b}[38;5;3m  [2019-08-26 13:00:00] Downgraded\n    0.0.1\u{1b}[39m\n\u{1b}[38;5;1m  [2019-08-26 14:00:00] Removed\n    0.0.1\u{1b}[39m\n"
        )
    }

    #[test]
    fn should_print_to_stdout_colored_no_details() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Plain {
            with_colors: true,
            without_details: true,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "\u{1b}[38;5;2mfoo\u{1b}[39m\n")
    }

    #[test]
    fn should_print_to_stdout_no_colors() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Plain {
            with_colors: false,
            without_details: false,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "foo\n  [2019-08-26 12:00:00] Installed\n    0.0.1\n")
    }

    #[test]
    fn should_print_to_stdout_no_colors_no_details() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Plain {
            with_colors: false,
            without_details: true,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "foo\n")
    }

    #[test]
    fn should_print_compact_to_stdout() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![
                Event {
                    a: String::from("Upgraded"),
                    v: String::from("0.0.2"),
                    d: String::from("2019-08-26 12:00:00"),
                },
                Event {
                    a: String::from("Downgraded"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 13:00:00"),
                },
                Event {
                    a: String::from("Removed"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 14:00:00"),
                },
            ],
        }];
        let mut stdout = Vec::new();
        Format::Compact {
            with_colors: true,
            without_details: false,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(
            str,
            "\u{1b}[38;5;2m|foo|2019-08-26 12:00:00|Upgraded  |0.0.2|\u{1b}[39m\n\
             \u{1b}[38;5;3m|foo|2019-08-26 13:00:00|Downgraded|0.0.1|\u{1b}[39m\n\
             \u{1b}[38;5;1m|foo|2019-08-26 14:00:00|Removed   |0.0.1|\u{1b}[39m\n"
        )
    }

    #[test]
    fn should_print_compact_to_stdout_no_colors() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Upgraded"),
                v: String::from("0.0.2"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Compact {
            with_colors: false,
            without_details: false,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "|foo|2019-08-26 12:00:00|Upgraded|0.0.2|\n")
    }

    #[test]
    fn should_print_compact_to_stdout_no_details() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![
                Event {
                    a: String::from("Installed"),
                    v: String::from("0.0.2"),
                    d: String::from("2019-08-26 12:00:00"),
                },
                Event {
                    a: String::from("Downgraded"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 13:00:00"),
                },
                Event {
                    a: String::from("Removed"),
                    v: String::from("0.0.1"),
                    d: String::from("2019-08-26 14:00:00"),
                },
            ],
        }];
        let mut stdout = Vec::new();
        Format::Compact {
            with_colors: true,
            without_details: true,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(
            str,
            "\u{1b}[38;5;2m|foo|\u{1b}[39m\n\
             \u{1b}[38;5;3m|foo|\u{1b}[39m\n\
             \u{1b}[38;5;1m|foo|\u{1b}[39m\n"
        )
    }

    #[test]
    fn should_print_compact_to_stdout_no_details_no_colors() {
        let package_histories = vec![PackageHistory {
            p: String::from("foo"),
            e: vec![Event {
                a: String::from("Installed"),
                v: String::from("0.0.1"),
                d: String::from("2019-08-26 12:00:00"),
            }],
        }];
        let mut stdout = Vec::new();
        Format::Compact {
            with_colors: false,
            without_details: true,
        }
        .print(&mut stdout, &package_histories)
        .unwrap();
        let str = String::from_utf8(stdout).unwrap();
        assert_eq!(str, "|foo|\n")
    }

    #[test]
    fn should_get_max_lens() {
        let package_histories = vec![
            PackageHistory {
                p: String::from("foo"),
                e: vec![
                    Event {
                        a: String::from("Installed"),
                        v: String::from("0.0.1"),
                        d: String::from("2019-08-26 12:00:00"),
                    },
                    Event {
                        a: String::from("Upgraded"),
                        v: String::from("0.0.2"),
                        d: String::from("2019-08-30 13:30:00"),
                    },
                ],
            },
            PackageHistory {
                p: String::from("another"),
                e: vec![
                    Event {
                        a: String::from("Installed"),
                        v: String::from("1.0.1"),
                        d: String::from("2019-08-27 12:00:00"),
                    },
                    Event {
                        a: String::from("Upgraded"),
                        v: String::from("1.0.2-deadbeef"),
                        d: String::from("2019-09-01 13:30:00"),
                    },
                ],
            },
        ];
        let (p_max, d_max, a_max, v_max) = max_lens(&package_histories);
        assert_eq!(p_max, 7);
        assert_eq!(d_max, 19);
        assert_eq!(a_max, 9);
        assert_eq!(v_max, 14)
    }

    #[test]
    fn should_get_last_action_removed() {
        let package_history = PackageHistory {
            p: String::from("another"),
            e: vec![
                Event {
                    a: String::from("Installed"),
                    v: String::from("1.0.1"),
                    d: String::from("2019-08-27 12:00:00"),
                },
                Event {
                    a: String::from("Removed"),
                    v: String::from("1.0.2-deadbeef"),
                    d: String::from("2019-09-01 13:30:00"),
                },
            ],
        };
        let action = last_action(&package_history);
        assert_eq!(action, Action::Removed)
    }

    #[test]
    fn should_get_last_action_upgraded() {
        let package_history = PackageHistory {
            p: String::from("another"),
            e: vec![
                Event {
                    a: String::from("Installed"),
                    v: String::from("1.0.1"),
                    d: String::from("2019-08-27 12:00:00"),
                },
                Event {
                    a: String::from("Upgraded"),
                    v: String::from("1.0.2-deadbeef"),
                    d: String::from("2019-09-01 13:30:00"),
                },
            ],
        };
        let action = last_action(&package_history);
        assert_eq!(action, Action::Upgraded)
    }
}
