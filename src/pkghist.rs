use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use serde_json;

use crate::config::Config;
use crate::config::Format;
use crate::error::Error;
use crate::pacman;
use crate::pacman::action::Action;
use crate::pacman::filter::Filter;
use crate::pacman::PacmanEvent;
use std::io::Write;
use termion::color;

pub fn run(config: Config) -> Result<(), Error> {
    let logfile_path = &config.logfile;
    let pacman_events = pacman::from_file(Path::new(logfile_path))
        .unwrap_or_else(|_| panic!("Unable to open {}", logfile_path));

    let groups = pacman_events.filter_packages(&config);

    groups.iter().for_each(|g| log::debug!("{:?}", g));

    let mut package_histories = Vec::new();

    for (_, mut events) in groups {
        events.sort();
        let package_history = from_pacman_events(events);
        package_histories.push(package_history);
    }
    package_histories.sort_by(|h1, h2| h1.p.cmp(&h2.p));

    config.format.print(&package_histories)?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub v: String,
    pub d: String,
    pub a: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageHistory {
    pub p: String,
    pub e: Vec<Event>,
}

fn from_pacman_events(pacman_events: Vec<&PacmanEvent>) -> PackageHistory {
    let e: Vec<Event> = pacman_events
        .iter()
        .map(|e| Event {
            v: e.printable_version(),
            d: e.date.to_string(),
            a: e.action.to_string(),
        })
        .collect();
    let p = pacman_events.first().unwrap().package.clone();
    PackageHistory { p, e }
}

fn format_json(packages_with_version: &Vec<PackageHistory>) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(packages_with_version).unwrap();
    writeln!(std::io::stdout(), "{}", json).unwrap();
    Ok(())
}

fn format_plain(package_histories: &Vec<PackageHistory>, with_colors: bool) -> Result<(), Error> {
    for package_history in package_histories {
        if with_colors {
            // check if last event is a removal
            match package_history.e.last() {
                Some(last_event) => {
                    let last_action: Action = last_event.a.parse().unwrap();
                    match last_action {
                        Action::Removed => print!("{red}", red = color::Fg(color::Red)),
                        _ => print!("{green}", green = color::Fg(color::Green)),
                    }
                }
                None => (),
            }
            println!(
                "{package}{reset}",
                package = package_history.p,
                reset = color::Fg(color::Reset)
            )
        } else {
            println!("{}", package_history.p)
        }
        for event in &package_history.e {
            if with_colors {
                match event.a.parse().unwrap() {
                    Action::Removed => print!("{red}", red = color::Fg(color::Red)),
                    _ => (), // no coloring in the default case
                };
                println!(
                    "  [{date}] {action}{reset}",
                    date = event.d,
                    action = event.a,
                    reset = color::Fg(color::Reset)
                )
            } else {
                println!("  [{date}] {action}", date = event.d, action = event.a,)
            }
        }
    }
    Ok(())
}

trait Printer {
    fn print(&self, package_histories: &Vec<PackageHistory>) -> Result<(), Error>;
}

impl Printer for Format {
    fn print(&self, package_histories: &Vec<PackageHistory>) -> Result<(), Error> {
        match *self {
            Format::Plain { with_colors } => format_plain(package_histories, with_colors),
            Format::Json => format_json(package_histories),
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
            format: Format::Plain { with_colors: true },
            no_colors: false,
        };
        let result = run(config);
        assert_eq!(result.is_ok(), true);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

}
