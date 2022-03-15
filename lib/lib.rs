#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[doc(hidden)]
pub const VERSION: &str = "1.1.0*";

// Export Internal Functions
#[macro_use]
pub mod internal;

// Import Internal Functions
mod handle;
mod thread_pool;
use internal::common;
use internal::http;
use internal::path;

// The main server
mod server;
pub use self::server::Server;

// HTTP Header relates things
mod header;
pub use self::header::Header;

// Different types of requests e.g. GET, POST, PUT, DELETE
mod method;
pub use self::method::Method;

// Routing - the main way of getting things done
mod route;
pub use self::route::Route;

// A request object to hold all the information about a request
mod request;
pub use self::request::Request;

// A response object that is used to define data to send to the client
mod response;
pub use self::response::Response;

// Query string stuff
mod query;
pub use self::query::Query;

// Content Types
mod content_type;
pub use content_type::Content;

// Middleware and stuff
pub mod middleware;
pub use middleware::Middleware;

// Cookies 🍪
#[cfg(feature = "cookies")]
mod cookie;
#[cfg(feature = "cookies")]
pub use self::cookie::{Cookie, SetCookie};

// TODO: Finish Discription
/// Prelude
pub mod prelude {
    pub use crate::{
        middleware::{MiddleRequest, MiddleResponse, Middleware},
        Content, Header, Method, Request, Response, Server,
    };
}

// Extra Features
mod extensions;

#[cfg(feature = "rate_limit")]
pub use extensions::ratelimit::RateLimiter;

#[cfg(feature = "logging")]
pub use extensions::logger::{Level, Logger};

#[cfg(feature = "serve_static")]
pub use extensions::serve_static;
#[cfg(feature = "serve_static")]
pub use extensions::serve_static::ServeStatic;

// Unit Tests
#[cfg(test)]
mod tests;
