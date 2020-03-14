use serde::de;
use serde::ser;
use std::fmt::{self, Debug, Display};
use std::{error, io};

#[derive(Debug)]
pub enum EncoderError {
    Io(io::Error),
    Unknown(String),
}

impl From<io::Error> for EncoderError {
    fn from(err: io::Error) -> EncoderError {
        EncoderError::Io(err)
    }
}

impl From<EncoderError> for io::Error {
    fn from(err: EncoderError) -> io::Error {
        match err {
            EncoderError::Io(e) => e,
            EncoderError::Unknown(e) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}

impl error::Error for EncoderError {
    fn description(&self) -> &str {
        match *self {
            EncoderError::Io(ref inner) => inner.description(),
            EncoderError::Unknown(ref inner) => inner,
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            EncoderError::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}

impl ser::Error for EncoderError {
    fn custom<T: Display>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.to_string())
    }
}

impl de::Error for EncoderError {
    fn custom<T: Display>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.to_string())
    }
}

impl Display for EncoderError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EncoderError::Unknown(ref s) => write!(fmt, "{}", s),
            EncoderError::Io(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

pub type EncoderResult<T> = Result<T, EncoderError>;
pub type DecoderResult<T> = Result<T, EncoderError>;
