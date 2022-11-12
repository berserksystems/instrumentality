#![feature(is_some_and)]
pub mod boot;
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

#[tokio::main]
async fn main() {
    boot::instrumentality().await;
}
