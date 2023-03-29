use crate::config::load_config;

mod config;
mod server;

fn main() {
    let config = load_config();
}