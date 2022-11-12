//! Functions for the configuration file.

use std::collections::HashMap;
use std::time::Duration;

use mongodb::options::{ClientOptions, Credential, ServerAddress};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct IConfig {
    pub mongodb: MDBConfig,
    pub content_types: HashMap<String, Vec<String>>,
    pub presence_types: HashMap<String, Vec<String>>,
    #[serde(default = "Settings::default")]
    pub settings: Settings,
    pub network: NetworkConfig,
    pub tls: TLSConfig,
}

impl IConfig {
    pub fn valid_presence_type(
        &self,
        platform: &str,
        presence_type: &str,
    ) -> bool {
        self.presence_types
            .get(platform)
            .is_some_and(|p| p.contains(&presence_type.to_string()))
    }

    pub fn valid_content_type(
        &self,
        platform: &str,
        content_type: &str,
    ) -> bool {
        self.content_types
            .get(platform)
            .is_some_and(|p| p.contains(&content_type.to_string()))
    }

    pub fn valid_platform(&self, platform: &str) -> bool {
        self.content_types.contains_key(platform)
            || self.presence_types.contains_key(platform)
    }
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "Settings::default_log_level")]
    pub log_level: String,
    #[serde(default = "Settings::default_queue_timeout_secs")]
    pub queue_timeout_secs: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_level: Self::default_log_level(),
            queue_timeout_secs: Self::default_queue_timeout_secs(),
        }
    }
}

impl Settings {
    pub fn default_log_level() -> String {
        "INFO".to_string()
    }

    pub fn default_queue_timeout_secs() -> i64 {
        30
    }
}

#[derive(Clone, Deserialize)]
pub struct TLSConfig {
    pub cert: String,
    pub key: String,
}

#[derive(Clone, Deserialize)]
pub struct NetworkConfig {
    pub address: String,
    pub port: String,
}

#[derive(Clone, Deserialize)]
pub struct MDBConfig {
    pub address: String,
    pub port: String,
    pub database: String,
    pub credentials: Option<Credentials>,
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub auth_database: String,
}

impl MDBConfig {
    pub fn client_opts(&self) -> ClientOptions {
        let server_addr =
            ServerAddress::parse(format!("{}:{}", self.address, self.port))
                .unwrap();

        if let Some(credentials) = &self.credentials {
            let creds = Credential::builder()
                .username(credentials.username.to_string())
                .password(credentials.password.to_string())
                .source(credentials.auth_database.to_string())
                .build();

            ClientOptions::builder()
                .credential(creds)
                .hosts(vec![server_addr])
                .connect_timeout(Duration::new(1, 0))
                .heartbeat_freq(Duration::new(1, 0))
                .server_selection_timeout(Duration::new(1, 0))
                .build()
        } else {
            ClientOptions::builder()
                .hosts(vec![server_addr])
                .connect_timeout(Duration::new(1, 0))
                .heartbeat_freq(Duration::new(1, 0))
                .server_selection_timeout(Duration::new(1, 0))
                .build()
        }
    }
}

pub fn open(config_path: &str) -> Result<IConfig, Box<dyn std::error::Error>> {
    let config_str = &std::fs::read_to_string(config_path)?;
    let config: IConfig = toml::from_str(config_str)?;
    Ok(config)
}
