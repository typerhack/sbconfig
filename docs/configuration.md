# Configuration

> sing-box client configuration generation details for sbconfig.

## Overview

sbconfig generates sing-box client configurations using the SSH outbound protocol. This document explains the configuration structure and generation logic.

## Operating Modes

sbconfig supports two operating modes that affect how configs are generated:

### Development Mode
- **Server address:** `localhost` or `127.0.0.1`
- **Use case:** Local testing, development
- **SSH port:** Any available port on local machine
- Generated configs connect to local SSH server

### Production Mode
- **Server address:** Domain name or public IP
- **Use case:** Real VPS deployment
- **SSH port:** Custom port (10000-60000 range, NOT port 22)
- Generated configs connect to remote VPS

```rust
pub enum OperatingMode {
    Development,  // server = "localhost"
    Production,   // server = domain/IP from settings
}
```

## SSH Port Strategy

> **Important:** sbconfig uses a dedicated custom SSH port for proxy users, NOT the default port 22.

| Port | Usage |
|------|-------|
| 22 | Default SSH (admin access, NOT for sing-box) |
| 7344 (example) | Custom port for sing-box proxy users |

This separation:
- Keeps admin SSH access separate from proxy users
- Allows different authentication methods per port
- Makes it easier to manage firewall rules

## How sing-box SSH Works

```
┌─────────────────┐                      ┌─────────────────┐
│                 │    SSH Connection    │                 │
│  Client Device  │◄────────────────────►│   Your VPS      │
│  (sing-box)     │    (encrypted)       │   (OpenSSH)     │
│                 │                      │                 │
└────────┬────────┘                      └────────┬────────┘
         │                                        │
         │ Local SOCKS5/HTTP                      │ Direct
         │ Proxy (127.0.0.1:10808)               │ Connection
         │                                        │
         ▼                                        ▼
┌─────────────────┐                      ┌─────────────────┐
│   Application   │                      │    Internet     │
│ (Browser, etc.) │                      │                 │
└─────────────────┘                      └─────────────────┘
```

**Key Points:**
- Client runs sing-box with SSH outbound
- Server only needs standard OpenSSH (no sing-box server needed)
- Traffic is tunneled through SSH connection
- Uses SSH key authentication (ED25519)

## Generated Configuration Structure

### Complete Client Config

```json
{
  "log": {
    "level": "info",
    "timestamp": true
  },
  "dns": {
    "servers": [
      {
        "tag": "google",
        "address": "8.8.8.8"
      },
      {
        "tag": "cloudflare",
        "address": "1.1.1.1"
      }
    ],
    "rules": [
      {
        "outbound": "any",
        "server": "google"
      }
    ]
  },
  "inbounds": [
    {
      "type": "mixed",
      "tag": "mixed-in",
      "listen": "127.0.0.1",
      "listen_port": 10808
    }
  ],
  "outbounds": [
    {
      "type": "ssh",
      "tag": "ssh-out",
      "server": "liberty.zyberis.com",
      "server_port": 7344,
      "user": "user_beta",
      "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n...\n-----END OPENSSH PRIVATE KEY-----",
      "host_key_algorithms": [
        "ssh-ed25519",
        "ecdsa-sha2-nistp256",
        "rsa-sha2-512",
        "rsa-sha2-256",
        "ssh-rsa"
      ],
      "client_version": "SSH-2.0-OpenSSH_9.0"
    },
    {
      "type": "direct",
      "tag": "direct"
    },
    {
      "type": "block",
      "tag": "block"
    }
  ],
  "route": {
    "rules": [
      {
        "protocol": "dns",
        "outbound": "dns-out"
      },
      {
        "ip_is_private": true,
        "outbound": "direct"
      }
    ],
    "final": "ssh-out"
  }
}
```

## Configuration Sections

### Log

Controls logging behavior.

```json
{
  "log": {
    "level": "info",
    "timestamp": true
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `level` | string | Log level: `debug`, `info`, `warn`, `error` |
| `timestamp` | bool | Include timestamps in logs |

### DNS

DNS resolution configuration.

```json
{
  "dns": {
    "servers": [
      {
        "tag": "google",
        "address": "8.8.8.8"
      }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `servers` | array | List of DNS servers |
| `servers[].tag` | string | Identifier for the server |
| `servers[].address` | string | DNS server address |

### Inbounds

Local proxy listener configuration.

```json
{
  "inbounds": [
    {
      "type": "mixed",
      "tag": "mixed-in",
      "listen": "127.0.0.1",
      "listen_port": 10808
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Inbound type: `mixed` (SOCKS5 + HTTP) |
| `tag` | string | Identifier for the inbound |
| `listen` | string | Listen address (use `127.0.0.1` for local only) |
| `listen_port` | int | Local proxy port |

**Platform-Specific Inbound Settings:**

| Platform | Listen | Port | Notes |
|----------|--------|------|-------|
| iOS | `127.0.0.1` | `10808` | sing-box app handles system proxy |
| Android | `127.0.0.1` | `10808` | sing-box app handles VPN |
| Windows | `127.0.0.1` | `10808` | Configure apps to use proxy |
| macOS | `127.0.0.1` | `10808` | Configure apps to use proxy |
| Linux | `127.0.0.1` | `10808` | Configure apps to use proxy |

### Outbounds

Connection handling configuration.

#### SSH Outbound

The main proxy connection.

```json
{
  "type": "ssh",
  "tag": "ssh-out",
  "server": "liberty.zyberis.com",
  "server_port": 7344,
  "user": "user_beta",
  "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n...\n-----END OPENSSH PRIVATE KEY-----",
  "host_key_algorithms": [
    "ssh-ed25519",
    "ecdsa-sha2-nistp256",
    "rsa-sha2-512",
    "rsa-sha2-256",
    "ssh-rsa"
  ],
  "client_version": "SSH-2.0-OpenSSH_9.0"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Must be `ssh` |
| `tag` | string | Identifier for the outbound |
| `server` | string | Server hostname or IP |
| `server_port` | int | SSH port on server |
| `user` | string | SSH username |
| `private_key` | string | Full private key content (embedded) |
| `host_key_algorithms` | array | Accepted host key algorithms |
| `client_version` | string | SSH client version string |

**Alternative: File-based Private Key**

For desktop platforms, you can use a file path instead:

```json
{
  "private_key_path": "/path/to/private_key"
}
```

> **Note:** sbconfig uses embedded keys (`private_key`) for all platforms to ensure compatibility with mobile apps.

#### Direct Outbound

For traffic that should bypass the proxy.

```json
{
  "type": "direct",
  "tag": "direct"
}
```

#### Block Outbound

For traffic that should be blocked.

```json
{
  "type": "block",
  "tag": "block"
}
```

### Route

Traffic routing rules.

```json
{
  "route": {
    "rules": [
      {
        "protocol": "dns",
        "outbound": "dns-out"
      },
      {
        "ip_is_private": true,
        "outbound": "direct"
      }
    ],
    "final": "ssh-out"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `rules` | array | List of routing rules |
| `final` | string | Default outbound if no rules match |

**Common Routing Rules:**

| Rule | Purpose |
|------|---------|
| `protocol: dns` | Route DNS queries |
| `ip_is_private: true` | Direct access to LAN |
| `domain_suffix: .local` | Direct access to local domains |

## sing-box URI Format

For QR codes and easy sharing, sbconfig generates sing-box URIs:

```
ssh://user@server:port?private_key=BASE64_ENCODED_KEY&host_key_algorithms=...
```

### URI Components

| Component | Description |
|-----------|-------------|
| `ssh://` | Protocol scheme |
| `user` | SSH username |
| `server` | Server hostname |
| `port` | SSH port |
| `private_key` | Base64-encoded private key |
| `host_key_algorithms` | Comma-separated list |

### Example URI

```
ssh://user_beta@liberty.zyberis.com:7344?private_key=LS0tLS1CRUdJTiBPUEVOU1NIIFBSSVZBVEUgS0VZLS0tLS0K...&host_key_algorithms=ssh-ed25519,ecdsa-sha2-nistp256
```

## Configuration Templates

### Rust Template Code

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq)]
pub enum OperatingMode {
    Development,
    Production,
}

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub log: LogConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub route: RouteConfig,
}

impl ClientConfig {
    pub fn generate(user: &User, settings: &Settings, mode: OperatingMode) -> Self {
        // Determine server address based on mode
        let server_address = match mode {
            OperatingMode::Development => "localhost".to_string(),
            OperatingMode::Production => settings.server_domain.clone()
                .unwrap_or_else(|| settings.server_ip.clone()),
        };
        
        Self {
            log: LogConfig {
                level: "info".to_string(),
                timestamp: true,
            },
            dns: DnsConfig {
                servers: settings.dns_servers.iter().enumerate().map(|(i, addr)| {
                    DnsServer {
                        tag: format!("dns-{}", i),
                        address: addr.clone(),
                    }
                }).collect(),
            },
            inbounds: vec![
                Inbound::Mixed {
                    tag: "mixed-in".to_string(),
                    listen: "127.0.0.1".to_string(),
                    listen_port: settings.proxy_port,
                }
            ],
            outbounds: vec![
                Outbound::Ssh {
                    tag: "ssh-out".to_string(),
                    server: server_address,  // Mode-dependent!
                    server_port: settings.ssh_port,  // Custom port, NOT 22
                    user: user.username.clone(),
                    private_key: user.private_key.clone(),
                    host_key_algorithms: default_host_key_algorithms(),
                    client_version: "SSH-2.0-OpenSSH_9.0".to_string(),
                },
                Outbound::Direct { tag: "direct".to_string() },
                Outbound::Block { tag: "block".to_string() },
            ],
            route: RouteConfig {
                rules: vec![
                    RouteRule {
                        protocol: Some("dns".to_string()),
                        outbound: "dns-out".to_string(),
                        ..Default::default()
                    },
                    RouteRule {
                        ip_is_private: Some(true),
                        outbound: "direct".to_string(),
                        ..Default::default()
                    },
                ],
                final_outbound: "ssh-out".to_string(),
            },
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
    
    pub fn to_uri(&self) -> String {
        // Extract SSH outbound settings
        if let Some(Outbound::Ssh { server, server_port, user, private_key, .. }) = 
            self.outbounds.iter().find(|o| matches!(o, Outbound::Ssh { .. })) 
        {
            let encoded_key = base64::encode(private_key);
            format!(
                "ssh://{}@{}:{}?private_key={}",
                user, server, server_port, encoded_key
            )
        } else {
            String::new()
        }
    }
}

fn default_host_key_algorithms() -> Vec<String> {
    vec![
        "ssh-ed25519".to_string(),
        "ecdsa-sha2-nistp256".to_string(),
        "rsa-sha2-512".to_string(),
        "rsa-sha2-256".to_string(),
        "ssh-rsa".to_string(),
    ]
}
```

### Example: Development Mode Config

```json
{
  "outbounds": [
    {
      "type": "ssh",
      "tag": "ssh-out",
      "server": "localhost",
      "server_port": 2222,
      "user": "test_user",
      "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n..."
    }
  ]
}
```

### Example: Production Mode Config

```json
{
  "outbounds": [
    {
      "type": "ssh",
      "tag": "ssh-out",
      "server": "liberty.zyberis.com",
      "server_port": 7344,
      "user": "user_beta",
      "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n..."
    }
  ]
}
```

## Platform-Specific Notes

### iOS

- Import via QR code or `sing-box://` URL
- Config is stored within the app
- VPN mode automatically handles system traffic

### Android

- Import via QR code or `sing-box://` URL
- Config is stored within the app
- VPN mode or per-app proxy

### Windows

- Save config to file (e.g., `config.json`)
- Run: `sing-box run -c config.json`
- Configure browser/apps to use `127.0.0.1:10808`

### macOS

- Save config to file
- Run: `sing-box run -c config.json`
- Configure system proxy or use app-specific settings

### Linux

- Save config to file
- Run: `sing-box run -c config.json`
- Set environment variables:
  ```bash
  export http_proxy=http://127.0.0.1:10808
  export https_proxy=http://127.0.0.1:10808
  ```

## QR Code Generation

sbconfig generates QR codes for easy mobile import:

```rust
use qrcode::{QrCode, render::unicode};

pub fn generate_qr_code(uri: &str) -> String {
    let code = QrCode::new(uri.as_bytes()).unwrap();
    
    code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}
```

The QR code contains the full sing-box URI, which can be scanned by the mobile app.

## Routing

sbconfig includes powerful routing capabilities. For detailed documentation, see [docs/routing.md](./routing.md).

### Quick Overview

| Preset | Description |
|--------|-------------|
| **Default** | Proxy all, direct for private IPs |
| **Iran Direct** | Iranian sites direct, ads blocked |
| **Iran Block** | Block all Iranian traffic |
| **China Direct** | Chinese sites direct |
| **Custom** | User-defined rules |

### Default Route Config

```json
{
  "route": {
    "rules": [
      { "ip_is_private": true, "outbound": "direct" }
    ],
    "final": "ssh-out"
  }
}
```

### With Routing Preset (Iran Direct)

```json
{
  "route": {
    "rule_set": [
      {
        "tag": "iran-geosite-ads",
        "type": "remote",
        "format": "binary",
        "url": "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-ads.srs"
      },
      {
        "tag": "iran-geosite-all",
        "type": "remote",
        "format": "binary",
        "url": "https://github.com/bootmortis/sing-geosite/releases/latest/download/geosite-all.srs"
      }
    ],
    "rules": [
      { "ip_is_private": true, "outbound": "direct" },
      { "rule_set": ["iran-geosite-ads"], "outbound": "block" },
      { "rule_set": ["iran-geosite-all"], "outbound": "direct" }
    ],
    "final": "ssh-out"
  }
}
```

## Validation

Before generating configs, sbconfig validates:

1. **User exists** and is active
2. **Private key** is valid and decryptable
3. **Server settings** are configured (domain/IP, port)
4. **Port numbers** are in valid range (1-65535)

## References

- [sing-box SSH Outbound Documentation](https://sing-box.sagernet.org/configuration/outbound/ssh/)
- [sing-box Configuration Introduction](https://sing-box.sagernet.org/configuration/)
- [sing-box Mixed Inbound](https://sing-box.sagernet.org/configuration/inbound/mixed/)
- [sing-box Route Configuration](https://sing-box.sagernet.org/configuration/route/)

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Database](./database.md) - User and config storage
- [Features](./features.md) - Feature specifications
- [Routing](./routing.md) - Traffic routing configuration (presets, custom rules, rule sets)
