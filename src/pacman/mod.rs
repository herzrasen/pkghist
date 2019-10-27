use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use chrono::DateTime;
use chrono::NaiveDateTime;

use lazy_static::*;
use regex::*;
use std::error::Error as StdError;

use crate::error::{Error, ErrorDetail};
use crate::pacman::action::Action;

pub mod action;
pub mod filter;
pub mod group;
pub mod newest;
pub mod range;

lazy_static! {
    static ref REGEX: Regex = Regex::new(r"^\[(?P<date>(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}|\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{4}))\]\s\[.+\]\s(?P<action>upgraded|installed|removed|reinstalled|downgraded)\s(?P<package>.+)\s\((?P<from>.+?)(\s->\s(?P<to>.+))?\)").unwrap();
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PacmanEvent {
    pub date: NaiveDateTime,
    pub action: Action,
    pub package: String,
    pub from: String,
    pub to: Option<String>,
}

impl PacmanEvent {
    pub fn new(
        date: NaiveDateTime,
        action: Action,
        package: String,
        from: String,
        to: Option<String>,
    ) -> PacmanEvent {
        PacmanEvent {
            date,
            action,
            package,
            from,
            to,
        }
    }

    pub fn printable_version(&self) -> String {
        if self.to.is_some() {
            self.to.clone().unwrap()
        } else {
            self.from.clone()
        }
    }
}

impl Ord for PacmanEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.package == other.package {
            self.date.cmp(&other.date)
        } else {
            self.package.cmp(&other.package)
        }
    }
}

impl PartialOrd for PacmanEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for PacmanEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if REGEX.is_match(s) {
            match REGEX.captures(s) {
                Some(captures) => {
                    let date = parse_date(captures.name("date").unwrap().as_str());
                    let action =
                        Action::from_str(captures.name("action").unwrap().as_str()).unwrap();
                    let package = String::from(captures.name("package").unwrap().as_str());
                    let from = String::from(captures.name("from").unwrap().as_str());
                    let to = match captures.name("to") {
                        Some(to) => Some(String::from(to.as_str())),
                        None => None,
                    };
                    Ok(PacmanEvent::new(date, action, package, from, to))
                }
                None => Err(Error::new(ErrorDetail::InvalidFormat)),
            }
        } else {
            Err(Error::new(ErrorDetail::InvalidFormat))
        }
    }
}

fn parse_date(date_str: &str) -> NaiveDateTime {
    let d = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M");
    match d {
        Ok(n) => n,
        Err(_) => {
            let date_time = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%z").unwrap();
            date_time.naive_local()
        }
    }
}

pub fn from_file(path: &Path) -> std::io::Result<Vec<PacmanEvent>> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let pacman_events: Vec<PacmanEvent> =
        file.lines()
            .enumerate()
            .fold(vec![], |mut current, (idx, l)| match l {
                Ok(line) => match PacmanEvent::from_str(line.as_str()) {
                    Ok(pacman_event) => {
                        current.push(pacman_event);
                        current
                    }
                    Err(_) => current,
                },
                Err(e) => {
                    eprintln!("Skipping line #{} ({:?})", idx + 1, e.description());
                    current
                }
            });
    Ok(pacman_events)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use std::io::Write;
    use std::path;

    use chrono::{NaiveDate, NaiveTime};
    use filepath::FilePath;

    use super::*;

    #[test]
    fn should_parse_new_date_format() {
        let p1: PacmanEvent = "[2019-10-23T20:25:18+0200] [ALPM] installed yay (9.4.2-1)"
            .parse()
            .unwrap();
        assert_eq!(
            p1.date,
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 10, 23),
                NaiveTime::from_hms(20, 25, 18)
            )
        )
    }

    #[test]
    fn should_order_pacman_events_by_date() {
        let p1: PacmanEvent = "[2019-07-16 21:07] [ALPM] installed nvidia (430.26)"
            .parse()
            .unwrap();
        let p2: PacmanEvent = "[2019-07-16 21:08] [ALPM] upgraded nvidia (430.26 -> 430.26-5)"
            .parse()
            .unwrap();
        let p3: PacmanEvent = "[2019-07-16 21:09] [ALPM] upgraded nvidia (430.26-9 -> 430.26-10)"
            .parse()
            .unwrap();
        let mut p = [&p2, &p3, &p1].to_vec();

        p.sort();

        let should_match = [&p1, &p2, &p3];
        assert_eq!(p.as_slice(), should_match)
    }

    #[test]
    fn should_order_pacman_events_by_date_and_package() {
        let p1: PacmanEvent =
            "[2019-05-23 07:00] [ALPM] installed intellij-idea-community-edition (2:2019.1.2-1)"
                .parse()
                .unwrap();
        let p2: PacmanEvent = "[2019-05-29 22:25] [ALPM] upgraded intellij-idea-community-edition (2:2019.1.2-1 -> 2:2019.1.3-1)".parse().unwrap();
        let p3: PacmanEvent = "[2019-07-25 01:17] [ALPM] upgraded intellij-idea-community-edition (2:2019.1.3-1 -> 2:2019.1.3-2)".parse().unwrap();
        let p4: PacmanEvent = "[2019-07-25 23:38] [ALPM] upgraded intellij-idea-community-edition (2:2019.1.3-2 -> 2:2019.2-1)".parse().unwrap();

        let p5: PacmanEvent =
            "[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)"
                .parse()
                .unwrap();
        let p6: PacmanEvent =
            "[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)"
                .parse()
                .unwrap();
        let p7: PacmanEvent =
            "[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)"
                .parse()
                .unwrap();
        let p8: PacmanEvent =
            "[2019-07-25 01:16] [ALPM] upgraded linux (5.2.1.arch1-1 -> 5.2.2.arch1-1)"
                .parse()
                .unwrap();

        let mut p = [&p5, &p3, &p8, &p6, &p1, &p4, &p2, &p7].to_vec();

        p.sort();

        let should_match = [&p1, &p2, &p3, &p4, &p5, &p6, &p7, &p8];
        assert_eq!(p.as_slice(), should_match)
    }

    #[test]
    fn should_extract_a_pacman_event_with_from_and_to() {
        let line: PacmanEvent = "[2019-07-05 22:10] [ALPM] upgraded libva (2.4.1-1 -> 2.5.0-1)"
            .parse()
            .unwrap();
        let expected_pacman_event = PacmanEvent {
            date: NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 7, 5),
                NaiveTime::from_hms(22, 10, 0),
            ),
            action: Action::Upgraded,
            package: String::from("libva"),
            from: String::from("2.4.1-1"),
            to: Some(String::from("2.5.0-1")),
        };
        assert_eq!(line, expected_pacman_event)
    }

    #[test]
    fn should_extract_a_pacman_install_event() {
        let line: PacmanEvent = "[2019-06-26 10:47] [ALPM] installed ansible (2.8.1-1)"
            .parse()
            .unwrap();
        let exptected_pacman_event = PacmanEvent {
            date: NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 6, 26),
                NaiveTime::from_hms(10, 47, 0),
            ),
            action: Action::Installed,
            package: String::from("ansible"),
            from: String::from("2.8.1-1"),
            to: None,
        };
        assert_eq!(line, exptected_pacman_event)
    }

    #[test]
    fn should_extract_a_pacman_downgraded_event() {
        let line: PacmanEvent =
            "[2018-12-15 00:22] [ALPM] downgraded mps-youtube (0.2.8-2 -> 0.2.8-1)"
                .parse()
                .unwrap();
        let expected_pacman_event = PacmanEvent {
            date: NaiveDateTime::new(
                NaiveDate::from_ymd(2018, 12, 15),
                NaiveTime::from_hms(0, 22, 0),
            ),
            action: Action::Downgraded,
            package: String::from("mps-youtube"),
            from: String::from("0.2.8-2"),
            to: Some(String::from("0.2.8-1")),
        };
        assert_eq!(line, expected_pacman_event)
    }

    #[test]
    fn should_extract_a_pacman_reinstall_event() {
        let line: PacmanEvent = "[2019-06-26 10:47] [ALPM] reinstalled ansible (2.8.1-1)"
            .parse()
            .unwrap();
        let exptected_pacman_event = PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 6, 26),
                NaiveTime::from_hms(10, 47, 0),
            ),
            Action::Reinstalled,
            String::from("ansible"),
            String::from("2.8.1-1"),
            None,
        );
        assert_eq!(line, exptected_pacman_event)
    }

    #[test]
    fn should_extract_a_removed_pacman_event_with_from() {
        let line: PacmanEvent = "[2019-07-04 14:05] [ALPM] removed gnome-common (3.18.0-3)"
            .parse()
            .unwrap();
        let expected_pacman_event = PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 7, 4),
                NaiveTime::from_hms(14, 5, 0),
            ),
            Action::Removed,
            String::from("gnome-common"),
            String::from("3.18.0-3"),
            None,
        );
        assert_eq!(line, expected_pacman_event)
    }

    #[test]
    fn should_not_extract_a_pacman_event() {
        let r = PacmanEvent::from_str("[2019-07-04 14:05] I AM NOT MATCHING");
        assert_eq!(r.is_err(), true)
    }

    #[test]
    fn should_result_in_an_error() {
        let res = from_file(path::Path::new(&String::from("/not/found")));
        assert_eq!(res.is_err(), true)
    }

    #[test]
    fn should_extract_the_valid_lines() {
        let mut file = File::create(uuid::Uuid::new_v4().to_string()).unwrap();
        writeln!(
            file,
            "[2019-07-14 21:33] [PACMAN] synchronizing package lists\n[2019-07-14 21:33] [PACMAN] starting full system upgrade\n[2019-07-14 21:33] [ALPM] transaction started\n[2019-07-14 21:33] [ALPM] upgraded feh (3.1.3-1 -> 3.2-1)\n[2019-07-14 21:33] [ALPM] upgraded libev (4.25-1 -> 4.27-1)\n[2019-07-14 21:33] [ALPM] upgraded iso-codes (4.2-1 -> 4.3-1)"
        )
        .unwrap();

        let pacman_events = from_file(&file.path().unwrap()).unwrap();

        assert_eq!(pacman_events.len(), 3);

        let packages: Vec<String> = pacman_events.iter().map(|p| p.package.clone()).collect();
        assert_eq!(
            packages.as_slice(),
            [
                String::from("feh"),
                String::from("libev"),
                String::from("iso-codes"),
            ]
        );
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    #[test]
    fn should_skip_invalid_line() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2018-12-15 00:19] [PACMAN] Running 'pacman -U ^��sA��'"
        )
        .unwrap();

        let pacman_events = from_file(Path::new(&file_name)).unwrap();

        assert_eq!(pacman_events.len(), 0);

        fs::remove_file(file.path().unwrap()).unwrap()
    }
}
