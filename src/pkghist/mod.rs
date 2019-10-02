mod format;

use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;
use crate::opt::Config;
use crate::pacman;
use crate::pacman::filter::Filter;
use crate::pacman::PacmanEvent;
use itertools::Itertools;
use std::io::stdout;

use crate::pkghist::format::Printer;

pub fn run(config: Config) -> Result<(), Error> {
    let logfile_path = &config.logfile;
    let pacman_events = pacman::from_file(Path::new(logfile_path)).unwrap_or_else(|_| {
        eprintln!("Unable to open {}", logfile_path);
        std::process::exit(2)
    });

    let groups = pacman_events.filter_packages(&config);

    let mut package_histories = Vec::new();

    let sorted: Vec<Vec<&PacmanEvent>> = groups
        .iter()
        .sorted_by(|(p1, e1), (p2, e2)| {
            let d1 = e1.last().unwrap().date;
            let d2 = e2.last().unwrap().date;
            if d1 == d2 {
                p1.cmp(p2)
            } else {
                d1.cmp(&d2)
            }
        })
        .map(|(_, e)| e.clone())
        .collect();

    for mut events in sorted {
        events.sort();
        let package_history = PackageHistory::from_pacman_events(events);
        package_histories.push(package_history);
    }

    match config.format.print(&mut stdout(), &package_histories) {
        _ => Ok(()),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Event {
    pub v: String,
    pub d: String,
    pub a: String,
}

impl Event {
    fn new(version: String, date: String, action: String) -> Event {
        Event {
            v: version,
            d: date,
            a: action,
        }
    }

    fn from_pacman_event(pacman_event: &PacmanEvent) -> Event {
        Event::new(
            pacman_event.printable_version(),
            pacman_event.date.to_string(),
            pacman_event.action.to_string(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageHistory {
    pub p: String,
    pub e: Vec<Event>,
}

impl PackageHistory {
    fn new(p: String, e: Vec<Event>) -> PackageHistory {
        PackageHistory { p, e }
    }

    fn from_pacman_events(pacman_events: Vec<&PacmanEvent>) -> PackageHistory {
        let e: Vec<Event> = pacman_events
            .iter()
            .map(|e| Event::from_pacman_event(e))
            .collect();
        let p = pacman_events.first().unwrap().package.clone();
        PackageHistory::new(p, e)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    use filepath::FilePath;

    use crate::pacman::action::Action;

    use super::*;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    #[test]
    fn should_create_package_histories_with_new() {
        let ev1 = Event::new(
            String::from("1.2.1"),
            String::from("2019-10-01 12:30:00"),
            String::from("Upgraded"),
        );
        let ev2 = Event::new(
            String::from("1.2.1"),
            String::from("2019-10-01 13:30:00"),
            String::from("Removed"),
        );

        let package_histories =
            PackageHistory::new(String::from("foo"), vec![ev1.clone(), ev2.clone()]);

        assert_eq!(package_histories.p, "foo");
        assert_eq!(package_histories.e.len(), 2);
        assert!(package_histories.e.contains(&ev1));
        assert!(package_histories.e.contains(&ev2))
    }

    #[test]
    fn should_create_package_histories() {
        let ev1 = PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 9, 1),
                NaiveTime::from_hms(12, 30, 0),
            ),
            Action::Installed,
            String::from("test"),
            String::from("0.1.0"),
            None,
        );
        let ev2 = PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 9, 1),
                NaiveTime::from_hms(18, 30, 10),
            ),
            Action::Upgraded,
            String::from("test"),
            String::from("0.1.0"),
            Some(String::from("0.1.1")),
        );

        let pacman_events = vec![&ev1, &ev2];
        let package_history = PackageHistory::from_pacman_events(pacman_events);
        assert_eq!(package_history.p, "test");
        assert_eq!(package_history.e.len(), 2);
        assert_eq!(
            package_history.e,
            vec![
                Event::new(
                    String::from("0.1.0"),
                    String::from("2019-09-01 12:30:00"),
                    String::from("Installed")
                ),
                Event::new(
                    String::from("0.1.1"),
                    String::from("2019-09-01 18:30:10"),
                    String::from("Upgraded")
                )
            ]
        )
    }

    #[test]
    fn should_be_ok_1() {
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

        let mut config = Config::new();
        config.logfile = file_name;

        let result = run(config);
        assert_eq!(result.is_ok(), true);
        fs::remove_file(file.path().unwrap()).unwrap()
    }
}
