use std::{error, fmt};

pub struct Error(String);

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
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
