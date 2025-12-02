# Home Assistant Installer

## Overview

A cross-platform desktop application that simplifies installing Home Assistant OS across multiple platforms. The app guides users through an intent-based flow, automatically handling image downloads, device preparation, and VM provisioning where supported.

Built with Tauri (Rust backend + web frontend).

---

## Design Philosophy: Visual-First

**The installer should be usable almost without reading.**

This means:

### Every option has a visual identity

- Device selection shows **actual product photos** (Pi 5, Yellow, Green, NUC, etc.)
- Installation paths have **distinctive icons** representing the action
- Progress states use **clear visual indicators**, not just text

### Minimal text, maximum clarity

- Headlines are short (2-4 words)
- Descriptions are optional reinforcement, not required to understand
- Button labels are action verbs with icons

### Visual hierarchy guides the eye

- Large, tappable/clickable targets
- Clear visual distinction between options
- Selected state is immediately obvious
- Progress is shown graphically

### Icons and imagery everywhere

- Each device category has an icon
- Each board/device has a product photo or illustration
- Actions (flash, download, connect) have consistent iconography
- Success/warning/error states use universal visual language (✓, ⚠️, ✕)

---

## Visual Assets Required

### Device Images

High-quality product photos or illustrations for:

| Device | Image needed |
|--------|--------------|
| Raspberry Pi 5 | Product photo |
| Raspberry Pi 4 | Product photo |
| Raspberry Pi 3 | Product photo |
| ODROID-N2+ | Product photo |
| ODROID-M1S | Product photo |
| Home Assistant Yellow | Product photo |
| Home Assistant Green | Product photo |
| Generic mini PC / NUC | Generic illustration |
| Generic x86-64 | Icon/illustration |
| Generic ARM64 | Icon/illustration |

### Category Icons

| Category | Icon concept |
|----------|--------------|
| Single Board Computer | Chip/board icon |
| Mini PC / NUC | Small desktop icon |
| Home Assistant Hardware | HA logo variant |
| Proxmox | Proxmox logo |
| Virtual Machine (UTM) | VM/container icon |
| Other Options | Help/docs icon |

### Action Icons

| Action | Icon |
|--------|------|
| Flash/Write | Lightning bolt or write arrow |
| Download | Download arrow |
| Connect | Link/plug icon |
| Configure | Gear/sliders |
| Success | Checkmark |
| Warning | Triangle alert |
| Error | X or exclamation |
| Refresh | Circular arrows |
| Back | Left arrow |
| Next/Continue | Right arrow |

### Logos

- Home Assistant logo (main branding, welcome screen)
- Open Home Foundation logo (welcome screen, below "Let's go" button)
- Proxmox logo (for Proxmox option)
- UTM logo (for macOS VM option)

### Mascot: Casita

The Home Assistant mascot "Casita" (the friendly house character) should appear throughout the app to add personality and guide users:

| Where | Usage |
|-------|-------|
| Welcome screen | Could appear subtly in background or as part of illustration |
| Progress screens | Animated casita showing activity (downloading, writing, waiting) |
| Success screen | Happy/celebrating casita |
| Error states | Concerned casita with helpful expression |
| Empty states | Casita pointing or guiding |
| Loading states | Casita with activity indicator |

Casita helps make the app feel friendly and approachable, especially for non-technical users. The mascot provides visual continuity with the Home Assistant brand and makes waiting states feel less tedious.

---

## UI Mockups: Visual-First Approach

### Welcome Screen (First Launch)

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                                                                     │
│                                                                     │
│                      [Home Assistant Logo]                          │
│                                                                     │
│                          Installer                                  │
│                                                                     │
│                                                                     │
│            Welcome to your privacy-first, local,                    │
│               home automation journey.                              │
│                                                                     │
│         This tool will help you install Home Assistant              │
│          on your hardware in just a few steps.                      │
│                                                                     │
│                                                                     │
│                                                                     │
│                   ┌─────────────────────┐                           │
│                   │                     │                           │
│                   │      Let's go →     │                           │
│                   │                     │                           │
│                   └─────────────────────┘                           │
│                                                                     │
│                  [Open Home Foundation logo]                        │
│                                                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Installation Path Selection (After "Let's go")

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  [HA logo]  Installer                                               │
│                                                                     │
│                    What are you installing on?                      │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │                 │  │                 │  │                 │      │
│  │   [Pi image]    │  │  [NUC image]    │  │ [Yellow image]  │      │
│  │                 │  │                 │  │                 │      │
│  │ Raspberry Pi    │  │   Mini PC       │  │  Home Assistant │      │
│  │ & other boards  │  │                 │  │    Hardware     │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │                 │  │                 │  │                 │      │
│  │ [Proxmox logo]  │  │  [UTM logo]     │  │    [? icon]     │      │
│  │                 │  │                 │  │                 │      │
│  │    Proxmox      │  │  Virtual        │  │     Other       │      │
│  │    Server       │  │  Machine        │  │    Options      │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
│                                            (macOS only)             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Device Selection (SBC)

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ← Back                     Select your board                       │
│                                                                     │
│  ┌───────────────────────┐  ┌───────────────────────┐               │
│  │                       │  │                       │               │
│  │  ┌─────────────────┐  │  │  ┌─────────────────┐  │               │
│  │  │                 │  │  │  │                 │  │               │
│  │  │ [Pi 5 photo]    │  │  │  │ [Pi 4 photo]    │  │               │
│  │  │                 │  │  │  │                 │  │               │
│  │  └─────────────────┘  │  │  └─────────────────┘  │               │
│  │                       │  │                       │               │
│  │   Raspberry Pi 5      │  │   Raspberry Pi 4      │               │
│  │                       │  │                       │               │
│  └───────────────────────┘  └───────────────────────┘               │
│                                                                     │
│  ┌───────────────────────┐  ┌───────────────────────┐               │
│  │                       │  │                       │               │
│  │  ┌─────────────────┐  │  │  ┌─────────────────┐  │               │
│  │  │                 │  │  │  │                 │  │               │
│  │  │ [Pi 3 photo]    │  │  │  │ [ODROID photo]  │  │               │
│  │  │                 │  │  │  │                 │  │               │
│  │  └─────────────────┘  │  │  └─────────────────┘  │               │
│  │                       │  │                       │               │
│  │   Raspberry Pi 3      │  │   ODROID-N2+          │               │
│  │                       │  │                       │               │
│  └───────────────────────┘  └───────────────────────┘               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Drive Selection

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ← Back                    Select SD card                           │
│                                                                     │
│           [Illustration: SD card going into Pi]                     │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ○   [SD icon]   Samsung SD Card                    32 GB   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ○   [SD icon]   SanDisk Ultra                      64 GB   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ○   [USB icon]  Kingston DataTraveler              16 GB   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
│                                                                     │
│     ⚠️ All data will be erased                                      │
│                                                                     │
│  [🔄 Refresh]                                      [Next →]         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Progress Screen

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                                                                     │
│                      [Animated Casita mascot                        │
│                       with download indicator]                      │
│                                                                     │
│                                                                     │
│         ████████████████████░░░░░░░░░░░░░░░░░░░  45%                │
│                                                                     │
│                      Downloading...                                 │
│                                                                     │
│                   234 MB  /  512 MB                                 │
│                                                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Success Screen

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                    [Happy Casita mascot with                        │
│                     checkmark / celebration]                        │
│                                                                     │
│                         Ready!                                      │
│                                                                     │
│         [Illustration: SD card + Pi + arrow to screen               │
│          showing Home Assistant logo]                               │
│                                                                     │
│             1. Insert SD card into Pi                               │
│             2. Connect power                                        │
│             3. Wait ~20 minutes for first boot                      │
│             4. Visit homeassistant.local:8123                       │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                                                                │ │
│  │   While you wait, get the companion apps:                      │ │
│  │                                                                │ │
│  │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │ │
│  │   │             │    │             │    │             │        │ │
│  │   │  [QR code]  │    │  [QR code]  │    │  [Desktop   │        │ │
│  │   │             │    │             │    │   icon]     │        │ │
│  │   │             │    │             │    │             │        │ │
│  │   └─────────────┘    └─────────────┘    └─────────────┘        │ │
│  │      iOS App            Android           Mac App              │ │
│  │                                          [Install]             │ │
│  │                                                                │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│       [🔄 Flash Another]              [✓ Done]                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Platform-specific behavior:**

| Running on | Show |
|------------|------|
| macOS | iOS QR, Android QR, Mac App install button |
| Windows | iOS QR, Android QR, Windows App download link |
| Linux | iOS QR, Android QR only |

**Mac App installation:**

On macOS, the "Install" button for the Mac App can:
1. Check if already installed
2. If not, open Mac App Store link
3. If installed, show "✓ Installed" instead

**QR Code URLs:**

- iOS: `https://apps.apple.com/app/home-assistant/id1099568401`
- Android: `https://play.google.com/store/apps/details?id=io.homeassistant.companion.android`
- Mac App Store: `https://apps.apple.com/app/home-assistant/id1099568401`

**Implementation:**

```typescript
// Generate QR codes using a library like qrcode
import QRCode from 'qrcode';

const IOS_APP_URL = 'https://apps.apple.com/app/home-assistant/id1099568401';
const ANDROID_APP_URL = 'https://play.google.com/store/apps/details?id=io.homeassistant.companion.android';

// In component
async generateQRCodes() {
  this.iosQR = await QRCode.toDataURL(IOS_APP_URL, { width: 120 });
  this.androidQR = await QRCode.toDataURL(ANDROID_APP_URL, { width: 120 });
}
```

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

## Project Setup

### Prerequisites

- Rust (via rustup)
- Node.js 18+
- Platform-specific dependencies (see Tauri docs)

### Initialize Project

There's a Tauri + Lit starter template available:

```bash
# Clone the Lit + Tauri starter
git clone https://github.com/riipandi/tauri-start-lit ha-installer
cd ha-installer

# Install dependencies
npm install

# Add Web Awesome
npm install @aspect-ui/webawesome

# Start development
npm run tauri dev
```

Or start from scratch:

```bash
# Create Tauri app
npm create tauri-app@latest ha-installer -- --template vanilla-ts

cd ha-installer

# Add Lit and Web Awesome
npm install lit @aspect-ui/webawesome

# Start development
npm run tauri dev
```

### Project Configuration

`tauri.conf.json` key settings:

```json
{
  "productName": "Home Assistant Installer",
  "identifier": "io.home-assistant.installer",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Home Assistant Installer",
        "width": 800,
        "height": 600,
        "resizable": true,
        "minWidth": 600,
        "minHeight": 500
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

---

## Target Platforms

The installer itself runs on:
- macOS (Apple Silicon and Intel)
- Windows
- Linux

---

## Installation Paths

### Fully Automated

| Path | Description | How it works |
|------|-------------|--------------|
| Single Board Computers | Pi 3/4/5, ODROID, etc. | Flash SD card |
| Mini PC / NUC | Generic x86-64 or ARM64 | Flash connected SSD/NVMe via USB adapter |
| Home Assistant Hardware | Yellow, Green | Flash or restore device |
| Proxmox | VM on Proxmox VE | API-driven VM creation |
| macOS VM | UTM on macOS | Automated VM setup |

### Docs Redirect

| Path | Reason |
|------|--------|
| Windows VMs | Hyper-V complexity, not a priority |
| Linux VMs | Users can handle it |
| Containers | Different product (HA Container, not HAOS) |
| Unraid | Limited API, good community docs exist |
| Synology | Poor API, good community docs exist |
| Portainer | Container-based, not HAOS |

---

## User Flow

### Entry Point

On launch, detect the host platform (macOS/Windows/Linux) and present options accordingly.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Home Assistant Installer                                  │
│                                                             │
│   How do you want to run Home Assistant?                    │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  On a Raspberry Pi or similar board                 │   │
│   │  Flash an SD card                                   │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  On a mini PC or NUC                                │   │
│   │  Write directly to an SSD/NVMe                      │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  On Home Assistant hardware (Yellow, Green)         │   │
│   │  Flash or restore your device                       │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  On a Proxmox server                                │   │
│   │  We'll create and configure the VM for you          │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  In a virtual machine on this Mac                   │   │  <- macOS only
│   │  Set up UTM automatically                           │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Other options                                      │   │
│   │  VMs on Windows/Linux, containers, NAS devices      │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Flow: Single Board Computer

### Step 1: Select Device

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Select your board                                         │
│                                                             │
│   ┌─────────────────┐  ┌─────────────────┐                  │
│   │  Raspberry Pi 5 │  │  Raspberry Pi 4 │                  │
│   └─────────────────┘  └─────────────────┘                  │
│   ┌─────────────────┐  ┌─────────────────┐                  │
│   │  Raspberry Pi 3 │  │  ODROID-N2+     │                  │
│   └─────────────────┘  └─────────────────┘                  │
│   ┌─────────────────┐  ┌─────────────────┐                  │
│   │  ODROID-M1S     │  │  Other...       │                  │
│   └─────────────────┘  └─────────────────┘                  │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 2: Select Target Drive

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Select the SD card to flash                               │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  ○  Samsung SD Card - 32 GB (/dev/disk4)            │   │
│   └─────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  ○  SanDisk Ultra - 64 GB (/dev/disk5)              │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ⚠️  All data on the selected drive will be erased         │
│                                                             │
│   [ Refresh ]                          [ Back ] [ Next ]    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 3: Confirm and Flash

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Ready to install                                          │
│                                                             │
│   Device:     Raspberry Pi 5                                │
│   Target:     Samsung SD Card - 32 GB                       │
│   Image:      Home Assistant OS 14.0                        │
│                                                             │
│   ⚠️  This will erase all data on the SD card               │
│                                                             │
│                                      [ Back ] [ Install ]   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 4: Progress

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Installing Home Assistant                                 │
│                                                             │
│   ████████████████████░░░░░░░░░░░░░░░░░░░░  45%             │
│                                                             │
│   Downloading image...                                      │
│                                                             │
│   Downloaded: 234 MB / 512 MB                               │
│   Speed: 12.4 MB/s                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 5: Complete

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ✓  Installation complete                                  │
│                                                             │
│   Your SD card is ready. Next steps:                        │
│                                                             │
│   1. Insert the SD card into your Raspberry Pi 5            │
│   2. Connect ethernet (recommended) or prepare WiFi         │
│   3. Power on the device                                    │
│   4. Wait ~20 minutes for first boot                        │
│   5. Visit http://homeassistant.local:8123                  │
│                                                             │
│                              [ Flash Another ] [ Done ]     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Flow: Mini PC / NUC

### Step 1: Clarify Setup

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   How will you install?                                     │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  I can connect the target drive to this computer    │   │
│   │  I'll plug in the SSD/NVMe via USB adapter          │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  I need to boot from USB to install                 │   │
│   │  The drive is internal and can't be removed         │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

If "boot from USB" is selected:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   USB Boot Installation                                     │
│                                                             │
│   This installer doesn't create bootable USB installers.    │
│                                                             │
│   For mini PCs where you can't remove the drive, follow     │
│   our guide for creating a bootable USB:                    │
│                                                             │
│   [ Open Installation Guide ]                               │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

If "connect drive" is selected, continue to:

### Step 2: Select Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Select your system type                                   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Generic x86-64 (most mini PCs, NUCs, etc.)         │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Generic ARM64 (some SBCs without specific builds)  │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Then continues to drive selection (same as SBC flow).

---

## Flow: Home Assistant Hardware

### Step 1: Select Device

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Select your device                                        │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Home Assistant Yellow                              │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Home Assistant Green                               │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Then continues to drive selection and flashing.

---

## Flow: Proxmox

### Step 1: Connect

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Connect to your Proxmox server                            │
│                                                             │
│   Server URL   [ https://192.168.1.100:8006 ]               │
│                                                             │
│   Username     [ root@pam                   ]               │
│   Password     [ ••••••••••••••             ]               │
│                                                             │
│                                    [ Back ] [ Connect ]     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 2: Configure VM

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Configure the Home Assistant VM                           │
│                                                             │
│   Node          [ pve             ▼ ]                       │
│   Storage       [ local-lvm       ▼ ]                       │
│   VM ID         [ 100               ]  (auto-suggested)     │
│                                                             │
│   Resources                                                 │
│   CPU cores     [ 2                 ]                       │
│   Memory        [ 2048         ] MB                         │
│   Disk size     [ 32           ] GB                         │
│                                                             │
│   ☑ Start VM after creation                                 │
│                                                             │
│                                 [ Back ] [ Create VM ]      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 3: Progress

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Creating Home Assistant VM                                │
│                                                             │
│   ✓  Connected to Proxmox                                   │
│   ✓  Downloaded HAOS image                                  │
│   ⟳  Uploading image to storage...                          │
│   ○  Creating VM                                            │
│   ○  Starting VM                                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 4: Complete

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ✓  Home Assistant VM created                              │
│                                                             │
│   VM ID: 100                                                │
│   Status: Running                                           │
│                                                             │
│   Next steps:                                               │
│                                                             │
│   1. Wait ~20 minutes for first boot                        │
│   2. Visit http://homeassistant.local:8123                  │
│      (or check the VM console for the IP address)           │
│                                                             │
│                              [ Create Another ] [ Done ]    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Flow: macOS VM (UTM)

### Step 1: Check UTM

If UTM is not installed:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   UTM Required                                              │
│                                                             │
│   To run Home Assistant in a virtual machine on your Mac,   │
│   you'll need UTM installed.                                │
│                                                             │
│   UTM is a free, open-source virtualizer for macOS.         │
│                                                             │
│   [ Download UTM ]                    [ I've installed it ] │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

If UTM is installed:

### Step 2: Configure VM

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Configure the Home Assistant VM                           │
│                                                             │
│   CPU cores     [ 2                 ]                       │
│   Memory        [ 2048         ] MB                         │
│   Disk size     [ 32           ] GB                         │
│                                                             │
│   ☑ Start VM after creation                                 │
│                                                             │
│                                 [ Back ] [ Create VM ]      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Step 3: Progress and Complete

Similar to Proxmox flow.

---

## Flow: Other Options

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Other Installation Methods                                │
│                                                             │
│   This installer supports direct flashing and automated     │
│   VM setup for Proxmox and UTM. For other platforms,        │
│   we have detailed guides:                                  │
│                                                             │
│   Virtual Machines                                          │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Windows (Hyper-V, VirtualBox)        [ View Guide ]│   │
│   └─────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Linux (KVM, VirtualBox)              [ View Guide ]│   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   NAS Devices                                               │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Unraid                               [ View Guide ]│   │
│   └─────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Synology                             [ View Guide ]│   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   Containers                                                │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Docker / Portainer                   [ View Guide ]│   │
│   │  Note: This runs HA Container, not full HAOS        │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                             [ Back ]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Technical Architecture

### Stack

- **Framework**: Tauri 2.x
- **Backend**: Rust
- **Frontend**: Lit (Web Components) + TypeScript
- **UI Components**: Web Awesome (the library Home Assistant uses, successor to Shoelace)
- **Build**: Vite

This stack matches what Home Assistant uses for their frontend, ensuring visual and technical consistency.

### Rust Backend Modules

```
src-tauri/
├── main.rs
├── lib.rs
├── commands/
│   ├── mod.rs
│   ├── devices.rs      # List block devices
│   ├── flash.rs        # Image writing
│   ├── download.rs     # Image downloads with progress
│   ├── proxmox.rs      # Proxmox API integration
│   └── utm.rs          # UTM automation (macOS)
├── platforms/
│   ├── mod.rs
│   ├── macos.rs        # macOS-specific disk handling
│   ├── windows.rs      # Windows-specific disk handling
│   └── linux.rs        # Linux-specific disk handling
└── models/
    ├── mod.rs
    ├── device.rs       # Device/board definitions
    └── image.rs        # Image metadata
```

### Key Tauri Commands

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
```

### Frontend Structure (Lit + Web Awesome)

```
src/
├── index.html
├── main.ts                     # App entry point
├── styles/
│   ├── theme.css               # HA brand colors, tokens
│   └── app.css
├── state/
│   ├── wizard-state.ts         # Current step, selections
│   └── progress-state.ts       # Download/flash progress
├── api/
│   ├── tauri-commands.ts       # Tauri invoke wrappers
│   ├── devices.ts
│   ├── flash.ts
│   ├── proxmox.ts
│   └── utm.ts
├── components/
│   ├── ha-installer-app.ts     # Root app component
│   ├── wizard-shell.ts         # Wizard container/navigation
│   ├── step-indicator.ts       # Progress breadcrumb
│   ├── device-selector.ts      # Device/board picker
│   ├── drive-selector.ts       # Target drive picker
│   ├── progress-bar.ts         # Download/write progress
│   └── option-card.ts          # Selectable option card
├── views/
│   ├── home-view.ts            # Main selection screen
│   ├── sbc/
│   │   ├── select-device.ts
│   │   ├── select-drive.ts
│   │   ├── confirm.ts
│   │   └── progress.ts
│   ├── minipc/
│   │   ├── setup-type.ts
│   │   ├── select-arch.ts
│   │   └── ...
│   ├── proxmox/
│   │   ├── connect.ts
│   │   ├── configure.ts
│   │   └── progress.ts
│   ├── utm/
│   │   ├── check-utm.ts
│   │   ├── configure.ts
│   │   └── progress.ts
│   └── other/
│       └── other-options.ts
└── utils/
    └── format.ts               # Bytes formatting, etc.
```

### Example Lit Component

```typescript
import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import '@aspect-ui/webawesome/components/wa-button/wa-button.js';
import '@aspect-ui/webawesome/components/wa-card/wa-card.js';

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

### Web Awesome Integration

Web Awesome (formerly Shoelace) provides the component library. Key components we'll use:

- `<wa-button>` - Actions and navigation
- `<wa-card>` - Option cards, info panels
- `<wa-input>` - Text inputs (Proxmox URL, credentials)
- `<wa-select>` - Dropdowns (node selection, storage)
- `<wa-progress-bar>` - Download/flash progress
- `<wa-alert>` - Warnings, errors, success messages
- `<wa-spinner>` - Loading states
- `<wa-icon>` - Icons throughout
- `<wa-dialog>` - Confirmation dialogs

Install via npm:
```bash
npm install @aspect-ui/webawesome
```

Or use the CDN for prototyping:
```html
<link rel="stylesheet" href="https://cdn.webawesome.com/dist/themes/default.css" />
<script type="module" src="https://cdn.webawesome.com/dist/webawesome.js"></script>
```

---

## Device and Image Data

Devices and images are fetched from `version.home-assistant.io` and cached locally.

### Manifest Structure

The app fetches and parses the manifest from Home Assistant's version API. Example expected structure:

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
// Pseudocode for manifest fetching
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

---

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

---

## Security Considerations

- Never store credentials (Proxmox creds are session-only, in memory)
- Verify image checksums before flashing
- Warn clearly before destructive operations
- Request minimal necessary privileges

---

## Future Considerations (Post-V1)

- Network configuration (WiFi credentials pre-provisioned)
- Backup restoration (flash a backup directly)
- Additional VM platforms if there's demand
- Auto-update mechanism for the installer itself
- Translations / i18n

---

## Testing Strategy

### Overview

Testing is split into four layers: Rust unit tests, frontend component tests, Tauri integration tests, and Playwright E2E tests. A mock mode enables testing flows without real hardware.

### 1. Rust Unit Tests

Test core backend logic in isolation.

```rust
// src-tauri/src/manifest.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{"devices": [{"id": "rpi5", "name": "Raspberry Pi 5"}]}"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.devices[0].id, "rpi5");
    }

    #[test]
    fn test_checksum_validation_success() {
        let data = b"test data";
        let hash = "sha256:...";
        assert!(validate_checksum(data, hash).is_ok());
    }

    #[test]
    fn test_checksum_validation_failure() {
        let data = b"test data";
        let wrong_hash = "sha256:wrong";
        assert!(validate_checksum(data, wrong_hash).is_err());
    }

    #[tokio::test]
    async fn test_manifest_cache_fallback() {
        // Test that cached manifest is used when fetch fails
    }
}
```

**Run:** `cargo test` in `src-tauri/`

### 2. Frontend Component Tests (Web Test Runner)

Test Lit components in isolation.

```typescript
// test/unit/option-card.test.ts
import { fixture, html, expect } from '@open-wc/testing';
import '../../src/components/option-card.js';

describe('OptionCard', () => {
  it('renders title and description', async () => {
    const el = await fixture(html`
      <option-card 
        title="Raspberry Pi" 
        description="Flash an SD card">
      </option-card>
    `);
    
    expect(el.shadowRoot?.textContent).to.include('Raspberry Pi');
    expect(el.shadowRoot?.textContent).to.include('Flash an SD card');
  });

  it('dispatches select event on click', async () => {
    const el = await fixture(html`<option-card></option-card>`);
    
    let fired = false;
    el.addEventListener('select', () => fired = true);
    el.click();
    
    expect(fired).to.be.true;
  });

  it('applies selected state visually', async () => {
    const el = await fixture(html`<option-card selected></option-card>`);
    const card = el.shadowRoot?.querySelector('wa-card');
    
    expect(card?.hasAttribute('data-selected')).to.be.true;
  });
});
```

**Config:** `web-test-runner.config.js`
```javascript
export default {
  files: 'test/unit/**/*.test.ts',
  nodeResolve: true,
  plugins: [esbuildPlugin({ ts: true })],
};
```

**Run:** `npm run test:unit`

### 3. Tauri Integration Tests

Test frontend-backend communication with mocked IPC.

```typescript
// test/integration/commands.test.ts
import { invoke } from '@tauri-apps/api/core';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { expect } from '@open-wc/testing';

describe('Tauri Commands', () => {
  afterEach(() => clearMocks());

  it('list_block_devices returns device list', async () => {
    mockIPC((cmd) => {
      if (cmd === 'list_block_devices') {
        return [
          { id: 'disk1', name: 'SD Card', size: 32000000000 }
        ];
      }
    });

    const devices = await invoke('list_block_devices');
    expect(devices).to.have.length(1);
    expect(devices[0].name).to.equal('SD Card');
  });

  it('proxmox_connect validates credentials', async () => {
    mockIPC((cmd, args) => {
      if (cmd === 'proxmox_connect') {
        if (args.password === 'correct') {
          return { success: true, nodes: ['pve'] };
        }
        throw new Error('Authentication failed');
      }
    });

    const result = await invoke('proxmox_connect', {
      url: 'https://proxmox.local:8006',
      username: 'root@pam',
      password: 'correct'
    });
    
    expect(result.success).to.be.true;
  });
});
```

**Run:** `npm run test:integration`

### 4. E2E Tests (Playwright)

Test complete user flows through the app.

```typescript
// test/e2e/welcome.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Welcome Screen', () => {
  test('displays welcome message and proceeds on button click', async ({ page }) => {
    await page.goto('/');
    
    // Verify welcome screen content
    await expect(page.getByText('Welcome to your privacy-first')).toBeVisible();
    await expect(page.getByRole('button', { name: "Let's go" })).toBeVisible();
    await expect(page.locator('[alt="Open Home Foundation"]')).toBeVisible();
    
    // Click through
    await page.getByRole('button', { name: "Let's go" }).click();
    
    // Verify navigation to path selection
    await expect(page.getByText('What are you installing on?')).toBeVisible();
  });
});
```

```typescript
// test/e2e/sbc-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Single Board Computer Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Enable mock mode
    await page.goto('/?mock=true');
    await page.getByRole('button', { name: "Let's go" }).click();
  });

  test('complete flow from path selection to confirmation', async ({ page }) => {
    // Select SBC path
    await page.getByRole('button', { name: /Raspberry Pi/i }).click();
    
    // Select specific device
    await page.getByRole('button', { name: 'Raspberry Pi 5' }).click();
    
    // Should show drive selection
    await expect(page.getByText('Select SD card')).toBeVisible();
    
    // Mock device should be available
    await page.getByText('Mock SD Card').click();
    await page.getByRole('button', { name: 'Next' }).click();
    
    // Should show confirmation
    await expect(page.getByText('Ready to install')).toBeVisible();
    await expect(page.getByText('Raspberry Pi 5')).toBeVisible();
    await expect(page.getByText('Mock SD Card')).toBeVisible();
  });

  test('back navigation works correctly', async ({ page }) => {
    await page.getByRole('button', { name: /Raspberry Pi/i }).click();
    await page.getByRole('button', { name: 'Raspberry Pi 5' }).click();
    
    // Go back
    await page.getByRole('button', { name: /back/i }).click();
    
    // Should be on device selection
    await expect(page.getByRole('button', { name: 'Raspberry Pi 5' })).toBeVisible();
  });
});
```

```typescript
// test/e2e/proxmox-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Proxmox Flow', () => {
  test('shows error on invalid credentials', async ({ page }) => {
    await page.goto('/?mock=true');
    await page.getByRole('button', { name: "Let's go" }).click();
    await page.getByRole('button', { name: /Proxmox/i }).click();
    
    // Fill invalid credentials
    await page.getByLabel('Server URL').fill('https://proxmox.local:8006');
    await page.getByLabel('Username').fill('root@pam');
    await page.getByLabel('Password').fill('wrong');
    await page.getByRole('button', { name: 'Connect' }).click();
    
    // Should show error
    await expect(page.getByText(/authentication failed/i)).toBeVisible();
  });
});
```

```typescript
// test/e2e/toolbox.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Toolbox Integration', () => {
  test('opens toolbox iframe and returns', async ({ page }) => {
    await page.goto('/');
    
    // Click toolbox button
    await page.getByRole('button', { name: /toolbox/i }).click();
    
    // Should show iframe with toolbox
    const iframe = page.frameLocator('iframe');
    await expect(iframe.locator('body')).toBeVisible();
    
    // Return to installer
    await page.getByRole('button', { name: /back to installer/i }).click();
    
    // Should be back on welcome
    await expect(page.getByText('Welcome to your privacy-first')).toBeVisible();
  });
});
```

**Config:** `playwright.config.ts`
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './test/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
  },
});
```

**Run:** `npm run test:e2e`

### 5. Mock Mode

Enable testing without real hardware.

```rust
// src-tauri/src/commands/devices.rs

#[tauri::command]
pub async fn list_block_devices() -> Result<Vec<BlockDevice>, String> {
    if std::env::var("HA_INSTALLER_MOCK").is_ok() {
        return Ok(mock_devices());
    }
    real_list_block_devices().await
}

fn mock_devices() -> Vec<BlockDevice> {
    vec![
        BlockDevice {
            id: "mock-sd-32".into(),
            name: "Mock SD Card".into(),
            size: 32_000_000_000,
            device_type: DeviceType::SdCard,
        },
        BlockDevice {
            id: "mock-usb-64".into(),
            name: "Mock USB Drive".into(),
            size: 64_000_000_000,
            device_type: DeviceType::Usb,
        },
    ]
}

#[tauri::command]
pub async fn flash_image(
    image_path: String,
    target_device: String,
    window: tauri::Window,
) -> Result<(), String> {
    if std::env::var("HA_INSTALLER_MOCK").is_ok() {
        return mock_flash(window).await;
    }
    real_flash_image(image_path, target_device, window).await
}

async fn mock_flash(window: tauri::Window) -> Result<(), String> {
    // Simulate download progress
    for i in 0..=100 {
        window.emit("flash-progress", FlashProgress {
            stage: if i < 50 { "downloading" } else { "writing" },
            percent: i,
        }).ok();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}
```

### Test Directory Structure

```
ha-installer/
├── src-tauri/
│   ├── src/
│   └── tests/
│       ├── manifest_test.rs
│       ├── checksum_test.rs
│       ├── proxmox_api_test.rs
│       └── utm_test.rs
├── src/
├── test/
│   ├── unit/
│   │   ├── option-card.test.ts
│   │   ├── wizard-shell.test.ts
│   │   ├── drive-selector.test.ts
│   │   └── progress-bar.test.ts
│   ├── integration/
│   │   ├── commands.test.ts
│   │   └── state.test.ts
│   └── e2e/
│       ├── welcome.spec.ts
│       ├── sbc-flow.spec.ts
│       ├── minipc-flow.spec.ts
│       ├── proxmox-flow.spec.ts
│       ├── utm-flow.spec.ts
│       └── toolbox.spec.ts
├── playwright.config.ts
├── web-test-runner.config.js
└── package.json
```

### npm Scripts

```json
{
  "scripts": {
    "test": "npm run test:unit && npm run test:integration && npm run test:e2e",
    "test:unit": "web-test-runner",
    "test:integration": "web-test-runner --config wtr.integration.config.js",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:headed": "playwright test --headed"
  }
}
```

---

## CI/CD (GitHub Actions)

### Principles

- **Immutable releases**: Once a version is released, it cannot be changed
- **Signed artifacts**: All release binaries are signed
- **Reproducible builds**: Same commit = same output
- **Multi-platform**: Build and test on macOS, Windows, Linux

### Workflow: Test (on every push/PR)

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test-rust:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      
      - name: Run tests with coverage
        run: cargo llvm-cov --lcov --output-path lcov.info
        working-directory: src-tauri
      
      - name: Upload Rust coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: src-tauri/lcov.info
          flags: rust
          token: ${{ secrets.CODECOV_TOKEN }}

  test-frontend:
    name: Frontend Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run unit tests with coverage
        run: npm run test:unit -- --coverage
      
      - name: Run integration tests with coverage
        run: npm run test:integration -- --coverage
      
      - name: Upload frontend coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage/lcov.info
          flags: frontend
          token: ${{ secrets.CODECOV_TOKEN }}

  test-e2e:
    name: E2E Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium
      
      - name: Install Tauri dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run E2E tests
        run: npm run test:e2e
        env:
          HA_INSTALLER_MOCK: "true"
      
      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 7

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Lint frontend
        run: npm run lint
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      
      - name: Rust format check
        run: cargo fmt --check
        working-directory: src-tauri
      
      - name: Rust clippy
        run: cargo clippy -- -D warnings
        working-directory: src-tauri
```

### Codecov Configuration

```yaml
# codecov.yml
coverage:
  precision: 2
  round: down
  status:
    project:
      default:
        target: auto
        threshold: 2%
    patch:
      default:
        target: 80%

flags:
  rust:
    paths:
      - src-tauri/src/
    carryforward: true
  frontend:
    paths:
      - src/
    carryforward: true

comment:
  layout: "reach,diff,flags,files"
  behavior: default
  require_changes: true
```

### Workflow: Release (on version tag)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  # Validate tag matches version in Cargo.toml
  validate-version:
    name: Validate Version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Check version consistency
        run: |
          TAG_VERSION="${GITHUB_REF#refs/tags/v}"
          CARGO_VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
          PKG_VERSION=$(node -p "require('./package.json').version")
          
          if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) doesn't match Cargo.toml ($CARGO_VERSION)"
            exit 1
          fi
          
          if [ "$TAG_VERSION" != "$PKG_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) doesn't match package.json ($PKG_VERSION)"
            exit 1
          fi
          
          echo "Version $TAG_VERSION validated"

  # Run all tests before release
  test:
    name: Test
    needs: validate-version
    uses: ./.github/workflows/test.yml

  # Build for each platform
  build:
    name: Build (${{ matrix.os }})
    needs: test
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: ha-installer_linux-x64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: ha-installer_macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: ha-installer_macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: ha-installer_windows-x64

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install dependencies (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
      
      - name: Install frontend dependencies
        run: npm ci
      
      - name: Build frontend
        run: npm run build
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          # macOS signing
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v__VERSION__
          releaseName: 'Home Assistant Installer v__VERSION__'
          releaseBody: 'See the assets below to download and install.'
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }}
      
      - name: Generate checksums
        shell: bash
        run: |
          cd src-tauri/target/${{ matrix.target }}/release/bundle
          find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) -exec sha256sum {} \; > checksums.txt
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.dmg
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.app.tar.gz
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.msi
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.exe
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.deb
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.AppImage
            src-tauri/target/${{ matrix.target }}/release/bundle/**/checksums.txt
          retention-days: 5

  # Create GitHub release with all artifacts
  publish:
    name: Publish Release
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
      id-token: write  # Required for cosign keyless signing
    steps:
      - uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
      
      - name: Install cosign
        uses: sigstore/cosign-installer@v3
      
      - name: Generate release checksums
        run: |
          cd artifacts
          find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) -exec sha256sum {} \; | sort > SHA256SUMS.txt
      
      - name: Sign artifacts with cosign
        run: |
          cd artifacts
          # Sign each artifact with keyless signing (uses GitHub OIDC)
          for file in $(find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \)); do
            cosign sign-blob --yes --output-signature "${file}.sig" --output-certificate "${file}.pem" "${file}"
          done
          # Also sign the checksums file
          cosign sign-blob --yes --output-signature SHA256SUMS.txt.sig --output-certificate SHA256SUMS.txt.pem SHA256SUMS.txt
      
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          draft: true
          generate_release_notes: true
          files: |
            artifacts/**/*.dmg
            artifacts/**/*.app.tar.gz
            artifacts/**/*.msi
            artifacts/**/*.exe
            artifacts/**/*.deb
            artifacts/**/*.AppImage
            artifacts/**/*.sig
            artifacts/**/*.pem
            artifacts/SHA256SUMS.txt
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

### Cosign Verification

Users can verify the authenticity of downloads using cosign:

```bash
# Install cosign
# macOS: brew install cosign
# Linux: See https://docs.sigstore.dev/cosign/installation/

# Verify a downloaded artifact
cosign verify-blob \
  --signature ha-installer_macos-arm64.dmg.sig \
  --certificate ha-installer_macos-arm64.dmg.pem \
  --certificate-identity-regexp "https://github.com/home-assistant/ha-installer/" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  ha-installer_macos-arm64.dmg

# Verify the checksums file
cosign verify-blob \
  --signature SHA256SUMS.txt.sig \
  --certificate SHA256SUMS.txt.pem \
  --certificate-identity-regexp "https://github.com/home-assistant/ha-installer/" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  SHA256SUMS.txt
```

Cosign keyless signing uses GitHub's OIDC identity, so signatures are tied to:
- The specific GitHub repository
- The GitHub Actions workflow that created them
- The exact commit and workflow run

No private keys to manage or rotate.
```

### Immutable Release Protections

1. **Tag protection rules** (GitHub Settings > Branches > Tag protection):
   - Pattern: `v*`
   - Prevents deletion or force-push of version tags

2. **Branch protection on main**:
   - Require PR reviews
   - Require status checks (all tests must pass)
   - No force pushes

3. **Release process**:
   - Releases are created as drafts
   - Manual review before publishing
   - Once published, artifacts cannot be modified

4. **Checksums**:
   - SHA256 checksums for all artifacts
   - Published with each release
   - Users can verify download integrity

5. **Cosign signatures**:
   - All artifacts signed using Sigstore cosign
   - Keyless signing via GitHub OIDC (no secrets to manage)
   - Signatures prove artifacts were built by the official CI pipeline
   - Publicly verifiable via Sigstore transparency log

### Required Secrets

| Secret | Description |
|--------|-------------|
| `CODECOV_TOKEN` | Codecov upload token (from codecov.io) |
| `TAURI_SIGNING_PRIVATE_KEY` | Key for signing Tauri updates |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for signing key |
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | e.g., "Developer ID Application: Open Home Foundation" |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

### Release Process

1. Update version in `package.json` and `src-tauri/Cargo.toml`
2. Create PR with version bump
3. Merge to main after review
4. Create and push tag: `git tag v1.0.0 && git push origin v1.0.0`
5. CI builds and creates draft release
6. Review draft release and artifacts
7. Publish release (makes it immutable)

---

## Contributor Tooling & Automation

### AI Agent Instructions

```markdown
# .github/copilot-instructions.md

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
```

```markdown
# .github/claude-instructions.md

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
```

### GitHub Issue Forms

```yaml
# .github/ISSUE_TEMPLATE/bug_report.yml
name: Bug Report
description: Report a problem with the installer
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug!
        Please fill out the information below to help us investigate.

  - type: dropdown
    id: platform
    attributes:
      label: Operating System
      description: What OS are you running the installer on?
      options:
        - macOS (Apple Silicon)
        - macOS (Intel)
        - Windows 11
        - Windows 10
        - Linux (Ubuntu/Debian)
        - Linux (Fedora/RHEL)
        - Linux (Other)
    validations:
      required: true

  - type: dropdown
    id: installation-path
    attributes:
      label: Installation Path
      description: What were you trying to install?
      options:
        - Raspberry Pi / SBC
        - Mini PC / NUC
        - Home Assistant Yellow
        - Home Assistant Green
        - Proxmox VM
        - UTM VM (macOS)
        - Other
    validations:
      required: true

  - type: input
    id: version
    attributes:
      label: Installer Version
      description: Which version of the installer? (shown in app or filename)
      placeholder: "v1.0.0"
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: What happened?
      description: Describe the bug clearly
      placeholder: "When I clicked 'Flash', the app showed an error..."
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: What did you expect?
      description: What should have happened instead?
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this?
      placeholder: |
        1. Open the installer
        2. Select Raspberry Pi 5
        3. Insert SD card
        4. Click Flash
        5. See error
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Logs or Screenshots
      description: Paste any error messages or attach screenshots
    validations:
      required: false

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      options:
        - label: I have searched existing issues to avoid duplicates
          required: true
        - label: I am running the latest version of the installer
          required: false
```

```yaml
# .github/ISSUE_TEMPLATE/feature_request.yml
name: Feature Request
description: Suggest a new feature or improvement
labels: ["enhancement", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for your suggestion! Please describe your idea below.

  - type: textarea
    id: problem
    attributes:
      label: Problem or Use Case
      description: What problem does this solve? What are you trying to do?
      placeholder: "I want to install Home Assistant on my XYZ device, but..."
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: How would you like this to work?
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Any other approaches you've thought about?
    validations:
      required: false

  - type: dropdown
    id: area
    attributes:
      label: Area
      description: What part of the installer does this relate to?
      options:
        - New device support
        - New installation method
        - User interface
        - Performance
        - Documentation
        - Other
    validations:
      required: true

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      options:
        - label: I have searched existing issues to avoid duplicates
          required: true
```

```yaml
# .github/ISSUE_TEMPLATE/device_request.yml
name: Device Support Request
description: Request support for a new device or board
labels: ["device-support", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Want to see a new device supported? Let us know!

  - type: input
    id: device-name
    attributes:
      label: Device Name
      description: Full name of the device
      placeholder: "ODROID-M2"
    validations:
      required: true

  - type: input
    id: manufacturer
    attributes:
      label: Manufacturer
      placeholder: "Hardkernel"
    validations:
      required: true

  - type: dropdown
    id: device-type
    attributes:
      label: Device Type
      options:
        - Single Board Computer (SBC)
        - Mini PC / NUC
        - Official Home Assistant Hardware
        - Other
    validations:
      required: true

  - type: input
    id: haos-support
    attributes:
      label: Home Assistant OS Support
      description: Is there already a HAOS image for this device?
      placeholder: "Yes - https://github.com/home-assistant/operating-system/releases"
    validations:
      required: true

  - type: textarea
    id: details
    attributes:
      label: Additional Details
      description: Any other relevant information (links, specs, etc.)
    validations:
      required: false
```

```yaml
# .github/ISSUE_TEMPLATE/config.yml
blank_issues_enabled: false
contact_links:
  - name: Home Assistant Community
    url: https://community.home-assistant.io/
    about: Ask questions and get help from the community
  - name: Home Assistant Discord
    url: https://discord.gg/home-assistant
    about: Chat with the community in real-time
  - name: Documentation
    url: https://www.home-assistant.io/installation/
    about: Official installation documentation
```

### Pull Request Template

```markdown
# .github/PULL_REQUEST_TEMPLATE.md

## Description

<!-- Describe your changes -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] CI/CD changes

## Related Issues

<!-- Link any related issues: Fixes #123, Relates to #456 -->

## Testing

<!-- How did you test this? -->

- [ ] Unit tests added/updated
- [ ] E2E tests added/updated
- [ ] Manually tested on macOS
- [ ] Manually tested on Windows
- [ ] Manually tested on Linux

## Screenshots

<!-- If UI changes, add before/after screenshots -->

## Checklist

- [ ] My code follows the project's code style
- [ ] I have run `cargo fmt` and `npm run lint`
- [ ] I have added tests for my changes
- [ ] All new and existing tests pass
- [ ] I have updated documentation if needed
- [ ] My commits follow conventional commit format
```

### Copilot Code Review

```yaml
# .github/copilot-review.yml
# Configuration for GitHub Copilot code review

reviews:
  # Auto-review settings
  auto_review:
    enabled: true
    
  # What to focus on
  focus_areas:
    - security
    - error_handling
    - performance
    - accessibility
    
  # Language-specific guidance
  languages:
    rust:
      - Check for proper error handling with Result types
      - Ensure no unwrap() in production code paths
      - Verify async code properly handles cancellation
      - Check for potential panics
    typescript:
      - Ensure proper typing (no `any`)
      - Check for accessibility in components
      - Verify event cleanup in Lit lifecycle
      - Check for memory leaks in subscriptions

  # Custom review prompts
  prompts:
    - "Does this change handle offline/network error scenarios?"
    - "Are there any potential security issues with user input?"
    - "Is the UI accessible (keyboard navigation, screen readers)?"
    - "Does this follow the visual-first design principle?"
```

### Renovate Configuration

```json5
// renovate.json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": [
    "config:recommended",
    ":semanticCommits",
    ":preserveSemverRanges",
    "group:allNonMajor"
  ],
  
  "labels": ["dependencies"],
  
  "schedule": ["before 6am on monday"],
  
  "timezone": "UTC",
  
  "prHourlyLimit": 2,
  "prConcurrentLimit": 5,
  
  "packageRules": [
    {
      "description": "Group Tauri packages",
      "matchPackagePatterns": ["^@tauri-apps/", "^tauri"],
      "groupName": "tauri",
      "groupSlug": "tauri"
    },
    {
      "description": "Group Lit packages",
      "matchPackagePatterns": ["^lit", "^@lit/"],
      "groupName": "lit",
      "groupSlug": "lit"
    },
    {
      "description": "Group Web Awesome packages",
      "matchPackagePatterns": ["webawesome", "shoelace"],
      "groupName": "web-awesome",
      "groupSlug": "web-awesome"
    },
    {
      "description": "Group Playwright packages",
      "matchPackagePatterns": ["^@playwright/", "^playwright"],
      "groupName": "playwright",
      "groupSlug": "playwright"
    },
    {
      "description": "Group testing packages",
      "matchPackagePatterns": ["^@web/test-runner", "^@open-wc/testing", "^sinon"],
      "groupName": "testing",
      "groupSlug": "testing"
    },
    {
      "description": "Group Rust crates",
      "matchManagers": ["cargo"],
      "groupName": "rust-dependencies",
      "groupSlug": "rust-deps"
    },
    {
      "description": "Group GitHub Actions",
      "matchManagers": ["github-actions"],
      "groupName": "github-actions",
      "groupSlug": "gha"
    },
    {
      "description": "Auto-merge patch updates",
      "matchUpdateTypes": ["patch"],
      "automerge": true,
      "automergeType": "pr",
      "platformAutomerge": true
    },
    {
      "description": "Auto-merge minor dev dependencies",
      "matchDepTypes": ["devDependencies"],
      "matchUpdateTypes": ["minor"],
      "automerge": true
    }
  ],
  
  "vulnerabilityAlerts": {
    "enabled": true,
    "labels": ["security"]
  },
  
  "rust": {
    "enabled": true
  }
}
```

### CODEOWNERS

```
# .github/CODEOWNERS

# Default owners for everything
* @home-assistant/installer-team

# Rust backend
/src-tauri/ @home-assistant/installer-backend

# Frontend components
/src/ @home-assistant/installer-frontend

# CI/CD and infrastructure
/.github/ @home-assistant/installer-infra
/renovate.json @home-assistant/installer-infra

# Documentation
/docs/ @home-assistant/docs-team
*.md @home-assistant/docs-team
```

### Contributing Guide

```markdown
# CONTRIBUTING.md

# Contributing to Home Assistant Installer

Thank you for your interest in contributing! This document provides guidelines
and instructions for contributing.

## Getting Started

### Prerequisites

- Rust (via rustup)
- Node.js 20+
- Platform-specific Tauri dependencies

### Setup

```bash
# Clone the repository
git clone https://github.com/home-assistant/ha-installer.git
cd ha-installer

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Running Tests

```bash
# All tests
npm test

# Unit tests only
npm run test:unit

# E2E tests (with mock mode)
HA_INSTALLER_MOCK=true npm run test:e2e

# Rust tests
cd src-tauri && cargo test
```

## Development Guidelines

### Code Style

- **Rust**: Follow `rustfmt` defaults, run `cargo fmt` before committing
- **TypeScript**: Follow ESLint config, run `npm run lint` before committing
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(proxmox): add node selection dropdown`
- `fix(flash): handle USB disconnect during write`
- `docs: update installation instructions`

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all tests pass
5. Submit a pull request

### Design Principles

When contributing UI changes, remember:

1. **Visual-first**: The UI should be understandable without reading text
2. **Use icons and images**: Every option should have a visual identity
3. **Follow HA branding**: Use Home Assistant colors and style
4. **Include Casita**: Use the mascot for personality in appropriate places

## Getting Help

- [GitHub Discussions](https://github.com/home-assistant/ha-installer/discussions)
- [Home Assistant Discord](https://discord.gg/home-assistant)
- [Community Forum](https://community.home-assistant.io/)

## License

By contributing, you agree that your contributions will be licensed under the
Apache 2.0 License.
```

### Repository Structure Summary

```
ha-installer/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.yml
│   │   ├── feature_request.yml
│   │   ├── device_request.yml
│   │   └── config.yml
│   ├── workflows/
│   │   ├── test.yml
│   │   └── release.yml
│   ├── CODEOWNERS
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── copilot-instructions.md
│   ├── copilot-review.yml
│   └── claude-instructions.md
├── src-tauri/
│   └── ...
├── src/
│   └── ...
├── test/
│   └── ...
├── renovate.json
├── CONTRIBUTING.md
├── README.md
└── LICENSE
```

---

### Manifest / Image Metadata

- **Source**: Fetched from `version.home-assistant.io`
- **Caching**: Downloaded manifest is cached locally
- **Offline fallback**: If fetch fails and a cached version exists, use the cached version
- **Internet required**: App requires internet connection; check connectivity on launch and show appropriate message if offline and no cache exists

### Network Connectivity Check

On launch:
1. Attempt to fetch manifest from `version.home-assistant.io`
2. If successful: use fresh manifest, update cache
3. If failed + cache exists: show warning, proceed with cached manifest
4. If failed + no cache: show error, explain internet is required, offer retry

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   ⚠️  No Internet Connection                                │
│                                                             │
│   Home Assistant Installer requires an internet connection  │
│   to download the latest system images.                     │
│                                                             │
│   Please check your connection and try again.               │
│                                                             │
│                                        [ Retry ]            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Pre-release / Beta Images

- **Not supported in v1**
- Users can upgrade to beta/dev channels through Home Assistant's own UI after installation
- Keeps the installer simple and focused on stable releases

### App Updates

**No auto-update in v1.** This is a "run once" utility for most users, so a full update mechanism adds unnecessary complexity.

Instead, implement a simple version check with download prompt:

1. On launch, check for newer installer version (can be included in `version.home-assistant.io` response)
2. If newer version exists, show a non-intrusive banner:

```
┌─────────────────────────────────────────────────────────────────────┐
│  ℹ️ A new version is available (v1.2.0)        [ Download ]  [ ✕ ] │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                      [Normal app content]                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

3. "Download" opens the releases page in system browser
4. Banner is dismissable and doesn't block usage
5. User can continue using current version if they choose

**Beta versions:**

Users can opt-in to receive beta updates via a settings toggle.

```
┌─────────────────────────────────────────────────────────────────────┐
│  Settings                                                           │
│                                                                     │
│  ☐ Receive beta updates                                             │
│    Get early access to new features. Beta versions may be unstable. │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

When enabled, beta releases are included in version checks. Beta banner is visually distinct:

```
┌─────────────────────────────────────────────────────────────────────┐
│  🧪 Beta available (v1.3.0-beta.1)             [ Download ]  [ ✕ ] │
├─────────────────────────────────────────────────────────────────────┤
```

**Version tagging convention:**
- Stable: `v1.0.0`, `v1.1.0`, `v1.2.0`
- Beta: `v1.3.0-beta.1`, `v1.3.0-beta.2`
- Release candidates: `v1.3.0-rc.1`

**Implementation:**

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

```typescript
// On app launch
const settings = await loadSettings();
const update = await invoke('check_for_updates', { 
  includeBeta: settings.receiveBetaUpdates 
});

if (update) {
  showUpdateBanner(update.version, update.downloadUrl, update.isBeta);
}
```

**CI handling:**

Both stable and beta releases are built by the same workflow. The version string determines the release type:

```yaml
# In release.yml
- name: Determine release type
  run: |
    VERSION="${GITHUB_REF#refs/tags/v}"
    if [[ "$VERSION" == *"-beta"* ]] || [[ "$VERSION" == *"-rc"* ]]; then
      echo "PRERELEASE=true" >> $GITHUB_ENV
    else
      echo "PRERELEASE=false" >> $GITHUB_ENV
    fi

- name: Create release
  uses: softprops/action-gh-release@v1
  with:
    draft: true
    prerelease: ${{ env.PRERELEASE }}
    # ...
```

If usage patterns later show the app is used regularly, Tauri's built-in updater can be added.

### Branding

- Full name: **Home Assistant Installer**
- Short name: **HAI** (also means "Hi!" in Dutch, fitting for a welcoming app)
- Repository: `ha-installer` or `hai`
- Binary/app name: `hai` (lowercase)
- Follow Home Assistant brand guidelines for colors, typography, iconography
- Use HA blue (#03A9F4 / #41BDF5) as primary accent
- Match the visual language of the HA website and app

### Toolbox Integration

The Open Home Foundation Toolbox (https://toolbox.openhomefoundation.org/) is accessible via a button in the bottom-right corner of the app.

**UI placement:**
```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                         [Main app content]                          │
│                                                                     │
│                                                                     │
│                                                                     │
│                                                                     │
│                                                      ┌───────────┐  │
│                                                      │ 🧰 Toolbox│  │
│                                                      └───────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

**Behavior:**
- Clicking the button opens the Toolbox in an embedded iframe view within the app
- Back button or close returns to the installer flow
- Requires internet connection (show message if offline)

**Implementation:**
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

**CSP Configuration** (tauri.conf.json):
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; frame-src https://toolbox.openhomefoundation.org/"
    }
  }
}
```

### Distribution

All channels:

| Platform | Distribution Method |
|----------|---------------------|
| macOS | Direct download (.dmg), Homebrew Cask |
| Windows | Direct download (.msi/.exe), Microsoft Store |
| Linux | Direct download (.AppImage), Flathub, possibly .deb/.rpm |

Build automation via GitHub Actions to produce all artifacts on release.
