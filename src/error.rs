use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    InvalidArgument(&'static str),
    IoError(io::ErrorKind),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidArgument(arg) => write!(f, "Invalid argument: {}", arg),
            Error::IoError(ref cause) => write!(f, "Io Error: {}", cause),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error.kind())
    }
}
