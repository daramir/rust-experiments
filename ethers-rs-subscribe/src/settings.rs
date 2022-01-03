// adapted from https://blog.logrocket.com/configuration-management-in-rust-web-services/

use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Addresses {
    pub game: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExternalContracts {
    pub addresses: Addresses,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventSignatureHashes {
    pub some_event: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EthLogs {
    pub event_signature_hashes: EventSignatureHashes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeRpc {
    pub wss: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Avalanche {
    pub node_rpc: NodeRpc,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub rule_set: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub avalanche: Avalanche,
    pub external_contracts: ExternalContracts,
    pub eth_logs: EthLogs,
    pub rules: Vec<Rule>,
    pub log: Log,
    // pub env: ENV,
}

const CONFIG_FILE_PATH: &str = "./config/Default.toml";
const CONFIG_FILE_PREFIX: &str = "./config/";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "Default".into());  //thread 'main' 
        // panicked at 'config can't be loaded: configuration file "./config/Development" not found'.
        //  Hence swapped to Default

        let mut s = Config::new();
        s.set("env", env.clone())?;

        s.merge(File::with_name(CONFIG_FILE_PATH))?;
        s.merge(File::with_name(&format!("{}{}", CONFIG_FILE_PREFIX, env)))?;

        // This makes it so "EA_SERVER__PORT overrides server.port
        // s.merge(Environment::with_prefix("ea").separator("__"))?;

        s.try_into()
    }
}