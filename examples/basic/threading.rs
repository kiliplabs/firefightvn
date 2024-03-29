use std::num::NonZeroU64;
use std::thread;

use afire::{Content, Method, Response, Server};

use crate::Example;

// You can run this example with `cargo run --example basic -- threading`

// Create a new basic server like in example 01
// However, we want to use a thread pool to handle the requests

// In production, you would probably want to use a reverse proxy like nginx
// or something similar to split the load across multiple servers if you have a lot of traffic
// But just a thread pool is a good way to get started

pub struct Threading;

impl Example for Threading {
    fn name(&self) -> &'static str {
        "threading"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a handler for GET "/"
        server.route(Method::GET, "/", |_req| {
            Response::new()
                // hopefully the ThreadId.as_u64 method will become stable
                // until then im stuck with this mess for the example
                // It just gets the thread ID to show the user what thread is handling the request
                .text(format!(
                    "Hello from thread number {:#?}!",
                    unsafe { std::mem::transmute::<_, NonZeroU64>(thread::current().id()) }.get()
                        - 1
                ))
                .content(Content::TXT)
        });

        // Start the server with 8 threads
        // This will block the current thread
        server.start_threaded(8).unwrap();

        // If you go to http://localhost:8080 you should see the thread ID change with each refresh
    }
}
