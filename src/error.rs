use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    InvalidArgument,
    NoSuchElement,
    UnknownType,
    IoError(io::ErrorKind),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidArgument => write!(f, "Invalid argument."),
            Error::NoSuchElement => write!(f, "No such element."),
            Error::UnknownType => write!(f, "Unknown type."),
            Error::IoError(ref cause) => write!(f, "Io Error: {}", cause),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error.kind())
    }
}