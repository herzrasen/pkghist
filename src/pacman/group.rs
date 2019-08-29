use crate::pacman::PacmanEvent;
use std::collections::HashMap;

pub trait Group {
    type Event;
    fn group(&self) -> HashMap<&String, Vec<&Self::Event>>;
}

impl Group for Vec<PacmanEvent> {
    type Event = PacmanEvent;

    fn group(&self) -> HashMap<&String, Vec<&PacmanEvent>> {
        let mut groups: HashMap<&String, Vec<&PacmanEvent>> = HashMap::new();
        for event in self {
            if groups.contains_key(&event.package) {
                let current_pacman_events: &Vec<&PacmanEvent> = groups.get(&event.package).unwrap();
                let mut new_vec = Vec::from(current_pacman_events.as_slice());
                new_vec.push(event);
                groups.insert(&event.package, new_vec);
            } else {
                let mut value = Vec::new();
                value.push(event);
                groups.insert(&event.package, value);
            }
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Group;

    #[test]
    fn should_group_relevant() {
        let p1: PacmanEvent = "[2019-01-01 00:00] [ALPM] installed a (1.0.0)"
            .parse()
            .unwrap();
        let p2: PacmanEvent = "[2019-01-01 00:00] [ALPM] installed b (1.0.0)"
            .parse()
            .unwrap();
        let p3: PacmanEvent = "[2019-01-02 00:00] [ALPM] upgraded b (1.0.1)"
            .parse()
            .unwrap();
        let p4: PacmanEvent = "[2019-01-02 00:00] [ALPM] installed c (1.0.0)"
            .parse()
            .unwrap();

        let pacman_events = [p1, p2, p3, p4].to_vec();

        let groups = pacman_events.group();
        assert_eq!(groups.keys().len(), 3)
    }

}
