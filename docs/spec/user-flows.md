# User Flows

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
