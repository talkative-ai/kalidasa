extern crate snips_nlu_lib;
extern crate redis;
extern crate hyper;
extern crate futures;
extern crate serde_json;
extern crate url;

use std::boxed::{Box};
use std::collections::HashMap;

use futures::future::Future;
use futures::Stream;

use url::form_urlencoded;

use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};

use snips_nlu_lib::{BytesBasedConfiguration, SnipsNluEngine};

use redis::Commands;

static MISSING: &[u8] = b"Missing field";

struct Kalidasa;

const PHRASE: &'static str = "Hello, World!";

impl Service for Kalidasa {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

        let response = Response::new();

        match (req.method(), req.path()) {
            (&Method::Post, "/v1/parse") => {
                Box::new(
                    req.body().concat2().map(|b| {
                        let params = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
                        let context = if let Some(n) = params.get("context") {
                            n
                        } else {
                            return Response::new()
                                .with_body(MISSING)
                        };
                        let query = if let Some(n) = params.get("query") {
                            n
                        } else {
                            return Response::new()
                                .with_body(MISSING)
                        };

                        let config_string = if let Ok(n) = get_trained_data(context) {
                            n
                        } else {
                            return Response::new()
                                .with_body(MISSING)
                        };

                        let configuration = match BytesBasedConfiguration::new(config_string.as_bytes()) {
                            Ok(conf) => conf,
                            Err(e) => panic!(format!("{}", e)),
                        };

                        let nlu_engine = SnipsNluEngine::new(configuration).unwrap();

                        let result = nlu_engine.parse(query, None).unwrap();

                        Response::new()
                            .with_header(ContentType::json())
                            .with_body(format!("{}", serde_json::to_string(&result).unwrap()))
                    })
                )
            },
            _ => {
                Box::new(futures::future::ok(response))
            }
        }
    }
}

fn get_trained_data(key: &str) -> redis::RedisResult<String> {
    let client = try!(redis::Client::open("redis://redis/"));
    let con = try!(client.get_connection());
    con.get(key)
}


fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Kalidasa)).unwrap();
    println!("Starting Kalidasa server");
    server.run().unwrap();
}
