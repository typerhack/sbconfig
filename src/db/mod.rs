// src/db/mod.rs
// Database module

mod database;
mod encryption;
mod migrations;
mod models;
mod queries;
mod schema;

pub use database::Database;
pub use encryption::{decrypt, encrypt};
pub use models::*;
