// src/singbox/mod.rs
// sing-box detection, service control, and config generation

mod config;
mod detect;
mod install;
mod service;

pub use config::*;
pub use detect::*;
pub use install::*;
pub use service::*;
