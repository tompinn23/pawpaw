use std::collections::HashMap;
use std::net::SocketAddr;
use clap::{Parser};
use figment::{Figment, providers::{Serialized, Yaml}};
use figment::providers::{Env, Format};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListenConfig {
    address: SocketAddr,
    #[serde(default = "def_false")]
    tls: bool,
    tls_cert: Option<String>,
    tls_key: Option<String>,
}

fn def_false() -> bool {
    false
}

#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[clap(short, long, value_parser, default_value = "config.yml")]
    // Path to the config file.
    config: String,
    #[clap(short = 'n', long, value_parser, default_value = "localhost")]
    //Hostname of the IRC server
    hostname: String,
    #[clap(skip)]
    listeners: HashMap<String, ListenConfig>
}


pub fn load_config() -> Config {
    let args = Config::parse();
    let config_path = args.config.clone();
    Figment::new()
        .merge(Serialized::defaults(args))
        .merge(Yaml::file(config_path))
        .merge(Env::prefixed("PAW_"))
        .extract().expect("Failed to load config")
}