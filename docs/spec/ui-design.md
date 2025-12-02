# UI Design

## Design Philosophy: Visual-First

**The installer should be usable almost without reading.**

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

## Branding

- Full name: **Home Assistant Installer**
- Short name: **HAI** (also means "Hi!" in Dutch, fitting for a welcoming app)
- Repository: `ha-installer` or `hai`
- Binary/app name: `hai` (lowercase)
- Follow Home Assistant brand guidelines for colors, typography, iconography
- Use HA blue (#03A9F4 / #41BDF5) as primary accent
- Match the visual language of the HA website and app

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

## UI Mockups

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

**Platform-specific companion apps:**

| Running on | Show |
|------------|------|
| macOS | iOS QR, Android QR, Mac App install button |
| Windows | iOS QR, Android QR, Windows App download link |
| Linux | iOS QR, Android QR only |

**QR Code URLs:**

- iOS: `https://apps.apple.com/app/home-assistant/id1099568401`
- Android: `https://play.google.com/store/apps/details?id=io.homeassistant.companion.android`
- Mac App Store: `https://apps.apple.com/app/home-assistant/id1099568401`

---

## Toolbox Integration

The Open Home Foundation Toolbox (https://toolbox.openhomefoundation.org/) is accessible via a button in the bottom-right corner of the app.

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

---

## Update Banner

When a new version is available:

```
┌─────────────────────────────────────────────────────────────────────┐
│  ℹ️ A new version is available (v1.2.0)        [ Download ]  [ ✕ ] │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                      [Normal app content]                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

Beta version banner (for users who opted in):

```
┌─────────────────────────────────────────────────────────────────────┐
│  🧪 Beta available (v1.3.0-beta.1)             [ Download ]  [ ✕ ] │
├─────────────────────────────────────────────────────────────────────┤
```

---

## No Internet Screen

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   ⚠️  No Internet Connection                                        │
│                                                                     │
│   Home Assistant Installer requires an internet connection          │
│   to download the latest system images.                             │
│                                                                     │
│   Please check your connection and try again.                       │
│                                                                     │
│                                        [ Retry ]                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```
