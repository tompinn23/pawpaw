extern crate core;

use crate::config::load_config;
use crate::server::Server;
use env_logger::Env;
use log::error;
use std::sync::Arc;
use tokio::runtime::Builder;

mod client;
mod config;
mod details;
mod proto;
mod server;

macro_rules! return_err {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(e) => {
                error!("client err: {}", e);
                return;
            }
        }
    };
}

macro_rules! break_err {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(e) => {
                error!("client err: {}", e);
                break;
            }
        }
    };
}

fn main() {
    if cfg!(debug_assertions) {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    }
    let config = load_config();
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async move {
        let mut server = Server::new(config)
            .await
            .expect("Failed to construct server");
        server.server_loop().await;
        let server = Arc::new(server);
        loop {
            let mut client = server.accept().await.expect("Failed to accept client");
            let server = server.clone();
            tokio::spawn(async move {
                return_err!(client.send_notice("*** Attempting lookup of your hostname..."));
                return_err!(client.poll_nowait().await);
                let hostname = match server.resolver().reverse_lookup(client.address()).await {
                    Ok(val) => val
                        .iter()
                        .nth(0)
                        .expect("Failed to get hostname from hostname??")
                        .to_string(),
                    Err(e) => client.address().to_string(),
                };
                client.set_hostname(hostname.clone());
                return_err!(client.send_notice(format!("*** Found hostname using {}.", hostname)));
                return_err!(client.poll_nowait().await);
                loop {
                    break_err!(client.poll().await);
                }
            });
        }
    });
}
