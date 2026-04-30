# Architecture

## Tech Stack

- **Framework**: Tauri 2.x
- **Backend**: Rust (Cargo workspace with hai-core + hai-desktop)
- **Frontend**: Lit (Web Components) + TypeScript
- **UI Components**: Web Awesome (the library Home Assistant uses, successor to Shoelace)
- **Build**: Vite

This stack matches what Home Assistant uses for their frontend, ensuring visual and technical consistency.

## Workspace Architecture

HAI uses a Cargo workspace to separate concerns and enable code reuse:

```
┌─────────────────────────────────────────────────────────────────┐
│                       HAI Workspace                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    hai-core                             │   │
│   │              (Shared Rust Library)                      │   │
│   │                                                         │   │
│   │   • Device enumeration (macOS/Linux/Windows)            │   │
│   │   • Image download + verification                       │   │
│   │   • Disk writing + verification                         │   │
│   │   • Proxmox VE API integration                          │   │
│   │   • UTM automation (macOS)                              │   │
│   │   • HA readiness checks                                 │   │
│   │   • Mock mode support                                   │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                            ▲                                    │
│                            │ uses                               │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    hai-desktop                          │   │
│   │               (Tauri Desktop App)                       │   │
│   │                                                         │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │              Frontend (Web)                     │   │   │
│   │   │    Lit components, Web Awesome UI               │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   │                        │                                │   │
│   │                        │ invoke('command', args)        │   │
│   │                        ▼                                │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │         Tauri Commands (Thin Wrappers)          │   │   │
│   │   │    Adapts hai-core to Tauri IPC channels        │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Why This Architecture?

1. **Code Reuse**: Core logic can be shared with future TUI installer for live USB
2. **Better Testing**: hai-core can be tested independently without Tauri
3. **Cleaner Separation**: Business logic separate from UI framework concerns
4. **Maintainability**: Fix once in hai-core, benefit everywhere

## Project Structure

```
home-assistant-installer/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── hai-core/                 # Shared Rust library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API exports
│   │       ├── error.rs          # Unified error types
│   │       ├── types.rs          # Shared data types
│   │       ├── devices.rs        # Block device enumeration
│   │       ├── download.rs       # Image download + verification
│   │       ├── flash.rs          # Disk writing + verification
│   │       ├── proxmox.rs        # Proxmox VE API
│   │       ├── utm.rs            # UTM automation (macOS)
│   │       ├── network.rs        # HA readiness checks
│   │       └── mock.rs           # Mock mode support
│   │
│   └── hai-desktop/              # Tauri desktop app
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── build.rs
│       ├── capabilities/
│       ├── icons/
│       ├── src/
│       │   ├── main.rs
│       │   ├── lib.rs
│       │   └── commands.rs       # Thin wrappers around hai-core
│       └── frontend/             # Web UI
│           ├── package.json
│           ├── index.html
│           ├── vite.config.ts
│           ├── tsconfig.json
│           └── src/
│               ├── main.ts
│               ├── styles.css
│               ├── state/
│               │   └── wizard-state.ts
│               ├── api/
│               │   ├── commands.ts
│               │   ├── types.ts
│               │   ├── mock-data.ts
│               │   └── index.ts
│               ├── components/
│               │   ├── app-shell.ts
│               │   ├── wizard-shell.ts
│               │   ├── step-indicator.ts
│               │   ├── device-card.ts
│               │   ├── drive-card.ts
│               │   ├── progress-bar.ts
│               │   ├── option-card.ts
│               │   ├── confirm-dialog.ts
│               │   └── info-dialog.ts
│               └── views/
│                   ├── welcome-view.ts
│                   ├── path-selection-view.ts
│                   ├── other-options-view.ts
│                   ├── sbc/
│                   ├── minipc/
│                   ├── ha-hardware/
│                   ├── proxmox/
│                   └── utm/
│
├── docs/
│   ├── spec/                     # This documentation
│   └── project.md                # Roadmap
├── test/
│   ├── unit/                     # Frontend unit tests
│   └── e2e/                      # Playwright E2E tests
├── .github/
│   └── workflows/
├── package.json                  # Root package.json with scripts
├── playwright.config.ts
└── ...config files
```

## hai-core Crate

The shared library containing all business logic.

### Modules

| Module | Description |
|--------|-------------|
| `types` | Shared data types (`BlockDevice`, `FlashProgress`, etc.) |
| `error` | Unified error handling with `thiserror` |
| `devices` | Platform-specific block device enumeration |
| `download` | Image download, verification, extraction, caching |
| `flash` | Disk writing with progress and verification |
| `proxmox` | Proxmox VE API client |
| `utm` | UTM automation via AppleScript (macOS) |
| `network` | Connectivity and HA readiness checks |
| `mock` | Mock data for testing |

### Progress Callback Trait

Core uses a generic trait for progress reporting instead of Tauri-specific channels:

```rust
pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, progress: FlashProgress);
}

// Works with closures
impl<F> ProgressCallback for F
where
    F: Fn(FlashProgress) + Send + Sync,
{
    fn on_progress(&self, progress: FlashProgress) {
        self(progress);
    }
}
```

This allows hai-desktop to adapt it to Tauri channels, while a future TUI can adapt it to terminal UI updates.

## hai-desktop Crate

The Tauri desktop application with thin command wrappers.

### Command Pattern

Commands are thin wrappers that:
1. Call hai-core functions
2. Adapt progress callbacks to Tauri channels
3. Convert errors to strings for frontend

```rust
use hai_core::{devices, FlashProgress, ProgressCallback};
use tauri::ipc::Channel;

struct TauriProgressAdapter(Channel<FlashProgress>);

impl ProgressCallback for TauriProgressAdapter {
    fn on_progress(&self, progress: FlashProgress) {
        self.0.send(progress).ok();
    }
}

#[tauri::command]
pub async fn list_block_devices() -> Result<Vec<BlockDevice>, String> {
    hai_core::devices::list_block_devices()
        .await
        .map_err(|e| e.to_string())
}
```

## Project Configuration

### tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Home Assistant Installer",
  "version": "0.1.0",
  "identifier": "org.openhomefoundation.hai",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Home Assistant Installer",
        "width": 900,
        "height": 780,
        "resizable": true,
        "minWidth": 800,
        "minHeight": 750
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; frame-src https://toolbox.openhomefoundation.org/"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Prerequisites

- Rust (via rustup)
- Node.js 20+
- Platform-specific dependencies (see Tauri docs)

### Initialize Project

```bash
# Create Tauri app
npm create tauri-app@latest hai -- --template vanilla-ts

cd hai

# Add Lit and Web Awesome
npm install lit
npm install qrcode

# Start development
npm run tauri dev
```

## Web Awesome Integration

Web Awesome (formerly Shoelace) provides the component library. Key components:

- `<wa-button>` - Actions and navigation
- `<wa-card>` - Option cards, info panels
- `<wa-input>` - Text inputs (Proxmox URL, credentials)
- `<wa-select>` - Dropdowns (node selection, storage)
- `<wa-progress-bar>` - Download/flash progress
- `<wa-alert>` - Warnings, errors, success messages
- `<wa-spinner>` - Loading states
- `<wa-icon>` - Icons throughout
- `<wa-dialog>` - Confirmation dialogs

## Example Lit Component

```typescript
import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('option-card')
export class OptionCard extends LitElement {
  @property() title = '';
  @property() description = '';
  @property({ type: Boolean }) selected = false;

  static styles = css`
    :host {
      display: block;
    }
    wa-card {
      cursor: pointer;
      transition: border-color 0.2s;
    }
    wa-card:hover {
      border-color: var(--wa-color-primary-500);
    }
    wa-card[data-selected] {
      border-color: var(--wa-color-primary-600);
      background: var(--wa-color-primary-50);
    }
  `;

  render() {
    return html`
      <wa-card ?data-selected=${this.selected} @click=${this._onClick}>
        <div slot="header">${this.title}</div>
        <p>${this.description}</p>
      </wa-card>
    `;
  }

  private _onClick() {
    this.dispatchEvent(new CustomEvent('select', { bubbles: true }));
  }
}
```

## Platform-Specific Considerations

### macOS

- Disk access requires elevated privileges (`diskutil`, direct writes)
- UTM integration via `utmctl` CLI or AppleScript
- Code signing and notarization required for distribution

### Windows

- Disk access requires Administrator privileges
- Use `CreateFile` with `\\.\PhysicalDriveN` for raw access
- Consider Windows Defender / SmartScreen implications

### Linux

- Disk access requires root or appropriate permissions
- Direct `/dev/sdX` access
- AppImage or Flatpak distribution

## Security Considerations

- Never store credentials (Proxmox creds are session-only, in memory)
- Verify image checksums before flashing
- Warn clearly before destructive operations
- Request minimal necessary privileges
