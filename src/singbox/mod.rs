// src/singbox/mod.rs
// sing-box detection, service control, and config generation

mod config;
mod detect;
mod install;
mod service;
mod server_config;

pub use config::*;
pub use detect::*;
pub use install::*;
pub use service::*;
pub use server_config::*;
