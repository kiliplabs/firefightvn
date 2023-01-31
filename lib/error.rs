//! Errors that can occur in the process of connectioning to clients, parseing HTTP and handling requests.

use std::result;

use crate::{Method, Request};

/// Easy way to use a Result<T, Error>
pub type Result<T> = result::Result<T, Error>;

/// Errors that can occur,,,
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Stream error
    Stream(StreamError),

    /// Error while handling a Request
    Handle(Box<HandleError>),

    /// Error while parsing request HTTP
    Parse(ParseError),

    /// IO Errors
    Io(String),

    /// Response does not exist (probably because of an error with the request)
    None,
}

/// Errors thet can arize while handling a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandleError {
    /// Route matching request path not found
    NotFound(Method, String),

    /// A route or middleware paniced while running
    Panic(Box<Result<Request>>, String),
}

/// Error that can occur while parsing the HTTP of a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// No `\r\n\r\n` found in request to separate metadata from body
    NoSeparator,

    /// No Method found in request HTTP
    NoMethod,

    /// No Path found in request HTTP
    NoPath,

    /// No Version found in request HTTP
    NoVersion,

    /// No Request Line found in HTTP
    NoRequestLine,

    /// Invalid Query in Path
    InvalidQuery,

    /// Invalid Method in Request HTTP
    InvalidMethod,

    /// Invalid Header in Request HTTP
    InvalidHeader,
}

/// Error that can occur while reading or writing to a stream
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// The stream ended unexpectedly
    UnexpectedEof,
}

impl From<StreamError> for Error {
    fn from(e: StreamError) -> Self {
        Error::Stream(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<HandleError> for Error {
    fn from(e: HandleError) -> Self {
        Error::Handle(Box::new(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}
