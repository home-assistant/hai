# Backend (Rust)

## Workspace Structure

The Rust backend is split into two crates:

### hai-core (Shared Library)

```
crates/hai-core/src/
├── lib.rs              # Public API exports and traits
├── error.rs            # Unified error types
├── types.rs            # Shared type definitions
├── devices.rs          # Device enumeration (platform-specific)
├── download.rs         # Image downloads with progress and extraction
├── flash.rs            # Image writing with progress and verification
├── proxmox.rs          # Proxmox VE API integration
├── utm.rs              # UTM automation (macOS only)
├── network.rs          # HA readiness checks
└── mock.rs             # Mock mode support for testing
```

### hai-desktop (Tauri App)

```
crates/hai-desktop/src/
├── main.rs             # Application entry point
├── lib.rs              # Tauri plugin setup and command registration
└── commands.rs         # Thin wrappers around hai-core
```

---

## hai-core Public API

### Error Handling

```rust
// crates/hai-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device is busy: {0}")]
    DeviceBusy(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Proxmox API error: {0}")]
    ProxmoxApi(String),

    #[error("UTM error: {0}")]
    Utm(String),

    #[error("Drive disconnected")]
    DriveDisconnected,

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Progress Callback Trait

```rust
// crates/hai-core/src/lib.rs
use crate::types::FlashProgress;

/// Trait for receiving progress updates during long operations.
/// Implemented by Tauri channel adapters, TUI update handlers, etc.
pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, progress: FlashProgress);
}

/// Convenience implementation for closures
impl<F> ProgressCallback for F
where
    F: Fn(FlashProgress) + Send + Sync,
{
    fn on_progress(&self, progress: FlashProgress) {
        self(progress);
    }
}
```

### Device Enumeration

```rust
// crates/hai-core/src/devices.rs

/// List all block devices suitable for flashing
pub async fn list_block_devices() -> Result<Vec<BlockDevice>>;

/// List only removable devices (SD cards, USB drives) for SBC flow
pub async fn list_removable_devices() -> Result<Vec<BlockDevice>>;

/// List only internal devices (NVMe, SATA) for mini PC flow
pub async fn list_internal_devices() -> Result<Vec<BlockDevice>>;
```

### Image Download

```rust
// crates/hai-core/src/download.rs

/// Fetch current stable HAOS version
pub async fn fetch_stable_version() -> Result<String>;

/// Fetch release information for a specific board
pub async fn fetch_release_info(board: &str) -> Result<HaosRelease>;

/// Download image with progress reporting
pub async fn download_image(
    release: &HaosRelease,
    callback: &dyn ProgressCallback,
) -> Result<PathBuf>;

/// Verify checksum of downloaded file
pub async fn verify_checksum(path: &Path, expected: &str) -> Result<bool>;

/// Extract xz-compressed image
pub async fn extract_image(
    source: &Path,
    callback: &dyn ProgressCallback,
) -> Result<PathBuf>;
```

### Disk Flashing

```rust
// crates/hai-core/src/flash.rs

/// Flash an image to a target device
pub async fn flash_image(
    request: FlashRequest,
    callback: &dyn ProgressCallback,
) -> Result<FlashResult>;

/// Validate a device is safe to write to (not system drive)
pub fn validate_device(device: &str) -> Result<()>;

/// Unmount a device before flashing
pub async fn unmount_device(device: &str) -> Result<()>;

/// Eject a device after flashing (macOS)
pub async fn eject_device(device: &str) -> Result<()>;
```

### Proxmox Integration

```rust
// crates/hai-core/src/proxmox.rs

pub struct ProxmoxClient {
    // ... internal state
}

impl ProxmoxClient {
    /// Connect to a Proxmox server
    pub async fn connect(credentials: ProxmoxCredentials) -> Result<Self>;

    /// List available nodes
    pub async fn list_nodes(&self) -> Result<Vec<ProxmoxNode>>;

    /// List storage on a node
    pub async fn list_storage(&self, node: &str) -> Result<Vec<ProxmoxStorage>>;

    /// Get next available VM ID
    pub async fn get_next_vm_id(&self) -> Result<u32>;

    /// Upload image to storage with progress
    pub async fn upload_image(
        &self,
        node: &str,
        storage: &str,
        path: &Path,
        callback: &dyn ProgressCallback,
    ) -> Result<String>;

    /// Create a VM
    pub async fn create_vm(
        &self,
        config: ProxmoxVmConfig,
        callback: &dyn ProgressCallback,
    ) -> Result<ProxmoxVmResult>;

    /// Start a VM
    pub async fn start_vm(&self, node: &str, vmid: u32) -> Result<()>;

    /// Wait for VM to get an IP address
    pub async fn wait_for_ip(&self, node: &str, vmid: u32) -> Result<String>;
}
```

### UTM Integration (macOS)

```rust
// crates/hai-core/src/utm.rs

pub struct UtmClient;

impl UtmClient {
    /// Check if UTM is installed
    pub fn is_installed() -> bool;

    /// Get UTM version
    pub fn get_version() -> Result<String>;

    /// Create a new VM
    pub async fn create_vm(
        config: UtmConfig,
        callback: &dyn ProgressCallback,
    ) -> Result<String>;

    /// Start a VM
    pub async fn start_vm(name: &str) -> Result<()>;

    /// Stop a VM
    pub async fn stop_vm(name: &str) -> Result<()>;

    /// Get VM status and IP
    pub async fn get_vm_status(name: &str) -> Result<UtmVmStatus>;
}
```

### Network Checks

```rust
// crates/hai-core/src/network.rs

/// Check if we have internet connectivity
pub async fn check_connectivity() -> ConnectivityStatus;

/// Check if Home Assistant is ready (port 8123 responding)
pub async fn check_ha_ready(ip: &str) -> Result<bool>;

/// Check if Home Assistant has finished updating (manifest.json)
pub async fn check_ha_updated(ip: &str) -> Result<bool>;
```

---

## hai-desktop Commands

Thin wrappers that adapt hai-core to Tauri IPC:

```rust
// crates/hai-desktop/src/commands.rs
use hai_core::{self, BlockDevice, FlashProgress, ProgressCallback};
use tauri::ipc::Channel;

/// Adapter to convert Tauri Channel to ProgressCallback
struct TauriProgressAdapter(Channel<FlashProgress>);

impl ProgressCallback for TauriProgressAdapter {
    fn on_progress(&self, progress: FlashProgress) {
        self.0.send(progress).ok();
    }
}
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
