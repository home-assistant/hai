//! Mock data for testing and development
//!
//! This module provides mock implementations of devices, manifests, and
//! other data for testing the application without real hardware.
//!
//! Mock mode is enabled when the `HA_INSTALLER_MOCK` environment variable
//! is set to "1" or "true".

use crate::types::{
    BlockDevice, Device, DeviceCategory, DeviceManifest, DeviceType, HaosConfig, HaosImage,
    HaosRelease, StableVersionInfo, UpdateInfo,
};
use std::collections::HashMap;

/// Returns mock block devices for testing
pub fn get_mock_block_devices() -> Vec<BlockDevice> {
    vec![
        BlockDevice {
            id: "mock-sd-card-32gb".to_string(),
            name: "SD Card 32GB".to_string(),
            size: 32 * 1024 * 1024 * 1024, // 32 GB
            device_type: DeviceType::SdCard,
            removable: true,
            model: Some("SanDisk Ultra".to_string()),
            vendor: Some("SanDisk".to_string()),
        },
        BlockDevice {
            id: "mock-sd-card-64gb".to_string(),
            name: "SD Card 64GB".to_string(),
            size: 64 * 1024 * 1024 * 1024, // 64 GB
            device_type: DeviceType::SdCard,
            removable: true,
            model: Some("Samsung EVO Plus".to_string()),
            vendor: Some("Samsung".to_string()),
        },
        BlockDevice {
            id: "mock-usb-drive-128gb".to_string(),
            name: "USB Drive 128GB".to_string(),
            size: 128 * 1024 * 1024 * 1024, // 128 GB
            device_type: DeviceType::UsbDrive,
            removable: true,
            model: Some("USB Flash Drive".to_string()),
            vendor: Some("Kingston".to_string()),
        },
        BlockDevice {
            id: "mock-ssd-256gb".to_string(),
            name: "External SSD 256GB".to_string(),
            size: 256 * 1024 * 1024 * 1024, // 256 GB
            device_type: DeviceType::Ssd,
            removable: true,
            model: Some("Portable SSD T7".to_string()),
            vendor: Some("Samsung".to_string()),
        },
        BlockDevice {
            id: "mock-nvme-500gb".to_string(),
            name: "NVMe Drive 500GB".to_string(),
            size: 500 * 1024 * 1024 * 1024, // 500 GB
            device_type: DeviceType::NvMe,
            removable: false,
            model: Some("970 EVO Plus".to_string()),
            vendor: Some("Samsung".to_string()),
        },
    ]
}

/// Returns mock device manifest for testing
pub fn get_mock_manifest() -> DeviceManifest {
    DeviceManifest {
        version: 1,
        devices: vec![
            // Raspberry Pi devices
            Device {
                id: "rpi5".to_string(),
                name: "Raspberry Pi 5".to_string(),
                category: DeviceCategory::RaspberryPi,
                image_url: Some("/assets/devices/raspberry_pi_5.png".to_string()),
                haos: HaosConfig {
                    board: "rpi5-64".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi5-64-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "rpi4".to_string(),
                name: "Raspberry Pi 4".to_string(),
                category: DeviceCategory::RaspberryPi,
                image_url: Some("/assets/devices/raspberry_pi_4.png".to_string()),
                haos: HaosConfig {
                    board: "rpi4-64".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi4-64-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "rpi3".to_string(),
                name: "Raspberry Pi 3".to_string(),
                category: DeviceCategory::RaspberryPi,
                image_url: Some("/assets/devices/raspberry_pi_3.png".to_string()),
                haos: HaosConfig {
                    board: "rpi3-64".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi3-64-{version}.img.xz".to_string(),
                },
            },
            // ODROID devices
            Device {
                id: "odroid-n2".to_string(),
                name: "ODROID-N2/N2+".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-n2.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-n2".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-n2-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "odroid-c2".to_string(),
                name: "ODROID-C2".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-c2.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-c2".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-c2-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "odroid-c4".to_string(),
                name: "ODROID-C4".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-c4.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-c4".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-c4-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "odroid-m1".to_string(),
                name: "ODROID-M1".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-m1.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-m1".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-m1-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "odroid-m1s".to_string(),
                name: "ODROID-M1S".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-m1s.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-m1s".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-m1s-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "odroid-xu4".to_string(),
                name: "ODROID-XU4".to_string(),
                category: DeviceCategory::Odroid,
                image_url: Some("/assets/devices/hardkernel_odroid-xu4.png".to_string()),
                haos: HaosConfig {
                    board: "odroid-xu".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-xu-{version}.img.xz".to_string(),
                },
            },
            // Khadas devices
            Device {
                id: "khadas-vim3".to_string(),
                name: "Khadas VIM3".to_string(),
                category: DeviceCategory::Khadas,
                image_url: Some("/assets/devices/khadas_vim3.png".to_string()),
                haos: HaosConfig {
                    board: "khadas-vim3".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_khadas-vim3-{version}.img.xz".to_string(),
                },
            },
            // ASUS devices
            Device {
                id: "asus-tinker".to_string(),
                name: "ASUS Tinker Board".to_string(),
                category: DeviceCategory::Asus,
                image_url: Some("/assets/devices/asus_tinker.png".to_string()),
                haos: HaosConfig {
                    board: "tinker".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_tinker-{version}.img.xz".to_string(),
                },
            },
            // Home Assistant Hardware
            Device {
                id: "ha-green".to_string(),
                name: "Home Assistant Green".to_string(),
                category: DeviceCategory::HomeAssistantHardware,
                image_url: Some("/assets/devices/homeassistant_green.png".to_string()),
                haos: HaosConfig {
                    board: "green".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_green-{version}.img.xz".to_string(),
                },
            },
            Device {
                id: "ha-yellow".to_string(),
                name: "Home Assistant Yellow".to_string(),
                category: DeviceCategory::HomeAssistantHardware,
                image_url: Some("/assets/devices/homeassistant_yellow.png".to_string()),
                haos: HaosConfig {
                    board: "yellow".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_yellow-{version}.img.xz".to_string(),
                },
            },
            // Generic x86-64
            Device {
                id: "generic-x86-64".to_string(),
                name: "Intel/AMD (x86-64)".to_string(),
                category: DeviceCategory::GenericX86,
                image_url: Some("/assets/icons/cpu-64-bit.svg".to_string()),
                haos: HaosConfig {
                    board: "generic-x86-64".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_generic-x86-64-{version}.img.xz".to_string(),
                },
            },
            // Generic ARM64
            Device {
                id: "generic-aarch64".to_string(),
                name: "ARM (aarch64)".to_string(),
                category: DeviceCategory::GenericArm64,
                image_url: Some("/assets/icons/chip.svg".to_string()),
                haos: HaosConfig {
                    board: "generic-aarch64".to_string(),
                    download_url: "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_generic-aarch64-{version}.img.xz".to_string(),
                },
            },
        ],
    }
}

/// Returns mock update info for testing
pub fn get_mock_update_info() -> UpdateInfo {
    UpdateInfo {
        update_available: false,
        current_version: "0.1.0".to_string(),
        latest_version: "0.1.0".to_string(),
        download_url: Some(
            "https://github.com/home-assistant/home-assistant-installer/releases".to_string(),
        ),
        release_notes_url: Some(
            "https://github.com/home-assistant/home-assistant-installer/releases".to_string(),
        ),
        is_beta: false,
    }
}

/// Returns mock stable version info (simulating version.home-assistant.io/stable.json)
pub fn get_mock_stable_version() -> StableVersionInfo {
    let mut hassos = HashMap::new();
    // All boards have the same version in stable releases
    let version = "16.3".to_string();
    hassos.insert("rpi5-64".to_string(), version.clone());
    hassos.insert("rpi4-64".to_string(), version.clone());
    hassos.insert("rpi4".to_string(), version.clone());
    hassos.insert("rpi3-64".to_string(), version.clone());
    hassos.insert("rpi3".to_string(), version.clone());
    hassos.insert("rpi2".to_string(), version.clone());
    hassos.insert("odroid-n2".to_string(), version.clone());
    hassos.insert("odroid-c2".to_string(), version.clone());
    hassos.insert("odroid-c4".to_string(), version.clone());
    hassos.insert("odroid-m1".to_string(), version.clone());
    hassos.insert("odroid-m1s".to_string(), version.clone());
    hassos.insert("odroid-xu4".to_string(), version.clone());
    hassos.insert("khadas-vim3".to_string(), version.clone());
    hassos.insert("tinker".to_string(), version.clone());
    hassos.insert("green".to_string(), version.clone());
    hassos.insert("yellow".to_string(), version.clone());
    hassos.insert("generic-x86-64".to_string(), version.clone());
    hassos.insert("generic-aarch64".to_string(), version);

    StableVersionInfo { hassos }
}

/// Returns mock HAOS release info based on real 16.3 release data
pub fn get_mock_haos_release() -> HaosRelease {
    HaosRelease {
        version: "16.3".to_string(),
        images: vec![
            HaosImage {
                board: "rpi5-64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi5-64-16.3.img.xz".to_string(),
                size: 331_899_792,
                sha256: "5ade653232aa1c4504e52b56347b389fb0b24d9edc69134a860edb84f41ea9e9".to_string(),
            },
            HaosImage {
                board: "rpi4-64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi4-64-16.3.img.xz".to_string(),
                size: 322_239_272,
                sha256: "3ebed523708dc1dad5b5399707ee74d0a54b9604b7d4cae5d591d75c85b35013".to_string(),
            },
            HaosImage {
                board: "rpi3-64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi3-64-16.3.img.xz".to_string(),
                size: 311_438_560,
                sha256: "f21d5da83a94a5045d4d36822da77d2bee3539ab5150a7074c562d922f81e0de".to_string(),
            },
            HaosImage {
                board: "odroid-n2".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-n2-16.3.img.xz".to_string(),
                size: 298_412_092,
                sha256: "f97b188d9fd2c239269c886e53031ad8bc38828296f1eaede2e89fd4b89207b7".to_string(),
            },
            HaosImage {
                board: "green".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_green-16.3.img.xz".to_string(),
                size: 336_860_104,
                sha256: "fd41fb3432fb5d64d916b04f6ab18c39824b128fd996d55ea207e393fc65c943".to_string(),
            },
            HaosImage {
                board: "yellow".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_yellow-16.3.img.xz".to_string(),
                size: 322_261_788,
                sha256: "145f252403a00a50391ed4074242e5b770c59477b66f2a2ea33927f68bef0e98".to_string(),
            },
            HaosImage {
                board: "generic-x86-64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_generic-x86-64-16.3.img.xz".to_string(),
                size: 396_451_208,
                sha256: "afe591a859a068eb25dcef15be9e7b2236f9c06f515cac3706681db900cb02df".to_string(),
            },
            HaosImage {
                board: "generic-aarch64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_generic-aarch64-16.3.img.xz".to_string(),
                size: 341_537_340,
                sha256: "4769532f71886f8b41c4520b3c0c8f974f5bbf583782a2dc7b16a8e2743315ed".to_string(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_mock_mode_disabled_by_default() {
        // Clear the env var if set
        std::env::remove_var("HA_INSTALLER_MOCK");
        assert!(!crate::is_mock_enabled());
    }

    #[test]
    fn test_mock_block_devices_not_empty() {
        let devices = get_mock_block_devices();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_mock_manifest_has_devices() {
        let manifest = get_mock_manifest();
        assert!(!manifest.devices.is_empty());
    }

    #[test]
    fn test_mock_block_devices_have_valid_sizes() {
        let devices = get_mock_block_devices();
        for device in devices {
            assert!(device.size > 0);
        }
    }

    #[test]
    #[serial]
    fn test_mock_mode_enabled_with_set() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        assert!(crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    fn test_mock_block_devices_have_unique_ids() {
        let devices = get_mock_block_devices();
        let mut ids = std::collections::HashSet::new();
        for device in &devices {
            assert!(
                ids.insert(device.id.clone()),
                "Duplicate device ID found: {}",
                device.id
            );
        }
    }

    #[test]
    fn test_mock_manifest_has_unique_device_ids() {
        let manifest = get_mock_manifest();
        let mut ids = std::collections::HashSet::new();
        for device in &manifest.devices {
            assert!(
                ids.insert(device.id.clone()),
                "Duplicate device ID found: {}",
                device.id
            );
        }
    }

    #[test]
    fn test_mock_haos_release_has_images() {
        let release = get_mock_haos_release();
        assert!(
            !release.images.is_empty(),
            "HAOS release should have at least one image"
        );
    }

    #[test]
    fn test_mock_haos_release_images_have_valid_checksums() {
        let release = get_mock_haos_release();
        for image in &release.images {
            assert_eq!(
                image.sha256.len(),
                64,
                "SHA256 checksum for board {} should be 64 characters",
                image.board
            );
            assert!(
                image.sha256.chars().all(|c| c.is_ascii_hexdigit()),
                "SHA256 checksum for board {} should only contain hex digits",
                image.board
            );
        }
    }

    #[test]
    fn test_mock_update_info_versions_valid() {
        let update_info = get_mock_update_info();
        assert!(
            !update_info.current_version.is_empty(),
            "Current version should not be empty"
        );
        assert!(
            !update_info.latest_version.is_empty(),
            "Latest version should not be empty"
        );
    }

    #[test]
    fn test_mock_update_info_has_urls() {
        let update_info = get_mock_update_info();
        assert!(
            update_info.download_url.is_some(),
            "Download URL should be present"
        );
        assert!(
            update_info.release_notes_url.is_some(),
            "Release notes URL should be present"
        );
    }

    #[test]
    fn test_mock_stable_version_has_all_boards() {
        let stable = get_mock_stable_version();
        // Test that all major board types are present
        assert!(stable.hassos.contains_key("rpi5-64"));
        assert!(stable.hassos.contains_key("rpi4-64"));
        assert!(stable.hassos.contains_key("rpi3-64"));
        assert!(stable.hassos.contains_key("odroid-n2"));
        assert!(stable.hassos.contains_key("green"));
        assert!(stable.hassos.contains_key("yellow"));
        assert!(stable.hassos.contains_key("generic-x86-64"));
        assert!(stable.hassos.contains_key("generic-aarch64"));
    }

    #[test]
    fn test_mock_stable_version_consistent_version() {
        let stable = get_mock_stable_version();
        // All boards should have the same version
        let first_version = stable.hassos.values().next().unwrap();
        for (board, version) in &stable.hassos {
            assert_eq!(
                version, first_version,
                "Board {} has inconsistent version",
                board
            );
        }
    }

    #[test]
    fn test_mock_haos_release_version_matches() {
        let release = get_mock_haos_release();
        assert_eq!(release.version, "16.3");
    }

    #[test]
    fn test_mock_haos_release_has_download_urls() {
        let release = get_mock_haos_release();
        for image in &release.images {
            assert!(
                image.download_url.starts_with("https://"),
                "Image for {} should have HTTPS URL",
                image.board
            );
            assert!(
                image.download_url.contains(&release.version),
                "Image URL for {} should contain version",
                image.board
            );
        }
    }

    #[test]
    fn test_mock_haos_release_images_have_valid_sizes() {
        let release = get_mock_haos_release();
        for image in &release.images {
            assert!(
                image.size > 0,
                "Image size for {} should be positive",
                image.board
            );
            assert!(
                image.size > 100_000_000,
                "Image size for {} should be at least 100MB",
                image.board
            );
        }
    }

    #[test]
    fn test_mock_block_devices_device_types() {
        let devices = get_mock_block_devices();
        let has_sd_card = devices.iter().any(|d| d.device_type == DeviceType::SdCard);
        let has_usb = devices.iter().any(|d| d.device_type == DeviceType::UsbDrive);
        let has_ssd = devices.iter().any(|d| d.device_type == DeviceType::Ssd);
        let has_nvme = devices.iter().any(|d| d.device_type == DeviceType::NvMe);

        assert!(has_sd_card, "Should have at least one SD card");
        assert!(has_usb, "Should have at least one USB drive");
        assert!(has_ssd, "Should have at least one SSD");
        assert!(has_nvme, "Should have at least one NVMe");
    }

    #[test]
    fn test_mock_block_devices_have_vendor_and_model() {
        let devices = get_mock_block_devices();
        for device in &devices {
            assert!(device.vendor.is_some(), "Device {} should have vendor", device.id);
            assert!(device.model.is_some(), "Device {} should have model", device.id);
        }
    }

    #[test]
    fn test_mock_manifest_version() {
        let manifest = get_mock_manifest();
        assert_eq!(manifest.version, 1);
    }

    #[test]
    fn test_mock_manifest_has_all_categories() {
        let manifest = get_mock_manifest();
        let categories: std::collections::HashSet<_> =
            manifest.devices.iter().map(|d| &d.category).collect();

        assert!(
            categories.contains(&DeviceCategory::RaspberryPi),
            "Should have Raspberry Pi devices"
        );
        assert!(
            categories.contains(&DeviceCategory::Odroid),
            "Should have ODROID devices"
        );
        assert!(
            categories.contains(&DeviceCategory::HomeAssistantHardware),
            "Should have Home Assistant hardware"
        );
        assert!(
            categories.contains(&DeviceCategory::GenericX86),
            "Should have generic x86"
        );
        assert!(
            categories.contains(&DeviceCategory::GenericArm64),
            "Should have generic ARM64"
        );
    }

    #[test]
    fn test_mock_manifest_devices_have_haos_config() {
        let manifest = get_mock_manifest();
        for device in &manifest.devices {
            assert!(
                !device.haos.board.is_empty(),
                "Device {} should have HAOS board",
                device.id
            );
            assert!(
                !device.haos.download_url.is_empty(),
                "Device {} should have HAOS download URL",
                device.id
            );
            assert!(
                device.haos.download_url.contains("{version}"),
                "Device {} download URL should have version placeholder",
                device.id
            );
        }
    }

    #[test]
    fn test_mock_manifest_devices_have_image_urls() {
        let manifest = get_mock_manifest();
        for device in &manifest.devices {
            assert!(
                device.image_url.is_some(),
                "Device {} should have image URL",
                device.id
            );
        }
    }

    #[test]
    #[serial]
    fn test_mock_mode_enabled_with_true_string() {
        std::env::set_var("HA_INSTALLER_MOCK", "true");
        assert!(crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_enabled_with_true_uppercase() {
        std::env::set_var("HA_INSTALLER_MOCK", "TRUE");
        assert!(crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_disabled_with_zero() {
        std::env::set_var("HA_INSTALLER_MOCK", "0");
        assert!(!crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_disabled_with_false() {
        std::env::set_var("HA_INSTALLER_MOCK", "false");
        assert!(!crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_disabled_with_random_string() {
        std::env::set_var("HA_INSTALLER_MOCK", "random");
        assert!(!crate::is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }
}
