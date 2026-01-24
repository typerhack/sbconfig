// src/singbox/config.rs
// sing-box client configuration generation

use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutingPreset {
    Default,     // All traffic through proxy
    IranDirect,  // Iranian sites bypass proxy
    IranBlock,   // Block Iranian government sites
    ChinaDirect, // Chinese sites bypass proxy
}

impl RoutingPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutingPreset::Default => "default",
            RoutingPreset::IranDirect => "iran_direct",
            RoutingPreset::IranBlock => "iran_block",
            RoutingPreset::ChinaDirect => "china_direct",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RoutingPreset::Default => "All traffic through proxy",
            RoutingPreset::IranDirect => "Iranian sites bypass proxy",
            RoutingPreset::IranBlock => "Block Iranian government sites",
            RoutingPreset::ChinaDirect => "Chinese sites bypass proxy",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub log: LogConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub route: RouteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    pub tag: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inbound {
    #[serde(rename = "type")]
    pub inbound_type: String,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniff_override_destination: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outbound {
    #[serde(rename = "type")]
    pub outbound_type: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_algorithms: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub rules: Vec<RouteRule>,
    #[serde(rename = "final")]
    pub final_outbound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geosite: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<Vec<String>>,
    pub outbound: String,
}

/// Generate client configuration
pub fn generate_config(
    server: &str,
    port: u16,
    username: &str,
    private_key: &str,
    preset: RoutingPreset,
) -> Result<ClientConfig> {
    let config = ClientConfig {
        log: LogConfig {
            level: "info".to_string(),
            timestamp: true,
        },
        dns: DnsConfig {
            servers: vec![DnsServer {
                tag: "google".to_string(),
                address: "8.8.8.8".to_string(),
            }],
        },
        inbounds: vec![Inbound {
            inbound_type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: "127.0.0.1".to_string(),
            listen_port: 10808,
            sniff: Some(true),
            sniff_override_destination: Some(true),
        }],
        outbounds: vec![
            Outbound {
                outbound_type: "ssh".to_string(),
                tag: "proxy".to_string(),
                server: Some(server.to_string()),
                server_port: Some(port),
                user: Some(username.to_string()),
                private_key: Some(private_key.to_string()),
                host_key_algorithms: Some(vec![
                    "ssh-ed25519".to_string(),
                    "rsa-sha2-512".to_string(),
                    "rsa-sha2-256".to_string(),
                ]),
            },
            Outbound {
                outbound_type: "direct".to_string(),
                tag: "direct".to_string(),
                server: None,
                server_port: None,
                user: None,
                private_key: None,
                host_key_algorithms: None,
            },
            Outbound {
                outbound_type: "block".to_string(),
                tag: "block".to_string(),
                server: None,
                server_port: None,
                user: None,
                private_key: None,
                host_key_algorithms: None,
            },
        ],
        route: generate_route_rules(preset),
    };

    Ok(config)
}

fn generate_route_rules(preset: RoutingPreset) -> RouteConfig {
    let mut rules = vec![
        // Always direct DNS queries
        RouteRule {
            protocol: Some(vec!["dns".to_string()]),
            geosite: None,
            geoip: None,
            outbound: "direct".to_string(),
        },
    ];

    match preset {
        RoutingPreset::Default => {
            // All traffic through proxy (no additional rules)
        }
        RoutingPreset::IranDirect => {
            rules.push(RouteRule {
                protocol: None,
                geosite: Some(vec!["ir".to_string()]),
                geoip: None,
                outbound: "direct".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: Some(vec!["ir".to_string()]),
                outbound: "direct".to_string(),
            });
        }
        RoutingPreset::IranBlock => {
            rules.push(RouteRule {
                protocol: None,
                geosite: Some(vec!["ir".to_string()]),
                geoip: None,
                outbound: "block".to_string(),
            });
        }
        RoutingPreset::ChinaDirect => {
            rules.push(RouteRule {
                protocol: None,
                geosite: Some(vec!["cn".to_string()]),
                geoip: None,
                outbound: "direct".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: Some(vec!["cn".to_string()]),
                outbound: "direct".to_string(),
            });
        }
    }

    RouteConfig {
        rules,
        final_outbound: "proxy".to_string(),
    }
}

/// Generate configuration as JSON string
pub fn generate_config_json(
    server: &str,
    port: u16,
    username: &str,
    private_key: &str,
    preset: RoutingPreset,
) -> Result<String> {
    let config = generate_config(server, port, username, private_key, preset)?;
    let json = serde_json::to_string_pretty(&config)?;
    Ok(json)
}
