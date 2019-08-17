use crate::config::Config;
use crate::pacman::PacmanEvent;
use std::collections::HashMap;

pub trait Group {
    type Event;
    fn group_relevant(&self, config: &Config) -> HashMap<&String, Vec<&Self::Event>>;
}

impl Group for Vec<PacmanEvent> {
    type Event = PacmanEvent;

    fn group_relevant(&self, config: &Config) -> HashMap<&String, Vec<&PacmanEvent>> {
        let mut groups: HashMap<&String, Vec<&PacmanEvent>> = HashMap::new();
        for event in self {
            if config.is_relevant_package(&event.package) {
                if groups.contains_key(&event.package) {
                    let current_pacman_events: &Vec<&PacmanEvent> =
                        groups.get(&event.package).unwrap();
                    let mut new_vec = Vec::from(current_pacman_events.as_slice());
                    new_vec.push(event);
                    groups.insert(&event.package, new_vec);
                } else {
                    let mut value = Vec::new();
                    value.push(event);
                    groups.insert(&event.package, value);
                }
            }
        }
        groups
    }
}
