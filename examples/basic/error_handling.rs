use std::sync::atomic::{AtomicUsize, Ordering};

use afire::{Content, Method, Response, Server, Status};

use crate::Example;

// You can run this example with `cargo run --example basic -- error_handling`

// Don't crash thread from a panic in a route
// This does not apply to the error handler itself
// afire will catch any panic in a route and return a 500 error by default

pub struct ErrorHandling;

impl Example for ErrorHandling {
    fn name(&self) -> &'static str {
        "error_handling"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a route that will panic
        server.route(Method::GET, "/panic", |_req| panic!("This is a panic!"));

        // Give the server a main page
        server.route(Method::GET, "/", |_req| {
            Response::new()
                .text(r#"<a href="/panic">PANIC</a>"#)
                .content(Content::HTML)
        });

        // You can optionally define a custom error handler
        // This can be defined anywhere in the server and will take affect for all routes
        // Its like a normal route, but it will only be called if the route panics
        let errors = AtomicUsize::new(1);
        server.error_handler(move |_state, _req, err| {
            Response::new()
                .status(Status::InternalServerError)
                .text(format!(
                    "<h1>Internal Server Error #{}</h1><br>Panicked at '{}'",
                    errors.fetch_add(1, Ordering::Relaxed),
                    err
                ))
                .content(Content::HTML)
        });

        // You can now goto http://localhost:8080/panic
        // This will cause the route to panic and return a 500 error

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
