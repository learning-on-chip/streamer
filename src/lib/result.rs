use std::{error, fmt, result};

/// An error.
pub struct Error(String);

/// A result.
pub type Result<T> = result::Result<T, Error>;

impl Error {
    /// Create an error.
    #[inline]
    pub fn new<T: ToString>(message: T) -> Error {
        Error(message.to_string())
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        &self.0
    }
}
