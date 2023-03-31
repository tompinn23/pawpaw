use std::sync::Arc;
use crate::config::load_config;
use env_logger::Env;
use tokio::runtime::Builder;
use crate::server::Server;

mod config;
mod server;
mod client;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = load_config();
    let runtime = Builder::new_multi_thread()
        .build()
        .unwrap();
    runtime.block_on(async move {
        let server = Server::new(config).await.expect("Failed to construct server");

        let server = Arc::new(server);
        loop {
            let client = server.accept().await;
        }
    });
}

