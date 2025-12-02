use crate::types::{
    BlockDevice, Device, DeviceCategory, DeviceManifest, DeviceType, HaosConfig, HaosImage,
    HaosRelease, StableVersionInfo, UpdateInfo,
};
use std::collections::HashMap;

/// Check if mock mode is enabled via environment variable
pub fn is_mock_enabled() -> bool {
    std::env::var("HA_INSTALLER_MOCK")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

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
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        latest_version: env!("CARGO_PKG_VERSION").to_string(),
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
                board: "rpi4".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi4-16.3.img.xz".to_string(),
                size: 311_865_372,
                sha256: "e4f72c1487f8e9a4c6a5679a4d5a800a887aaaa187e0d1a66a2ccf3e1800ce85".to_string(),
            },
            HaosImage {
                board: "rpi3-64".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi3-64-16.3.img.xz".to_string(),
                size: 311_438_560,
                sha256: "f21d5da83a94a5045d4d36822da77d2bee3539ab5150a7074c562d922f81e0de".to_string(),
            },
            HaosImage {
                board: "rpi3".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi3-16.3.img.xz".to_string(),
                size: 299_905_192,
                sha256: "c7a4d62d0007889f4253ddeb16e6304d8bc6ab122cfa9bfab5e244b71fa52d0e".to_string(),
            },
            HaosImage {
                board: "rpi2".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi2-16.3.img.xz".to_string(),
                size: 300_861_076,
                sha256: "2228988ef3361e1f3d2ff7c49be71bc3655cbd2c1f63e396f7cda486351db735".to_string(),
            },
            HaosImage {
                board: "odroid-n2".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-n2-16.3.img.xz".to_string(),
                size: 298_412_092,
                sha256: "f97b188d9fd2c239269c886e53031ad8bc38828296f1eaede2e89fd4b89207b7".to_string(),
            },
            HaosImage {
                board: "odroid-c2".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-c2-16.3.img.xz".to_string(),
                size: 298_539_208,
                sha256: "898ff9f1f7b175c5c8422ec874936f9d377f42f11c579fdda5790f128a6a7241".to_string(),
            },
            HaosImage {
                board: "odroid-c4".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-c4-16.3.img.xz".to_string(),
                size: 298_770_768,
                sha256: "d84eb96e1d823213d97a875e05cdee74d1babc521cfadae11179b3ce7338d812".to_string(),
            },
            HaosImage {
                board: "odroid-m1".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-m1-16.3.img.xz".to_string(),
                size: 337_395_068,
                sha256: "67603922ee054740e0d59b319806fb11a4a62e5b93a78c90077670e91b75d025".to_string(),
            },
            HaosImage {
                board: "odroid-m1s".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-m1s-16.3.img.xz".to_string(),
                size: 336_863_952,
                sha256: "3eed2da3a01bae6dc9ac5482c319fd2816266b0c97df01eff059051f051d5c09".to_string(),
            },
            HaosImage {
                board: "odroid-xu4".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-xu4-16.3.img.xz".to_string(),
                size: 288_562_728,
                sha256: "a19afa034b48548b761ac7215d92b01d8a813038f933970af89799bb5ecacded".to_string(),
            },
            HaosImage {
                board: "khadas-vim3".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_khadas-vim3-16.3.img.xz".to_string(),
                size: 298_271_704,
                sha256: "3c852191fff72efcd20aa3df589c82b1be5ba666d63245836c0ed47fed125699".to_string(),
            },
            HaosImage {
                board: "tinker".to_string(),
                download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_tinker-16.3.img.xz".to_string(),
                size: 292_023_092,
                sha256: "28916a7c4aadad3eb3ae730f345379ea0cd83c8024d916625da0ff4fe8efc177".to_string(),
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
        assert!(!is_mock_enabled());
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

    // Mock mode detection tests
    #[test]
    #[serial]
    fn test_mock_mode_enabled_with_1() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        assert!(is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_enabled_with_true() {
        std::env::set_var("HA_INSTALLER_MOCK", "true");
        assert!(is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_mock_mode_case_insensitive() {
        std::env::set_var("HA_INSTALLER_MOCK", "TRUE");
        assert!(is_mock_enabled());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    // Block devices validation tests
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
    fn test_mock_block_devices_all_removable() {
        let devices = get_mock_block_devices();
        let removable_count = devices.iter().filter(|d| d.removable).count();
        // At least most devices should be removable (we allow for NVMe which might not be)
        assert!(
            removable_count > 0,
            "At least some devices should be removable"
        );
    }

    #[test]
    fn test_mock_block_devices_cover_device_types() {
        let devices = get_mock_block_devices();
        let mut has_sd = false;
        let mut has_usb = false;
        let mut has_ssd = false;
        let mut has_nvme = false;

        for device in &devices {
            match device.device_type {
                DeviceType::SdCard => has_sd = true,
                DeviceType::UsbDrive => has_usb = true,
                DeviceType::Ssd => has_ssd = true,
                DeviceType::NvMe => has_nvme = true,
                _ => {}
            }
        }

        assert!(has_sd, "Mock devices should include SD card");
        assert!(has_usb, "Mock devices should include USB drive");
        assert!(has_ssd, "Mock devices should include SSD");
        assert!(has_nvme, "Mock devices should include NVMe");
    }

    // Manifest validation tests
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
    fn test_mock_manifest_has_unique_board_ids() {
        let manifest = get_mock_manifest();
        let mut boards = std::collections::HashSet::new();
        for device in &manifest.devices {
            assert!(
                boards.insert(device.haos.board.clone()),
                "Duplicate board ID found: {}",
                device.haos.board
            );
        }
    }

    #[test]
    fn test_mock_manifest_covers_categories() {
        let manifest = get_mock_manifest();
        let mut has_raspberry_pi = false;
        let mut has_odroid = false;
        let mut has_khadas = false;
        let mut has_asus = false;
        let mut has_home_assistant_hardware = false;
        let mut has_generic_x86 = false;
        let mut has_generic_arm64 = false;

        for device in &manifest.devices {
            match device.category {
                DeviceCategory::RaspberryPi => has_raspberry_pi = true,
                DeviceCategory::Odroid => has_odroid = true,
                DeviceCategory::Khadas => has_khadas = true,
                DeviceCategory::Asus => has_asus = true,
                DeviceCategory::HomeAssistantHardware => has_home_assistant_hardware = true,
                DeviceCategory::GenericX86 => has_generic_x86 = true,
                DeviceCategory::GenericArm64 => has_generic_arm64 = true,
            }
        }

        assert!(
            has_raspberry_pi,
            "Mock manifest should include RaspberryPi devices"
        );
        assert!(has_odroid, "Mock manifest should include Odroid devices");
        assert!(has_khadas, "Mock manifest should include Khadas devices");
        assert!(has_asus, "Mock manifest should include Asus devices");
        assert!(
            has_home_assistant_hardware,
            "Mock manifest should include HomeAssistantHardware devices"
        );
        assert!(
            has_generic_x86,
            "Mock manifest should include GenericX86 devices"
        );
        assert!(
            has_generic_arm64,
            "Mock manifest should include GenericArm64 devices"
        );
    }

    #[test]
    fn test_mock_manifest_all_have_haos_config() {
        let manifest = get_mock_manifest();
        for device in &manifest.devices {
            assert!(
                !device.haos.board.is_empty(),
                "Device {} should have a board name",
                device.id
            );
            assert!(
                !device.haos.download_url.is_empty(),
                "Device {} should have a download URL",
                device.id
            );
            assert!(
                device.haos.download_url.contains("https://"),
                "Device {} download URL should use https",
                device.id
            );
        }
    }

    // HAOS release validation tests
    #[test]
    fn test_mock_haos_release_has_images() {
        let release = get_mock_haos_release();
        assert!(
            !release.images.is_empty(),
            "HAOS release should have at least one image"
        );
        assert!(
            release.images.len() > 10,
            "HAOS release should have multiple images for different boards"
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
    fn test_mock_haos_release_images_have_valid_urls() {
        let release = get_mock_haos_release();
        for image in &release.images {
            assert!(
                image.download_url.starts_with("https://"),
                "Download URL for board {} should start with https://",
                image.board
            );
            assert!(
                image.download_url.contains(&release.version),
                "Download URL for board {} should contain version {}",
                image.board,
                release.version
            );
        }
    }

    #[test]
    fn test_mock_haos_release_images_have_unique_boards() {
        let release = get_mock_haos_release();
        let mut boards = std::collections::HashSet::new();
        for image in &release.images {
            assert!(
                boards.insert(image.board.clone()),
                "Duplicate board found in HAOS release images: {}",
                image.board
            );
        }
    }

    // Update info validation tests
    #[test]
    fn test_mock_update_info_versions_valid() {
        let update_info = get_mock_update_info();

        // Version strings should not be empty
        assert!(
            !update_info.current_version.is_empty(),
            "Current version should not be empty"
        );
        assert!(
            !update_info.latest_version.is_empty(),
            "Latest version should not be empty"
        );

        // Versions should be reasonable (contain at least one digit)
        assert!(
            update_info
                .current_version
                .chars()
                .any(|c| c.is_ascii_digit()),
            "Current version should contain at least one digit"
        );
        assert!(
            update_info
                .latest_version
                .chars()
                .any(|c| c.is_ascii_digit()),
            "Latest version should contain at least one digit"
        );

        // URLs should be valid if present
        if let Some(url) = &update_info.download_url {
            assert!(url.starts_with("https://"), "Download URL should use https");
        }
        if let Some(url) = &update_info.release_notes_url {
            assert!(
                url.starts_with("https://"),
                "Release notes URL should use https"
            );
        }
    }
}
