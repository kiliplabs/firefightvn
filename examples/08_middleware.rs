use afire::{
    internal::common::remove_address_port,
    middleware::{MiddleRequest, Middleware},
    Header, Method, Request, Response, Server,
};

// In afire Middleware is a trait that is implemented and can modify / overwrite Requests and Response
// The Middleware functions for this are `pre` and `post` for before and after the routes
// These functions will return a MiddleResponse / MiddleRequest which is an enum with the following
//  - `Continue` to not affect the Request / Response
//  - `Add(Request | Response)` to modify the Request / Response and continue to run Middleware (if any)
//  - `Send(Response)` to send a Response to the client immediately
// Just like routes, middleware is executed in reverse order that they are defined.
// So the most recently defined middleware is executed first.

// You could use middleware to log requests, check if a user is logged in, Implement RateLimiting, add Analytics, etc.
// In this example, we will make a very simple Request Logger

// First we define the Struct to Implement Middleware on
// It can have values, but in this case that is not needed
struct Log;

// Now we will Implement Middleware for Log
impl Middleware for Log {
    // Redefine the `pre` function
    // (Runs before Routes)
    fn pre(&mut self, req: Request) -> MiddleRequest {
        // Print some info
        println!(
            "[{}] {} {}",
            remove_address_port(req.address),
            req.method,
            req.path
        );
        // Note: req.address also has the client port
        // This is being removed with
        // Ex: 127.0.0.1:6264 => 127.0.0.1

        // Continue to forward the request to the next middleware or route
        MiddleRequest::Continue
    }
}

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a basic route
    server.route(Method::GET, "/", |_req| {
        Response::new()
            .status(200)
            .text("Hello World!")
            .header(Header::new("Content-Type", "text/plain"))
    });

    // Here is where we will attach our Middleware to the Server
    // This is super easy
    Log.attach(&mut server);

    // You can now goto http://localhost:8080 you should see that the request is printed to the console
    // It should look something like this: `[127.0.0.1:xxxxx] GET `

    println!("[08] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
