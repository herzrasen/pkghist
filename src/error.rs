use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorDetail {
    IOError { msg: String },
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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::new(ErrorDetail::IOError {
            msg: format!("{:?} -> {}", error.kind(), error.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn should_convert_the_fmt_error() {
        let e = std::fmt::Error::default();
        let error = Error::from(e);
        assert_eq!(
            error.detail,
            ErrorDetail::FormattingError {
                msg: String::from("an error occurred when formatting an argument")
            }
        )
    }

    #[test]
    fn should_convert_the_io_error() {
        let e = std::io::Error::new(ErrorKind::BrokenPipe, "Pipe is broken");
        let error = Error::from(e);
        assert_eq!(
            error.detail,
            ErrorDetail::IOError {
                msg: String::from("BrokenPipe -> Pipe is broken")
            }
        )
    }

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
