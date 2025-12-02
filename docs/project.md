# HAI - Home Assistant Installer Roadmap

## Overview

This document outlines the phased implementation of HAI (Home Assistant Installer). Each phase builds on the previous one, allowing incremental development and testing.

---

## Phase 0: Project Foundation

**Goal:** Working Tauri app with basic structure, CI, and tooling in place.

### Project Setup

- [x] Initialize Tauri + Lit project
- [x] Configure TypeScript strict mode
- [x] Add Web Awesome UI library
- [x] Set up project directory structure (components, views, state, api)
- [x] Configure Vite build

### Developer Experience

- [x] Add `.mise.toml` configuration (Node, Rust, zizmor)
- [x] Add ESLint + Prettier config
- [x] Add `rustfmt` config
- [x] Set up VS Code workspace settings
- [x] Add `.github/copilot-instructions.md`
- [x] Add `.github/claude-instructions.md`
- [x] Add `.github/copilot-review.yml`

### CI Foundation

- [x] Add GitHub Actions test workflow (skeleton)
- [x] Add lint job (cargo clippy, eslint)
- [x] Add format check job (cargo fmt, prettier)
- [x] Set up Codecov integration
- [x] Pin GitHub Actions to SHA (with Renovate auto-update)
- [x] Add zizmor for GitHub Actions security linting

### Repository Setup

- [x] Add README.md
- [x] Add LICENSE (Apache 2.0)
- [x] Add CONTRIBUTING.md
- [x] Add issue templates (bug, feature, device request)
- [x] Add PR template
- [x] Add CODEOWNERS
- [x] Add renovate.json

### Verification

- [x] `npm run tauri dev` opens a window
- [x] `npm run lint` passes
- [x] `cargo clippy` passes
- [ ] CI workflow runs successfully

---

## Phase 1: Welcome Screen

**Goal:** User can see the welcome screen and click "Let's go".

### Welcome View

- [x] Create welcome view component
- [x] Add Home Assistant logo
- [x] Add "Installer" title
- [x] Add welcome message text
- [x] Add "Let's go" button
- [x] Add Open Home Foundation logo below button
- [x] Style according to HA brand guidelines

### App Shell

- [x] Create app shell component (root)
- [x] Add basic routing/view switching logic
- [x] Add state management for current view

### Assets

- [x] Add Home Assistant logo (SVG)
- [x] Add Open Home Foundation logo (SVG)
- [x] Add placeholder Casita mascot graphic

### Verification

- [x] App launches with welcome screen
- [x] "Let's go" button is visible and clickable
- [x] Clicking "Let's go" transitions (even to empty view)

---

## Phase 2: Path Selection

**Goal:** User can select an installation path after clicking "Let's go".

### Path Selection View

- [x] Create path selection view component
- [x] Create option card component (reusable)
- [x] Add "Raspberry Pi & other boards" option with image
- [x] Add "Mini PC" option with image
- [x] Add "Home Assistant Hardware" option with image
- [x] Add "Proxmox Server" option with logo
- [x] Add "Virtual Machine" option (macOS only, conditional)
- [x] Add "Other options" link

### Platform Detection

- [x] Detect host OS (macOS/Windows/Linux)
- [x] Conditionally show UTM option on macOS only

### Navigation

- [x] Add back button component
- [x] Wire up back navigation to welcome screen
- [x] Track selected path in state

### Visual Assets

- [x] Add/create placeholder images for each path
- [x] Add Proxmox logo
- [x] Add UTM logo (or placeholder)

### Verification

- [x] All options visible and clickable
- [x] UTM option only shows on macOS
- [x] Back button returns to welcome
- [x] Selected path is tracked

---

## Phase 3: Wizard Shell & Navigation

**Goal:** Robust wizard navigation with step indicators and consistent UX.

### Wizard Shell

- [x] Create wizard shell component
- [x] Add step indicator / breadcrumb component
- [x] Add consistent header with back button
- [x] Add consistent footer with action buttons

### State Management

- [x] Create wizard state store
- [x] Track current flow (sbc, minipc, proxmox, etc.)
- [x] Track current step within flow
- [x] Track selections (device, drive, etc.)

### Navigation Logic

- [x] Implement step forward
- [x] Implement step back
- [x] Implement cancel / start over
- [ ] Handle browser back button (if applicable)

### Verification

- [x] Can navigate forward and back
- [x] Step indicator shows current position
- [x] State persists during navigation

---

## Phase 4: Mock Mode Infrastructure

**Goal:** Can test all flows without real hardware.

### Rust Mock Commands

- [x] Add `HA_INSTALLER_MOCK` environment variable check
- [x] Create mock `list_block_devices` returning fake devices
- [x] Create mock `flash_image` with simulated progress
- [x] Create mock `check_for_updates` response

### Frontend Mock Support

- [x] Add `?mock=true` URL parameter support
- [x] Pass mock flag to Tauri commands

### Mock Data

- [x] Define mock device list (SD cards, USB drives)
- [x] Define mock manifest data

### VS Code Launch Configurations

- [x] Add "Tauri App" launcher (real mode)
- [x] Add "Tauri App (No Cache)" launcher with `HA_INSTALLER_NO_CACHE=true`
- [x] Add "Tauri App (Mock Mode)" launcher with `HA_INSTALLER_MOCK=true`
- [x] Add "Tauri (Debug Rust)" launcher with LLDB
- [x] Add "Tauri (Debug Rust, Mock Mode)" launcher

### Verification

- [x] `HA_INSTALLER_MOCK=true npm run tauri dev` shows mock devices
- [x] Mock flash completes with progress updates

---

## Phase 5: SBC Flow - Device Selection

**Goal:** User can select a single board computer (Pi, ODROID, etc.).

### Device Selection View

- [x] Create device selection view for SBC flow
- [x] Display devices in visual grid with images
- [x] Show device name under each image
- [x] Handle device selection

### Manifest Integration

- [x] Define manifest schema (TypeScript types)
- [x] Create manifest fetch command (Rust)
- [ ] Implement manifest caching
- [ ] Handle offline/cache fallback
- [x] Parse and expose device list to frontend

### Device Data

- [x] Add Raspberry Pi 5 to manifest
- [x] Add Raspberry Pi 4 to manifest
- [x] Add Raspberry Pi 3 to manifest
- [x] Add ODROID devices to manifest
- [x] Add placeholder images for each device

### Verification

- [x] Device list loads from manifest (or mock)
- [x] Devices display with images
- [x] Selection works and is tracked

---

## Phase 6: SBC Flow - Drive Selection

**Goal:** User can select target SD card / USB drive.

### Drive Selection View

- [x] Create drive selection view
- [x] Display available drives with icons
- [x] Show drive name, size, and identifier
- [x] Add "Refresh" button
- [x] Add warning about data erasure
- [x] Handle drive selection
- [x] Sort drives (non-selectable at bottom with warning icon)

### Rust: Drive Enumeration

- [x] Implement `list_block_devices` for macOS
- [x] Implement `list_block_devices` for Linux
- [x] Implement `list_block_devices` for Windows
- [x] Filter out system drives
- [x] Return drive metadata (name, size, type)

### Drive Display

- [x] Create drive icon (SD card vs USB)
- [x] Format drive size (GB)
- [x] Show drive type indicator

### Verification

- [x] Drives are enumerated on all platforms
- [x] System drives are filtered out
- [x] Selection works
- [x] Refresh button updates list

---

## Phase 7: SBC Flow - Confirmation

**Goal:** User sees summary and confirms before flashing.

### Confirmation View

- [x] Create confirmation view
- [x] Show selected device (with image)
- [x] Show selected drive
- [x] Show HAOS version to be installed (fetched from backend)
- [x] Add prominent warning about data loss
- [x] Add macOS password prompt explanation
- [x] Add "Install" button
- [x] Add "Back" button

### Verification

- [x] All selections displayed correctly
- [x] Warning is visible
- [x] Install button proceeds to flash
- [x] Back button returns to drive selection

---

## Phase 8: SBC Flow - Flashing

**Goal:** Image is downloaded and written to drive with progress feedback.

### Progress View

- [x] Create progress view
- [x] Add Casita mascot with thinking cloud
- [x] Add Casita blinking and tongue animations
- [x] Add progress bar component
- [x] Show current stage (downloading, extracting, writing, verifying, finalizing)
- [x] Show stage indicator dots with active/complete states
- [x] Show percentage and size progress
- [x] Show transfer speed (MB/s)
- [x] Show ETA (time remaining)
- [ ] Handle cancel (if possible)

### Rust: Image Download

- [x] Create download module with reqwest client
- [x] Fetch stable version from version.home-assistant.io/stable.json
- [x] Fetch release details from GitHub API
- [x] Implement image download function with progress events
- [x] Implement SHA256 checksum verification function
- [x] Implement image caching functions
- [x] Add `get_haos_release` command to expose release info to frontend
- [x] Add mock HAOS release data (based on real 16.3 release)
- [x] Wire up download to flash_image command
- [x] Add `HA_INSTALLER_NO_CACHE` env var to skip cache for testing
- [x] Implement xz image extraction with progress events
- [ ] Support resume on failure (nice to have)

### Rust: Image Writing

- [x] Implement raw disk write for macOS (using /dev/rdiskN for raw access)
- [x] Implement raw disk write for Linux (direct write to /dev/sdX)
- [x] Implement raw disk write for Windows (using \\.\PhysicalDriveN)
- [x] Emit progress events during write (every 10MB)
- [x] Handle privilege escalation (error messages for permission denied)
- [x] Implement optional verification after write
- [x] Auto-unmount/clean disk before write
- [x] Auto-eject after completion (macOS)
- [x] Optimize with large write buffers (4MB for SD cards, 64MB for NVMe/SSDs)
- [x] Single authorization for write+verify (no double password prompt)
- [x] Compute SHA256 during write phase for efficient verification
- [x] Flow-aware minimum drive size (2GB for SBC, 16GB for mini PC)

### Progress Events

- [x] Define progress event types
- [x] Emit events from Rust to frontend (mock mode)
- [x] Update UI reactively

### Error Handling

- [x] Handle download failure (UI)
- [x] Handle write failure (UI)
- [x] Handle drive disconnect
- [x] Show error view with retry option

### Verification

- [x] Download progress shows correctly (mock mode)
- [x] Write progress shows correctly (mock mode)
- [x] Errors are handled gracefully (UI)
- [x] Can retry after failure

---

## Phase 9: SBC Flow - Success

**Goal:** User sees success screen with next steps and companion app prompts.

### Success View

- [x] Create success view
- [x] Add happy Casita mascot with blinking animation
- [x] Show "You're all set!" message
- [x] Show next steps list (remove drive, insert, wait, open browser)
- [x] Add homeassistant.local:8123 as clickable link
- [x] Add "Done" button in footer

### Companion App Section

- [x] Add App Store link button for iOS
- [x] Add Google Play link button for Android
- [ ] Show Mac App install button (macOS only)
- [ ] Check if Mac App already installed
- [ ] Open Mac App Store on click

### Platform-Specific

- [ ] Show appropriate apps per platform
- [ ] Handle Windows app link (if applicable)

### Verification

- [x] Success screen displays after flash
- [x] App store links work
- [x] "Done" returns to welcome
- [x] E2E tests pass for success view

---

## Phase 10: Mini PC Flow

**Goal:** User can flash a connected SSD/NVMe for generic x86-64 or ARM64.

### Setup Type Selection

- [x] Create setup type view
- [x] "I can connect the drive" option
- [x] "I need to boot from USB" option (links to docs)

### Architecture Selection

- [x] Create architecture selection view
- [x] Generic x86-64 option
- [x] Generic ARM64 option

### Flow Integration

- [x] Reuse drive selection from SBC flow
- [x] Flow-aware messaging (NVMe/SSD vs SD card terminology)
- [x] Reuse confirmation from SBC flow
- [x] Reuse flash/progress from SBC flow
- [x] Reuse success from SBC flow
- [x] Use correct image for selected architecture

### Verification

- [x] Full mini PC flow works end-to-end
- [x] Correct image is downloaded for architecture

---

## Phase 11: Home Assistant Hardware Flow

**Goal:** User can flash/restore Yellow or Green devices.

### Device Selection

- [ ] Create HA hardware selection view
- [ ] Add Home Assistant Yellow option with image
- [ ] Add Home Assistant Green option with image

### Flow Integration

- [ ] Reuse drive selection
- [ ] Reuse confirmation
- [ ] Reuse flash/progress
- [ ] Reuse success
- [ ] Use correct image for device

### Verification

- [ ] Yellow and Green flows work end-to-end
- [ ] Correct images used

---

## Phase 12: Other Options View

**Goal:** Users with unsupported paths are directed to documentation.

### Other Options View

- [x] Create other options view
- [x] Docker Container section → link to docs
- [x] Synology NAS section → link to docs
- [x] QNAP NAS section → link to docs
- [x] Linux VMs section → link to docs
- [x] Windows VMs section → link to docs
- [x] Official brand icons from simple-icons

### Verification

- [x] All links work and open in browser
- [x] Clear messaging about what this tool does/doesn't do

---

## Phase 13: Proxmox Flow

**Goal:** User can create a Home Assistant VM on their Proxmox server.

### Connect View

- [x] Create Proxmox connect view
- [x] Server URL input
- [x] Username input
- [x] Password input
- [x] Add "Connect" button
- [x] Show connection errors
- [x] Loading indicator while connecting

### Rust: Proxmox API

- [x] Implement Proxmox authentication
- [x] Implement node listing
- [x] Implement storage listing
- [x] Implement VM ID suggestion
- [x] Implement image upload (chunked streaming with progress)
- [x] Implement VM creation
- [x] Implement VM start
- [x] Implement network IP detection
- [x] Implement HA webserver readiness check
- [x] Implement HA update completion check (manifest.json)

### Configure View

- [x] Create Proxmox configure view
- [x] VM name input
- [x] Node selector dropdown
- [x] Storage selector dropdown
- [x] VM ID input (with auto-suggestion)
- [x] CPU cores slider
- [x] Memory slider
- [x] Disk size slider

### Progress View

- [x] Create Proxmox progress component
- [x] Show per-stage progress (0-100% per stage)
- [x] Indeterminate progress for waiting stages
- [x] Stage indicator dots with hover tooltips
- [x] Proxmox-specific stages:
  - [x] Downloading Home Assistant OS
  - [x] Uploading image to Proxmox
  - [x] Creating virtual machine
  - [x] Starting Home Assistant
  - [x] Waiting for network connection
  - [x] Waiting for Home Assistant
  - [x] Updating to the latest version (checks manifest.json)

### Success View

- [x] Create Proxmox success component
- [x] Show VM ID
- [x] Show IP address
- [x] Link to Home Assistant web UI
- [x] Show Proxmox-specific next steps

### Error Handling

- [x] Handle authentication failure
- [ ] Handle 2FA (show message about limitations)
- [x] Handle upload failure
- [x] Handle VM creation failure
- [x] Retry button on error

### Verification

- [x] Full Proxmox flow works end-to-end
- [x] VM is created and starts on real Proxmox server

---

## Phase 14: UTM Flow (macOS)

**Goal:** User can create a Home Assistant VM in UTM on their Mac.

### UTM Check View

- [x] Check if UTM is installed
- [x] If not: show download prompt with link
- [x] If yes: proceed to configure
- [x] Warning card about VM limitations (best for testing)

### Rust: UTM Integration

- [x] Detect UTM installation (check_utm_status)
- [x] Get system info for VM limits (get_system_info)
- [x] Download HAOS qcow2 image (download_utm_image)
- [x] Get Mac architecture (get_mac_architecture)
- [x] Create UTM VM configuration (create_utm_vm)
- [x] Resize VM disk (resize_utm_vm_disk)
- [x] Use utmctl for automation
- [x] Start VM after creation (start_utm_vm)
- [x] List UTM VMs (list_utm_vms)
- [x] Get VM status and IP (get_utm_vm_status)
- [x] Check HA webserver ready (check_ha_ready)
- [x] Check HA update complete (check_ha_updated)

### Configure View

- [x] Create UTM configure view
- [x] Display name input
- [x] CPU cores slider (2 to system max, step 2)
- [x] Memory slider (2GB to system max - 2GB reserve)
- [x] Disk size slider (32GB to 512GB)
- [x] MDI icons for each setting (label, chip, memory, database)
- [x] Dynamic descriptions based on selected values
- [x] Tick marks on sliders

### Progress View

- [x] Create UTM progress component
- [x] Show per-stage progress (0-100% per stage)
- [x] Indeterminate progress for waiting stages
- [x] Stage indicator dots with hover tooltips
- [x] UTM-specific stages:
  - [x] Downloading Home Assistant OS
  - [x] Extracting the image
  - [x] Creating virtual machine
  - [x] Starting Home Assistant
  - [x] Waiting for network connection
  - [x] Waiting for Home Assistant
  - [x] Updating to the latest version (checks manifest.json)

### Success View

- [x] Show VM running status
- [x] Show IP address when available
- [x] Link to Home Assistant web UI
- [x] Show UTM-specific next steps

### Verification

- [x] UTM detection works
- [x] VM is created in UTM
- [x] VM starts successfully
- [x] Can connect to Home Assistant after boot

---

## Phase 15: Toolbox Integration

**Goal:** User can access Open Home Toolbox from within the app.

### Toolbox Button

- [x] Add toolbox button to app (bottom-right)
- [x] Button visible on all screens
- [x] Opens toolbox in external browser (full Web Serial support)

### Verification

- [x] Toolbox opens in system browser
- [x] Button has tooltip on hover

---

## Phase 16: Settings & Updates

**Goal:** User can access settings and see update notifications.

### Settings View

- [ ] Create settings view
- [ ] Add "Receive beta updates" toggle
- [ ] Persist settings locally

### Version Check

- [ ] Implement version check on app launch
- [ ] Fetch latest version from version.home-assistant.io
- [ ] Compare with current version
- [ ] Support beta version channel

### Update Banner

- [ ] Create update banner component
- [ ] Show when update available
- [ ] Different styling for stable vs beta
- [ ] "Download" button opens releases page
- [ ] Dismissable

### Verification

- [ ] Settings persist across app restarts
- [ ] Beta toggle affects version checking
- [ ] Update banner appears when outdated
- [ ] Download link works

---

## Phase 17: Connectivity & Offline Handling

**Goal:** App handles network issues gracefully.

### Connectivity Check

- [ ] Check internet on app launch
- [ ] Attempt manifest fetch as connectivity test

### Offline States

- [ ] No internet + no cache: show error with retry
- [ ] No internet + cache exists: show warning, proceed with cache
- [ ] Show appropriate messaging

### Offline View

- [ ] Create "No Internet" view
- [ ] Explain what's needed
- [ ] Add retry button

### Verification

- [ ] App handles airplane mode gracefully
- [ ] Cached manifest works offline
- [ ] Clear messaging for users

---

## Phase 18: Visual Polish

**Goal:** App looks professional and matches Home Assistant brand.

### Branding

- [ ] Apply HA color palette
- [ ] Apply HA typography
- [ ] Consistent spacing and layout

### Casita Mascot

- [x] Add proper Casita graphics (from HA team)
- [x] Casita on welcome screen
- [x] Casita on progress screens (animated placeholder)
- [x] Casita on success screens
- [x] Casita on error screens

### Device Images

- [x] Add proper product photos for all devices
- [x] Consistent image sizing and treatment

### Icons

- [x] App icon (official HA macOS icon from iOS repository)
- [x] Brand icons for architecture selection (Intel, ARM from Simple Icons)
- [x] Brand icons for Other Options (Docker, Synology, QNAP from Simple Icons)
- [x] MDI icons for drive/USB/info throughout
- [ ] Consistent icon set throughout
- [ ] Action icons (flash, download, etc.)
- [ ] Status icons (success, error, warning)

### Animations

- [x] Consistent animations
- [ ] Smooth view transitions
- [ ] Button hover/press states
- [ ] Progress bar animation
- [ ] (Future) Lottie animations for Casita

### Verification

- [ ] Visual review against HA brand guidelines
- [ ] Consistent look across all screens

---

## Phase 19: Testing

**Goal:** Comprehensive test coverage.

### Rust Unit Tests

- [x] Disk writer device path parsing tests (macOS/Linux/Windows)
- [x] Disk writer safety validation tests (system drive protection)
- [x] Disk writer file copy and verification tests
- [x] Disk writer drive disconnect detection tests
- [x] Download module board filename parsing tests
- [x] Download module SHA256 checksum tests
- [x] Block device enumeration tests
- [x] Mock data tests
- [ ] Manifest parsing tests
- [ ] Proxmox API request formatting tests

### Frontend Component Tests

- [ ] Option card component tests
- [ ] Progress bar component tests
- [ ] Wizard shell tests
- [ ] Drive selector tests

### E2E Tests (Playwright)

- [ ] Welcome screen test
- [ ] Path selection test
- [ ] SBC flow test (mock mode)
- [ ] Mini PC flow test (mock mode)
- [ ] Proxmox flow test (mock mode)
- [ ] UTM flow test (mock mode, macOS only)
- [ ] Settings test
- [ ] Toolbox test

### CI Integration

- [ ] All tests run in CI
- [ ] Coverage reports to Codecov
- [ ] Test failures block merge

### Verification

- [ ] Good coverage on critical paths
- [ ] Tests are reliable (no flaky tests)

---

## Phase 20: Release Infrastructure

**Goal:** Automated builds and signed releases.

### Build Workflow

- [ ] Build for macOS (Intel)
- [ ] Build for macOS (Apple Silicon)
- [ ] Build for Windows
- [ ] Build for Linux (AppImage)
- [ ] Build for Linux (deb)

### Signing

- [ ] macOS code signing
- [ ] macOS notarization
- [ ] Windows signing (if applicable)
- [ ] Cosign signing for all artifacts

### Release Workflow

- [ ] Trigger on version tag
- [ ] Version validation
- [ ] Run all tests before build
- [ ] Generate checksums
- [ ] Create GitHub release (draft)
- [ ] Upload all artifacts

### Release Artifacts

- [ ] .dmg for macOS
- [ ] .msi for Windows
- [ ] .AppImage for Linux
- [ ] .deb for Linux
- [ ] SHA256SUMS.txt
- [ ] Cosign signatures (.sig, .pem)

### Verification

- [ ] Tag push triggers release
- [ ] All platforms build successfully
- [ ] Artifacts are signed and verifiable
- [ ] Draft release created with all files

---

## Phase 21: Distribution

**Goal:** App available through multiple channels.

### Direct Download

- [ ] GitHub Releases page
- [ ] Link from Home Assistant website (coordinate with team)

### macOS

- [ ] Homebrew Cask formula
- [ ] Submit to Homebrew

### Windows

- [ ] Microsoft Store listing (optional)
- [ ] Winget package (optional)

### Linux

- [ ] Flathub submission
- [ ] AppImage hosting

### Verification

- [ ] Install via each distribution method
- [ ] Updates work from each method

---

## Phase 22: Documentation

**Goal:** Users and contributors have clear documentation.

### User Documentation

- [ ] Installation guide for HAI itself
- [ ] Usage guide with screenshots
- [ ] Troubleshooting guide
- [ ] FAQ

### Developer Documentation

- [ ] Architecture overview
- [ ] Development setup guide
- [ ] Testing guide
- [ ] Release process guide

### Integration

- [ ] Coordinate with HA docs team
- [ ] Link from official installation page

### Verification

- [ ] Docs reviewed by non-developers
- [ ] All common issues covered

---

## Post-Launch

### Monitoring

- [ ] Track download counts
- [ ] Monitor GitHub issues
- [ ] Community feedback collection

### Iteration

- [ ] Address user feedback
- [ ] Bug fixes
- [ ] New device support as needed

### Future Considerations

- [ ] Lottie animations for Casita
- [ ] WiFi pre-configuration
- [ ] Backup restoration
- [ ] Additional VM platforms if demanded
- [ ] Tauri auto-updater (if usage patterns warrant)
