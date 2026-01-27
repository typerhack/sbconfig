// src/dns_presets.rs
// DNS server presets for config generation

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsPreset {
    Google,
    Cloudflare,
    AdGuard,
    Quad9,
    Shecan,      // Iran
    Electro,     // Iran
    FourZeroThree, // 403.online (Iran)
}

impl DnsPreset {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Google,
            Self::Cloudflare,
            Self::AdGuard,
            Self::Quad9,
            Self::Shecan,
            Self::Electro,
            Self::FourZeroThree,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Google => "Google DNS",
            Self::Cloudflare => "Cloudflare DNS",
            Self::AdGuard => "AdGuard DNS",
            Self::Quad9 => "Quad9 DNS",
            Self::Shecan => "Shecan DNS",
            Self::Electro => "Electro DNS",
            Self::FourZeroThree => "403.online DNS",
        }
    }

    pub fn servers(&self) -> Vec<String> {
        match self {
            Self::Google => vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            Self::Cloudflare => vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()],
            Self::AdGuard => vec!["94.140.14.14".to_string(), "94.140.15.15".to_string()],
            Self::Quad9 => vec!["9.9.9.9".to_string(), "149.112.112.112".to_string()],
            Self::Shecan => vec!["178.22.122.100".to_string(), "185.51.200.2".to_string()],
            Self::Electro => vec!["78.157.42.100".to_string(), "78.157.42.101".to_string()],
            Self::FourZeroThree => vec!["10.202.10.202".to_string(), "10.202.10.102".to_string()],
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Google => "8.8.8.8, 8.8.4.4",
            Self::Cloudflare => "1.1.1.1, 1.0.0.1",
            Self::AdGuard => "94.140.14.14, 94.140.15.15 (blocks ads)",
            Self::Quad9 => "9.9.9.9, 149.112.112.112 (blocks malware)",
            Self::Shecan => "178.22.122.100, 185.51.200.2 (Iran)",
            Self::Electro => "78.157.42.100, 78.157.42.101 (Iran)",
            Self::FourZeroThree => "10.202.10.202, 10.202.10.102 (Iran)",
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self::Google | Self::Cloudflare)
    }

    #[allow(dead_code)]
    pub fn is_iran(&self) -> bool {
        matches!(self, Self::Shecan | Self::Electro | Self::FourZeroThree)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DnsSelection {
    pub selected_presets: Vec<DnsPreset>,
    pub custom: Vec<String>,
}

impl Default for DnsSelection {
    fn default() -> Self {
        Self {
            selected_presets: vec![DnsPreset::Google, DnsPreset::Cloudflare],
            custom: vec![],
        }
    }
}

impl DnsSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self, preset: DnsPreset) {
        if let Some(pos) = self.selected_presets.iter().position(|p| *p == preset) {
            self.selected_presets.remove(pos);
        } else {
            self.selected_presets.push(preset);
        }
    }

    pub fn is_selected(&self, preset: DnsPreset) -> bool {
        self.selected_presets.contains(&preset)
    }

    pub fn to_servers(&self) -> Vec<String> {
        let mut servers = Vec::new();
        for preset in &self.selected_presets {
            servers.extend(preset.servers());
        }
        servers.extend(self.custom.clone());
        servers
    }

    pub fn from_servers(servers: &[String]) -> Self {
        let mut selection = Self::new();
        selection.selected_presets.clear();

        // Try to match servers to presets
        for preset in DnsPreset::all() {
            let preset_servers = preset.servers();
            if servers.iter().any(|s| preset_servers.contains(s)) {
                selection.selected_presets.push(preset);
            }
        }

        // If no presets matched, use default
        if selection.selected_presets.is_empty() {
            selection.selected_presets = vec![DnsPreset::Google, DnsPreset::Cloudflare];
        }

        selection
    }
}
