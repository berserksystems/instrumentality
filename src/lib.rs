//! This is the API documentation for the Instrumentality server. It contains
//! technical details about the internal operation of Instrumentality, and is
//! solely aimed at developers looking to understand or extend the server's
//! capabilities.
//!
//! You can find documentation on installation and running the server at:
//! <https://docs.berserksystems.com/>
//!
//! Instrumentality makes heavy use of [Axum] and [MongoDB]. MongoDB is
//! probably not the correct choice for this system at scale, and a re-write
//! using PostgreSQL would likely be more performant. However, whilst iterating,
//! being able to add and remove fields ad-hoc is useful.
//!
//! [MongoDB]: https://www.mongodb.com/
//! [Axum]: https://github.com/tokio-rs/axum/

pub mod boot;
pub mod concepts;
pub mod config;
pub mod database;
#[macro_use]
pub mod routes;
pub mod server;
pub mod utils;
