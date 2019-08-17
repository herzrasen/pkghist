use crate::pacman::PacmanEvent;
use std::collections::HashMap;

pub trait Latest {
    type Event;

    fn latest(&mut self) -> &Self::Event;
}

impl Latest for Vec<&PacmanEvent> {
    type Event = PacmanEvent;

    fn latest(&mut self) -> &PacmanEvent {
        self.sort();
        self.last().unwrap()
    }
}

pub fn select_latest<'a>(
    groups: HashMap<&'a String, Vec<&'a PacmanEvent>>,
) -> HashMap<&'a String, PacmanEvent> {
    let mut latest = HashMap::new();
    for (package, mut pacman_events) in groups {
        let latest_event = pacman_events.latest().clone();
        latest.insert(package, latest_event);
    }
    latest
}
