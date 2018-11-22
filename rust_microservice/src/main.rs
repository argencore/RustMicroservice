extern crate hyper;
extern crate futures;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate url;

use std::collections::HashMap;
use std::io;

use hyper::{Chunk, StatusCode};
use hyper::Method::{Get, Post};
use hyper::server::{Request, Response, Service};

use futures::Stream;
use futures::future::{Future, FutureResult};

struct Microservice;

impl Service for Microservice{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        match (request.method(), request.path()) {
            (&Post, "/") => {
                let future = request
                    .body()
                    .concat2()
                    .and_then(parse_form)
                    .and_then(write_to_db)
                    .then(make_post_response);
                Box::new(future)
            }
            _ => Box::new(futures::future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        }
    }
}

struct NewMessage{
    username: String,
    message: String,
}

fn parse_form(form_chunk: Chunk) -> FutureResult<NewMessage, hyper::Error>{
        let mut form = url::form_urlencoded::parse(form_chunk.as_ref())
            .into_owned()
            .collect::<HashMap<String,String>>();

        if let Some(message) = form.remove("message"){
            let username = form.remove("username").unwrap_or(String::from("anonymous"));
            futures::future::ok(NewMessage{
                username,
                message,
            })
        }else{
            futures::future::err(hyper::Error::from(io::Error::new(io::ErrorKind::InvalidInput, "missing field message",)))
        }
}

fn write_to_db(entry: NewMessage) -> FutureResult<i64, hyper::Error>{
    futures::future::ok(0)
}

fn make_post_response(result: Result<i64, hyper::Error>) -> FutureResult<hyper::Response, hyper::Error>{
    futures::future::ok(Response::new().with_status(StatusCode::NotFound))
}


fn main() {
    env_logger::init();
    let address = "127.0.0.1:8080".parse().unwrap();
    let server = hyper::server::Http::new()
        .bind(&address, || Ok(Microservice {}))
        .unwrap();
    info!("running microservice at {}", address);
    server.run().unwrap();
}
