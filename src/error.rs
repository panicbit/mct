use std::error::{Error,FromError};
use std::fmt;
use std::result;
use std::io::IoError;

use self::WrappedError::*;

pub enum WrappedError {
    Simple(String),
    Wrapped(Box<Error>)
}

impl Error for WrappedError {
    fn description(&self) -> &str {
        match *self {
            Simple(ref msg) => msg.as_slice(),
            Wrapped(ref e) => e.description()
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            &Simple(_) => None,
            &Wrapped(ref e) => e.detail()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            Simple(_) => None,
            Wrapped(ref e) => e.cause()
        }
    }
}

impl fmt::Show for WrappedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Simple(ref msg) => write!(f, "{}", msg),
            Wrapped(ref e) => {
                match e.detail() {
                    Some(detail) => write!(f, "{} ({})", e.description(), detail),
                    None => write!(f, "{}", e.description())
                }
            }
        }
    }
}

impl FromError<IoError> for WrappedError {
    fn from_error(e: IoError) -> WrappedError {
        wrap_error(e)
    }
}

pub fn error(msg: &str) -> WrappedError {
    Simple(msg.to_string())
}


pub fn wrap_error<E: Error>(e: E) -> WrappedError {
    Wrapped(Box::new(e))
}

pub type Result<T> = result::Result<T, WrappedError>;
