# Home Assistant Installer - Copilot Instructions

## Project Overview
This is a Tauri desktop application for installing Home Assistant OS.
- Backend: Rust (src-tauri/)
- Frontend: Lit web components + Web Awesome UI library (src/)
- Testing: Playwright for E2E, Web Test Runner for unit tests

## Code Style

### Rust
- Use `rustfmt` defaults
- Prefer `thiserror` for error types
- Use `tokio` for async runtime
- All public functions need doc comments

### TypeScript/Lit
- Use TypeScript strict mode
- Components use `@customElement` decorator
- Styles use CSS-in-JS via Lit's `css` tagged template
- Prefer Web Awesome components (`<wa-*>`) over custom implementations

## Architecture Guidelines
- Tauri commands go in `src-tauri/src/commands/`
- Each command module handles one domain (devices, flash, proxmox, utm)
- Frontend state management uses simple stores in `src/state/`
- Components are in `src/components/`, views in `src/views/`

## Testing
- All Tauri commands need unit tests
- All components need basic render tests
- User flows need Playwright E2E tests
- Use mock mode (`HA_INSTALLER_MOCK=true`) for testing without hardware

## Common Tasks
- Add new device: Update manifest schema and device selector component
- Add new installation path: Create new view folder, add to wizard routing
- Fix flashing issue: Check platform-specific code in `src-tauri/src/platforms/`
