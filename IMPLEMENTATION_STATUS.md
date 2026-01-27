# Config Settings Separation - Implementation Status

## Overview
Implementing separate Client and Server configuration settings with improved UI including DNS presets multi-select.

## Completed Changes

### 1. Database Schema (✓ DONE)
- **File**: `src/db/migrations.rs`
- Added migration #3 with separate client/server settings:
  - `client_routing_preset`, `client_dns_servers`, `client_block_ads`, `client_proxy_port`
  - `server_routing_preset`, `server_dns_servers`, `server_block_ads`, `server_block_iran`
- Backward compatibility: migrates old `config_*` keys to `client_*` keys

### 2. DNS Presets Module (✓ DONE)
- **File**: `src/dns_presets.rs` (NEW)
- Created `DnsPreset` enum with:
  - Google, Cloudflare, AdGuard, Quad9
  - Shecan, Electro, 403.online (Iran-specific)
- `DnsSelection` struct for managing selected DNS servers
- Helper methods: `is_default()`, `is_iran()`, `to_servers()`, `from_servers()`

### 3. App Structure Updates (✓ DONE)
- **File**: `src/ui/app.rs`
- Updated `Screen` enum:
  - Changed `ConfigSettings` → `ClientConfigSettings` + `ServerConfigSettings`
- Created new state structs:
  - `ClientConfigState` with `ClientConfigFocus`
  - `ServerConfigState` with `ServerConfigFocus`
- Updated `App` struct to use new states:
  - `config_settings_state` → `client_config_state` + `server_config_state`

### 4. Menu Updates (✓ DONE)
- Updated configs menu to show 4 options:
  - Client Config Settings
  - Server Config Settings
  - Generate Configs
  - Help / How to Use
- Updated navigation handling for new menu structure

### 5. UI Rendering Setup (✓ DONE)
- Updated title generation for both screens
- Updated draw call routing
- Updated footer navigation hints

## Remaining Work

###  6. Screen Rendering Functions (TODO)
Need to create these functions in `src/ui/app.rs`:

```rust
fn draw_client_config_settings(&self, frame: &mut Frame, area: Rect) {
    // Render client config settings with:
    // - Routing preset selection (radio buttons with descriptions)
    // - DNS presets multi-select (checkboxes)
    // - Custom DNS input
    // - Block ads toggle
    // - Proxy port input
    // - Organized sections with clear defaults
}

fn draw_server_config_settings(&self, frame: &mut Frame, area: Rect) {
    // Render server config settings with:
    // - Routing preset selection
    // - DNS presets multi-select
    // - Custom DNS input
    // - Block ads toggle
    // - Block Iran toggle
    // - Organized sections with clear defaults
}
```

### 7. Input Handling Functions (TODO)
Need to create in `src/ui/app.rs`:

```rust
fn handle_client_config_input(&mut self, key: KeyCode) {
    // Handle:
    // - Tab/Shift+Tab for focus navigation
    // - Up/Down for list selection
    // - Space for toggle checkboxes
    // - Char input for text fields
    // - Enter for routing preset cycle
}

fn handle_server_config_input(&mut self, key: KeyCode) {
    // Similar to client but with server-specific fields
}
```

### 8. Load/Save Functions (TODO)
Need to create in `src/ui/app.rs`:

```rust
fn load_client_config_settings(&mut self) {
    // Load from settings:
    // - client_routing_preset
    // - client_dns_servers → parse to DnsSelection
    // - client_block_ads
    // - client_proxy_port
}

fn save_client_config_settings(&mut self) -> Result<()> {
    // Save to database:
    // - Serialize DNS selection to comma-separated string
    // - Validate proxy port
    // - Store all client_* settings
}

fn load_server_config_settings(&mut self) {
    // Load server settings from database
}

fn save_server_config_settings(&mut self) -> Result<()> {
    // Save server settings to database
}
```

### 9. Input Routing Updates (TODO)
Need to update the match statement around line 2293-2300:

```rust
match self.screen {
    // ... existing screens ...
    Screen::ClientConfigSettings => self.handle_client_config_input(key.code),
    Screen::ServerConfigSettings => self.handle_server_config_input(key.code),
    // ... rest of screens ...
}
```

### 10. Ctrl+S Save Handling (TODO)
Update around line 2278-2292 to handle both screens:

```rust
if self.screen == Screen::ClientConfigSettings
    && key.modifiers.contains(KeyModifiers::CONTROL)
    && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
{
    if let Err(err) = self.save_client_config_settings() {
        self.client_config_state.error = Some(err.to_string());
        // ... error handling
    } else {
        self.client_config_state.error = None;
        self.status_message = Some("Client settings saved.".to_string());
        // ... success handling
    }
    return Ok(());
}

// Similar block for ServerConfigSettings
```

### 11. Config Generation Updates (TODO)
Update `generate_config_json` calls to use client settings:

- Change references from `config_routing_preset` to `client_routing_preset`
- Change references from `config_dns_servers` to `client_dns_servers`
- Change references from `config_iran_adblock_enabled` to `client_block_ads`

### 12. Server Config Generation (TODO - Optional Phase 2)
**File**: `src/singbox/config.rs`

Create server-side config generation:
```rust
pub fn generate_server_config(
    listen_port: u16,
    preset: RoutingPreset,
    dns_servers: &[String],
    block_ads: bool,
    block_iran: bool,
) -> Result<ServerConfig> {
    // Generate sing-box server config with:
    // - SSH inbound
    // - Direct/block outbounds
    // - Server-side routing rules
}
```

## UI Design Improvements

### DNS Multi-Select Pattern
```
┌─ DNS Servers ──────────────────────────────────────────────────┐
│ Select one or more DNS providers:                               │
│                                                                  │
│  [✓] Google DNS        8.8.8.8, 8.8.4.4           [DEFAULT]     │
│  [✓] Cloudflare DNS    1.1.1.1, 1.0.0.1           [DEFAULT]     │
│  [ ] AdGuard DNS       94.140.14.14 (blocks ads)                │
│  [ ] Quad9 DNS         9.9.9.9 (blocks malware)                 │
│  [ ] Shecan DNS        178.22.122.100 (Iran)                    │
│  [ ] Electro DNS       78.157.42.100 (Iran)                     │
│  [ ] 403.online DNS    10.202.10.202 (Iran)                     │
│  [ ] Custom:           [____________________]                   │
│                                                                  │
│  Currently active: 8.8.8.8, 8.8.4.4, 1.1.1.1, 1.0.0.1          │
└──────────────────────────────────────────────────────────────────┘
```

### Routing Preset Pattern
```
┌─ Routing Preset ───────────────────────────────────────────────┐
│                                                                 │
│  (•) Default         - Proxy all, direct for private IPs       │
│                        [DEFAULT] [FASTEST SETUP]                │
│                                                                 │
│  ( ) Iran Direct     - Iranian sites direct, ads blocked       │
│                        Recommended for users in Iran            │
│                                                                 │
│  ( ) Iran Block      - Block all Iranian traffic               │
│                        For privacy/avoiding government sites    │
│                                                                 │
│  ( ) China Direct    - Chinese sites direct                    │
│                        Recommended for users in China           │
│                                                                 │
└──────────────────────────────────────────────────────────────────┘
```

## Testing Checklist

- [ ] Navigation between Client and Server config screens works
- [ ] DNS presets can be toggled with checkboxes
- [ ] Routing preset selection shows current selection with (•)
- [ ] Settings persist correctly to database
- [ ] Client config generation uses client_* settings
- [ ] Server config generation uses server_* settings (if implemented)
- [ ] Migration from old config_* settings works
- [ ] UI shows [DEFAULT] badges appropriately
- [ ] Ctrl+S saves settings correctly
- [ ] Error messages display properly

## Files Modified

1. ✓ `src/db/migrations.rs` - Added migration #3
2. ✓ `src/dns_presets.rs` - NEW file
3. ✓ `src/main.rs` - Added dns_presets module
4. ✓ `src/ui/app.rs` - Partial (structure updated, functions needed)
5. ✓ `Cargo.toml` - Bumped version to 0.1.16
6. TODO `src/singbox/config.rs` - Update config generation (optional: add server config)

## Next Steps

1. **Implement draw functions** - Start with `draw_client_config_settings()`
2. **Implement input handlers** - Handle keyboard navigation and toggles
3. **Implement load/save functions** - Database persistence
4. **Update config generation** - Use new client settings keys
5. **Test thoroughly** - Ensure backward compatibility
6. **Document** - Update docs/configuration.md with new UI

## Notes

- The new UI is more user-friendly with clear visual indicators
- DNS preset system makes configuration easier
- Separation of client/server configs enables different policies
- Migration ensures existing setups continue working
- Server config generation is optional for Phase 2
