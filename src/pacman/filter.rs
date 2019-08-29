use crate::opt::Config;
use crate::pacman::group::Group;
use crate::pacman::latest::Latest;
use crate::pacman::PacmanEvent;
use itertools::Itertools;
use std::collections::HashMap;

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
            let latest_event = events.latest();
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
            let latest_event = events.latest();
            if latest_event.action.is_removed() {
                without_removed.remove(package);
            }
        }
        without_removed
    }

    fn filter_packages(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>> {
        let mut packages = if config.removed_only {
            self.without_installed()
        } else if !config.with_removed {
            self.without_removed()
        } else {
            self.group()
        };

        let mut last_n_packages = last_n_pacman_events(&mut packages, config.last);
        limit_pacman_events(&mut last_n_packages, config.limit)
    }
}

fn last_n_pacman_events<'a>(
    grouped: &mut HashMap<&'a String, Vec<&'a PacmanEvent>>,
    last_n: Option<u32>,
) -> HashMap<&'a String, Vec<&'a PacmanEvent>> {
    match last_n {
        Some(n) => {
            let filters: Vec<&String> = grouped
                .into_iter()
                .sorted_by(|(p1, e1), (p2, e2)| {
                    let d1 = e1.last().unwrap().date;
                    let d2 = e2.last().unwrap().date;
                    if d1 == d2 {
                        p1.cmp(p2)
                    } else {
                        d1.cmp(&d2)
                    }
                })
                .map(|(p, _)| p.clone())
                .rev()
                .unique()
                .take(n as usize)
                .collect();
            println!("filters: {:?}", filters);
            let mut filtered = HashMap::new();
            grouped
                .into_iter()
                .filter(|(p, _)| filters.contains(*p))
                .for_each(|(p, e)| {
                    filtered.insert(p.clone(), e.clone());
                });
            filtered
        }
        None => grouped.clone(),
    }
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
            action: pacman::action::Action::Installed,
            from: String::from("0.0.2"),
            to: None,
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
            package: String::from("another-package"),
            action: pacman::action::Action::Removed,
            from: String::from("0.0.2"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: pacman::action::Action::Installed,
            from: String::from("0.0.2"),
            to: None,
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

    //    #[test]
    //    fn should_get_filters_for_last_n_events_without_removed() {
    //        // given
    //        let pacman_events = some_pacman_events();
    //        let mut group = pacman_events.group();
    //
    //
    //        // when
    //        let filtered = last_n_pacman_events(&mut group, Some(2));
    //
    //        // then
    //        assert_eq!(filtered.keys().len(), 2);
    //        assert_eq!(
    //            filtered.keys(),
    //            [
    //                String::from("another-package"),
    //                String::from("some-package")
    //            ]
    //            .to_vec()
    //        )
    //    }

    //    #[test]
    //    fn should_get_filters_for_last_n_events_with_removed() {
    //        // given
    //        let pacman_events = some_pacman_events();
    //
    //        // when
    //        let filters = get_filters_for_last_n_pacman_events(3, false, true, &pacman_events);
    //
    //        // then
    //        assert_eq!(filters.len(), 3);
    //        assert_eq!(
    //            filters,
    //            [
    //                String::from("no-longer-used"),
    //                String::from("another-package"),
    //                String::from("some-package"),
    //            ]
    //            .to_vec()
    //        )
    //    }

    //    #[test]
    //    fn should_get_filters_for_last_n_events_removed_only() {
    //        // given
    //        let pacman_events = some_pacman_events();
    //
    //        // when
    //        let filters = get_filters_for_last_n_pacman_events(2, true, false, &pacman_events);
    //
    //        // then
    //        assert_eq!(filters.len(), 2);
    //        assert_eq!(
    //            filters,
    //            [
    //                String::from("no-longer-used"),
    //                String::from("another-package")
    //            ]
    //            .to_vec()
    //        )
    //    }

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
}
