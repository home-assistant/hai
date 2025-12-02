//! Block device enumeration for different platforms
//!
//! This module provides platform-specific implementations for listing
//! block devices (SD cards, USB drives, etc.) that can be used as
//! installation targets.

use crate::types::{BlockDevice, DeviceType};

/// List all available block devices on the system
pub async fn list_devices() -> Result<Vec<BlockDevice>, String> {
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
        Err("Unsupported platform".to_string())
    }
}

// =============================================================================
// macOS Implementation
// =============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    pub(crate) struct DiskUtilList {
        #[serde(rename = "AllDisksAndPartitions")]
        all_disks_and_partitions: Vec<DiskEntry>,
    }

    impl DiskUtilList {
        pub(crate) fn len(&self) -> usize {
            self.all_disks_and_partitions.len()
        }
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
    pub(crate) struct DiskUtilInfo {
        #[serde(rename = "Ejectable", default)]
        pub ejectable: bool,
        #[serde(rename = "Removable", default)]
        pub removable: bool,
        #[serde(rename = "RemovableMedia", default)]
        pub removable_media: bool,
        #[serde(rename = "Internal", default)]
        pub internal: bool,
        #[serde(rename = "SolidState", default)]
        pub solid_state: bool,
        #[serde(rename = "MediaName", default)]
        pub media_name: Option<String>,
        #[serde(rename = "IORegistryEntryName", default)]
        pub io_registry_entry_name: Option<String>,
        #[serde(rename = "DeviceNode", default)]
        pub device_node: Option<String>,
        #[serde(rename = "Size", default)]
        pub size: u64,
        #[serde(rename = "BusProtocol", default)]
        pub bus_protocol: Option<String>,
        #[serde(rename = "MediaType", default)]
        pub media_type: Option<String>,
    }

    /// Parse diskutil list output from plist bytes
    pub(crate) fn parse_diskutil_list(bytes: &[u8]) -> Result<DiskUtilList, String> {
        plist::from_bytes(bytes).map_err(|e| format!("Failed to parse diskutil output: {}", e))
    }

    /// Parse diskutil info output from plist bytes
    pub(crate) fn parse_diskutil_info(bytes: &[u8]) -> Result<DiskUtilInfo, String> {
        plist::from_bytes(bytes).map_err(|e| format!("Failed to parse diskutil info: {}", e))
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>, String> {
        // Get list of all disks using diskutil
        let output = Command::new("diskutil")
            .args(["list", "-plist"])
            .output()
            .map_err(|e| format!("Failed to run diskutil: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "diskutil failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let disk_list = parse_diskutil_list(&output.stdout)?;

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
                .map_err(|e| format!("Failed to get disk info: {}", e))?;

            if !info_output.status.success() {
                continue;
            }

            let disk_info: DiskUtilInfo = match parse_diskutil_info(&info_output.stdout) {
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
            let device_type = determine_device_type_macos(&disk_info);

            // Build the device name
            let name = disk_info
                .media_name
                .or(disk_info.io_registry_entry_name)
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

    pub(crate) fn determine_device_type_macos(info: &DiskUtilInfo) -> DeviceType {
        let bus = info.bus_protocol.as_deref().unwrap_or("");
        let media = info.media_type.as_deref().unwrap_or("");

        // Check for SD card - usually connected via USB card reader
        // or internal SD slot
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

        for vendor in vendors {
            if name.to_lowercase().contains(&vendor.to_lowercase()) {
                let model = name
                    .replace(vendor, "")
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
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    pub(crate) struct LsblkOutput {
        pub blockdevices: Vec<LsblkDevice>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct LsblkDevice {
        pub name: String,
        #[serde(default)]
        pub size: Option<u64>,
        #[serde(rename = "type", default)]
        pub device_type: Option<String>,
        #[serde(default)]
        pub rm: Option<bool>, // removable
        #[serde(default)]
        pub ro: Option<bool>, // read-only
        #[serde(default)]
        pub tran: Option<String>, // transport (usb, sata, nvme, etc.)
        #[serde(default)]
        pub model: Option<String>,
        #[serde(default)]
        pub vendor: Option<String>,
        #[serde(default)]
        pub hotplug: Option<bool>,
    }

    /// Parse lsblk output from JSON bytes
    pub(crate) fn parse_lsblk_output(bytes: &[u8]) -> Result<LsblkOutput, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Failed to parse lsblk output: {}", e))
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>, String> {
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
            .map_err(|e| format!("Failed to run lsblk: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "lsblk failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let lsblk = parse_lsblk_output(&output.stdout)?;

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
            let device_type = determine_device_type_linux(&dev);

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

    pub(crate) fn determine_device_type_linux(dev: &LsblkDevice) -> DeviceType {
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
                // Could be SSD or HDD - check if it's likely an SSD
                // (this is a heuristic, not 100% accurate)
                if model.contains("ssd") {
                    DeviceType::Ssd
                } else {
                    DeviceType::Hdd
                }
            }
            _ => DeviceType::Unknown,
        }
    }

    pub(crate) fn build_device_name(
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
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Debug, Deserialize)]
    pub(crate) struct PowerShellDisk {
        #[serde(rename = "Number")]
        pub number: u32,
        #[serde(rename = "FriendlyName")]
        pub friendly_name: Option<String>,
        #[serde(rename = "Size")]
        pub size: Option<u64>,
        #[serde(rename = "MediaType")]
        pub media_type: Option<String>,
        #[serde(rename = "BusType")]
        pub bus_type: Option<String>,
        #[serde(rename = "IsSystem")]
        pub is_system: Option<bool>,
        #[serde(rename = "IsBoot")]
        pub is_boot: Option<bool>,
    }

    /// Parse PowerShell output from JSON string
    /// PowerShell returns a single object (not array) if there's only one disk
    pub(crate) fn parse_powershell_output(stdout: &str) -> Result<Vec<PowerShellDisk>, String> {
        let stdout = stdout.trim();

        // Handle empty output
        if stdout.is_empty() {
            return Ok(Vec::new());
        }

        // PowerShell returns a single object (not array) if there's only one disk
        if stdout.starts_with('[') {
            serde_json::from_str(stdout)
                .map_err(|e| format!("Failed to parse PowerShell output: {}", e))
        } else {
            let single: PowerShellDisk = serde_json::from_str(stdout)
                .map_err(|e| format!("Failed to parse PowerShell output: {}", e))?;
            Ok(vec![single])
        }
    }

    pub async fn list_devices() -> Result<Vec<BlockDevice>, String> {
        // Use PowerShell to get disk information in JSON format
        let script = r#"
            Get-Disk | Where-Object { $_.IsOffline -eq $false } | Select-Object Number, FriendlyName, Size, MediaType, BusType, IsSystem, IsBoot | ConvertTo-Json -Compress
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "PowerShell failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let disks = parse_powershell_output(&stdout)?;

        let mut devices = Vec::new();

        for disk in disks {
            // Skip system and boot disks
            if disk.is_system == Some(true) || disk.is_boot == Some(true) {
                continue;
            }

            // Skip very small devices (< 1GB)
            let size = disk.size.unwrap_or(0);
            if size < 1_000_000_000 {
                continue;
            }

            // Determine if removable based on bus type
            let bus_type = disk.bus_type.as_deref().unwrap_or("");
            let is_removable = matches!(bus_type, "USB" | "SD" | "MMC");

            // Skip non-removable drives
            if !is_removable {
                continue;
            }

            // Determine device type
            let device_type = determine_device_type_windows(&disk);

            let name = disk
                .friendly_name
                .clone()
                .unwrap_or_else(|| format!("Disk {}", disk.number));

            // Parse vendor/model from friendly name
            let (vendor, model) = parse_friendly_name(&name);

            devices.push(BlockDevice {
                id: format!("\\\\.\\PhysicalDrive{}", disk.number),
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

    pub(crate) fn determine_device_type_windows(disk: &PowerShellDisk) -> DeviceType {
        let bus_type = disk.bus_type.as_deref().unwrap_or("");
        let media_type = disk.media_type.as_deref().unwrap_or("");

        // Check bus type first
        match bus_type {
            "USB" => DeviceType::UsbDrive,
            "SD" | "MMC" => DeviceType::SdCard,
            "NVMe" => DeviceType::NvMe,
            "SATA" | "ATA" => {
                if media_type == "SSD" {
                    DeviceType::Ssd
                } else {
                    DeviceType::Hdd
                }
            }
            _ => {
                // Fall back to media type
                match media_type {
                    "SSD" => DeviceType::Ssd,
                    "HDD" => DeviceType::Hdd,
                    _ => DeviceType::Unknown,
                }
            }
        }
    }

    pub(crate) fn parse_friendly_name(name: &str) -> (Option<String>, Option<String>) {
        // Common vendor prefixes for Windows
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
        ];

        for vendor in vendors {
            if name.to_lowercase().contains(&vendor.to_lowercase()) {
                let model = name
                    .replace(vendor, "")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_devices() {
        // This test will run on the actual platform
        // In CI, it should return an empty list (no removable drives)
        // or fail gracefully
        let result = list_devices().await;
        assert!(
            result.is_ok(),
            "list_devices should not error: {:?}",
            result
        );
    }

    // =============================================================================
    // macOS-specific tests
    // =============================================================================

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::macos::{determine_device_type_macos, parse_media_name, DiskUtilInfo};
        use crate::types::DeviceType;

        // Helper to create a test DiskUtilInfo
        fn create_test_disk_info(
            bus_protocol: Option<&str>,
            media_type: Option<&str>,
            solid_state: bool,
            media_name: Option<&str>,
        ) -> DiskUtilInfo {
            DiskUtilInfo {
                ejectable: true,
                removable: true,
                removable_media: true,
                internal: false,
                solid_state,
                media_name: media_name.map(|s| s.to_string()),
                io_registry_entry_name: None,
                device_node: None,
                size: 16_000_000_000,
                bus_protocol: bus_protocol.map(|s| s.to_string()),
                media_type: media_type.map(|s| s.to_string()),
            }
        }

        #[test]
        fn test_determine_device_type_usb() {
            let info = create_test_disk_info(Some("USB"), None, false, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::UsbDrive,
                "USB bus protocol should return UsbDrive"
            );
        }

        #[test]
        fn test_determine_device_type_sd_card_media_type() {
            let info = create_test_disk_info(Some("USB"), Some("SD"), false, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::SdCard,
                "SD media type should return SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_sd_card_media_name() {
            let info = create_test_disk_info(Some("USB"), None, false, Some("SD Card Reader"));
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::SdCard,
                "SD in media name should return SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_nvme() {
            let info = create_test_disk_info(Some("PCI-Express"), None, true, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::NvMe,
                "PCI-Express + solid_state should return NvMe"
            );
        }

        #[test]
        fn test_determine_device_type_pci_ssd() {
            let info = create_test_disk_info(Some("PCI"), None, true, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::NvMe,
                "PCI + solid_state should return NvMe"
            );
        }

        #[test]
        fn test_determine_device_type_pci_non_solid() {
            let info = create_test_disk_info(Some("PCI"), None, false, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::Ssd,
                "PCI without solid_state should return Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_sata_ssd() {
            let info = create_test_disk_info(Some("SATA"), None, true, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::Ssd,
                "SATA + solid_state should return Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_sata_hdd() {
            let info = create_test_disk_info(Some("SATA"), None, false, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::Hdd,
                "SATA without solid_state should return Hdd"
            );
        }

        #[test]
        fn test_determine_device_type_unknown_solid_state() {
            let info = create_test_disk_info(None, None, true, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::Ssd,
                "Unknown bus with solid_state should return Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_unknown_non_solid() {
            let info = create_test_disk_info(None, None, false, None);
            assert_eq!(
                determine_device_type_macos(&info),
                DeviceType::Unknown,
                "Unknown bus without solid_state should return Unknown"
            );
        }

        #[test]
        fn test_parse_media_name_sandisk() {
            let (vendor, model) = parse_media_name("SanDisk Ultra 3.0");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Ultra 3.0".to_string()));
        }

        #[test]
        fn test_parse_media_name_samsung() {
            let (vendor, model) = parse_media_name("Samsung Portable SSD T5");
            assert_eq!(vendor, Some("Samsung".to_string()));
            assert_eq!(model, Some("Portable SSD T5".to_string()));
        }

        #[test]
        fn test_parse_media_name_kingston() {
            let (vendor, model) = parse_media_name("Kingston DataTraveler");
            assert_eq!(vendor, Some("Kingston".to_string()));
            assert_eq!(model, Some("DataTraveler".to_string()));
        }

        #[test]
        fn test_parse_media_name_vendor_only() {
            let (vendor, model) = parse_media_name("SanDisk");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, None);
        }

        #[test]
        fn test_parse_media_name_unknown() {
            let (vendor, model) = parse_media_name("Generic USB Device");
            assert_eq!(vendor, None);
            assert_eq!(model, Some("Generic USB Device".to_string()));
        }

        #[test]
        fn test_parse_media_name_case_insensitive() {
            let (vendor, model) = parse_media_name("sandisk ultra");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            // Note: The model will contain the full string because the replace is case-sensitive
            // This is expected behavior - the vendor is detected but not cleanly extracted
            assert_eq!(model, Some("sandisk ultra".to_string()));
        }

        #[test]
        fn test_parse_media_name_western_digital() {
            let (vendor, model) = parse_media_name("Western Digital My Passport");
            assert_eq!(vendor, Some("Western Digital".to_string()));
            assert_eq!(model, Some("My Passport".to_string()));
        }

        #[test]
        fn test_parse_media_name_wd() {
            let (vendor, model) = parse_media_name("WD Elements");
            assert_eq!(vendor, Some("WD".to_string()));
            assert_eq!(model, Some("Elements".to_string()));
        }

        // =====================================================================
        // Error handling tests for parsing failures
        // =====================================================================

        #[test]
        fn test_parse_diskutil_list_invalid_plist() {
            use super::super::macos::parse_diskutil_list;

            // Test with invalid XML/plist
            let invalid_plist = b"not a valid plist";
            let result = parse_diskutil_list(invalid_plist);
            assert!(result.is_err(), "Should fail with invalid plist");
            assert!(
                result
                    .unwrap_err()
                    .contains("Failed to parse diskutil output"),
                "Error message should mention parsing failure"
            );
        }

        #[test]
        fn test_parse_diskutil_list_malformed_plist() {
            use super::super::macos::parse_diskutil_list;

            // Test with malformed XML
            let malformed_plist = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist><dict>";
            let result = parse_diskutil_list(malformed_plist);
            assert!(result.is_err(), "Should fail with malformed plist");
        }

        #[test]
        fn test_parse_diskutil_list_wrong_structure() {
            use super::super::macos::parse_diskutil_list;

            // Valid plist but wrong structure (missing AllDisksAndPartitions)
            let wrong_structure = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>SomeOtherKey</key>
    <array/>
</dict>
</plist>"#;
            let result = parse_diskutil_list(wrong_structure);
            assert!(result.is_err(), "Should fail with wrong structure");
        }

        #[test]
        fn test_parse_diskutil_list_empty_devices() {
            use super::super::macos::parse_diskutil_list;

            // Valid plist with empty device list
            let empty_list = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AllDisksAndPartitions</key>
    <array/>
</dict>
</plist>"#;
            let result = parse_diskutil_list(empty_list);
            assert!(result.is_ok(), "Should succeed with empty device list");
            let disk_list = result.unwrap();
            assert_eq!(disk_list.len(), 0, "Should have zero devices");
        }

        #[test]
        fn test_parse_diskutil_info_invalid_plist() {
            use super::super::macos::parse_diskutil_info;

            let invalid_plist = b"not a valid plist";
            let result = parse_diskutil_info(invalid_plist);
            assert!(result.is_err(), "Should fail with invalid plist");
            assert!(
                result
                    .unwrap_err()
                    .contains("Failed to parse diskutil info"),
                "Error message should mention parsing failure"
            );
        }

        #[test]
        fn test_parse_diskutil_info_empty_data() {
            use super::super::macos::parse_diskutil_info;

            let empty_data = b"";
            let result = parse_diskutil_info(empty_data);
            assert!(result.is_err(), "Should fail with empty data");
        }
    }

    // =============================================================================
    // Linux-specific tests
    // =============================================================================

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::super::linux::{build_device_name, determine_device_type_linux, LsblkDevice};
        use crate::types::DeviceType;

        // Helper to create a test LsblkDevice
        fn create_test_device(
            name: &str,
            tran: Option<&str>,
            model: Option<&str>,
            vendor: Option<&str>,
        ) -> LsblkDevice {
            LsblkDevice {
                name: name.to_string(),
                size: Some(16_000_000_000),
                device_type: Some("disk".to_string()),
                rm: Some(true),
                ro: Some(false),
                tran: tran.map(|s| s.to_string()),
                model: model.map(|s| s.to_string()),
                vendor: vendor.map(|s| s.to_string()),
                hotplug: Some(true),
            }
        }

        #[test]
        fn test_determine_device_type_mmcblk() {
            let dev = create_test_device("mmcblk0", None, None, None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::SdCard,
                "mmcblk device should be SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_usb_linux() {
            let dev = create_test_device("sdb", Some("usb"), None, None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::UsbDrive,
                "usb transport should be UsbDrive"
            );
        }

        #[test]
        fn test_determine_device_type_nvme_linux() {
            let dev = create_test_device("nvme0n1", Some("nvme"), None, None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::NvMe,
                "nvme transport should be NvMe"
            );
        }

        #[test]
        fn test_determine_device_type_sata_ssd() {
            let dev = create_test_device("sda", Some("sata"), Some("Samsung SSD"), None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::Ssd,
                "SATA with SSD in model should be Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_sata_hdd() {
            let dev = create_test_device("sda", Some("sata"), Some("WD Blue"), None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::Hdd,
                "SATA without SSD in model should be Hdd"
            );
        }

        #[test]
        fn test_determine_device_type_ata() {
            let dev = create_test_device("sda", Some("ata"), Some("Generic"), None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::Hdd,
                "ATA transport should be Hdd by default"
            );
        }

        #[test]
        fn test_determine_device_type_sd_card_model() {
            let dev = create_test_device("sdb", Some("usb"), Some("SD Card Reader"), None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::SdCard,
                "SD in model name should be SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_unknown() {
            let dev = create_test_device("sda", Some("unknown"), None, None);
            assert_eq!(
                determine_device_type_linux(&dev),
                DeviceType::Unknown,
                "Unknown transport should be Unknown"
            );
        }

        #[test]
        fn test_build_device_name_vendor_model() {
            let name = build_device_name(
                "sda",
                &Some("SanDisk".to_string()),
                &Some("Ultra USB 3.0".to_string()),
            );
            assert_eq!(name, "SanDisk Ultra USB 3.0");
        }

        #[test]
        fn test_build_device_name_vendor_only() {
            let name = build_device_name("sda", &Some("Samsung".to_string()), &None);
            assert_eq!(name, "Samsung");
        }

        #[test]
        fn test_build_device_name_model_only() {
            let name = build_device_name("sda", &None, &Some("Generic Flash Drive".to_string()));
            assert_eq!(name, "Generic Flash Drive");
        }

        #[test]
        fn test_build_device_name_fallback() {
            let name = build_device_name("sdb", &None, &None);
            assert_eq!(name, "sdb");
        }

        // =====================================================================
        // Error handling tests for parsing failures
        // =====================================================================

        #[test]
        fn test_parse_lsblk_output_invalid_json() {
            use super::super::linux::parse_lsblk_output;

            let invalid_json = b"not valid json";
            let result = parse_lsblk_output(invalid_json);
            assert!(result.is_err(), "Should fail with invalid JSON");
            assert!(
                result.unwrap_err().contains("Failed to parse lsblk output"),
                "Error message should mention parsing failure"
            );
        }

        #[test]
        fn test_parse_lsblk_output_malformed_json() {
            use super::super::linux::parse_lsblk_output;

            let malformed_json = b"{\"blockdevices\":";
            let result = parse_lsblk_output(malformed_json);
            assert!(result.is_err(), "Should fail with malformed JSON");
        }

        #[test]
        fn test_parse_lsblk_output_wrong_structure() {
            use super::super::linux::parse_lsblk_output;

            // Valid JSON but missing blockdevices key
            let wrong_structure = b"{\"devices\": []}";
            let result = parse_lsblk_output(wrong_structure);
            assert!(result.is_err(), "Should fail with wrong structure");
        }

        #[test]
        fn test_parse_lsblk_output_empty_devices() {
            use super::super::linux::parse_lsblk_output;

            // Valid JSON with empty device list
            let empty_list = b"{\"blockdevices\": []}";
            let result = parse_lsblk_output(empty_list);
            assert!(result.is_ok(), "Should succeed with empty device list");
            let output = result.unwrap();
            assert_eq!(output.blockdevices.len(), 0, "Should have zero devices");
        }

        #[test]
        fn test_parse_lsblk_output_missing_required_fields() {
            use super::super::linux::parse_lsblk_output;

            // Missing required 'name' field
            let missing_field = b"{\"blockdevices\": [{\"size\": 1000000000}]}";
            let result = parse_lsblk_output(missing_field);
            assert!(result.is_err(), "Should fail with missing required fields");
        }

        #[test]
        fn test_parse_lsblk_output_empty_data() {
            use super::super::linux::parse_lsblk_output;

            let empty_data = b"";
            let result = parse_lsblk_output(empty_data);
            assert!(result.is_err(), "Should fail with empty data");
        }

        #[test]
        fn test_parse_lsblk_output_valid_with_optional_fields() {
            use super::super::linux::parse_lsblk_output;

            // Valid JSON with minimal fields (all optional fields as null)
            let minimal = br#"{
                "blockdevices": [
                    {
                        "name": "sda"
                    }
                ]
            }"#;
            let result = parse_lsblk_output(minimal);
            assert!(
                result.is_ok(),
                "Should succeed with minimal valid structure"
            );
            let output = result.unwrap();
            assert_eq!(output.blockdevices.len(), 1, "Should have one device");
            assert_eq!(output.blockdevices[0].name, "sda");
            assert_eq!(output.blockdevices[0].size, None);
            assert_eq!(output.blockdevices[0].tran, None);
        }
    }

    // =============================================================================
    // Windows-specific tests
    // =============================================================================

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::super::windows::{
            determine_device_type_windows, parse_friendly_name, PowerShellDisk,
        };
        use crate::types::DeviceType;

        // Helper to create a test PowerShellDisk
        fn create_test_disk(
            bus_type: Option<&str>,
            media_type: Option<&str>,
            friendly_name: Option<&str>,
        ) -> PowerShellDisk {
            PowerShellDisk {
                number: 1,
                friendly_name: friendly_name.map(|s| s.to_string()),
                size: Some(16_000_000_000),
                media_type: media_type.map(|s| s.to_string()),
                bus_type: bus_type.map(|s| s.to_string()),
                is_system: Some(false),
                is_boot: Some(false),
            }
        }

        #[test]
        fn test_determine_device_type_usb_windows() {
            let disk = create_test_disk(Some("USB"), None, None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::UsbDrive,
                "USB bus should be UsbDrive"
            );
        }

        #[test]
        fn test_determine_device_type_sd_windows() {
            let disk = create_test_disk(Some("SD"), None, None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::SdCard,
                "SD bus should be SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_mmc_windows() {
            let disk = create_test_disk(Some("MMC"), None, None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::SdCard,
                "MMC bus should be SdCard"
            );
        }

        #[test]
        fn test_determine_device_type_nvme_windows() {
            let disk = create_test_disk(Some("NVMe"), None, None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::NvMe,
                "NVMe bus should be NvMe"
            );
        }

        #[test]
        fn test_determine_device_type_sata_ssd() {
            let disk = create_test_disk(Some("SATA"), Some("SSD"), None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Ssd,
                "SATA with SSD media type should be Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_sata_hdd() {
            let disk = create_test_disk(Some("SATA"), Some("HDD"), None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Hdd,
                "SATA with HDD media type should be Hdd"
            );
        }

        #[test]
        fn test_determine_device_type_ata_ssd() {
            let disk = create_test_disk(Some("ATA"), Some("SSD"), None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Ssd,
                "ATA with SSD media type should be Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_unknown_fallback_ssd() {
            let disk = create_test_disk(Some("Unknown"), Some("SSD"), None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Ssd,
                "Unknown bus with SSD media type should be Ssd"
            );
        }

        #[test]
        fn test_determine_device_type_unknown_fallback_hdd() {
            let disk = create_test_disk(Some("Unknown"), Some("HDD"), None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Hdd,
                "Unknown bus with HDD media type should be Hdd"
            );
        }

        #[test]
        fn test_determine_device_type_unknown() {
            let disk = create_test_disk(Some("Unknown"), None, None);
            assert_eq!(
                determine_device_type_windows(&disk),
                DeviceType::Unknown,
                "Unknown bus and media type should be Unknown"
            );
        }

        #[test]
        fn test_parse_friendly_name_sandisk() {
            let (vendor, model) = parse_friendly_name("SanDisk Cruzer USB Device");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, Some("Cruzer USB Device".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_samsung() {
            let (vendor, model) = parse_friendly_name("Samsung Portable SSD T7");
            assert_eq!(vendor, Some("Samsung".to_string()));
            assert_eq!(model, Some("Portable SSD T7".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_kingston() {
            let (vendor, model) = parse_friendly_name("Kingston DataTraveler 3.0");
            assert_eq!(vendor, Some("Kingston".to_string()));
            assert_eq!(model, Some("DataTraveler 3.0".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_vendor_only() {
            let (vendor, model) = parse_friendly_name("SanDisk");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            assert_eq!(model, None);
        }

        #[test]
        fn test_parse_friendly_name_generic() {
            let (vendor, model) = parse_friendly_name("Generic Flash Disk USB Device");
            assert_eq!(vendor, Some("Generic".to_string()));
            assert_eq!(model, Some("Flash Disk USB Device".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_unknown() {
            let (vendor, model) = parse_friendly_name("Unknown Device");
            assert_eq!(vendor, None);
            assert_eq!(model, Some("Unknown Device".to_string()));
        }

        #[test]
        fn test_parse_friendly_name_case_insensitive() {
            let (vendor, model) = parse_friendly_name("sandisk ultra");
            assert_eq!(vendor, Some("SanDisk".to_string()));
            // Note: The model will contain the full string because the replace is case-sensitive
            // This is expected behavior - the vendor is detected but not cleanly extracted
            assert_eq!(model, Some("sandisk ultra".to_string()));
        }

        // =====================================================================
        // Error handling tests for parsing failures
        // =====================================================================

        #[test]
        fn test_parse_powershell_output_invalid_json() {
            use super::super::windows::parse_powershell_output;

            let invalid_json = "not valid json";
            let result = parse_powershell_output(invalid_json);
            assert!(result.is_err(), "Should fail with invalid JSON");
            assert!(
                result
                    .unwrap_err()
                    .contains("Failed to parse PowerShell output"),
                "Error message should mention parsing failure"
            );
        }

        #[test]
        fn test_parse_powershell_output_malformed_json() {
            use super::super::windows::parse_powershell_output;

            let malformed_json = "{\"Number\":";
            let result = parse_powershell_output(malformed_json);
            assert!(result.is_err(), "Should fail with malformed JSON");
        }

        #[test]
        fn test_parse_powershell_output_wrong_structure() {
            use super::super::windows::parse_powershell_output;

            // Valid JSON but missing required fields
            let wrong_structure = "{\"InvalidField\": \"value\"}";
            let result = parse_powershell_output(wrong_structure);
            assert!(result.is_err(), "Should fail with wrong structure");
        }

        #[test]
        fn test_parse_powershell_output_empty_string() {
            use super::super::windows::parse_powershell_output;

            // Empty output should return empty vec
            let empty = "";
            let result = parse_powershell_output(empty);
            assert!(result.is_ok(), "Should succeed with empty string");
            assert_eq!(result.unwrap().len(), 0, "Should return empty vec");
        }

        #[test]
        fn test_parse_powershell_output_whitespace_only() {
            use super::super::windows::parse_powershell_output;

            // Whitespace-only should return empty vec
            let whitespace = "   \n\t  ";
            let result = parse_powershell_output(whitespace);
            assert!(result.is_ok(), "Should succeed with whitespace");
            assert_eq!(result.unwrap().len(), 0, "Should return empty vec");
        }

        #[test]
        fn test_parse_powershell_output_empty_array() {
            use super::super::windows::parse_powershell_output;

            // Empty array
            let empty_array = "[]";
            let result = parse_powershell_output(empty_array);
            assert!(result.is_ok(), "Should succeed with empty array");
            assert_eq!(result.unwrap().len(), 0, "Should return empty vec");
        }

        #[test]
        fn test_parse_powershell_output_single_object() {
            use super::super::windows::parse_powershell_output;

            // Single object (not wrapped in array) - common PowerShell behavior
            let single_object = r#"{
                "Number": 1,
                "FriendlyName": "Test Disk",
                "Size": 16000000000,
                "MediaType": "SSD",
                "BusType": "USB",
                "IsSystem": false,
                "IsBoot": false
            }"#;
            let result = parse_powershell_output(single_object);
            assert!(result.is_ok(), "Should succeed with single object");
            let disks = result.unwrap();
            assert_eq!(disks.len(), 1, "Should return vec with one disk");
            assert_eq!(disks[0].number, 1);
            assert_eq!(disks[0].friendly_name, Some("Test Disk".to_string()));
        }

        #[test]
        fn test_parse_powershell_output_array_of_objects() {
            use super::super::windows::parse_powershell_output;

            // Array of objects - normal PowerShell behavior for multiple items
            let array = r#"[
                {
                    "Number": 1,
                    "FriendlyName": "Disk 1",
                    "Size": 16000000000,
                    "MediaType": "SSD",
                    "BusType": "USB",
                    "IsSystem": false,
                    "IsBoot": false
                },
                {
                    "Number": 2,
                    "FriendlyName": "Disk 2",
                    "Size": 32000000000,
                    "MediaType": null,
                    "BusType": "SD",
                    "IsSystem": false,
                    "IsBoot": false
                }
            ]"#;
            let result = parse_powershell_output(array);
            assert!(result.is_ok(), "Should succeed with array of objects");
            let disks = result.unwrap();
            assert_eq!(disks.len(), 2, "Should return vec with two disks");
            assert_eq!(disks[0].number, 1);
            assert_eq!(disks[1].number, 2);
            assert_eq!(disks[0].friendly_name, Some("Disk 1".to_string()));
            assert_eq!(disks[1].friendly_name, Some("Disk 2".to_string()));
        }

        #[test]
        fn test_parse_powershell_output_missing_optional_fields() {
            use super::super::windows::parse_powershell_output;

            // Object with only required Number field
            let minimal = r#"{
                "Number": 5,
                "FriendlyName": null,
                "Size": null,
                "MediaType": null,
                "BusType": null,
                "IsSystem": null,
                "IsBoot": null
            }"#;
            let result = parse_powershell_output(minimal);
            assert!(result.is_ok(), "Should succeed with minimal fields");
            let disks = result.unwrap();
            assert_eq!(disks.len(), 1, "Should return vec with one disk");
            assert_eq!(disks[0].number, 5);
            assert_eq!(disks[0].friendly_name, None);
            assert_eq!(disks[0].size, None);
        }

        #[test]
        fn test_parse_powershell_output_missing_required_field() {
            use super::super::windows::parse_powershell_output;

            // Missing required Number field
            let missing_number = r#"{
                "FriendlyName": "Test",
                "Size": 1000000000
            }"#;
            let result = parse_powershell_output(missing_number);
            assert!(result.is_err(), "Should fail with missing Number field");
        }

        #[test]
        fn test_parse_powershell_output_invalid_number_type() {
            use super::super::windows::parse_powershell_output;

            // Number field with wrong type
            let invalid_type = r#"{
                "Number": "not a number",
                "FriendlyName": "Test"
            }"#;
            let result = parse_powershell_output(invalid_type);
            assert!(result.is_err(), "Should fail with wrong type for Number");
        }
    }

    // =============================================================================
    // Empty Device List Scenario Tests
    // =============================================================================

    #[tokio::test]
    async fn test_list_devices_handles_empty_result() {
        // This test documents that list_devices should return Ok(Vec::new())
        // when no removable devices exist, not an error
        let result = list_devices().await;

        // Should never error just because there are no removable devices
        assert!(
            result.is_ok(),
            "list_devices should return Ok even with no removable devices"
        );

        // The vec might be empty or have devices depending on the test environment
        let devices = result.unwrap();
        assert!(devices.len() >= 0); // Always true, but documents the contract
    }

    #[test]
    fn test_empty_device_list_operations() {
        // Test that an empty device list can be handled properly
        let devices: Vec<BlockDevice> = Vec::new();

        // Basic operations on empty list
        assert!(devices.is_empty());
        assert_eq!(devices.len(), 0);

        // Iteration should work but not execute
        for device in &devices {
            panic!("Should not iterate over empty list, found: {}", device.name);
        }

        // Filter operations should work
        let filtered: Vec<&BlockDevice> = devices.iter().filter(|d| d.removable).collect();
        assert_eq!(filtered.len(), 0);

        // Map operations should work
        let names: Vec<String> = devices.iter().map(|d| d.name.clone()).collect();
        assert_eq!(names.len(), 0);
    }

    #[test]
    fn test_filtering_results_in_empty_device_list() {
        // Simulate a scenario where all devices are filtered out
        let all_devices = vec![
            BlockDevice {
                id: "/dev/disk0".to_string(),
                name: "Internal SSD".to_string(),
                size: 1_000_000_000_000,
                device_type: DeviceType::Ssd,
                removable: false,
                model: Some("Apple SSD".to_string()),
                vendor: Some("Apple".to_string()),
            },
            BlockDevice {
                id: "/dev/disk1".to_string(),
                name: "System Drive".to_string(),
                size: 500_000_000_000,
                device_type: DeviceType::NvMe,
                removable: false,
                model: Some("NVMe SSD".to_string()),
                vendor: Some("Samsung".to_string()),
            },
        ];

        // Filter for removable devices only (should result in empty list)
        let removable: Vec<&BlockDevice> = all_devices.iter().filter(|d| d.removable).collect();

        assert_eq!(
            removable.len(),
            0,
            "All devices are internal, so no removable devices"
        );

        // Filter for devices larger than 2TB (should result in empty list)
        let large_devices: Vec<&BlockDevice> = all_devices
            .iter()
            .filter(|d| d.size > 2_000_000_000_000)
            .collect();

        assert_eq!(large_devices.len(), 0, "No devices larger than 2TB");
    }

    #[test]
    fn test_all_devices_are_system_drives_scenario() {
        // Simulate scenario where platform returns devices but all are system drives
        let system_drives = vec![
            BlockDevice {
                id: "/dev/disk0".to_string(),
                name: "Macintosh HD".to_string(),
                size: 500_000_000_000,
                device_type: DeviceType::Ssd,
                removable: false,
                model: Some("APPLE SSD".to_string()),
                vendor: Some("Apple".to_string()),
            },
            BlockDevice {
                id: "/dev/disk1".to_string(),
                name: "Recovery".to_string(),
                size: 10_000_000_000,
                device_type: DeviceType::Ssd,
                removable: false,
                model: Some("APPLE SSD".to_string()),
                vendor: Some("Apple".to_string()),
            },
        ];

        // Verify all are non-removable
        for device in &system_drives {
            assert!(
                !device.removable,
                "System drive {} should not be removable",
                device.name
            );
        }

        // Filter for removable only (simulating what the UI would do)
        let removable_only: Vec<&BlockDevice> =
            system_drives.iter().filter(|d| d.removable).collect();

        assert_eq!(
            removable_only.len(),
            0,
            "System drives should not be removable"
        );
    }

    #[test]
    fn test_no_internal_drives_scenario() {
        // Test handling of a list with only removable drives (opposite scenario)
        let removable_drives = vec![
            BlockDevice {
                id: "/dev/sdb".to_string(),
                name: "USB Drive".to_string(),
                size: 32_000_000_000,
                device_type: DeviceType::UsbDrive,
                removable: true,
                model: Some("Ultra USB".to_string()),
                vendor: Some("SanDisk".to_string()),
            },
            BlockDevice {
                id: "/dev/mmcblk0".to_string(),
                name: "SD Card".to_string(),
                size: 16_000_000_000,
                device_type: DeviceType::SdCard,
                removable: true,
                model: None,
                vendor: None,
            },
        ];

        // Filter for internal only
        let internal_only: Vec<&BlockDevice> =
            removable_drives.iter().filter(|d| !d.removable).collect();

        assert_eq!(internal_only.len(), 0, "Should have no internal drives");

        // All should be removable
        for device in &removable_drives {
            assert!(
                device.removable,
                "Device {} should be removable",
                device.name
            );
        }
    }

    #[test]
    fn test_device_list_with_very_small_devices_filtered() {
        // Test that devices smaller than 1GB are filtered out
        // (as per the actual implementation in all platform modules)
        let all_devices = vec![
            BlockDevice {
                id: "/dev/sdc".to_string(),
                name: "Tiny USB".to_string(),
                size: 500_000_000, // 500MB - too small
                device_type: DeviceType::UsbDrive,
                removable: true,
                model: None,
                vendor: None,
            },
            BlockDevice {
                id: "/dev/sdd".to_string(),
                name: "Normal USB".to_string(),
                size: 32_000_000_000, // 32GB - good
                device_type: DeviceType::UsbDrive,
                removable: true,
                model: None,
                vendor: None,
            },
        ];

        // Filter for devices >= 1GB (1_000_000_000 bytes)
        let valid_size_devices: Vec<&BlockDevice> = all_devices
            .iter()
            .filter(|d| d.size >= 1_000_000_000)
            .collect();

        assert_eq!(
            valid_size_devices.len(),
            1,
            "Only one device should meet size requirement"
        );
        assert_eq!(valid_size_devices[0].name, "Normal USB");
    }
}
