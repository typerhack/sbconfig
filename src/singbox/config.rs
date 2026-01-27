// src/singbox/config.rs
// sing-box client configuration generation
#![allow(dead_code)]

use crate::error::Result;
use base64::{engine::general_purpose, Engine as _};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<DnsRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    pub tag: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRule {
    pub outbound: String,
    pub server: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_set: Option<Vec<RouteRuleSet>>,
    pub rules: Vec<RouteRule>,
    #[serde(rename = "final")]
    pub final_outbound: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_interface: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRuleSet {
    pub tag: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geosite: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_set: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_suffix: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_is_private: Option<bool>,
    pub outbound: String,
}

/// Generate client configuration
pub fn generate_config(
    server: &str,
    port: u16,
    username: &str,
    private_key: &str,
    preset: RoutingPreset,
    dns_servers: &[String],
    iran_ads_block: bool,
) -> Result<ClientConfig> {
    let dns_servers = if dns_servers.is_empty() {
        vec![
            "8.8.8.8".to_string(),
            "1.1.1.1".to_string(),
        ]
    } else {
        dns_servers.to_vec()
    };

    let config = ClientConfig {
        log: LogConfig {
            level: "info".to_string(),
            timestamp: true,
        },
        dns: DnsConfig {
            servers: dns_servers
                .iter()
                .enumerate()
                .map(|(i, addr)| DnsServer {
                    tag: format!("dns-{}", i + 1),
                    address: addr.to_string(),
                })
                .collect(),
            rules: None,
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
                tag: "ssh-out".to_string(),
                server: Some(server.to_string()),
                server_port: Some(port),
                user: Some(username.to_string()),
                private_key: Some(private_key.to_string()),
                host_key_algorithms: Some(default_host_key_algorithms()),
                client_version: Some("SSH-2.0-OpenSSH_9.0".to_string()),
            },
            Outbound {
                outbound_type: "direct".to_string(),
                tag: "direct".to_string(),
                server: None,
                server_port: None,
                user: None,
                private_key: None,
                host_key_algorithms: None,
                client_version: None,
            },
            Outbound {
                outbound_type: "block".to_string(),
                tag: "block".to_string(),
                server: None,
                server_port: None,
                user: None,
                private_key: None,
                host_key_algorithms: None,
                client_version: None,
            },
            Outbound {
                outbound_type: "dns".to_string(),
                tag: "dns-out".to_string(),
                server: None,
                server_port: None,
                user: None,
                private_key: None,
                host_key_algorithms: None,
                client_version: None,
            },
        ],
        route: generate_route_rules(preset, iran_ads_block),
    };

    Ok(config)
}

fn generate_route_rules(preset: RoutingPreset, iran_ads_block: bool) -> RouteConfig {
    let mut rules = vec![
        RouteRule {
            protocol: Some(vec!["dns".to_string()]),
            geosite: None,
            geoip: None,
            rule_set: None,
            domain_suffix: None,
            ip_is_private: None,
            outbound: "dns-out".to_string(),
        },
        RouteRule {
            protocol: None,
            geosite: None,
            geoip: None,
            rule_set: None,
            domain_suffix: None,
            ip_is_private: Some(true),
            outbound: "direct".to_string(),
        },
    ];

    let mut rule_sets: Vec<RouteRuleSet> = vec![];

    match preset {
        RoutingPreset::Default => {}
        RoutingPreset::IranDirect => {
            if iran_ads_block {
                rule_sets.push(RouteRuleSet {
                    tag: "iran-geosite-ads".to_string(),
                    rule_type: "remote".to_string(),
                    format: "binary".to_string(),
                    update_interval: Some("7d".to_string()),
                    url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-ads.srs".to_string(),
                });
            }
            rule_sets.push(RouteRuleSet {
                tag: "iran-geosite-all".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                update_interval: Some("7d".to_string()),
                url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-all.srs".to_string(),
            });
            if iran_ads_block {
                rules.push(RouteRule {
                    protocol: None,
                    geosite: None,
                    geoip: None,
                    rule_set: Some(vec!["iran-geosite-ads".to_string()]),
                    domain_suffix: None,
                    ip_is_private: None,
                    outbound: "block".to_string(),
                });
            }
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: Some(vec!["iran-geosite-all".to_string()]),
                domain_suffix: None,
                ip_is_private: None,
                outbound: "direct".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: None,
                domain_suffix: Some(vec![".ir".to_string()]),
                ip_is_private: None,
                outbound: "direct".to_string(),
            });
        }
        RoutingPreset::IranBlock => {
            if iran_ads_block {
                rule_sets.push(RouteRuleSet {
                    tag: "iran-geosite-ads".to_string(),
                    rule_type: "remote".to_string(),
                    format: "binary".to_string(),
                    update_interval: Some("7d".to_string()),
                    url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-ads.srs".to_string(),
                });
            }
            rule_sets.push(RouteRuleSet {
                tag: "iran-geosite-all".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                update_interval: Some("7d".to_string()),
                url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-all.srs".to_string(),
            });
            rule_sets.push(RouteRuleSet {
                tag: "geoip-ir".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                update_interval: Some("7d".to_string()),
                url: "https://github.com/SagerNet/sing-geoip/releases/latest/download/geoip-ir.srs".to_string(),
            });
            if iran_ads_block {
                rules.push(RouteRule {
                    protocol: None,
                    geosite: None,
                    geoip: None,
                    rule_set: Some(vec!["iran-geosite-ads".to_string()]),
                    domain_suffix: None,
                    ip_is_private: None,
                    outbound: "block".to_string(),
                });
            }
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: Some(vec!["iran-geosite-all".to_string()]),
                domain_suffix: None,
                ip_is_private: None,
                outbound: "block".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: Some(vec!["geoip-ir".to_string()]),
                domain_suffix: None,
                ip_is_private: None,
                outbound: "block".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: None,
                domain_suffix: Some(vec![".ir".to_string()]),
                ip_is_private: None,
                outbound: "block".to_string(),
            });
        }
        RoutingPreset::ChinaDirect => {
            rule_sets.push(RouteRuleSet {
                tag: "geosite-cn".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                update_interval: Some("7d".to_string()),
                url: "https://github.com/SagerNet/sing-geosite/releases/latest/download/geosite-cn.srs".to_string(),
            });
            rule_sets.push(RouteRuleSet {
                tag: "geoip-cn".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                update_interval: Some("7d".to_string()),
                url: "https://github.com/SagerNet/sing-geoip/releases/latest/download/geoip-cn.srs".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: Some(vec!["geosite-cn".to_string()]),
                domain_suffix: None,
                ip_is_private: None,
                outbound: "direct".to_string(),
            });
            rules.push(RouteRule {
                protocol: None,
                geosite: None,
                geoip: None,
                rule_set: Some(vec!["geoip-cn".to_string()]),
                domain_suffix: None,
                ip_is_private: None,
                outbound: "direct".to_string(),
            });
        }
    }

    RouteConfig {
        rule_set: if rule_sets.is_empty() {
            None
        } else {
            Some(rule_sets)
        },
        rules,
        final_outbound: "ssh-out".to_string(),
        auto_detect_interface: Some(true),
    }
}

/// Generate configuration as JSON string
pub fn generate_config_json(
    server: &str,
    port: u16,
    username: &str,
    private_key: &str,
    preset: RoutingPreset,
    dns_servers: &[String],
    iran_ads_block: bool,
) -> Result<String> {
    let config = generate_config(
        server,
        port,
        username,
        private_key,
        preset,
        dns_servers,
        iran_ads_block,
    )?;
    let json = serde_json::to_string_pretty(&config)?;
    Ok(json)
}

pub fn generate_ssh_uri(
    server: &str,
    port: u16,
    username: &str,
    private_key: &str,
    host_key_algorithms: &[String],
) -> String {
    let encoded_key = general_purpose::URL_SAFE.encode(private_key.as_bytes());
    let mut params = vec![format!("private_key={}", encoded_key)];
    if !host_key_algorithms.is_empty() {
        params.push(format!(
            "host_key_algorithms={}",
            host_key_algorithms.join(",")
        ));
    }
    format!(
        "ssh://{}@{}:{}?{}",
        username,
        server,
        port,
        params.join("&")
    )
}

pub fn default_host_key_algorithms() -> Vec<String> {
    vec![
        "ssh-ed25519".to_string(),
        "ecdsa-sha2-nistp256".to_string(),
        "rsa-sha2-512".to_string(),
        "rsa-sha2-256".to_string(),
        "ssh-rsa".to_string(),
    ]
}
