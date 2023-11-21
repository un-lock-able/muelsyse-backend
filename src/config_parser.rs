use clap::Parser;
use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub db_name: DatabaseName,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default)]
    pub bind_address: ServerBindAddress,
    #[serde(default)]
    pub bind_port: ServerLitsenPort,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            bind_address: ServerBindAddress(String::from("127.0.0.1")),
            bind_port: ServerLitsenPort(29030),
        }
    }
}

#[derive(Deserialize)]
pub struct ServerBindAddress(pub String);

impl Default for ServerBindAddress {
    fn default() -> Self {
        ServerBindAddress(String::from("127.0.0.1"))
    }
}

#[derive(Deserialize)]
pub struct ServerLitsenPort(pub u16);

impl Default for ServerLitsenPort {
    fn default() -> Self {
        ServerLitsenPort(29030)
    }
}

#[derive(Deserialize)]
pub struct DatabaseName(pub String);

impl Default for DatabaseName {
    fn default() -> Self {
        DatabaseName(String::from("muelsyse.db"))
    }
}

#[derive(Parser)]
pub struct CmdArgs {
    #[arg(long, short, required = true)]
    pub config: String,
}
