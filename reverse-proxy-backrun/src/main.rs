use ethers::prelude::{k256, Signer, Wallet};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
// use std::sync::{Arc, RwLock};
use tokio::sync::*;

#[macro_use]
extern crate lazy_static;

mod backrun;
mod subscriber;

lazy_static! {
    static ref CONFIG: settings_mod::settings::Settings =
        settings_mod::settings::Settings::new().expect("config can't be loaded");
}

pub mod settings_mod {
    pub mod settings;
    // mod utils; // because it's not `pub` it won't be visible outside of `settings_mod`
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
    // provider: Provider<Http>,
    provider_url: String,
    chain_id: u64,
    signer: Wallet<k256::ecdsa::SigningKey>,
}

#[derive(Debug)]
pub struct LatestEventCounter {
    latest_event_id: u32,
}

impl AppContext {
    pub fn new(
        config: settings_mod::settings::Settings,
        https_rpc_url: String,
        wallet_path: &Path,
        password: String,
    ) -> Self {
        // connect to the network
        // let provider = Provider::<Http>::try_from(https_rpc_url.clone())
        //     .expect("could not instantiate HTTP Provider");
        println!("HTTP provider instantiated");
        let signer_wallet = backrun::setup_encrypted_json_wallet(&wallet_path, password);
        println!("Wallet {} decrypted", signer_wallet.clone().address());
        Self {
            // provider,
            provider_url: https_rpc_url,
            chain_id: config.avalanche.mainnet_node_rpc.chain_id,
            signer: signer_wallet,
        }
    }
}

// async fn handle_inner(
//     context: AppContext,
//     addr: SocketAddr,
//     req: Request<Body>,
// ) -> Result<Response<Body>, Error> {
//     Ok(Response::new(Body::from("Hello World")))
// }

async fn handle(
    client_ip: IpAddr,
    req: Request<Body>,
    context: AppContext,
    latest_event_counter: Arc<RwLock<LatestEventCounter>>,
) -> Result<Response<Body>, Infallible> {
    if req.uri().path().starts_with("/rpb-inspect-request") {
        // let _test_op_frontrun = match backrun::backrun_call(
        //     context.provider_url.clone(),
        //     context.chain_id,
        //     context.signer,
        // )
        // .await
        // {
        //     Ok(_) => println!("Backrun successful"),
        //     Err(e) => eprintln!("Error backrunning {}", e),
        // };
        // let latest_event_id: &LatestEventCounter = &latest_event_counter.read().unwrap();
        let latest_event_id: &u32 = &latest_event_counter.read().await.latest_event_id;
        println!("{:?}", latest_event_id);
        let _backrun_op = match backrun::backrun_call(
            context.provider_url,
            context.chain_id,
            context.signer,
            latest_event_id,
        ).await
        {
            Ok(_) => println!("Backrun successful"),
            Err(e) => eprintln!("Error backrunning {}", e),
        };
        debug_request(req)
    } else if req.uri().path().starts_with("/avalanche/c/rpc") {
        // will forward requests to port moralis speedy node HTTP RPC
        let proxy_op = match hyper_reverse_proxy::call(
            client_ip,
            &CONFIG.avalanche.mainnet_node_rpc.https,
            "/avalanche/c/rpc",
            req,
        )
        .await
        {
            Ok(response) => Ok(response),
            Err(error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        };
        let _backrun_op =
            backrun::backrun_call(context.provider_url, context.chain_id, 
                context.signer, 
                &1234_u32
            );

        return proxy_op;
    } else {
        // will forward requests to port moralis speedy node HTTP RPC
        let proxy_op = match hyper_reverse_proxy::call(
            client_ip,
            // &CONFIG.avalanche.mainnet_node_rpc.https,
            "https://httpbin.org",
            "",
            req,
        )
        .await
        {
            Ok(response) => Ok(response),
            Err(error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        };
        // state_counters.write().unwrap().latest_event_id += 1;
        return proxy_op;
    }
}

#[tokio::main]
async fn main() {
    println!("Starting service");

    let wallys_config = CONFIG.wallys.clone();
    let wally = match wallys_config
        .first()
        // .into_iter()
        // .find(|w| w.name.eq("bababa-0"))
    {
        Some(json_wallet) => json_wallet,
        None => {
            eprintln!("Wallet not found in path");
            panic!()
        }
    };
    let wally_path = Path::new(&wally.path);
    println!("Initializing context...");
    let context = AppContext::new(
        CONFIG.clone(),
        CONFIG.avalanche.mainnet_node_rpc.https.clone(),
        &wally_path,
        wally.clone().password,
    );
    println!("Context initialized");
    // AppContext {
    //     // provider: todo!(),
    //     // ...
    // };
    let main_latest_event_id: Arc<RwLock<LatestEventCounter>> =
        Arc::new(RwLock::new(LatestEventCounter { latest_event_id: 0 }));

    // This is our socket address...
    let addr = ([0, 0, 0, 0], 13900).into();

    let task_latest_event_id = Arc::clone(&main_latest_event_id);
    tokio::spawn(async move {
        // process(socket).await;
        let listener = subscriber::start_event_listener(CONFIG.clone(), task_latest_event_id);
        listener.await.unwrap()
    });

    println!("Making service...");
    // A `Service` is needed for every connection.
    // A `MakeService` that produces a `Service` to handle each connection.
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();

        // let svc_last_block_id = Arc::clone(&main_last_block_id);
        let svc_latest_event_id = Arc::clone(&main_latest_event_id);

        // You can grab the address of the incoming connection like so.
        let remote_addr = conn.remote_addr();

        // Create a `Service` for responding to the request.
        // let service = service_fn(move |req| {
        //     handle(context.clone(), addr, req)
        // });
        let service = service_fn(move |req: Request<Body>| {
            // let svc_last_block_id = Arc::clone(&svc_last_block_id);
            let svc_latest_event_id = Arc::clone(&svc_latest_event_id);
            handle(
                remote_addr.ip(),
                req,
                context.clone(),
                svc_latest_event_id,
            )
        });

        // Return the service to hyper.
        async move { Ok::<_, Error>(service) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    println!("Running server on {:?}", addr);

    // Run this server for... forever!
    // hyper::rt::run(server);
}
