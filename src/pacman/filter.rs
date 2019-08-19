use crate::config::Config;
use crate::pacman::group::Group;
use crate::pacman::latest::Latest;
use crate::pacman::PacmanEvent;
use std::collections::HashMap;

pub trait Filter {
    type Event;
    fn without_installed<'a>(
        &'a self,
        filters: &'a Vec<String>,
    ) -> HashMap<&'a String, Vec<&'a Self::Event>>;
    fn without_removed<'a>(
        &'a self,
        filters: &'a Vec<String>,
    ) -> HashMap<&'a String, Vec<&'a Self::Event>>;

    fn filter_packages<'a>(
        &'a self,
        config: &'a Config,
    ) -> HashMap<&'a String, Vec<&'a Self::Event>>;
}

impl Filter for Vec<PacmanEvent> {
    type Event = PacmanEvent;

    fn without_installed<'a>(
        &'a self,
        filters: &'a Vec<String>,
    ) -> HashMap<&'a String, Vec<&'a PacmanEvent>> {
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

    fn without_removed<'a>(
        &'a self,
        filters: &'a Vec<String>,
    ) -> HashMap<&'a String, Vec<&'a PacmanEvent>> {
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

    fn filter_packages<'a>(
        &'a self,
        config: &'a Config,
    ) -> HashMap<&'a String, Vec<&'a Self::Event>> {
        if config.removed_only {
            self.without_installed(&config.filters)
        } else if !config.with_removed {
            self.without_removed(&config.filters)
        } else {
            self.group_relevant(&config.filters)
        }
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
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)\n[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)\n[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)\n[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)\n[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)\n[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)\n[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)\n[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)\n[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)\n[2019-05-10 12:45] [ALPM] upgraded bash (5.0.003-1 -> 5.0.007-1)"
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
            "[2019-06-23 21:09] [ALPM] upgraded linux (5.1.12.arch1-1 -> 5.1.14.arch1-1)\n[2019-06-26 12:48] [ALPM] upgraded linux (5.1.14.arch1-1 -> 5.1.15.arch1-1)\n[2019-07-08 01:01] [ALPM] upgraded linux-firmware (20190618.acb56f2-1 -> 20190628.70e4394-1)\n[2019-07-08 01:01] [ALPM] upgraded linux (5.1.15.arch1-1 -> 5.1.16.arch1-1)\n[2019-07-11 22:08] [ALPM] upgraded linux (5.1.16.arch1-1 -> 5.2.arch2-1)\n[2019-07-16 21:09] [ALPM] upgraded linux (5.2.arch2-1 -> 5.2.1.arch1-1)\n[2019-03-03 10:02] [ALPM] installed bash (5.0.0-1)\n[2019-03-16 12:57] [ALPM] upgraded bash (5.0.0-1 -> 5.0.002-1)\n[2019-04-14 21:51] [ALPM] upgraded bash (5.0.002-1 -> 5.0.003-1)\n[2019-05-10 12:45] [ALPM] upgraded bash (5.0.003-1 -> 5.0.007-1)"
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
            package: String::from("no-longer-used"),
            action: pacman::action::Action::Removed,
            from: String::from("0.0.1"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events
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
}
