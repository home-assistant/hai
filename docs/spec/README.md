# HAI - Home Assistant Installer Specification

## Overview

HAI (Home Assistant Installer) is a cross-platform desktop application that simplifies installing Home Assistant OS across multiple platforms. The app guides users through an intent-based flow, automatically handling image downloads, device preparation, and VM provisioning where supported.

- **Full name**: Home Assistant Installer
- **Short name**: HAI (also means "Hi!" in Dutch)
- **Binary name**: `hai`

Built with Tauri (Rust backend + web frontend using Lit and Web Awesome).

## Documentation Structure

| Document | Description |
|----------|-------------|
| [architecture.md](./architecture.md) | Tech stack, project structure, Tauri setup |
| [ui-design.md](./ui-design.md) | Visual-first philosophy, mockups, assets, branding |
| [user-flows.md](./user-flows.md) | All installation flows (SBC, mini PC, Proxmox, UTM) |
| [backend.md](./backend.md) | Rust commands, platform-specific code, manifest handling |
| [testing.md](./testing.md) | Test strategy, Playwright, mock mode |
| [ci-cd.md](./ci-cd.md) | GitHub Actions, releases, signing, cosign |
| [contributing.md](./contributing.md) | Issue templates, renovate, AI instructions |

## Quick Links

- [Project Roadmap](../project.md) - Phased implementation plan
- [Contributing Guide](./contributing.md#contributing-guide)

## Target Platforms

The installer itself runs on:
- macOS (Apple Silicon and Intel)
- Windows
- Linux

## Installation Paths Supported

### Fully Automated
- Single Board Computers (Pi 3/4/5, ODROID, etc.) - Flash SD card
- Mini PC / NUC (Generic x86-64 or ARM64) - Flash connected SSD/NVMe
- Home Assistant Hardware (Yellow, Green) - Flash or restore
- Proxmox VE - API-driven VM creation
- macOS VM - Automated UTM setup

### Documentation Links Only
- Windows VMs (Hyper-V)
- Linux VMs (KVM, VirtualBox)
- Containers (Docker, Portainer)
- NAS devices (Unraid, Synology)
