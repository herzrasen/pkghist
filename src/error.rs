use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorDetail {
    InvalidFormat,
    InvalidAction,
    FormattingError { msg: String },
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    detail: ErrorDetail,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", &self.detail)
    }
}

impl Error {
    pub fn new(error_detail: ErrorDetail) -> Error {
        Error {
            detail: error_detail,
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Error::new(ErrorDetail::FormattingError {
            msg: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_set_the_error_message() {
        let error = Error::new(ErrorDetail::FormattingError {
            msg: String::from("This error is a test"),
        });
        assert_eq!(
            error.to_string(),
            "Error: FormattingError { msg: \"This error is a test\" }"
        )
    }

    #[test]
    fn should_set_the_error_detail() {
        let error = Error::new(ErrorDetail::InvalidFormat);
        assert_eq!(error.detail, ErrorDetail::InvalidFormat)
    }

    #[test]
    fn should_format_correctly() {
        let error = Error::new(ErrorDetail::InvalidFormat);
        let str = format!("{}", error);
        assert_eq!(str, String::from("Error: InvalidFormat"))
    }

    #[test]
    fn should_debug_format_correctly() {
        let error = Error::new(ErrorDetail::InvalidFormat);
        let str = format!("{:?}", error);
        assert_eq!(str, String::from("Error { detail: InvalidFormat }"))
    }
}
