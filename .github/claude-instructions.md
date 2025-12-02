# Home Assistant Installer - Claude Code Instructions

## Project Context
Home Assistant Installer is a cross-platform desktop app built with Tauri.
It helps users install Home Assistant OS on various hardware platforms.

## Tech Stack
- Tauri 2.x (Rust backend + web frontend)
- Lit for web components
- Web Awesome for UI components
- Playwright for E2E testing
- GitHub Actions for CI/CD

## Key Directories
- `src-tauri/src/commands/` - Tauri command handlers
- `src-tauri/src/platforms/` - Platform-specific implementations (disk access)
- `src/components/` - Reusable Lit components
- `src/views/` - Page-level view components
- `test/e2e/` - Playwright tests

## Design Principles
- Visual-first: UI should be usable without reading
- Every option needs an icon or image
- Use Casita mascot for personality (progress, success, error states)
- Follow Home Assistant brand guidelines

## When Making Changes
1. Check if similar patterns exist in codebase
2. Ensure Rust code passes `cargo clippy`
3. Ensure TypeScript passes `npm run lint`
4. Add tests for new functionality
5. Update relevant documentation

## Important Notes
- Mock mode available for testing: `HA_INSTALLER_MOCK=true`
- Manifest data comes from version.home-assistant.io
- No auto-update; version check with download prompt only
- Releases are signed with cosign
