use std::collections::HashMap;

use itertools::Itertools;

use crate::opt::Direction;
use crate::pacman::PacmanEvent;
use std::hash::BuildHasher;

pub fn range<'a, S: BuildHasher + Default>(
    grouped: &HashMap<&'a String, Vec<&'a PacmanEvent>, S>,
    direction: &Option<Direction>,
) -> HashMap<&'a String, Vec<&'a PacmanEvent>> {
    let sorted: Vec<&String> = grouped
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
        .map(|(p, _)| *p)
        .unique()
        .collect();

    let filters = match direction {
        Some(Direction::Forwards { n }) => sorted.into_iter().take(*n).collect(),
        Some(Direction::Backwards { n }) => sorted.into_iter().rev().take(*n).collect(),
        None => sorted,
    };

    let mut filtered = HashMap::new();
    grouped
        .iter()
        .filter(|(p, _)| filters.contains(*p))
        .for_each(|(p, e)| {
            filtered.insert(*p, e.clone());
        });
    filtered
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::pacman::action::Action;
    use crate::pacman::group::Group;
    use crate::pacman::PacmanEvent;

    use super::*;

    fn some_pacman_events() -> Vec<PacmanEvent> {
        let mut pacman_events = Vec::new();
        pacman_events.push(PacmanEvent {
            package: String::from("some-package"),
            action: Action::Installed,
            from: String::from("0.0.1"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: Action::Installed,
            from: String::from("0.0.2"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: Action::Upgraded,
            from: String::from("0.0.2"),
            to: Some(String::from("0.0.3")),
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: Action::Removed,
            from: String::from("0.0.2"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events.push(PacmanEvent {
            package: String::from("another-package"),
            action: Action::Installed,
            from: String::from("0.0.2"),
            to: None,
            date: Utc::now().naive_local(),
        });

        pacman_events.push(PacmanEvent {
            package: String::from("no-longer-used"),
            action: Action::Removed,
            from: String::from("0.0.1"),
            to: None,
            date: Utc::now().naive_local(),
        });
        pacman_events
    }

    #[test]
    fn should_get_last_n_packages() {
        // given
        let pacman_events = some_pacman_events();
        let group = pacman_events.group();

        // when
        let filtered = range(&group, &Some(Direction::Backwards { n: 2 }));

        // then
        assert_eq!(filtered.keys().len(), 2);
        assert!(filtered.get(&String::from("another-package")).is_some());
        assert!(filtered.get(&String::from("no-longer-used")).is_some())
    }
}
