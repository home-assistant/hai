# HAI - Home Assistant Installer

A cross-platform desktop application for installing Home Assistant OS on various hardware platforms.

## Features

- **Single Board Computers** - Flash SD cards for Raspberry Pi, ODROID, and more
- **Mini PCs** - Install on generic x86-64 or ARM64 devices
- **Home Assistant Hardware** - Flash or restore Yellow and Green devices
- **Proxmox VE** - Create Home Assistant VMs via API
- **UTM (macOS)** - Automated VM setup on Mac

## Installation

Download the latest release for your platform from the [Releases](https://github.com/home-assistant/hai/releases) page.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (via rustup)
- [Node.js](https://nodejs.org/) 22+
- Platform-specific [Tauri dependencies](https://tauri.app/start/prerequisites/)

### Setup

```bash
# Clone the repository
git clone https://github.com/home-assistant/hai.git
cd hai

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Commands

```bash
npm run tauri dev     # Start development server
npm run lint          # Run ESLint
npm run format        # Format code with Prettier
npm run test          # Run unit tests
npm run test:e2e      # Run E2E tests
```

### Mock Mode

For testing without real hardware:

```bash
HA_INSTALLER_MOCK=true npm run tauri dev
```

## Tech Stack

- **Framework**: [Tauri 2.x](https://tauri.app/)
- **Backend**: Rust
- **Frontend**: [Lit](https://lit.dev/) + TypeScript
- **UI Components**: [Web Awesome](https://webawesome.com/)
- **Build**: [Vite](https://vite.dev/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

Apache 2.0 - See [LICENSE](LICENSE) for details.

---

Part of the [Open Home Foundation](https://www.openhomefoundation.org/)
