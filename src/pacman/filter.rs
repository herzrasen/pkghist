use crate::opt::Config;
use crate::pacman::action::Action;
use crate::pacman::group::Group;
use crate::pacman::latest::Latest;
use crate::pacman::PacmanEvent;
use itertools::Itertools;
use std::collections::HashMap;

pub trait Filter {
    type Event;
    fn without_installed(&self, filters: &Vec<String>) -> HashMap<&String, Vec<&Self::Event>>;
    fn without_removed(&self, filters: &Vec<String>) -> HashMap<&String, Vec<&Self::Event>>;

    fn filter_packages(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>>;
}

impl Filter for Vec<PacmanEvent> {
    type Event = PacmanEvent;

    fn without_installed(&self, filters: &Vec<String>) -> HashMap<&String, Vec<&PacmanEvent>> {
        let groups = self.group_relevant(&filters);
        let mut without_installed = groups.clone();
        for (package, mut events) in groups {
            let latest_event = events.latest();
            log::debug!("Latest parsed line for {} -> {:?}", package, latest_event);
            if latest_event.action.is_installed() {
                log::info!(
                    "Removing {} from result since it is currently installed",
                    package
                );
                without_installed.remove(package);
            }
        }
        without_installed
    }

    fn without_removed(&self, filters: &Vec<String>) -> HashMap<&String, Vec<&PacmanEvent>> {
        let groups = self.group_relevant(&filters);
        let mut without_removed = groups.clone();
        for (package, mut events) in groups {
            let latest_event = events.latest();
            log::debug!("Latest parsed line for {} -> {:?}", package, latest_event);
            if latest_event.action.is_removed() {
                log::info!(
                    "Removing {} from result since it is currently not installed",
                    package
                );
                without_removed.remove(package);
            }
        }
        without_removed
    }

    fn filter_packages(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>> {
        let filters: Vec<String> = if config.last.is_some() {
            get_filters_for_last_n_pacman_events(
                config.last.unwrap(),
                config.removed_only,
                config.with_removed,
                self,
            )
        } else {
            config.filters.clone()
        };
        let mut packages = if config.removed_only {
            self.without_installed(&filters)
        } else if !config.with_removed {
            self.without_removed(&filters)
        } else {
            self.group_relevant(&filters)
        };
        limit_pacman_events(&mut packages, config.limit)
    }
}

fn get_filters_for_last_n_pacman_events(
    last_n: u32,
    removed_only: bool,
    with_removed: bool,
    pacman_events: &Vec<PacmanEvent>,
) -> Vec<String> {
    let mut filters = Vec::new();
    for pacman_event in pacman_events.iter().rev() {
        match pacman_event.action {
            Action::Removed if removed_only || with_removed => {
                filters.push(pacman_event.package.clone())
            }
            Action::Removed => (), // removed but not interested in removed elements
            _ if removed_only == false => filters.push(pacman_event.package.clone()),
            _ => (),
        }
    }
    filters
        .iter()
        .dedup()
        .take(last_n as usize)
        .map(|e| e.clone())
        .collect()
}

fn limit_pacman_events<'a>(
    packages: &mut HashMap<&'a String, Vec<&'a PacmanEvent>>,
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

pub fn is_relevant_package(filters: &Vec<String>, package: &str) -> bool {
    filters.is_empty() || filters.contains(&String::from(package))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use std::fs::File;
    use std::io::Write;

    use crate::pacman;
    use filepath::FilePath;
    use std::fs;
    use std::path::Path;

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

        let mut filters: Vec<String> = Vec::new();
        filters.push(String::from("bash"));
        filters.push(String::from("linux"));

        let mut config = Config::new();
        config.logfile = file_name.clone();
        config.filters = filters;

        let pacman_events = pacman::from_file(Path::new(&file_name))
            .unwrap_or_else(|_| panic!("Unable to open {}", &file_name));

        let groups = pacman_events.filter_packages(&config);
        assert_eq!(groups.keys().len(), 2);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

    fn some_pacman_events() -> Vec<PacmanEvent> {
        let mut pacman_events = Vec::new();
        pacman_events.push(PacmanEvent {
            package: String::from("some-package"),
            action: pacman::action::Action::Installed,
            from: String::from("0.0.1"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: pacman::action::Action::Upgraded,
            from: String::from("0.0.1"),
            to: Some(String::from("0.0.2")),
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: pacman::action::Action::Upgraded,
            from: String::from("0.0.2"),
            to: Some(String::from("0.0.3")),
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("no-longer-used"),
            action: pacman::action::Action::Removed,
            from: String::from("0.0.1"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events
    }

    #[test]
    fn should_get_filters_for_last_n_events_without_removed() {
        // given
        let pacman_events = some_pacman_events();

        // when
        let filters = get_filters_for_last_n_pacman_events(2, false, false, &pacman_events);

        // then
        assert_eq!(filters.len(), 2);
        assert_eq!(
            filters,
            [
                String::from("another-package"),
                String::from("some-package")
            ]
            .to_vec()
        )
    }

    #[test]
    fn should_get_filters_for_last_n_events_with_removed() {
        // given
        let pacman_events = some_pacman_events();

        // when
        let filters = get_filters_for_last_n_pacman_events(2, false, true, &pacman_events);

        // then
        assert_eq!(filters.len(), 2);
        assert_eq!(
            filters,
            [
                String::from("no-longer-used"),
                String::from("another-package")
            ]
            .to_vec()
        )
    }

    #[test]
    fn should_get_filters_for_last_n_events_removed_only() {
        // given
        let pacman_events = some_pacman_events();

        // when
        let filters = get_filters_for_last_n_pacman_events(2, true, false, &pacman_events);

        // then
        assert_eq!(filters.len(), 1);
        assert_eq!(filters, [String::from("no-longer-used"),].to_vec())
    }

    #[test]
    fn should_remove_installed() {
        // given
        let pacman_events = some_pacman_events();
        let filters = Vec::new();

        // when
        let without_installed = pacman_events.without_installed(&filters);

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
        let filters = Vec::new();

        // when
        let without_removed = pacman_events.without_removed(&filters);

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
        let filters = Vec::new();
        let mut group = pacman_events.without_removed(&filters);

        // when
        let limited = limit_pacman_events(&mut group, Some(1));

        // then
        for (_, l) in limited {
            assert_eq!(l.len(), 1)
        }
    }
}
