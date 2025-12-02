# Architecture

## Tech Stack

- **Framework**: Tauri 2.x
- **Backend**: Rust
- **Frontend**: Lit (Web Components) + TypeScript
- **UI Components**: Web Awesome (the library Home Assistant uses, successor to Shoelace)
- **Build**: Vite

This stack matches what Home Assistant uses for their frontend, ensuring visual and technical consistency.

## How Tauri Works

```
┌─────────────────────────────────────────────────────────────────┐
│                        Your Desktop App                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    Frontend (Web)                       │   │
│   │                                                         │   │
│   │   HTML + CSS + JavaScript/TypeScript                    │   │
│   │   Lit components, Web Awesome UI                        │   │
│   │                                                         │   │
│   │   This is what users SEE and INTERACT with              │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                            │                                    │
│                            │ invoke('command', args)            │
│                            ▼                                    │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    Backend (Rust)                       │   │
│   │                                                         │   │
│   │   System access, disk operations, network calls         │   │
│   │   Proxmox API, UTM automation, image flashing           │   │
│   │                                                         │   │
│   │   This does the HEAVY LIFTING and SYSTEM ACCESS         │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
home-assistant-installer/
├── .github/
│   └── workflows/
├── docs/
│   ├── spec/                   # This documentation
│   └── project.md              # Roadmap
├── src/                        # Frontend (TypeScript/Lit)
│   ├── main.ts
│   ├── state/
│   │   └── wizard-state.ts
│   ├── api/
│   │   ├── commands.ts         # Tauri command wrappers
│   │   ├── types.ts            # TypeScript interfaces
│   │   ├── mock-data.ts        # Mock data for testing
│   │   └── index.ts            # Re-exports
│   ├── components/
│   │   ├── app-shell.ts        # Main application shell
│   │   ├── wizard-shell.ts
│   │   ├── step-indicator.ts
│   │   ├── device-card.ts
│   │   ├── drive-card.ts
│   │   ├── progress-bar.ts
│   │   ├── option-card.ts
│   │   ├── confirm-dialog.ts
│   │   └── info-dialog.ts
│   └── views/
│       ├── welcome-view.ts
│       ├── path-selection-view.ts
│       ├── other-options-view.ts
│       ├── sbc/
│       │   ├── device-selection-view.ts
│       │   ├── drive-selection-view.ts
│       │   ├── confirmation-view.ts
│       │   ├── progress-view.ts
│       │   └── success-view.ts
│       ├── minipc/
│       │   ├── setup-method-view.ts
│       │   └── architecture-selection-view.ts
│       ├── ha-hardware/
│       │   └── device-selection-view.ts
│       ├── proxmox/
│       │   ├── proxmox-connect-view.ts
│       │   ├── proxmox-configure-view.ts
│       │   ├── proxmox-confirm-view.ts
│       │   ├── proxmox-progress-view.ts
│       │   └── proxmox-success-view.ts
│       └── utm/
│           ├── utm-check-view.ts
│           ├── utm-configure-view.ts
│           ├── utm-confirm-view.ts
│           ├── utm-progress-view.ts
│           └── utm-success-view.ts
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands.rs         # Tauri command handlers
│   │   ├── types.rs            # Shared types
│   │   ├── block_devices.rs    # Device enumeration
│   │   ├── disk_writer.rs      # Image writing
│   │   ├── download.rs         # Image downloads
│   │   ├── proxmox.rs          # Proxmox API
│   │   ├── utm.rs              # UTM automation (macOS)
│   │   └── mock.rs             # Mock mode support
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── test/
│   └── e2e/
│       ├── navigation.spec.ts
│       ├── proxmox-flow.spec.ts
│       └── utm-flow.spec.ts
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── playwright.config.ts
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
