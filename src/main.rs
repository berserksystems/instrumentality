#![feature(is_some_and)]

pub mod config;
pub mod data;
pub mod database;
pub mod group;
pub mod response;
pub mod routes;
pub mod server;
pub mod subject;
pub mod user;
pub mod utils;

use std::fs::File;
use std::io::Write;

pub const CONFIG_FILE_NAME: &str = "Instrumentality.toml";
pub const EXAMPLE_CONFIG_FILE_NAME: &str = "InstrumentalityExample.toml";

#[tokio::main]
async fn main() {
    instrumentality().await;
}

async fn instrumentality() {
    let config = config::open(CONFIG_FILE_NAME);
    if let Ok(config) = config {
        server::build_tracing(&config.settings.log_level);
        tracing::info!("Config file loaded.");

        let (app, tls_config, addr, handle) =
            server::build_server(&config).await;

        let server = axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service());

        tracing::info!("READY: https://{:?}.", addr);
        server.await.unwrap();
    } else {
        server::build_tracing(&config::Settings::default_log_level());

        tracing::error!(
            "Couldn't load {CONFIG_FILE_NAME}, creating an example at \
            {EXAMPLE_CONFIG_FILE_NAME}."
        );

        let mut file = File::create(EXAMPLE_CONFIG_FILE_NAME).unwrap();
        file.write_all(EXAMPLE_CONFIG_FILE).unwrap();
    }
}

const EXAMPLE_CONFIG_FILE: &[u8] = b"[content_types]
instagram = [\"post\", \"story\", \"live\"]
twitter = [\"tweet\", \"like\", \"retweet\", \"story\"]
last_fm = [\"scrobble\"]
twitch_tv = [\"stream_start\", \"video\", \"stream_end\"]

[presence_types]
twitter = [\"follower_count_increase\"]
last_fm = [\"now_playing\"]
twitch_tv = [\"live\"]

[mongodb]
hosts = \"127.0.0.1\"
port = \"27017\"
user = \"instrumentality\"
password = \"mankind\"
database = \"instrumentality\"
auth_database = \"admin\"

[settings]
log_level = \"INFO\"
queue_timeout_secs = 30

[network]
address = \"127.0.0.1\"
port = \"12321\"

[tls]
# Can be taken directly from Let's Encrypt.
cert = \"tls/cert.pem\"
key = \"tls/privkey.pem\"";
