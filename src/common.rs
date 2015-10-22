use std::error::Error;
use std::fmt;
use std::result;
use std::io;
use ::augeas;

use self::WrappedError::*;

#[derive(Debug)]
pub enum WrappedError {
    Simple(String),
    Wrapped(Box<Error>)
}

impl Error for WrappedError {
    fn description(&self) -> &str {
        match *self {
            Simple(ref msg) => &msg,
            Wrapped(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            Simple(_) => None,
            Wrapped(ref e) => e.cause()
        }
    }
}

impl fmt::Display for WrappedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Simple(ref msg) => write!(f, "{}", msg),
            Wrapped(ref e) => write!(f, "{}", e.description())
        }
    }
}

impl From<io::Error> for WrappedError {
    fn from(e: io::Error) -> WrappedError {
        wrap_error(e)
    }
}

impl From<augeas::Error> for WrappedError {
    fn from(e: augeas::Error) -> WrappedError {
        wrap_error(e)
    }
}

pub fn error<S: Into<String>>(msg: S) -> WrappedError {
    Simple(msg.into())
}


pub fn wrap_error<E: Error + 'static>(e: E) -> WrappedError {
    Wrapped(Box::new(e))
}

pub type Result<T> = result::Result<T, WrappedError>;
