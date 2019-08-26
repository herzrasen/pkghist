use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use serde_json;

use crate::error::Error;
use crate::opt::Config;
use crate::opt::Format;
use crate::pacman;
use crate::pacman::action::Action;
use crate::pacman::filter::Filter;
use crate::pacman::PacmanEvent;
use itertools::Itertools;
use std::io::stdout;
use std::io::Write;
use termion::color;

pub fn run(config: Config) -> Result<(), Error> {
    let logfile_path = &config.logfile;
    let pacman_events = pacman::from_file(Path::new(logfile_path))
        .unwrap_or_else(|_| panic!("Unable to open {}", logfile_path));

    let groups = pacman_events.filter_packages(&config);

    let mut package_histories = Vec::new();

    let sorted: Vec<Vec<&PacmanEvent>> = groups
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
        .map(|(_, e)| e.clone())
        .collect();

    for mut events in sorted {
        events.sort();
        let package_history = from_pacman_events(events);
        package_histories.push(package_history);
    }

    match config.format.print(&package_histories) {
        _ => Ok(()),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
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
    writeln!(stdout(), "{}", json)?;
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
                        Action::Removed => write!(stdout(), "{red}", red = color::Fg(color::Red))?,
                        _ => write!(stdout(), "{green}", green = color::Fg(color::Green))?,
                    }
                }
                None => (),
            }
            writeln!(
                stdout(),
                "{package}{reset}",
                package = package_history.p,
                reset = color::Fg(color::Reset)
            )?
        } else {
            writeln!(stdout(), "{}", package_history.p)?
        }
        for event in &package_history.e {
            if with_colors {
                match event.a.parse().unwrap() {
                    Action::Removed => write!(stdout(), "{red}", red = color::Fg(color::Red))?,
                    _ => (), // no coloring in the default case
                };
                writeln!(
                    stdout(),
                    "  [{date}] {action}",
                    date = event.d,
                    action = event.a,
                )?;
                writeln!(
                    stdout(),
                    "    {version}{reset}",
                    version = event.v,
                    reset = color::Fg(color::Reset)
                )?
            } else {
                writeln!(
                    stdout(),
                    "  [{date}] {action}",
                    date = event.d,
                    action = event.a
                )?;
                writeln!(stdout(), "    {version}", version = event.v)?
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

    use super::*;

    #[test]
    fn should_be_ok_1() {
        let file_name = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(&file_name).unwrap();
        writeln!(
            file,
            "[2019-07-14 21:33] [PACMAN] synchronizing package lists
[2019-07-14 21:33] [PACMAN] starting full system upgrade
[2019-07-14 21:33] [ALPM] transaction started
[2019-07-14 21:33] [ALPM] installed feh (3.1.3-1)
[2019-07-14 21:33] [ALPM] upgraded libev (4.25-1 -> 4.27-1)
[2019-07-14 21:33] [ALPM] upgraded iso-codes (4.2-1 -> 4.3-1)"
        )
        .unwrap();

        let mut config = Config::new();
        config.logfile = file_name;

        let result = run(config);
        assert_eq!(result.is_ok(), true);
        fs::remove_file(file.path().unwrap()).unwrap()
    }

}
