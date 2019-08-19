use std::str::FromStr;

use crate::error::{Error, ErrorDetail};

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone)]
pub enum Action {
    Installed,
    Reinstalled,
    Upgraded,
    Removed,
}

impl Action {
    pub fn is_removed(&self) -> bool {
        *self == Action::Removed
    }

    pub fn is_installed(&self) -> bool {
        !self.is_removed()
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "upgraded" => Ok(Action::Upgraded),
            "installed" => Ok(Action::Installed),
            "reinstalled" => Ok(Action::Reinstalled),
            "removed" => Ok(Action::Removed),
            _ => Err(Error::new(ErrorDetail::InvalidAction)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_removed() {
        let removed = Action::Removed;
        assert_eq!(removed.is_removed(), true);
        assert_eq!(removed.is_installed(), false)
    }

    #[test]
    fn should_be_installed() {
        let installed = Action::Upgraded;
        assert_eq!(installed.is_installed(), true);
        assert_eq!(installed.is_removed(), false)
    }

    #[test]
    fn should_parse_action_installed() {
        let action: Action = "installed".parse().unwrap();
        assert_eq!(action, Action::Installed)
    }

    #[test]
    fn should_parse_action_reinstalled() {
        let action: Action = "reinstalled".parse().unwrap();
        assert_eq!(action, Action::Reinstalled)
    }

    #[test]
    fn should_parse_action_removed() {
        let action: Action = "removed".parse().unwrap();
        assert_eq!(action, Action::Removed)
    }

    #[test]
    fn should_parse_action_upgraded() {
        let action: Action = "upgraded".parse().unwrap();
        assert_eq!(action, Action::Upgraded)
    }

    #[test]
    fn should_not_parse_an_action() {
        let action: Error = Action::from_str("foo").err().unwrap();
        assert_eq!(action, Error::new(ErrorDetail::InvalidAction))
    }
}
