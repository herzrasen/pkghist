use std::collections::HashMap;

use chrono::NaiveDateTime;

use regex::Regex;

use crate::opt::Config;
use crate::pacman::group::Group;
use crate::pacman::newest::Newest;
use crate::pacman::range;
use crate::pacman::PacmanEvent;

pub trait Filter {
    type Event;

    fn without_installed(&self) -> HashMap<&String, Vec<&Self::Event>>;

    fn without_removed(&self) -> HashMap<&String, Vec<&Self::Event>>;

    fn filter_packages(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>>;
}

impl Filter for Vec<PacmanEvent> {
    type Event = PacmanEvent;

    fn without_installed(&self) -> HashMap<&String, Vec<&PacmanEvent>> {
        let groups = self.group();
        let mut without_installed = groups.clone();
        for (package, mut events) in groups {
            let latest_event = events.newest();
            if latest_event.action.is_installed() {
                without_installed.remove(package);
            }
        }
        without_installed
    }

    fn without_removed(&self) -> HashMap<&String, Vec<&PacmanEvent>> {
        let groups = self.group();
        let mut without_removed = groups.clone();
        for (package, mut events) in groups {
            let latest_event = events.newest();
            if latest_event.action.is_removed() {
                without_removed.remove(package);
            }
        }
        without_removed
    }

    fn filter_packages(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>> {
        let packages = if config.removed_only {
            self.without_installed()
        } else if !config.with_removed {
            self.without_removed()
        } else {
            self.group()
        };

        let mut filtered_packages = HashMap::new();
        for (package, events) in packages {
            let filtered_events = filter_events(events.clone(), &config.after);
            if !filtered_events.is_empty()
                && (config.filters.is_empty() || matches_filter(package, &config.filters))
            {
                println!("Package {} matches the filter", package);
                filtered_packages.insert(package, filtered_events);
            }
        }

        limit_pacman_events(
            &range::range(&filtered_packages, &config.direction),
            config.limit,
        )
    }
}

fn matches_filter(package: &str, filters: &Vec<Regex>) -> bool {
    filters.into_iter().find(|f| f.is_match(package)).is_some()
}

fn filter_events<'a>(
    events: Vec<&'a PacmanEvent>,
    after: &Option<NaiveDateTime>,
) -> Vec<&'a PacmanEvent> {
    match after {
        Some(a) => events.into_iter().filter(|event| event.date > *a).collect(),
        None => events,
    }
}

fn limit_pacman_events<'a>(
    packages: &HashMap<&'a String, Vec<&'a PacmanEvent>>,
    limit: Option<u32>,
) -> HashMap<&'a String, Vec<&'a PacmanEvent>> {
    if let Some(l) = limit {
        let mut limited_packages = HashMap::new();
        for (package, pacman_events) in packages {
            let limited = pacman_events.iter().by_ref().rev().take(l as usize).fold(
                Vec::new(),
                |mut current, event| {
                    current.push(*event);
                    current
                },
            );
            limited_packages.insert(*package, limited);
        }
        limited_packages
    } else {
        packages.clone()
    }
}

pub fn is_relevant_package(filters: &[String], package: &str) -> bool {
    filters.is_empty() || filters.contains(&String::from(package))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use chrono::{NaiveDate, NaiveTime};
    use filepath::FilePath;

    use crate::pacman;

    use super::*;
    use crate::pacman::action::Action;

    #[test]
    fn should_match_package_starting_with() {
        let regex = Regex::new("^linux").unwrap();
        let mut filters = Vec::new();
        filters.push(regex);
        assert!(matches_filter("linux", &filters));
        assert_eq!(matches_filter("utils-linux", &filters), false)
    }

    #[test]
    fn should_not_filter_packages_when_no_filters_are_defined() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)
[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)
[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)
[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)
[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)
[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)
[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)
[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)
[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)
[2019-05-10 12:45] [ALPM] upgraded bash (5.0.003-1 -> 5.0.007-1)"
        )
        .unwrap();

        let mut config = Config::new();
        config.logfile = file_name.clone();

        let pacman_events = pacman::from_file(Path::new(&file_name))
            .unwrap_or_else(|_| panic!("Unable to open {}", &file_name));

        let groups = pacman_events.filter_packages(&config);
        assert_eq!(groups.keys().len(), 3);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    #[test]
    fn should_filter_packages_when_not_matching_filters() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)
[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)
[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)
[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)
[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)
[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)
[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)
[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)
[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)
[2019-05-10 12:45] [ALPM] upgraded bash (5.0.003-1 -> 5.0.007-1)"
        )
        .unwrap();

        let mut filters: Vec<Regex> = Vec::new();
        filters.push(Regex::new("bash").unwrap());
        filters.push(Regex::new("linux").unwrap());

        let mut config = Config::new();
        config.logfile = file_name.clone();
        config.filters = filters;

        let pacman_events = pacman::from_file(Path::new(&file_name))
            .unwrap_or_else(|_| panic!("Unable to open {}", &file_name));

        let groups = pacman_events.filter_packages(&config);
        println!("{:?}", groups);
        assert_eq!(groups.keys().len(), 2);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    #[test]
    fn should_filter_installed() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)
[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)
[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)
[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)
[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)
[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)
[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)
[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)
[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)
[2019-05-10 12:45] [ALPM] removed bash (5.0.003-1)"
        )
        .unwrap();

        let mut filters: Vec<Regex> = Vec::new();
        filters.push(Regex::new("bash").unwrap());
        filters.push(Regex::new("linux").unwrap());

        let mut config = Config::new();
        config.logfile = file_name.clone();
        config.filters = filters;
        config.removed_only = true;

        let pacman_events = pacman::from_file(Path::new(&file_name))
            .unwrap_or_else(|_| panic!("Unable to open {}", &file_name));

        let groups = pacman_events.filter_packages(&config);
        println!("{:?}", groups);
        assert_eq!(groups.keys().len(), 1);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    #[test]
    fn should_keep_removed_and_installed() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)
[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)
[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)
[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)
[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)
[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)
[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)
[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)
[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)
[2019-05-10 12:45] [ALPM] removed bash (5.0.003-1)"
        )
        .unwrap();

        let mut config = Config::new();
        config.logfile = file_name.clone();
        config.filters = Vec::new();
        config.with_removed = true;

        let pacman_events = pacman::from_file(Path::new(&file_name))
            .unwrap_or_else(|_| panic!("Unable to open {}", &file_name));

        let groups = pacman_events.filter_packages(&config);
        println!("{:?}", groups);
        assert_eq!(groups.keys().len(), 3);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    fn some_pacman_events() -> Vec<PacmanEvent> {
        let mut pacman_events = Vec::new();
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(11, 30, 0),
            ),
            Action::Installed,
            String::from("some-package"),
            String::from("0.0.1"),
            None,
        ));
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(11, 30, 0),
            ),
            Action::Installed,
            String::from("another-package"),
            String::from("0.0.2"),
            None,
        ));
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(12, 30, 0),
            ),
            Action::Installed,
            String::from("another-package"),
            String::from("0.0.2"),
            Some(String::from("0.0.3")),
        ));
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(12, 31, 0),
            ),
            Action::Removed,
            String::from("another-package"),
            String::from("0.0.2"),
            None,
        ));
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(12, 35, 0),
            ),
            Action::Installed,
            String::from("another-package"),
            String::from("0.0.2"),
            None,
        ));
        pacman_events.push(PacmanEvent::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 08, 30),
                NaiveTime::from_hms(12, 35, 0),
            ),
            Action::Removed,
            String::from("no-longer-used"),
            String::from("0.0.2"),
            None,
        ));
        pacman_events
    }

    #[test]
    fn should_remove_installed() {
        // given
        let pacman_events = some_pacman_events();

        // when
        let without_installed = pacman_events.without_installed();

        // then
        assert_eq!(
            without_installed.contains_key(&String::from("some-package")),
            false
        );
        assert_eq!(
            without_installed.contains_key(&String::from("another-package")),
            false
        );
        assert_eq!(
            without_installed.contains_key(&String::from("no-longer-used")),
            true
        )
    }

    #[test]
    fn should_keep_installed() {
        // given
        let pacman_events = some_pacman_events();

        // when
        let without_removed = pacman_events.without_removed();

        // then
        assert_eq!(
            without_removed.contains_key(&String::from("some-package")),
            true
        );
        assert_eq!(
            without_removed.contains_key(&String::from("another-package")),
            true
        );
        assert_eq!(
            without_removed.contains_key(&String::from("no-longer-used")),
            false
        )
    }

    #[test]
    fn should_be_relevant_when_filters_are_empty() {
        let filters = Vec::new();
        assert_eq!(is_relevant_package(&filters, "linux"), true)
    }

    #[test]
    fn should_not_be_relevant_with_filters() {
        let mut filters: Vec<String> = Vec::new();
        filters.push(String::from("vim"));
        assert_eq!(is_relevant_package(&filters, "linux"), false)
    }

    #[test]
    fn should_limit_pacman_events() {
        // given
        let pacman_events = some_pacman_events();
        let mut group = pacman_events.without_removed();

        // when
        let limited = limit_pacman_events(&mut group, Some(1));

        // then
        for (_, l) in limited {
            assert_eq!(l.len(), 1)
        }
    }

    #[test]
    fn should_filter_events_before_date() {
        let pacman_events = some_pacman_events();
        let refs = pacman_events.iter().collect();
        let filtered = filter_events(
            refs,
            &Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2019, 8, 30),
                NaiveTime::from_hms(12, 31, 0),
            )),
        );

        assert_eq!(filtered.len(), 2)
    }

    #[test]
    fn should_filter_no_events() {
        let pacman_events = some_pacman_events();
        let refs = pacman_events.iter().collect();
        let filtered = filter_events(refs, &None);

        assert_eq!(filtered.len(), 6)
    }
}
