use std::{
    cell::RefCell,
    io::Read,
    net::{Shutdown, TcpStream},
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    context::ContextFlag,
    error::{HandleError, ParseError, StreamError},
    internal::sync::{ForceLockMutex, ForceLockRwLock},
    response::ResponseFlag,
    socket::Socket,
    trace, Content, Context, Error, Request, Response, Server, Status,
};

pub(crate) type Writeable = Box<RefCell<dyn Read + Send>>;

// https://open.spotify.com/track/50txng2W8C9SycOXKIQP0D

pub(crate) fn handle<State>(stream: TcpStream, this: Arc<Server<State>>)
where
    State: 'static + Send + Sync,
{
    trace!(Level::Debug, "Opening socket {:?}", stream.peer_addr());
    stream.set_read_timeout(this.socket_timeout).unwrap();
    stream.set_write_timeout(this.socket_timeout).unwrap();
    let stream = Arc::new(Socket::new(stream));
    loop {
        let mut keep_alive = false;
        let req = Request::from_socket(stream.clone());

        if let Ok(req) = &req {
            keep_alive = req.keep_alive();
            trace!(
                Level::Debug,
                "{} {} {{ keep_alive: {} }}",
                req.method,
                req.path,
                keep_alive
            );
        }

        let req = match req.map(Arc::new) {
            Ok(req) => req,
            Err(e) => {
                if let Err(e) =
                    error_response(&e, this.clone()).write(stream, &this.default_headers)
                {
                    trace!(Level::Debug, "Error writing error response: {:?}", e);
                }
                return;
            }
        };

        // Handle Route
        let ctx = Context::new(this.clone(), req.clone());
        let mut flag = ResponseFlag::None;
        let mut matched = false;

        for route in this.routes.iter().rev() {
            if let Some(params) = route.matches(req.clone()) {
                matched = true;
                *req.path_params.force_write() = params;
                let result = (route.handler)(&ctx);

                if let Err(e) = result {
                    unimplemented!("Route error {e:?}");
                }

                flag = ctx.response.force_lock().flag;
                let has_response = ctx.flags.get(ContextFlag::ResponseSent);
                if has_response {
                    break;
                }

                if ctx.flags.get(ContextFlag::GuaranteedSend) {
                    let barrier = ctx.req.socket.barrier.clone();
                    barrier.force_read().as_ref().unwrap().wait();
                    break;
                }

                let mut res = Response::new()
                    .status(Status::NotImplemented)
                    .text("No response was sent");
                if let Err(e) = res.write(req.socket.clone(), &this.default_headers) {
                    trace!(
                        Level::Debug,
                        "Error writing 'No Response' response: {:?}",
                        e
                    );
                }
                break;
            }
        }

        // let route = this
        //     .routes
        //     .iter()
        //     .rev()
        //     .find_map(|route| route.matches(req.clone()).map(|x| (route, x)));

        if !matched {
            let mut res = Response::new()
                .status(Status::NotFound)
                .text(format!("Cannot {} {}", req.method, req.path))
                .content(Content::TXT);
            if let Err(e) = res.write(req.socket.clone(), &this.default_headers) {
                trace!(Level::Debug, "Error writing 'Not Found' response: {:?}", e);
            }
        }

        if flag == ResponseFlag::End {
            trace!(Level::Debug, "Ending socket");
            break;
        }

        if !keep_alive || flag == ResponseFlag::Close || !this.keep_alive {
            trace!(Level::Debug, "Closing socket");
            if let Err(e) = stream.lock().unwrap().shutdown(Shutdown::Both) {
                trace!(Level::Debug, "Error closing socket: {:?}", e);
            }
            break;
        }
    }
}

/// Gets a response if there is an error.
/// Can handle Parse, Handle and IO errors.
pub fn error_response<State>(err: &Error, server: Arc<Server<State>>) -> Response
where
    State: 'static + Send + Sync,
{
    match err {
        Error::None | Error::Startup(_) => {
            unreachable!("None and Startup errors should not be here")
        }
        Error::Stream(e) => match e {
            StreamError::UnexpectedEof => Response::new().status(400).text("Unexpected EOF"),
        },
        Error::Parse(e) => Response::new().status(400).text(match e {
            ParseError::NoSeparator => "No separator",
            ParseError::NoMethod => "No method",
            ParseError::NoPath => "No path",
            ParseError::NoVersion => "No HTTP version",
            ParseError::NoRequestLine => "No request line",
            ParseError::InvalidQuery => "Invalid query",
            ParseError::InvalidHeader => "Invalid header",
            ParseError::InvalidMethod => "Invalid method",
        }),
        Error::Handle(e) => match e.deref() {
            HandleError::NotFound(method, path) => Response::new()
                .status(Status::NotFound)
                .text(format!("Cannot {method} {path}"))
                .content(Content::TXT),
            HandleError::Panic(r, e) => {
                (server.error_handler)(server.state.clone(), r, e.to_owned())
            }
        },
        Error::Io(e) => Response::new().status(500).text(e),
    }
}
