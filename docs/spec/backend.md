# Backend (Rust)

## Rust Backend Modules

```
src-tauri/src/
├── main.rs             # Application entry point
├── lib.rs              # Tauri plugin setup and command registration
├── commands.rs         # All Tauri command handlers
├── types.rs            # Shared type definitions (BlockDevice, FlashProgress, etc.)
├── block_devices.rs    # Device enumeration (platform-specific)
├── disk_writer.rs      # Image writing with progress
├── download.rs         # Image downloads with progress and extraction
├── proxmox.rs          # Proxmox VE API integration
├── utm.rs              # UTM automation (macOS only)
└── mock.rs             # Mock mode support for testing
```

## Key Tauri Commands

```rust
// Device enumeration
#[tauri::command]
async fn list_block_devices() -> Result<Vec<BlockDevice>, String>

// Image operations
#[tauri::command]
async fn download_image(device_type: String, window: Window) -> Result<PathBuf, String>

#[tauri::command]
async fn flash_image(image_path: PathBuf, target_device: String, window: Window) -> Result<(), String>

// Proxmox
#[tauri::command]
async fn proxmox_connect(url: String, username: String, password: String) -> Result<ProxmoxSession, String>

#[tauri::command]
async fn proxmox_list_nodes(session: ProxmoxSession) -> Result<Vec<Node>, String>

#[tauri::command]
async fn proxmox_list_storage(session: ProxmoxSession, node: String) -> Result<Vec<Storage>, String>

#[tauri::command]
async fn proxmox_create_vm(session: ProxmoxSession, config: VmConfig, window: Window) -> Result<u32, String>

// UTM (macOS only)
#[tauri::command]
async fn utm_is_installed() -> bool

#[tauri::command]
async fn utm_create_vm(config: VmConfig, window: Window) -> Result<(), String>

// Updates
#[tauri::command]
async fn check_for_updates(include_beta: bool) -> Result<Option<UpdateInfo>, String>

// Companion apps (macOS)
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn is_ha_mac_app_installed() -> bool

#[tauri::command]
#[cfg(target_os = "macos")]
pub fn open_mac_app_store()
```

---

## Device and Image Manifest

### Source

Fetched from `version.home-assistant.io` and cached locally.

### Manifest Structure

```json
{
  "devices": [
    {
      "id": "rpi5",
      "name": "Raspberry Pi 5",
      "category": "sbc",
      "image": "rpi5",
      "icon": "rpi5.svg"
    },
    {
      "id": "rpi4",
      "name": "Raspberry Pi 4",
      "category": "sbc",
      "image": "rpi4",
      "icon": "rpi4.svg"
    },
    {
      "id": "generic-x86-64",
      "name": "Generic x86-64",
      "category": "generic",
      "image": "generic-x86-64",
      "icon": "x86.svg"
    },
    {
      "id": "yellow",
      "name": "Home Assistant Yellow",
      "category": "ha-hardware",
      "image": "yellow",
      "icon": "yellow.svg"
    }
  ],
  "images": [
    {
      "id": "rpi5",
      "url": "https://github.com/home-assistant/operating-system/releases/download/14.0/haos_rpi5-64-14.0.img.xz",
      "version": "14.0",
      "sha256": "..."
    }
  ]
}
```

### Caching Strategy

```rust
async fn get_manifest() -> Result<Manifest, Error> {
    let cache_path = app_data_dir().join("manifest_cache.json");
    
    match fetch_from_server("https://version.home-assistant.io/...").await {
        Ok(manifest) => {
            // Update cache
            save_to_cache(&cache_path, &manifest)?;
            Ok(manifest)
        }
        Err(e) => {
            // Try cached version
            if cache_path.exists() {
                let cached = load_from_cache(&cache_path)?;
                // Warn user they're using cached data
                emit_warning("Using cached device list. Some options may be outdated.");
                Ok(cached)
            } else {
                Err(Error::NoInternetAndNoCache)
            }
        }
    }
}
```

### Network Connectivity Check

On launch:
1. Attempt to fetch manifest from `version.home-assistant.io`
2. If successful: use fresh manifest, update cache
3. If failed + cache exists: show warning, proceed with cached manifest
4. If failed + no cache: show error, explain internet is required, offer retry

---

## App Updates

**No auto-update in v1.** Instead, version check with download prompt.

### Version Check Implementation

```rust
#[derive(Deserialize)]
struct InstallerVersions {
    stable: VersionInfo,
    beta: Option<VersionInfo>,
}

#[derive(Deserialize)]
struct VersionInfo {
    version: String,
    download_url: String,
}

#[tauri::command]
async fn check_for_updates(include_beta: bool) -> Result<Option<UpdateInfo>, String> {
    let current = env!("CARGO_PKG_VERSION");
    let versions = fetch_installer_versions().await?;
    
    // Check beta first if opted in
    if include_beta {
        if let Some(beta) = versions.beta {
            if is_newer(&beta.version, current) {
                return Ok(Some(UpdateInfo {
                    version: beta.version,
                    download_url: beta.download_url,
                    is_beta: true,
                }));
            }
        }
    }
    
    // Check stable
    if is_newer(&versions.stable.version, current) {
        return Ok(Some(UpdateInfo {
            version: versions.stable.version,
            download_url: versions.stable.download_url,
            is_beta: false,
        }));
    }
    
    Ok(None)
}

fn is_newer(available: &str, current: &str) -> bool {
    semver::Version::parse(available).ok()
        .zip(semver::Version::parse(current).ok())
        .map(|(a, c)| a > c)
        .unwrap_or(false)
}
```

---

## Companion App Detection (macOS)

```rust
// Check if Mac app is installed
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn is_ha_mac_app_installed() -> bool {
    std::path::Path::new("/Applications/Home Assistant.app").exists()
}

// Open Mac App Store
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn open_mac_app_store() {
    std::process::Command::new("open")
        .arg("macappstore://apps.apple.com/app/id1099568401")
        .spawn()
        .ok();
}
```

---

## QR Code Generation (Frontend)

```typescript
import QRCode from 'qrcode';

const IOS_APP_URL = 'https://apps.apple.com/app/home-assistant/id1099568401';
const ANDROID_APP_URL = 'https://play.google.com/store/apps/details?id=io.homeassistant.companion.android';

// In component
async generateQRCodes() {
  this.iosQR = await QRCode.toDataURL(IOS_APP_URL, { width: 120 });
  this.androidQR = await QRCode.toDataURL(ANDROID_APP_URL, { width: 120 });
}
```

---

## Toolbox Integration

```typescript
@customElement('toolbox-view')
export class ToolboxView extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }
    .toolbar {
      padding: 8px 16px;
      border-bottom: 1px solid var(--wa-color-neutral-200);
    }
    iframe {
      width: 100%;
      height: calc(100% - 48px);
      border: none;
    }
  `;

  render() {
    return html`
      <div class="toolbar">
        <wa-button variant="text" @click=${this._close}>
          ← Back to Installer
        </wa-button>
      </div>
      <iframe src="https://toolbox.openhomefoundation.org/"></iframe>
    `;
  }

  private _close() {
    this.dispatchEvent(new CustomEvent('close-toolbox'));
  }
}
```

CSP Configuration (tauri.conf.json):
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; frame-src https://toolbox.openhomefoundation.org/"
    }
  }
}
```
