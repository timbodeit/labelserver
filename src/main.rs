mod generator;
mod print;
mod server;

use hyper::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let server = Server::bind(&addr).serve(server::router());

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        log::error!("Server error: {}", err);
    }
}
