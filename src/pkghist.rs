use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use serde_json;

use crate::config::Config;
use crate::config::Format;
use crate::error::{Error, ErrorDetail};
use crate::pacman;
use crate::pacman::filter::Filter;
use crate::pacman::PacmanEvent;

pub fn run(config: Config) -> Result<(), Error> {
    let logfile_path = &config.logfile;
    let pacman_events = pacman::from_file(Path::new(logfile_path))
        .unwrap_or_else(|_| panic!("Unable to open {}", logfile_path));

    let groups = pacman_events.filter_packages(&config);

    groups.iter().for_each(|g| log::debug!("{:?}", g));

    let mut history_entries = Vec::new();

    for (_, mut events) in groups {
        events.sort();
        let history_entry = from_pacman_events(events);
        history_entries.push(history_entry);
    }
    history_entries.sort_by(|h1, h2| h1.p.cmp(&h2.p));

    history_entries.print(config.format)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub v: String,
    pub d: String,
    pub a: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub p: String,
    pub e: Vec<Event>,
}

fn from_pacman_events(pacman_events: Vec<&PacmanEvent>) -> HistoryEntry {
    let e: Vec<Event> = pacman_events
        .iter()
        .map(|e| Event {
            v: e.printable_version(),
            d: e.date.to_string(),
            a: e.action.to_string(),
        })
        .collect();
    let p = pacman_events.first().unwrap().package.clone();
    HistoryEntry { p, e }
}

fn format_json(packages_with_version: &Vec<HistoryEntry>) -> Result<String, Error> {
    match serde_json::to_string_pretty(packages_with_version) {
        Ok(json) => Ok(json),
        Err(e) => Err(Error::new(ErrorDetail::FormattingError {
            msg: e.to_string(),
        })),
    }
}

fn format_plain(history_entries: &Vec<HistoryEntry>) -> Result<String, Error> {
    let mut plain = String::new();
    for history_entry in history_entries {
        plain.push_str(format!("{}\n", history_entry.p).as_str());
        for event in &history_entry.e {
            plain.push_str(format!("  [{}] {}\n    {}\n", event.d, event.a, event.v).as_str());
        }
    }
    Ok(plain)
}

pub trait PactraceFormatter {
    fn format(&self, format: Format) -> Result<String, Error>;

    fn print(&self, format: Format) -> Result<(), Error> {
        match self.format(format) {
            Ok(format_str) => {
                println!("{}", format_str);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error formatting output: {:?}", e);
                Err(e)
            }
        }
    }
}

impl PactraceFormatter for Vec<HistoryEntry> {
    fn format(&self, format: Format) -> Result<String, Error> {
        match format {
            Format::Json => format_json(self),
            Format::Plain => format_plain(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    use filepath::FilePath;

    use crate::config::Format;

    use super::*;

    #[test]
    fn should_be_ok_1() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-07-14 21:33] [PACMAN] synchronizing package lists\n[2019-07-14 21:33] [PACMAN] starting full system upgrade\n[2019-07-14 21:33] [ALPM] transaction started\n[2019-07-14 21:33] [ALPM] installed feh (3.1.3-1)\n[2019-07-14 21:33] [ALPM] upgraded libev (4.25-1 -> 4.27-1)\n[2019-07-14 21:33] [ALPM] upgraded iso-codes (4.2-1 -> 4.3-1)"
        )
            .unwrap();

        let config = Config {
            with_removed: false,
            removed_only: false,
            logfile: file_name,
            filters: Vec::new(),
            format: Format::Plain,
        };
        let result = run(config);
        assert_eq!(result.is_ok(), true);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

}
