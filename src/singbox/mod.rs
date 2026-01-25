// src/singbox/mod.rs
// sing-box detection, service control, and config generation

mod config;
mod detect;
mod service;

pub use config::*;
pub use detect::*;
pub use service::*;
