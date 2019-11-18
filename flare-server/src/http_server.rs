// This example serves the docs from `target/doc/`.
//
// Run `cargo doc && cargo run --example doc_server`, then
// point your browser to http://localhost:3000/

use futures::{future, Async::*, Future, Poll};
use http::response::Builder as ResponseBuilder;
use http::{header, Request, Response, StatusCode};
use hyper::Body;
use hyper_staticfile::{Static, StaticFuture};
use std::io::Error;
use std::path::Path;

/// Future returned from `MainService`.
enum MainFuture {
    Root,
    Static(StaticFuture<Body>),
}

impl Future for MainFuture {
    type Item = Response<Body>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            MainFuture::Root => {
                let res = ResponseBuilder::new()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, "/index.html")
                    .body(Body::empty())
                    .expect("unable to build response");
                Ok(Ready(res))
            }
            MainFuture::Static(ref mut future) => future.poll(),
        }
    }
}

/// Hyper `Service` implementation that serves all requests.
struct MainService {
    static_: Static,
}

impl MainService {
    fn new(static_dir: &str) -> MainService {
        MainService {
            static_: Static::new(Path::new(static_dir)),
        }
    }
}

impl hyper::service::Service for MainService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = MainFuture;

    fn call(&mut self, req: Request<Body>) -> MainFuture {
//        if req.uri().path() == "/" {
//            MainFuture::Root
//        } else {
//            MainFuture::Static(self.static_.serve(req))
//        }
        MainFuture::Static(self.static_.serve(req))
    }
}

pub struct SimpleHttpServer {

}

impl SimpleHttpServer {
    pub fn start_server(){

        let mut static_dir = "static/";
        if let Ok(r) = std::fs::read_dir("res/static/") {
            static_dir = "res/static/";
        }
        println!("http static dir: {}", static_dir);

        let addr = ([0, 0, 0, 0], 3890).into();
        match hyper::Server::try_bind(&addr) {
            Ok(builder) => {
                let server = builder
                    .serve(move || future::ok::<_, Error>(MainService::new(static_dir)))
                    .map_err(|e| eprintln!("server error: {}", e));
                println!("Http server running on http://127.0.0.1:{}/", addr.port());
                println!("Simpleui: http://127.0.0.1:{}/simpleui/", addr.port());
                hyper::rt::run(server);
            },
            Err(e) => {
                println!("Start flare web server failed, bind addr: {}, error: {}", addr, e);
            }
        }

    }

}


// Application entry point.
//fn main() {
//    let addr = ([127, 0, 0, 1], 3000).into();
//    let server = hyper::Server::bind(&addr)
//        .serve(|| future::ok::<_, Error>(MainService::new()))
//        .map_err(|e| eprintln!("server error: {}", e));
//    eprintln!("Doc server running on http://{}/", addr);
//    hyper::rt::run(server);
//}