//! Type definitions for hai-core
//!
//! This module contains all the shared types used across the Home Assistant
//! Installer, including block device representations, flash progress,
//! and configuration types.

use serde::{Deserialize, Serialize};

/// Represents a block device (SD card, USB drive, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    /// Unique identifier (e.g., "/dev/sda" on Linux, "disk2" on macOS)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Size in bytes
    pub size: u64,
    /// Device type
    pub device_type: DeviceType,
    /// Whether this is a removable device
    pub removable: bool,
    /// Model name if available
    pub model: Option<String>,
    /// Vendor name if available
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    SdCard,
    UsbDrive,
    Ssd,
    Hdd,
    NvMe,
    Unknown,
}

/// Progress event sent during flashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashProgress {
    /// Current stage of the process
    pub stage: FlashStage,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes to process
    pub total_bytes: u64,
    /// Human-readable message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlashStage {
    Downloading,
    Extracting,
    Writing,
    Verifying,
    Finalizing,
    /// Waiting for Home Assistant to be ready
    Ready,
    /// Updating Home Assistant to latest version
    Updating,
    Complete,
    Error,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Whether an update is available
    pub update_available: bool,
    /// Current version
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Download URL for the latest version
    pub download_url: Option<String>,
    /// Release notes URL
    pub release_notes_url: Option<String>,
    /// Whether this is a beta release
    pub is_beta: bool,
}

/// Device manifest for supported devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceManifest {
    /// Version of the manifest format
    pub version: u32,
    /// List of supported devices
    pub devices: Vec<Device>,
}

/// A supported device in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Device category
    pub category: DeviceCategory,
    /// Image URL for the device photo
    pub image_url: Option<String>,
    /// HAOS image configuration
    pub haos: HaosConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCategory {
    RaspberryPi,
    Odroid,
    Khadas,
    Asus,
    HomeAssistantHardware,
    GenericX86,
    GenericArm64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaosConfig {
    /// Board identifier for the HAOS image
    pub board: String,
    /// Download URL template
    pub download_url: String,
}

/// Flash request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashRequest {
    /// Target device ID (block device path)
    pub device_id: String,
    /// Board identifier (e.g., "rpi5-64", "green")
    pub board: String,
    /// Whether to verify after writing
    pub verify: bool,
}

/// HAOS release information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaosRelease {
    /// Version string (e.g., "16.3")
    pub version: String,
    /// List of available images
    pub images: Vec<HaosImage>,
}

/// A single HAOS image file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaosImage {
    /// Board name (e.g., "rpi5-64", "green", "generic-x86-64")
    pub board: String,
    /// Download URL
    pub download_url: String,
    /// File size in bytes
    pub size: u64,
    /// SHA256 checksum (hex string)
    pub sha256: String,
}

/// GitHub release asset from API
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    /// Digest in format "sha256:hexstring"
    pub digest: Option<String>,
}

/// GitHub release from API
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

/// Result of a flash operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration in seconds
    pub duration_secs: u64,
}

/// Stable version info from version.home-assistant.io/stable.json
#[derive(Debug, Clone, Deserialize)]
pub struct StableVersionInfo {
    /// HAOS versions per board
    pub hassos: std::collections::HashMap<String, String>,
}

// ============================================================================
// Proxmox VE Types
// ============================================================================

/// Proxmox connection credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxCredentials {
    /// Proxmox server URL (e.g., https://192.168.1.100:8006)
    pub server_url: String,
    /// Username (e.g., root@pam)
    pub username: String,
    /// Password
    pub password: String,
}

/// Proxmox session (authentication result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxSession {
    /// Server URL for the session
    pub server_url: String,
    /// Authentication ticket
    pub ticket: String,
    /// CSRF prevention token
    pub csrf_token: String,
}

/// Proxmox node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxNode {
    /// Node name
    pub name: String,
    /// Node status (online/offline)
    pub status: String,
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    /// Memory usage in bytes
    pub memory_used: Option<u64>,
    /// Total memory in bytes
    pub memory_total: Option<u64>,
}

/// Proxmox storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxStorage {
    /// Storage name
    pub name: String,
    /// Storage type (local, nfs, cifs, etc.)
    pub storage_type: String,
    /// Content types (images, rootdir, iso, etc.)
    pub content: Vec<String>,
    /// Available space in bytes
    pub available: u64,
    /// Total space in bytes
    pub total: u64,
    /// Whether storage is active
    pub active: bool,
}

/// Configuration for creating a Proxmox VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxVmConfig {
    /// Target node name
    pub node: String,
    /// Target storage name
    pub storage: String,
    /// VM ID (e.g., 100)
    pub vm_id: u32,
    /// VM name
    pub name: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u32,
    /// Disk size in GB
    pub disk_size_gb: u32,
    /// Whether to start VM after creation
    pub auto_start: bool,
}

/// Proxmox VM creation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxVmResult {
    /// The created VM ID
    pub vm_id: u32,
    /// Node where VM was created
    pub node: String,
    /// IP address if available
    pub ip_address: Option<String>,
}

// ============================================================================
// UTM Types (macOS)
// ============================================================================

/// Configuration for creating a UTM virtual machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmVmConfig {
    /// VM name
    pub name: String,
    /// Path to the HAOS qcow2 image file
    pub image_path: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u32,
    /// Disk size in GB
    pub disk_size_gb: u32,
    /// Whether to start VM after creation
    pub auto_start: bool,
}

/// UTM VM creation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmVmResult {
    /// The created VM name
    pub name: String,
    /// Path to the VM bundle
    pub path: Option<String>,
}

/// UTM application status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmStatus {
    /// Whether UTM is installed
    pub installed: bool,
    /// UTM version if installed
    pub version: Option<String>,
    /// Path to UTM application
    pub path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // DeviceType enum tests
    #[test]
    fn test_device_type_serializes_snake_case() {
        let device_type = DeviceType::SdCard;
        let json = serde_json::to_string(&device_type).unwrap();
        assert_eq!(json, "\"sd_card\"");

        let device_type = DeviceType::UsbDrive;
        let json = serde_json::to_string(&device_type).unwrap();
        assert_eq!(json, "\"usb_drive\"");

        let device_type = DeviceType::NvMe;
        let json = serde_json::to_string(&device_type).unwrap();
        assert_eq!(json, "\"nv_me\"");
    }

    #[test]
    fn test_device_type_roundtrip() {
        let device_types = vec![
            DeviceType::SdCard,
            DeviceType::UsbDrive,
            DeviceType::Ssd,
            DeviceType::Hdd,
            DeviceType::NvMe,
            DeviceType::Unknown,
        ];

        for device_type in device_types {
            let json = serde_json::to_string(&device_type).unwrap();
            let deserialized: DeviceType = serde_json::from_str(&json).unwrap();
            assert_eq!(device_type, deserialized);
        }
    }

    // FlashStage enum tests
    #[test]
    fn test_flash_stage_serializes_snake_case() {
        let stage = FlashStage::Downloading;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"downloading\"");

        let stage = FlashStage::Extracting;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"extracting\"");

        let stage = FlashStage::Writing;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"writing\"");
    }

    #[test]
    fn test_flash_stage_roundtrip() {
        let stages = vec![
            FlashStage::Downloading,
            FlashStage::Extracting,
            FlashStage::Writing,
            FlashStage::Verifying,
            FlashStage::Finalizing,
            FlashStage::Ready,
            FlashStage::Updating,
            FlashStage::Complete,
            FlashStage::Error,
        ];

        for stage in stages {
            let json = serde_json::to_string(&stage).unwrap();
            let deserialized: FlashStage = serde_json::from_str(&json).unwrap();
            assert_eq!(stage, deserialized);
        }
    }

    // DeviceCategory enum tests
    #[test]
    fn test_device_category_serializes_snake_case() {
        let category = DeviceCategory::RaspberryPi;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"raspberry_pi\"");

        let category = DeviceCategory::HomeAssistantHardware;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"home_assistant_hardware\"");

        let category = DeviceCategory::GenericX86;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"generic_x86\"");

        let category = DeviceCategory::GenericArm64;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"generic_arm64\"");
    }

    #[test]
    fn test_device_category_roundtrip() {
        let categories = vec![
            DeviceCategory::RaspberryPi,
            DeviceCategory::Odroid,
            DeviceCategory::Khadas,
            DeviceCategory::Asus,
            DeviceCategory::HomeAssistantHardware,
            DeviceCategory::GenericX86,
            DeviceCategory::GenericArm64,
        ];

        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: DeviceCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(category, deserialized);
        }
    }

    // BlockDevice struct tests
    #[test]
    fn test_block_device_json_roundtrip() {
        let device = BlockDevice {
            id: "/dev/sda".to_string(),
            name: "SanDisk Ultra".to_string(),
            size: 32000000000,
            device_type: DeviceType::SdCard,
            removable: true,
            model: Some("Ultra 32GB".to_string()),
            vendor: Some("SanDisk".to_string()),
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: BlockDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(device.id, deserialized.id);
        assert_eq!(device.name, deserialized.name);
        assert_eq!(device.size, deserialized.size);
        assert_eq!(device.device_type, deserialized.device_type);
        assert_eq!(device.removable, deserialized.removable);
        assert_eq!(device.model, deserialized.model);
        assert_eq!(device.vendor, deserialized.vendor);
    }

    #[test]
    fn test_block_device_json_roundtrip_with_none() {
        let device = BlockDevice {
            id: "disk2".to_string(),
            name: "Unknown Device".to_string(),
            size: 16000000000,
            device_type: DeviceType::Unknown,
            removable: false,
            model: None,
            vendor: None,
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: BlockDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(device.id, deserialized.id);
        assert_eq!(device.name, deserialized.name);
        assert_eq!(device.size, deserialized.size);
        assert_eq!(device.device_type, deserialized.device_type);
        assert_eq!(device.removable, deserialized.removable);
        assert_eq!(device.model, deserialized.model);
        assert_eq!(device.vendor, deserialized.vendor);
    }

    // FlashProgress struct tests
    #[test]
    fn test_flash_progress_json_roundtrip() {
        let progress = FlashProgress {
            stage: FlashStage::Writing,
            progress: 45,
            bytes_processed: 1500000000,
            total_bytes: 3000000000,
            message: "Writing image to device...".to_string(),
        };

        let json = serde_json::to_string(&progress).unwrap();
        let deserialized: FlashProgress = serde_json::from_str(&json).unwrap();

        assert_eq!(progress.stage, deserialized.stage);
        assert_eq!(progress.progress, deserialized.progress);
        assert_eq!(progress.bytes_processed, deserialized.bytes_processed);
        assert_eq!(progress.total_bytes, deserialized.total_bytes);
        assert_eq!(progress.message, deserialized.message);
    }

    #[test]
    fn test_flash_progress_all_stages() {
        let stages = vec![
            (FlashStage::Downloading, "Downloading image..."),
            (FlashStage::Extracting, "Extracting archive..."),
            (FlashStage::Writing, "Writing to device..."),
            (FlashStage::Verifying, "Verifying write..."),
            (FlashStage::Finalizing, "Finalizing..."),
            (FlashStage::Ready, "Waiting for Home Assistant..."),
            (FlashStage::Updating, "Updating to latest version..."),
            (FlashStage::Complete, "Complete!"),
            (FlashStage::Error, "Error occurred"),
        ];

        for (stage, message) in stages {
            let progress = FlashProgress {
                stage: stage.clone(),
                progress: 50,
                bytes_processed: 1000,
                total_bytes: 2000,
                message: message.to_string(),
            };

            let json = serde_json::to_string(&progress).unwrap();
            let deserialized: FlashProgress = serde_json::from_str(&json).unwrap();
            assert_eq!(progress.stage, deserialized.stage);
        }
    }

    // HaosRelease struct tests
    #[test]
    fn test_haos_release_json_roundtrip() {
        let release = HaosRelease {
            version: "16.3".to_string(),
            images: vec![
                HaosImage {
                    board: "rpi5-64".to_string(),
                    download_url: "https://example.com/haos-rpi5-16.3.img.xz".to_string(),
                    size: 500000000,
                    sha256: "abc123def456".to_string(),
                },
                HaosImage {
                    board: "generic-x86-64".to_string(),
                    download_url: "https://example.com/haos-generic-x86-16.3.img.xz".to_string(),
                    size: 600000000,
                    sha256: "def789abc012".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&release).unwrap();
        let deserialized: HaosRelease = serde_json::from_str(&json).unwrap();

        assert_eq!(release.version, deserialized.version);
        assert_eq!(release.images.len(), deserialized.images.len());

        for (original, deserialized) in release.images.iter().zip(deserialized.images.iter()) {
            assert_eq!(original.board, deserialized.board);
            assert_eq!(original.download_url, deserialized.download_url);
            assert_eq!(original.size, deserialized.size);
            assert_eq!(original.sha256, deserialized.sha256);
        }
    }

    #[test]
    fn test_haos_release_empty_images() {
        let release = HaosRelease {
            version: "16.0".to_string(),
            images: vec![],
        };

        let json = serde_json::to_string(&release).unwrap();
        let deserialized: HaosRelease = serde_json::from_str(&json).unwrap();

        assert_eq!(release.version, deserialized.version);
        assert_eq!(release.images.len(), 0);
        assert_eq!(deserialized.images.len(), 0);
    }

    // FlashRequest struct tests
    #[test]
    fn test_flash_request_json_roundtrip() {
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: "rpi5-64".to_string(),
            verify: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: FlashRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.device_id, deserialized.device_id);
        assert_eq!(request.board, deserialized.board);
        assert_eq!(request.verify, deserialized.verify);
    }

    #[test]
    fn test_flash_request_verify_false() {
        let request = FlashRequest {
            device_id: "disk2".to_string(),
            board: "green".to_string(),
            verify: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: FlashRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.device_id, deserialized.device_id);
        assert_eq!(request.board, deserialized.board);
        assert!(!request.verify);
        assert!(!deserialized.verify);
    }

    // Additional comprehensive tests
    #[test]
    fn test_flash_result_roundtrip() {
        let result = FlashResult {
            success: true,
            error: None,
            duration_secs: 120,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: FlashResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.error, deserialized.error);
        assert_eq!(result.duration_secs, deserialized.duration_secs);
    }

    #[test]
    fn test_flash_result_with_error() {
        let result = FlashResult {
            success: false,
            error: Some("Device not found".to_string()),
            duration_secs: 5,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: FlashResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.error, deserialized.error);
        assert_eq!(result.duration_secs, deserialized.duration_secs);
    }

    #[test]
    fn test_update_info_roundtrip() {
        let update_info = UpdateInfo {
            update_available: true,
            current_version: "1.0.0".to_string(),
            latest_version: "1.1.0".to_string(),
            download_url: Some("https://example.com/download".to_string()),
            release_notes_url: Some("https://example.com/notes".to_string()),
            is_beta: false,
        };

        let json = serde_json::to_string(&update_info).unwrap();
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(update_info.update_available, deserialized.update_available);
        assert_eq!(update_info.current_version, deserialized.current_version);
        assert_eq!(update_info.latest_version, deserialized.latest_version);
        assert_eq!(update_info.download_url, deserialized.download_url);
        assert_eq!(
            update_info.release_notes_url,
            deserialized.release_notes_url
        );
        assert_eq!(update_info.is_beta, deserialized.is_beta);
    }

    #[test]
    fn test_device_manifest_roundtrip() {
        let manifest = DeviceManifest {
            version: 1,
            devices: vec![Device {
                id: "rpi5".to_string(),
                name: "Raspberry Pi 5".to_string(),
                category: DeviceCategory::RaspberryPi,
                image_url: Some("https://example.com/rpi5.png".to_string()),
                haos: HaosConfig {
                    board: "rpi5-64".to_string(),
                    download_url: "https://example.com/haos-{version}-rpi5.img.xz".to_string(),
                },
            }],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: DeviceManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.version, deserialized.version);
        assert_eq!(manifest.devices.len(), deserialized.devices.len());
        assert_eq!(manifest.devices[0].id, deserialized.devices[0].id);
        assert_eq!(manifest.devices[0].name, deserialized.devices[0].name);
        assert_eq!(
            manifest.devices[0].category,
            deserialized.devices[0].category
        );
    }

    // UTM types tests
    #[test]
    fn test_utm_vm_config_roundtrip() {
        let config = UtmVmConfig {
            name: "Home Assistant".to_string(),
            image_path: "/path/to/image.qcow2".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: UtmVmConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.cpu_cores, deserialized.cpu_cores);
        assert_eq!(config.memory_mb, deserialized.memory_mb);
        assert_eq!(config.disk_size_gb, deserialized.disk_size_gb);
        assert_eq!(config.auto_start, deserialized.auto_start);
    }

    #[test]
    fn test_utm_status_roundtrip() {
        let status = UtmStatus {
            installed: true,
            version: Some("4.0.0".to_string()),
            path: Some("/Applications/UTM.app".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: UtmStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.installed, deserialized.installed);
        assert_eq!(status.version, deserialized.version);
        assert_eq!(status.path, deserialized.path);
    }

    // ============================================================================
    // Edge case tests
    // ============================================================================

    // FlashProgress edge cases
    #[test]
    fn test_flash_progress_zero_values() {
        let progress = FlashProgress {
            stage: FlashStage::Downloading,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        let parsed: FlashProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.progress, 0);
        assert_eq!(parsed.bytes_processed, 0);
        assert_eq!(parsed.total_bytes, 0);
        assert_eq!(parsed.message, "");
    }

    #[test]
    fn test_flash_progress_max_values() {
        let progress = FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: u64::MAX,
            total_bytes: u64::MAX,
            message: "Done".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        let parsed: FlashProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.progress, 100);
        assert_eq!(parsed.bytes_processed, u64::MAX);
        assert_eq!(parsed.total_bytes, u64::MAX);
    }

    // BlockDevice edge cases
    #[test]
    fn test_block_device_all_optional_none() {
        let device = BlockDevice {
            id: "/dev/sdb".to_string(),
            name: "Unknown".to_string(),
            size: 1_000_000_000,
            device_type: DeviceType::Unknown,
            removable: true,
            model: None,
            vendor: None,
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"model\":null"));
        assert!(json.contains("\"vendor\":null"));

        let parsed: BlockDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "/dev/sdb");
        assert_eq!(parsed.model, None);
        assert_eq!(parsed.vendor, None);
    }

    // HaosImage edge cases
    #[test]
    fn test_haos_image_empty_sha256() {
        let image = HaosImage {
            board: "rpi5-64".to_string(),
            download_url: "https://example.com/image.xz".to_string(),
            size: 500_000_000,
            sha256: "".to_string(),
        };
        let json = serde_json::to_string(&image).unwrap();
        let parsed: HaosImage = serde_json::from_str(&json).unwrap();
        assert!(parsed.sha256.is_empty());
        assert_eq!(parsed.board, "rpi5-64");
        assert_eq!(parsed.size, 500_000_000);
    }

    // Device edge cases
    #[test]
    fn test_device_full_structure() {
        let device = Device {
            id: "rpi5".to_string(),
            name: "Raspberry Pi 5".to_string(),
            category: DeviceCategory::RaspberryPi,
            image_url: Some("/assets/rpi5.png".to_string()),
            haos: HaosConfig {
                board: "rpi5-64".to_string(),
                download_url: "https://github.com/.../haos_rpi5-64-{version}.img.xz".to_string(),
            },
        };
        let json = serde_json::to_string(&device).unwrap();
        let parsed: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "rpi5");
        assert_eq!(parsed.category, DeviceCategory::RaspberryPi);
        assert_eq!(parsed.name, "Raspberry Pi 5");
        assert_eq!(parsed.haos.board, "rpi5-64");
    }

    // ProxmoxVmConfig and ProxmoxVmResult edge cases
    #[test]
    fn test_proxmox_vm_config_roundtrip() {
        let config = ProxmoxVmConfig {
            vm_id: 100,
            name: "homeassistant".to_string(),
            node: "pve".to_string(),
            storage: "local-lvm".to_string(),
            cpu_cores: 4,
            memory_mb: 4096,
            disk_size_gb: 32,
            auto_start: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProxmoxVmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.vm_id, 100);
        assert_eq!(parsed.memory_mb, 4096);
        assert_eq!(parsed.cpu_cores, 4);
        assert_eq!(parsed.disk_size_gb, 32);
        assert!(parsed.auto_start);
    }

    #[test]
    fn test_proxmox_vm_result_with_ip() {
        let result = ProxmoxVmResult {
            vm_id: 100,
            node: "pve".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("192.168.1.100"));

        let parsed: ProxmoxVmResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.vm_id, 100);
        assert_eq!(parsed.ip_address, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_proxmox_vm_result_without_ip() {
        let result = ProxmoxVmResult {
            vm_id: 100,
            node: "pve".to_string(),
            ip_address: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ProxmoxVmResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.ip_address.is_none());
        assert_eq!(parsed.vm_id, 100);
        assert_eq!(parsed.node, "pve");
    }

    // UtmVmConfig and UtmVmResult edge cases
    #[test]
    fn test_utm_vm_config_roundtrip_full() {
        let config = UtmVmConfig {
            name: "Home Assistant".to_string(),
            image_path: "/path/to/image.qcow2".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: UtmVmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.disk_size_gb, 32);
        assert_eq!(parsed.name, "Home Assistant");
        assert_eq!(parsed.cpu_cores, 2);
        assert_eq!(parsed.memory_mb, 2048);
        assert!(!parsed.auto_start);
    }

    #[test]
    fn test_utm_vm_result_roundtrip_full() {
        let result = UtmVmResult {
            name: "Home Assistant".to_string(),
            path: Some("/Users/test/VMs/HA.utm".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: UtmVmResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Home Assistant");
        assert_eq!(parsed.path, Some("/Users/test/VMs/HA.utm".to_string()));
    }

    #[test]
    fn test_utm_vm_result_without_path() {
        let result = UtmVmResult {
            name: "Home Assistant".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: UtmVmResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Home Assistant");
        assert!(parsed.path.is_none());
    }
}
