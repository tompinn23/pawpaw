use std::collections::HashMap;
use std::net::SocketAddr;
use clap::{Parser};
use figment::{Figment, providers::{Serialized, Yaml}};
use figment::providers::{Env, Format};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListenConfig {
    pub address: SocketAddr,
    #[serde(default = "def_false")]
    pub tls: bool,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

fn def_false() -> bool {
    false
}

#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[clap(short, long, value_parser, default_value = "config.yml")]
    // Path to the config file.
    pub config: String,
    #[clap(short = 'n', long, value_parser, default_value = "localhost")]
    //Hostname of the IRC server
    pub hostname: String,
    #[clap(skip)]
    pub listeners: HashMap<String, ListenConfig>
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