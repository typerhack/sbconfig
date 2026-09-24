// src/singbox/server_config.rs
// sing-box server configuration generation

use crate::error::Result;
use serde::{Deserialize, Serialize};

#[allow(dead_code)] // Will be used for mode selection UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    SshOnly,      // OpenSSH only (current mode)
    SingboxServer, // sing-box server daemon
}

#[allow(dead_code)] // Will be used for mode selection UI
impl ServerMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerMode::SshOnly => "ssh_only",
            ServerMode::SingboxServer => "singbox_server",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "singbox_server" => ServerMode::SingboxServer,
            _ => ServerMode::SshOnly,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ServerMode::SshOnly => "SSH Tunnel Only (OpenSSH) - Client-side routing",
            ServerMode::SingboxServer => "sing-box Server - Server-side routing & protocols",
        }
    }
}

#[allow(dead_code)] // VMess and Trojan will be used for protocol selection UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProtocol {
    Shadowsocks,
    VMess,
    Trojan,
    // Future: VLESS, Hysteria, etc.
}

#[allow(dead_code)] // Will be used for protocol selection UI
impl ServerProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerProtocol::Shadowsocks => "shadowsocks",
            ServerProtocol::VMess => "vmess",
            ServerProtocol::Trojan => "trojan",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ServerProtocol::Shadowsocks => "Shadowsocks - Simple & fast",
            ServerProtocol::VMess => "VMess - V2Ray protocol with UUID auth",
            ServerProtocol::Trojan => "Trojan - Camouflaged as HTTPS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub log: LogConfig,
    pub dns: Option<DnsConfig>,
    pub inbounds: Vec<ServerInbound>,
    pub outbounds: Vec<ServerOutbound>,
    pub route: Option<ServerRouteConfig>,
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
#[serde(tag = "type")]
pub enum ServerInbound {
    #[serde(rename = "shadowsocks")]
    Shadowsocks {
        tag: String,
        listen: String,
        listen_port: u16,
        method: String,
        password: String,
    },
    #[serde(rename = "vmess")]
    VMess {
        tag: String,
        listen: String,
        listen_port: u16,
        users: Vec<VMessUser>,
    },
    #[serde(rename = "trojan")]
    Trojan {
        tag: String,
        listen: String,
        listen_port: u16,
        users: Vec<TrojanUser>,
        tls: TlsConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMessUser {
    pub uuid: String,
    #[serde(rename = "alterId")]
    pub alter_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanUser {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub server_name: Option<String>,
    pub certificate_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerOutbound {
    #[serde(rename = "direct")]
    Direct { tag: String },
    #[serde(rename = "block")]
    Block { tag: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRouteConfig {
    pub rule_set: Option<Vec<RuleSet>>,
    pub rules: Vec<RouteRule>,
    #[serde(rename = "final")]
    pub final_outbound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub tag: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub format: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_set: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_suffix: Option<Vec<String>>,
    pub outbound: String,
}

impl ServerConfig {
    /// Generate a basic sing-box server config
    pub fn generate(
        protocol: ServerProtocol,
        port: u16,
        dns_servers: &[String],
        block_ads: bool,
        block_iran_gov: bool,
        block_all_iran: bool,
        custom_blocklist: &[String],
    ) -> Result<Self> {
        let inbound = match protocol {
            ServerProtocol::Shadowsocks => ServerInbound::Shadowsocks {
                tag: "ss-in".to_string(),
                listen: "0.0.0.0".to_string(),
                listen_port: port,
                method: "aes-256-gcm".to_string(),
                password: generate_random_password(),
            },
            ServerProtocol::VMess => ServerInbound::VMess {
                tag: "vmess-in".to_string(),
                listen: "0.0.0.0".to_string(),
                listen_port: port,
                users: vec![VMessUser {
                    uuid: uuid::Uuid::new_v4().to_string(),
                    alter_id: 0,
                }],
            },
            ServerProtocol::Trojan => ServerInbound::Trojan {
                tag: "trojan-in".to_string(),
                listen: "0.0.0.0".to_string(),
                listen_port: port,
                users: vec![TrojanUser {
                    password: generate_random_password(),
                }],
                tls: TlsConfig {
                    enabled: true,
                    server_name: None,
                    certificate_path: Some("/etc/ssl/certs/server.crt".to_string()),
                    key_path: Some("/etc/ssl/private/server.key".to_string()),
                },
            },
        };

        let dns_config = if !dns_servers.is_empty() {
            Some(DnsConfig {
                servers: dns_servers
                    .iter()
                    .enumerate()
                    .map(|(i, addr)| DnsServer {
                        tag: format!("dns-{}", i),
                        address: addr.clone(),
                    })
                    .collect(),
            })
        } else {
            None
        };

        let mut rule_sets = Vec::new();
        let mut rules = Vec::new();

        // Add ads blocking rule set
        if block_ads {
            rule_sets.push(RuleSet {
                tag: "geosite-ads".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-ads.srs".to_string(),
            });
            rules.push(RouteRule {
                rule_set: Some(vec!["geosite-ads".to_string()]),
                domain: None,
                domain_suffix: None,
                outbound: "block".to_string(),
            });
        }

        // Add Iran government sites blocking
        if block_iran_gov {
            rule_sets.push(RuleSet {
                tag: "geosite-iran-gov".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-category-ir-gov.srs".to_string(),
            });
            rules.push(RouteRule {
                rule_set: Some(vec!["geosite-iran-gov".to_string()]),
                domain: None,
                domain_suffix: None,
                outbound: "block".to_string(),
            });
        }

        // Add block all Iran sites
        if block_all_iran {
            rule_sets.push(RuleSet {
                tag: "geosite-iran-all".to_string(),
                rule_type: "remote".to_string(),
                format: "binary".to_string(),
                url: "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-ir.srs".to_string(),
            });
            rules.push(RouteRule {
                rule_set: Some(vec!["geosite-iran-all".to_string()]),
                domain: None,
                domain_suffix: None,
                outbound: "block".to_string(),
            });
        }

        // Add custom blocked domains
        if !custom_blocklist.is_empty() {
            rules.push(RouteRule {
                rule_set: None,
                domain: None,
                domain_suffix: Some(custom_blocklist.to_vec()),
                outbound: "block".to_string(),
            });
        }

        let route = if !rules.is_empty() {
            Some(ServerRouteConfig {
                rule_set: if rule_sets.is_empty() {
                    None
                } else {
                    Some(rule_sets)
                },
                rules,
                final_outbound: "direct".to_string(),
            })
        } else {
            None
        };

        Ok(ServerConfig {
            log: LogConfig {
                level: "info".to_string(),
                timestamp: true,
            },
            dns: dns_config,
            inbounds: vec![inbound],
            outbounds: vec![
                ServerOutbound::Direct {
                    tag: "direct".to_string(),
                },
                ServerOutbound::Block {
                    tag: "block".to_string(),
                },
            ],
            route,
        })
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

fn generate_random_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
