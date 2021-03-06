use std::collections::HashMap;

use crate::pacman::PacmanEvent;
use std::hash::BuildHasher;

pub trait Newest {
    type Event;

    fn newest(&mut self) -> &Self::Event;
}

impl Newest for Vec<&PacmanEvent> {
    type Event = PacmanEvent;

    fn newest(&mut self) -> &PacmanEvent {
        self.sort();
        self.last().unwrap()
    }
}

pub fn select_newest<'a, S: BuildHasher>(
    groups: HashMap<&'a String, Vec<&'a PacmanEvent>, S>,
) -> HashMap<&'a String, PacmanEvent> {
    let mut newest = HashMap::new();
    for (package, mut pacman_events) in groups {
        let newest_event = pacman_events.newest().clone();
        newest.insert(package, newest_event);
    }
    newest
}

#[cfg(test)]
mod tests {
    use Newest;

    use crate::pacman::PacmanEvent;

    use super::*;

    #[test]
    fn should_select_newest() {
        let p1: PacmanEvent = "[2019-05-23 07:00] [ALPM] installed intellij-idea (2:2019.1.2-1)"
            .parse()
            .unwrap();
        let p2: PacmanEvent =
            "[2019-05-29 22:25] [ALPM] upgraded intellij-idea (2:2019.1.2-1 -> 2:2019.1.3-1)"
                .parse()
                .unwrap();
        let p3: PacmanEvent =
            "[2019-07-25 01:17] [ALPM] upgraded intellij-idea (2:2019.1.3-1 -> 2:2019.1.3-2)"
                .parse()
                .unwrap();
        let p4: PacmanEvent =
            "[2019-07-25 23:38] [ALPM] upgraded intellij-idea (2:2019.1.3-2 -> 2:2019.2-1)"
                .parse()
                .unwrap();

        let mut pacman_events = [&p4, &p2, &p1, &p3].to_vec();
        let latest = pacman_events.newest();
        assert_eq!(latest, &p4)
    }

    #[test]
    fn should_select_newest_for_each_package() {
        let p1: PacmanEvent = "[2019-05-23 07:00] [ALPM] installed intellij-idea (2:2019.1.2-1)"
            .parse()
            .unwrap();
        let p2: PacmanEvent =
            "[2019-05-29 22:25] [ALPM] upgraded intellij-idea (2:2019.1.2-1 -> 2:2019.1.3-1)"
                .parse()
                .unwrap();
        let p3: PacmanEvent =
            "[2019-07-25 01:17] [ALPM] upgraded intellij-idea (2:2019.1.3-1 -> 2:2019.1.3-2)"
                .parse()
                .unwrap();
        let p4: PacmanEvent =
            "[2019-07-25 23:38] [ALPM] upgraded intellij-idea (2:2019.1.3-2 -> 2:2019.2-1)"
                .parse()
                .unwrap();

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

        let mut groups = HashMap::new();
        let intellij_package = String::from("intellij-idea");
        let intellij_events = [&p3, &p1, &p4, &p2].to_vec();
        groups.insert(&intellij_package, intellij_events);

        let linux_package = String::from("linux");
        let linux_events = [&p8, &p5, &p7, &p6].to_vec();
        groups.insert(&linux_package, linux_events);

        let latest = select_newest(groups);
        assert_eq!(latest.get(&intellij_package), Some(&p4));
        assert_eq!(latest.get(&linux_package), Some(&p8))
    }
}
