pub mod boot;
pub mod concepts;
pub mod config;
pub mod database;
#[macro_use]
pub mod routes;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() {
    boot::instrumentality().await;
}
