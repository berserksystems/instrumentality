pub mod boot;
pub mod config;
pub mod data;
pub mod database;
pub mod group;
#[macro_use]
pub mod routes;
pub mod server;
pub mod subject;
pub mod user;
pub mod utils;

#[tokio::main]
async fn main() {
    boot::instrumentality().await;
}
