//! Block device enumeration for different platforms
//!
//! This module provides platform-specific implementations for listing
//! block devices (SD cards, USB drives, etc.) that can be used as
//! installation targets.

use crate::error::Result;
use crate::types::{BlockDevice, DeviceType};

/// List all available block devices on the system
///
/// Returns removable devices suitable for flashing (SD cards, USB drives, etc.)
/// Filters out internal and system drives for safety.
pub async fn list_devices() -> Result<Vec<BlockDevice>> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            return Ok(crate::mock::get_mock_block_devices());
        }
    }

    #[cfg(target_os = "macos")]
    {
        macos::list_devices().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::list_devices().await
    }

    #[cfg(target_os = "windows")]
    {
        windows::list_devices().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(crate::error::Error::UnsupportedPlatform(
            "Block device enumeration".to_string(),
        ))
    }
}

// =============================================================================
// macOS Implementation
// =============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::error::Error;
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    struct DiskUtilList {
        #[serde(rename = "AllDisksAndPartitions")]
        all_disks_and_partitions: Vec<DiskEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct DiskEntry {
        #[serde(rename = "DeviceIdentifier")]
        device_identifier: String,
        #[serde(rename = "Size", default)]
        _size: u64,
        #[serde(rename = "Content", default)]
        _content: Option<String>,
        #[serde(rename = "Partitions", default)]
        _partitions: Vec<PartitionEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct PartitionEntry {
        #[serde(rename = "DeviceIdentifier")]
        _device_identifier: String,
        #[serde(rename = "Size", default)]
        _size: u64,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct DiskUtilInfo {
        #[serde(rename = "Ejectable", default)]
        pub(super) ejectable: bool,
        #[serde(rename = "Removable", default)]
        pub(super) removable: bool,
        #[serde(rename = "RemovableMedia", default)]
        pub(super) removable_media: bool,
        #[serde(rename = "Internal", default)]
        pub(super) internal: bool,
        #[serde(rename = "SolidState", default)]
        pub(super) solid_state: bool,
        #[serde(rename = "MediaName", default)]
        pub(super) media_name: Option<String>,
        #[serde(rename = "IORegistryEntryName", default)]
        pub(super) io_registry_entry_name: Option<String>,
        #[serde(rename = "DeviceNode", default)]
        pub(super) device_node: Option<String>,
        #[serde(rename = "Size", default)]
        pub(super) size: u64,
        #[serde(rename = "BusProtocol", default)]
        pub(super) bus_protocol: Option<String>,
        #[serde(rename = "MediaType", default)]
        pub(super) media_type: Option<String>,
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>> {
        // Get list of all disks using diskutil
        let output = Command::new("diskutil")
            .args(["list", "-plist"])
            .output()
            .map_err(Error::Io)?;

        if !output.status.success() {
            return Err(Error::DeviceNotFound(format!(
                "diskutil failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let disk_list: DiskUtilList =
            plist::from_bytes(&output.stdout).map_err(|e| Error::InvalidConfig(e.to_string()))?;

        let mut devices = Vec::new();

        // Get detailed info for each whole disk (not partitions)
        for disk in disk_list.all_disks_and_partitions {
            // Skip synthesized disks (APFS containers, etc.)
            if disk.device_identifier.starts_with("synthesized") {
                continue;
            }

            // Get detailed disk info
            let info_output = Command::new("diskutil")
                .args(["info", "-plist", &disk.device_identifier])
                .output()
                .map_err(Error::Io)?;

            if !info_output.status.success() {
                continue;
            }

            let disk_info: DiskUtilInfo = match plist::from_bytes(&info_output.stdout) {
                Ok(info) => info,
                Err(_) => continue,
            };

            // Filter: only include removable/ejectable external media
            // Skip internal drives
            if disk_info.internal && !disk_info.removable_media {
                continue;
            }

            // Must be ejectable or removable
            if !disk_info.ejectable && !disk_info.removable && !disk_info.removable_media {
                continue;
            }

            // Skip very small devices (< 1GB) - likely not real storage
            if disk_info.size < 1_000_000_000 {
                continue;
            }

            // Determine device type based on bus protocol and other properties
            let device_type = determine_device_type(&disk_info);

            // Build the device name
            let name = disk_info
                .media_name
                .clone()
                .or(disk_info.io_registry_entry_name.clone())
                .unwrap_or_else(|| disk.device_identifier.clone());

            // Extract vendor and model from media name if possible
            let (vendor, model) = parse_media_name(&name);

            let device_path = disk_info
                .device_node
                .unwrap_or_else(|| format!("/dev/{}", disk.device_identifier));

            devices.push(BlockDevice {
                id: device_path,
                name,
                size: disk_info.size,
                device_type,
                removable: disk_info.removable || disk_info.removable_media || disk_info.ejectable,
                model,
                vendor,
            });
        }

        Ok(devices)
    }

    pub(super) fn determine_device_type(info: &DiskUtilInfo) -> DeviceType {
        let bus = info.bus_protocol.as_deref().unwrap_or("");
        let media = info.media_type.as_deref().unwrap_or("");

        // Check for SD card
        if media.to_lowercase().contains("sd")
            || info
                .media_name
                .as_ref()
                .map(|n| n.to_lowercase().contains("sd"))
                .unwrap_or(false)
        {
            return DeviceType::SdCard;
        }

        // Check bus protocol
        match bus {
            "USB" => DeviceType::UsbDrive,
            "PCI-Express" | "PCI" => {
                if info.solid_state {
                    DeviceType::NvMe
                } else {
                    DeviceType::Ssd
                }
            }
            "SATA" => {
                if info.solid_state {
                    DeviceType::Ssd
                } else {
                    DeviceType::Hdd
                }
            }
            _ => {
                if info.solid_state {
                    DeviceType::Ssd
                } else {
                    DeviceType::Unknown
                }
            }
        }
    }

    pub(crate) fn parse_media_name(name: &str) -> (Option<String>, Option<String>) {
        // Common vendor prefixes
        let vendors = [
            "SanDisk",
            "Samsung",
            "Kingston",
            "Lexar",
            "PNY",
            "Transcend",
            "Sony",
            "Toshiba",
            "Western Digital",
            "WD",
            "Seagate",
            "Crucial",
            "Micron",
        ];

        let name_lower = name.to_lowercase();

        for vendor in vendors {
            if let Some(pos) = name_lower.find(&vendor.to_lowercase()) {
                // Remove the vendor from the name (case-insensitive)
                let model = format!("{}{}", &name[..pos], &name[pos + vendor.len()..])
                    .trim()
                    .trim_start_matches(&[' ', '-', '_'][..])
                    .to_string();
                return (
                    Some(vendor.to_string()),
                    if model.is_empty() { None } else { Some(model) },
                );
            }
        }

        (None, Some(name.to_string()))
    }
}

// =============================================================================
// Linux Implementation
// =============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::error::Error;
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<LsblkDevice>,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct LsblkDevice {
        pub(super) name: String,
        #[serde(default)]
        pub(super) size: Option<u64>,
        #[serde(rename = "type", default)]
        pub(super) device_type: Option<String>,
        #[serde(default)]
        pub(super) rm: Option<bool>, // removable
        #[serde(default)]
        pub(super) ro: Option<bool>, // read-only
        #[serde(default)]
        pub(super) tran: Option<String>, // transport (usb, sata, nvme, etc.)
        #[serde(default)]
        pub(super) model: Option<String>,
        #[serde(default)]
        pub(super) vendor: Option<String>,
        #[serde(default)]
        pub(super) hotplug: Option<bool>,
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>> {
        // Use lsblk with JSON output for reliable parsing
        let output = Command::new("lsblk")
            .args([
                "-J", // JSON output
                "-b", // Size in bytes
                "-d", // Don't show partitions
                "-o", // Output columns
                "NAME,SIZE,TYPE,RM,RO,TRAN,MODEL,VENDOR,HOTPLUG",
            ])
            .output()
            .map_err(Error::Io)?;

        if !output.status.success() {
            return Err(Error::DeviceNotFound(format!(
                "lsblk failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let lsblk: LsblkOutput = serde_json::from_slice(&output.stdout)?;

        let mut devices = Vec::new();

        for dev in lsblk.blockdevices {
            // Only include disk devices (not partitions, loop devices, etc.)
            if dev.device_type.as_deref() != Some("disk") {
                continue;
            }

            // Skip read-only devices
            if dev.ro == Some(true) {
                continue;
            }

            // Skip non-removable, non-hotplug devices (likely system drives)
            let is_removable = dev.rm == Some(true) || dev.hotplug == Some(true);
            if !is_removable {
                continue;
            }

            // Skip very small devices (< 1GB)
            let size = dev.size.unwrap_or(0);
            if size < 1_000_000_000 {
                continue;
            }

            // Determine device type
            let device_type = determine_device_type(&dev);

            // Clean up model and vendor strings
            let model = dev
                .model
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let vendor = dev
                .vendor
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            // Build human-readable name
            let name = build_device_name(&dev.name, &vendor, &model);

            devices.push(BlockDevice {
                id: format!("/dev/{}", dev.name),
                name,
                size,
                device_type,
                removable: is_removable,
                model,
                vendor,
            });
        }

        Ok(devices)
    }

    pub(super) fn determine_device_type(dev: &LsblkDevice) -> DeviceType {
        let transport = dev.tran.as_deref().unwrap_or("");
        let model = dev.model.as_deref().unwrap_or("").to_lowercase();

        // Check for SD card
        if dev.name.starts_with("mmcblk") {
            return DeviceType::SdCard;
        }

        // Check model name for hints
        if model.contains("sd ") || model.contains("sd card") {
            return DeviceType::SdCard;
        }

        // Check transport type
        match transport {
            "usb" => DeviceType::UsbDrive,
            "nvme" => DeviceType::NvMe,
            "sata" | "ata" => {
                if model.contains("ssd") {
                    DeviceType::Ssd
                } else {
                    DeviceType::Hdd
                }
            }
            _ => DeviceType::Unknown,
        }
    }

    pub(super) fn build_device_name(
        dev_name: &str,
        vendor: &Option<String>,
        model: &Option<String>,
    ) -> String {
        match (vendor, model) {
            (Some(v), Some(m)) => format!("{} {}", v, m),
            (Some(v), None) => v.clone(),
            (None, Some(m)) => m.clone(),
            (None, None) => dev_name.to_string(),
        }
    }
}

// =============================================================================
// Windows Implementation
// =============================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use crate::error::Error;
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    struct PowerShellDisk {
        #[serde(rename = "Number")]
        number: u32,
        #[serde(rename = "FriendlyName")]
        friendly_name: Option<String>,
        #[serde(rename = "Size")]
        size: Option<u64>,
        #[serde(rename = "MediaType")]
        media_type: Option<String>,
        #[serde(rename = "BusType")]
        bus_type: Option<String>,
        #[serde(rename = "IsSystem")]
        is_system: Option<bool>,
        #[serde(rename = "IsBoot")]
        is_boot: Option<bool>,
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>> {
        // Use PowerShell to get disk information in JSON format
        let script = r#"
            Get-Disk | Where-Object { $_.IsOffline -eq $false } | Select-Object Number, FriendlyName, Size, MediaType, BusType, IsSystem, IsBoot | ConvertTo-Json -Compress
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|e| Error::Io(e))?;

        if !output.status.success() {
            return Err(Error::DeviceNotFound(format!(
                "PowerShell failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();

        // Handle empty output
        if stdout.is_empty() {
            return Ok(Vec::new());
        }

        // PowerShell returns a single object (not array) if there's only one disk
        let disks: Vec<PowerShellDisk> = if stdout.starts_with('[') {
            serde_json::from_str(stdout)?
        } else {
            let single: PowerShellDisk = serde_json::from_str(stdout)?;
            vec![single]
        };

        let mut devices = Vec::new();

        for disk in disks {
            // Skip system and boot drives
            if disk.is_system == Some(true) || disk.is_boot == Some(true) {
                continue;
            }

            // Skip very small drives (< 1GB)
            let size = disk.size.unwrap_or(0);
            if size < 1_000_000_000 {
                continue;
            }

            // Determine device type based on bus type
            let device_type = determine_device_type(&disk);

            // Parse vendor and model from the friendly name
            let friendly_name = disk
                .friendly_name
                .clone()
                .unwrap_or_else(|| format!("Disk {}", disk.number));
            let (vendor, model) = parse_friendly_name(&friendly_name);

            devices.push(BlockDevice {
                id: format!("\\\\.\\PhysicalDrive{}", disk.number),
                name: friendly_name,
                size,
                device_type,
                removable: matches!(
                    disk.bus_type.as_deref(),
                    Some("USB") | Some("SD") | Some("MMC")
                ),
                model,
                vendor,
            });
        }

        Ok(devices)
    }

    fn determine_device_type(disk: &PowerShellDisk) -> DeviceType {
        let bus = disk.bus_type.as_deref().unwrap_or("");
        let media = disk.media_type.as_deref().unwrap_or("");

        match bus {
            "USB" => DeviceType::UsbDrive,
            "SD" | "MMC" => DeviceType::SdCard,
            "NVMe" => DeviceType::NvMe,
            "SATA" | "ATA" => {
                if media == "SSD" {
                    DeviceType::Ssd
                } else {
                    DeviceType::Hdd
                }
            }
            _ => {
                if media == "SSD" {
                    DeviceType::Ssd
                } else {
                    DeviceType::Unknown
                }
            }
        }
    }

    /// Parse the FriendlyName from Windows to extract vendor and model.
    /// Windows often combines vendor and model in the FriendlyName field.
    pub(crate) fn parse_friendly_name(name: &str) -> (Option<String>, Option<String>) {
        // Common vendor prefixes found in Windows disk FriendlyNames
        let vendors = [
            "SanDisk",
            "Samsung",
            "Kingston",
            "Lexar",
            "PNY",
            "Transcend",
            "Sony",
            "Toshiba",
            "Western Digital",
            "WD",
            "Seagate",
            "Crucial",
            "Micron",
            "Generic",
            "USB",
        ];

        let name_lower = name.to_lowercase();

        for vendor in vendors {
            if name_lower.contains(&vendor.to_lowercase()) {
                // Find the vendor in the original string (preserving case)
                let vendor_start = name_lower.find(&vendor.to_lowercase()).unwrap();
                let vendor_end = vendor_start + vendor.len();

                // Model is whatever comes after the vendor name
                let model = name[vendor_end..]
                    .trim()
                    .trim_start_matches(&[' ', '-', '_'][..])
                    .to_string();

                return (
                    Some(vendor.to_string()),
                    if model.is_empty() { None } else { Some(model) },
                );
            }
        }

        // No known vendor found - use entire name as model
        (
            None,
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_device_type_values() {
        // Ensure all device types can be created
        let types = vec![
            DeviceType::SdCard,
            DeviceType::UsbDrive,
            DeviceType::Ssd,
            DeviceType::Hdd,
            DeviceType::NvMe,
            DeviceType::Unknown,
        ];

        for device_type in types {
            let json = serde_json::to_string(&device_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_block_device_creation() {
        let device = BlockDevice {
            id: "/dev/sdb".to_string(),
            name: "Test Device".to_string(),
            size: 32_000_000_000,
            device_type: DeviceType::UsbDrive,
            removable: true,
            model: Some("Test Model".to_string()),
            vendor: Some("Test Vendor".to_string()),
        };

        assert_eq!(device.id, "/dev/sdb");
        assert_eq!(device.size, 32_000_000_000);
        assert!(device.removable);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    #[serial]
    async fn test_list_devices_returns_mock_data() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let devices = list_devices().await.unwrap();
        assert!(!devices.is_empty());
        // Verify we get the expected mock devices
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    #[serial]
    async fn test_list_devices_mock_devices_have_valid_structure() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let devices = list_devices().await.unwrap();
        for device in &devices {
            assert!(!device.id.is_empty());
            assert!(!device.name.is_empty());
            assert!(device.size > 0);
        }
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    #[serial]
    async fn test_list_devices_mock_has_various_device_types() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let devices = list_devices().await.unwrap();
        // Check that we have different device types
        let has_sd = devices
            .iter()
            .any(|d| matches!(d.device_type, DeviceType::SdCard));
        let has_usb = devices
            .iter()
            .any(|d| matches!(d.device_type, DeviceType::UsbDrive));
        assert!(has_sd || has_usb, "Mock should have SD or USB devices");
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::macos::{determine_device_type, parse_media_name, DiskUtilInfo};
        use crate::types::DeviceType;

        #[test]
        fn test_parse_media_name_with_vendor() {
            let (vendor, model) = parse_media_name("SanDisk Ultra");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra".to_string()));
        }

        #[test]
        fn test_parse_media_name_vendor_only() {
            let (vendor, model) = parse_media_name("Samsung");
            assert_eq!(vendor, Some("Samsung".to_string()));
            assert_eq!(model, None);
        }

        #[test]
        fn test_parse_media_name_no_vendor() {
            let (vendor, model) = parse_media_name("Unknown Device");
            assert_eq!(vendor, None);
            assert_eq!(model, Some("Unknown Device".to_string()));
        }

        #[test]
        fn test_parse_media_name_case_insensitive() {
            let (vendor, model) = parse_media_name("SANDISK EXTREME PRO");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("EXTREME PRO".to_string()));
        }

        #[test]
        fn test_determine_device_type_sd_card_by_media_type() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: true,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: Some("USB".to_string()),
                media_type: Some("SD Card".to_string()),
            };
            assert_eq!(determine_device_type(&info), DeviceType::SdCard);
        }

        #[test]
        fn test_determine_device_type_sd_card_by_media_name() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: true,
                media_name: Some("SD Card Reader".to_string()),
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: None,
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::SdCard);
        }

        #[test]
        fn test_determine_device_type_usb_drive() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: false,
                media_name: Some("USB Drive".to_string()),
                io_registry_entry_name: None,
                device_node: None,
                size: 64_000_000_000,
                bus_protocol: Some("USB".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::UsbDrive);
        }

        #[test]
        fn test_determine_device_type_nvme_pcie() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: true,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 500_000_000_000,
                bus_protocol: Some("PCI-Express".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::NvMe);
        }

        #[test]
        fn test_determine_device_type_ssd_pcie() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: false,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 500_000_000_000,
                bus_protocol: Some("PCI".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Ssd);
        }

        #[test]
        fn test_determine_device_type_ssd_sata() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: true,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 256_000_000_000,
                bus_protocol: Some("SATA".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Ssd);
        }

        #[test]
        fn test_determine_device_type_hdd_sata() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: false,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 1_000_000_000_000,
                bus_protocol: Some("SATA".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Hdd);
        }

        #[test]
        fn test_determine_device_type_unknown_protocol_solid_state() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: true,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 128_000_000_000,
                bus_protocol: Some("Unknown".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Ssd);
        }

        #[test]
        fn test_determine_device_type_unknown() {
            let info = DiskUtilInfo {
                ejectable: false,
                removable: false,
                removable_media: false,
                internal: true,
                solid_state: false,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 128_000_000_000,
                bus_protocol: None,
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Unknown);
        }
    }

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::super::linux::{build_device_name, determine_device_type, LsblkDevice};
        use crate::types::DeviceType;

        #[test]
        fn test_determine_device_type_mmcblk_sd_card() {
            let dev = LsblkDevice {
                name: "mmcblk0".to_string(),
                size: Some(32_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: None,
                model: None,
                vendor: None,
                hotplug: Some(false),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::SdCard);
        }

        #[test]
        fn test_determine_device_type_usb_transport() {
            let dev = LsblkDevice {
                name: "sdb".to_string(),
                size: Some(64_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: Some("usb".to_string()),
                model: Some("USB Drive".to_string()),
                vendor: Some("Generic".to_string()),
                hotplug: Some(true),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::UsbDrive);
        }

        #[test]
        fn test_determine_device_type_nvme_transport() {
            let dev = LsblkDevice {
                name: "nvme0n1".to_string(),
                size: Some(512_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(false),
                ro: Some(false),
                tran: Some("nvme".to_string()),
                model: Some("Samsung 970 EVO".to_string()),
                vendor: Some("Samsung".to_string()),
                hotplug: Some(false),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::NvMe);
        }

        #[test]
        fn test_determine_device_type_sata_with_ssd_in_model() {
            let dev = LsblkDevice {
                name: "sda".to_string(),
                size: Some(256_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(false),
                ro: Some(false),
                tran: Some("sata".to_string()),
                model: Some("Samsung SSD 860".to_string()),
                vendor: Some("Samsung".to_string()),
                hotplug: Some(false),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::Ssd);
        }

        #[test]
        fn test_determine_device_type_sata_without_ssd() {
            let dev = LsblkDevice {
                name: "sda".to_string(),
                size: Some(1_000_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(false),
                ro: Some(false),
                tran: Some("sata".to_string()),
                model: Some("WD Blue".to_string()),
                vendor: Some("WD".to_string()),
                hotplug: Some(false),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::Hdd);
        }

        #[test]
        fn test_determine_device_type_ata_transport_with_ssd() {
            let dev = LsblkDevice {
                name: "sda".to_string(),
                size: Some(128_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(false),
                ro: Some(false),
                tran: Some("ata".to_string()),
                model: Some("Crucial SSD".to_string()),
                vendor: Some("Crucial".to_string()),
                hotplug: Some(false),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::Ssd);
        }

        #[test]
        fn test_determine_device_type_unknown_transport() {
            let dev = LsblkDevice {
                name: "sdc".to_string(),
                size: Some(64_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: Some("unknown".to_string()),
                model: None,
                vendor: None,
                hotplug: Some(true),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::Unknown);
        }

        #[test]
        fn test_determine_device_type_sd_in_model_name() {
            let dev = LsblkDevice {
                name: "sdb".to_string(),
                size: Some(32_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: Some("usb".to_string()),
                model: Some("SD Card Reader".to_string()),
                vendor: None,
                hotplug: Some(true),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::SdCard);
        }

        #[test]
        fn test_build_device_name_with_vendor_and_model() {
            let vendor = Some("SanDisk".to_string());
            let model = Some("Ultra".to_string());
            let result = build_device_name("sdb", &vendor, &model);
            assert_eq!(result, "SanDisk Ultra");
        }

        #[test]
        fn test_build_device_name_with_vendor_only() {
            let vendor = Some("Samsung".to_string());
            let model = None;
            let result = build_device_name("sdb", &vendor, &model);
            assert_eq!(result, "Samsung");
        }

        #[test]
        fn test_build_device_name_with_model_only() {
            let vendor = None;
            let model = Some("Generic USB Drive".to_string());
            let result = build_device_name("sdb", &vendor, &model);
            assert_eq!(result, "Generic USB Drive");
        }

        #[test]
        fn test_build_device_name_with_neither() {
            let vendor = None;
            let model = None;
            let result = build_device_name("sdb", &vendor, &model);
            assert_eq!(result, "sdb");
        }
    }

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::super::windows::parse_friendly_name;

        #[test]
        fn test_parse_friendly_name_with_vendor() {
            let (vendor, model) = parse_friendly_name("SanDisk Ultra USB");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra USB".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_vendor_only() {
            let (vendor, model) = parse_friendly_name("Kingston");
            assert_eq!(vendor, Some("Kingston".to_string()));
            assert_eq!(model, None);
        }

        #[test]
        fn test_parse_friendly_name_no_vendor() {
            let (vendor, model) = parse_friendly_name("Unknown Device");
            assert_eq!(vendor, None);
            assert_eq!(model, Some("Unknown Device".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_empty() {
            let (vendor, model) = parse_friendly_name("");
            assert_eq!(vendor, None);
            assert_eq!(model, None);
        }
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    #[serial]
    async fn test_list_devices_without_mock_env() {
        std::env::remove_var("HA_INSTALLER_MOCK");
        let devices = list_devices().await;
        // Should use the real platform implementation, not mock
        assert!(devices.is_ok());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[cfg(target_os = "macos")]
    mod macos_additional_tests {
        use super::super::macos::{determine_device_type, parse_media_name, DiskUtilInfo};
        use crate::types::DeviceType;

        #[test]
        fn test_parse_media_name_with_various_vendors() {
            // Test all vendor variations
            let vendors_to_test = [
                ("Kingston DataTraveler", "Kingston", "DataTraveler"),
                ("Lexar JumpDrive", "Lexar", "JumpDrive"),
                ("PNY USB Drive", "PNY", "USB Drive"),
                ("Transcend JetFlash", "Transcend", "JetFlash"),
                ("Sony Storage", "Sony", "Storage"),
                ("Toshiba Drive", "Toshiba", "Drive"),
                (
                    "Western Digital My Passport",
                    "Western Digital",
                    "My Passport",
                ),
                ("WD Elements", "WD", "Elements"),
                ("Seagate Backup Plus", "Seagate", "Backup Plus"),
                ("Crucial X6", "Crucial", "X6"),
                ("Micron M600", "Micron", "M600"),
            ];

            for (input, expected_vendor, expected_model) in vendors_to_test {
                let (vendor, model) = parse_media_name(input);
                assert_eq!(
                    vendor,
                    Some(expected_vendor.to_string()),
                    "Failed for input: {}",
                    input
                );
                assert_eq!(
                    model,
                    Some(expected_model.to_string()),
                    "Failed for input: {}",
                    input
                );
            }
        }

        #[test]
        fn test_parse_media_name_vendor_at_end() {
            let (vendor, model) = parse_media_name("Ultra SanDisk");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra".to_string()));
        }

        #[test]
        fn test_parse_media_name_with_hyphens_and_underscores() {
            let (vendor, model) = parse_media_name("SanDisk-Ultra-Pro");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra-Pro".to_string()));
        }

        #[test]
        fn test_parse_media_name_empty_model_after_vendor() {
            let (vendor, model) = parse_media_name("SanDisk   ");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, None);
        }

        #[test]
        fn test_determine_device_type_with_no_bus_protocol() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: false,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: None,
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Unknown);
        }

        #[test]
        fn test_determine_device_type_empty_bus_protocol() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: false,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: Some("".to_string()),
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::Unknown);
        }

        #[test]
        fn test_determine_device_type_sd_lowercase_in_media_type() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: true,
                media_name: None,
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: None,
                media_type: Some("sd".to_string()),
            };
            assert_eq!(determine_device_type(&info), DeviceType::SdCard);
        }

        #[test]
        fn test_determine_device_type_sd_uppercase_in_media_name() {
            let info = DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state: true,
                media_name: Some("SD READER".to_string()),
                io_registry_entry_name: None,
                device_node: None,
                size: 32_000_000_000,
                bus_protocol: None,
                media_type: None,
            };
            assert_eq!(determine_device_type(&info), DeviceType::SdCard);
        }

        #[test]
        fn test_parse_media_name_with_underscores() {
            let (vendor, model) = parse_media_name("SanDisk_Ultra_Pro");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra_Pro".to_string()));
        }

        #[test]
        fn test_parse_media_name_empty_string() {
            let (vendor, model) = parse_media_name("");
            assert_eq!(vendor, None);
            assert_eq!(model, Some("".to_string()));
        }
    }

    #[cfg(target_os = "linux")]
    mod linux_additional_tests {
        use super::super::linux::{build_device_name, determine_device_type, LsblkDevice};
        use crate::types::DeviceType;

        #[test]
        fn test_determine_device_type_sd_in_model_with_space() {
            let dev = LsblkDevice {
                name: "sdb".to_string(),
                size: Some(32_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: Some("usb".to_string()),
                model: Some("SD CARD".to_string()),
                vendor: None,
                hotplug: Some(true),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::SdCard);
        }

        #[test]
        fn test_determine_device_type_no_transport() {
            let dev = LsblkDevice {
                name: "sdb".to_string(),
                size: Some(32_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: None,
                model: None,
                vendor: None,
                hotplug: Some(true),
            };
            assert_eq!(determine_device_type(&dev), DeviceType::Unknown);
        }

        #[test]
        fn test_build_device_name_all_combinations() {
            // Test various vendor/model combinations
            assert_eq!(build_device_name("sda", &None, &None), "sda");
            assert_eq!(build_device_name("sdb", &Some("V".to_string()), &None), "V");
            assert_eq!(build_device_name("sdc", &None, &Some("M".to_string())), "M");
            assert_eq!(
                build_device_name("sdd", &Some("V".to_string()), &Some("M".to_string())),
                "V M"
            );
        }
    }

    #[cfg(target_os = "windows")]
    mod windows_additional_tests {
        use super::super::windows::parse_friendly_name;

        #[test]
        fn test_parse_friendly_name_generic_usb() {
            let (vendor, model) = parse_friendly_name("Generic USB Flash Disk");
            assert_eq!(vendor, Some("Generic".to_string()));
            assert_eq!(model, Some("Flash Disk".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_usb_prefix() {
            let (vendor, model) = parse_friendly_name("USB DISK");
            assert_eq!(vendor, Some("USB".to_string()));
            assert_eq!(model, Some("DISK".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_western_digital() {
            let (vendor, model) = parse_friendly_name("Western Digital My Passport");
            assert_eq!(vendor, Some("Western Digital".to_string()));
            assert_eq!(model, Some("My Passport".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_case_variations() {
            let (vendor, model) = parse_friendly_name("samsung evo");
            assert_eq!(vendor, Some("Samsung".to_string()));
            assert_eq!(model, Some("evo".to_string()));
        }
    }
}
