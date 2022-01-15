use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
#[macro_use]
extern crate lazy_static;

mod settings;

lazy_static! {
    static ref CONFIG: settings::Settings =
        settings::Settings::new().expect("config can't be loaded");
}

fn debug_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_str = format!("{:?}", req);
    println!("{}", body_str);
    let response = Response::new(Body::from(body_str));
    return Ok(response);
}

#[derive(Clone)]
struct AppContext {
    // Whatever data your application needs can go here
}

async fn handle_inner(
    context: AppContext,
    addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("Hello World")))
}

async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.uri().path().starts_with("/rpb-inspect-request") {
        debug_request(req)
    }
    else if req.uri().path().starts_with("/target/first") {
        // will forward requests to port 13901
        match hyper_reverse_proxy::call(client_ip, "https://httpbin.org", "/target/first", req).await {
            Ok(response) => {Ok(response)}
            Err(error) => {Ok(Response::builder()
                                  .status(StatusCode::INTERNAL_SERVER_ERROR)
                                  .body(Body::empty())
                                  .unwrap())}
        }
    } else if req.uri().path().starts_with("/target/second") {
        // will forward requests to port 13902
        match hyper_reverse_proxy::call(client_ip, "https://httpbin.org/headers", "/target/second", req).await {
            Ok(response) => {Ok(response)}
            Err(error) => {Ok(Response::builder()
                                  .status(StatusCode::INTERNAL_SERVER_ERROR)
                                  .body(Body::empty())
                                  .unwrap())}
        }
    }
    else {
        // will forward requests to port 13902
        match hyper_reverse_proxy::call(client_ip, "https://httpbin.org", "", req).await {
            Ok(response) => {Ok(response)}
            Err(error) => {Ok(Response::builder()
                                  .status(StatusCode::INTERNAL_SERVER_ERROR)
                                  .body(Body::empty())
                                  .unwrap())}
        }
    }
    //  else {
    //     debug_request(req)
    // }
}

#[tokio::main]
async fn main() {
    let context = AppContext {
        // ...
    };

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 13900).into();

    // A `Service` is needed for every connection.
    // A `MakeService` that produces a `Service` to handle each connection.
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();

        // You can grab the address of the incoming connection like so.
        let remote_addr = conn.remote_addr();

        // Create a `Service` for responding to the request.
        // let service = service_fn(move |req| {
        //     handle(context.clone(), addr, req)
        // });
        let service = service_fn(move |req: Request<Body>| {
            handle(remote_addr.ip(), req)
        });

        // Return the service to hyper.
        async move { Ok::<_, Error>(service) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    // .serve(make_svc)
    // .map_err(|e| eprintln!("server error: {}", e));

    println!("Running server on {:?}", addr);

    // Run this server for... forever!
    // hyper::rt::run(server);
}
