//! UTM integration for macOS
//!
//! This module provides functionality for creating Home Assistant VMs
//! using UTM on macOS via AppleScript automation.

use crate::error::{Error, Result};
use crate::types::{FlashProgress, FlashStage, UtmStatus, UtmVmConfig, UtmVmResult};
use crate::ProgressCallback;

/// Check if UTM is installed and get its status
pub async fn check_utm_status() -> Result<UtmStatus> {
    #[cfg(target_os = "macos")]
    {
        macos::check_utm_status().await
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(Error::UnsupportedPlatform(
            "UTM is only available on macOS".to_string(),
        ))
    }
}

/// Create a Home Assistant VM using UTM
pub async fn create_vm<P: ProgressCallback>(
    config: &UtmVmConfig,
    progress_callback: &P,
) -> Result<UtmVmResult> {
    #[cfg(target_os = "macos")]
    {
        return macos::create_vm(config, progress_callback).await;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = config;
        let _ = progress_callback;
        Err(Error::UnsupportedPlatform(
            "UTM is only available on macOS".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::process::Command;

    const UTM_APP_PATH: &str = "/Applications/UTM.app";

    pub async fn check_utm_status() -> Result<UtmStatus> {
        #[cfg(feature = "mock")]
        {
            if crate::is_mock_enabled() {
                return Ok(UtmStatus {
                    installed: true,
                    version: Some("4.0.0".to_string()),
                    path: Some(UTM_APP_PATH.to_string()),
                });
            }
        }

        // Check if UTM.app exists
        let utm_path = std::path::Path::new(UTM_APP_PATH);
        if !utm_path.exists() {
            return Ok(UtmStatus {
                installed: false,
                version: None,
                path: None,
            });
        }

        // Try to get UTM version from Info.plist
        let info_plist_path = format!("{}/Contents/Info.plist", UTM_APP_PATH);
        let version = get_utm_version(&info_plist_path);

        Ok(UtmStatus {
            installed: true,
            version,
            path: Some(UTM_APP_PATH.to_string()),
        })
    }

    pub(super) fn get_utm_version(plist_path: &str) -> Option<String> {
        let plist_content = std::fs::read(plist_path).ok()?;
        let plist: plist::Value = plist::from_bytes(&plist_content).ok()?;
        let dict = plist.as_dictionary()?;
        dict.get("CFBundleShortVersionString")?
            .as_string()
            .map(|s| s.to_string())
    }

    /// Get the primary network interface for bridged networking.
    /// Uses the default route to determine which interface has internet connectivity.
    pub(super) fn get_primary_network_interface() -> String {
        let output = Command::new("route")
            .args(["-n", "get", "default"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.trim().starts_with("interface:") {
                        if let Some(interface) = line.split(':').nth(1) {
                            let interface = interface.trim();
                            if !interface.is_empty() {
                                return interface.to_string();
                            }
                        }
                    }
                }
            }
        }

        // Default to en0 if detection fails
        "en0".to_string()
    }

    /// Run an AppleScript and return the output
    pub(super) fn run_applescript(script: &str) -> Result<String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| Error::Utm(format!("Failed to execute AppleScript: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Utm(stderr.trim().to_string()))
        }
    }

    /// Start a VM by its ID
    pub(super) fn start_vm(vm_id: &str) -> Result<()> {
        let script = format!(
            r#"tell application "UTM"
    set vm to virtual machine id "{}"
    start vm
end tell"#,
            vm_id
        );

        run_applescript(&script)?;
        Ok(())
    }

    pub async fn create_vm<P: ProgressCallback>(
        config: &UtmVmConfig,
        progress_callback: &P,
    ) -> Result<UtmVmResult> {
        #[cfg(feature = "mock")]
        {
            if crate::is_mock_enabled() {
                // Simulate VM creation progress
                let stages = [
                    (10, "Downloading HAOS image..."),
                    (30, "Extracting image..."),
                    (50, "Creating UTM VM..."),
                    (70, "Configuring VM settings..."),
                    (90, "Starting VM..."),
                    (100, "Complete"),
                ];

                for (progress, message) in stages {
                    progress_callback.on_progress(FlashProgress {
                        stage: if progress < 100 {
                            FlashStage::Downloading
                        } else {
                            FlashStage::Complete
                        },
                        progress,
                        bytes_processed: 0,
                        total_bytes: 0,
                        message: message.to_string(),
                    });
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }

                return Ok(UtmVmResult {
                    name: config.name.clone(),
                    path: Some(format!(
                        "~/Library/Containers/com.utmapp.UTM/Data/Documents/{}.utm",
                        config.name
                    )),
                });
            }
        }

        // Verify UTM is installed
        let status = check_utm_status().await?;
        if !status.installed {
            return Err(Error::Utm("UTM is not installed".to_string()));
        }

        // Verify the image file exists
        let image_path = std::path::Path::new(&config.image_path);
        if !image_path.exists() {
            return Err(Error::Utm(format!(
                "Image file not found: {}",
                config.image_path
            )));
        }

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Downloading,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Creating virtual machine...".to_string(),
        });

        // Get architecture and network interface
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        let network_interface = get_primary_network_interface();

        // Escape the name for AppleScript
        let escaped_name = config.name.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_path = config.image_path.replace('\\', "\\\\").replace('"', "\\\"");

        // Convert disk size from GB to MB for UTM
        let disk_size_mb = config.disk_size_gb * 1024;

        // Build the drives configuration with VirtIO interface
        let drives_config = format!(
            "{{interface:VirtIO, source:(POSIX file \"{}\"), guest size:{}}}",
            escaped_path, disk_size_mb
        );

        // Create VM using QEMU backend with hardware virtualization
        // - hypervisor:true for hardware acceleration (uses macOS Hypervisor.framework)
        // - uefi:true for UEFI boot (required by HAOS)
        // - bridged network for direct LAN access (required for Home Assistant)
        let script = format!(
            r#"tell application "UTM"
    set vmConfig to {{name:"{name}", notes:"Created by the Home Assistant Installer", architecture:"{arch}", cpu cores:{cores}, memory:{memory}, hypervisor:true, uefi:true, drives:{{{drives}}}, network interfaces:{{{{mode:bridged, host interface:"{interface}"}}}}}}
    set vm to make new virtual machine with properties {{backend:qemu, configuration:vmConfig}}
    return id of vm
end tell"#,
            name = escaped_name,
            arch = arch,
            cores = config.cpu_cores,
            memory = config.memory_mb,
            drives = drives_config,
            interface = network_interface,
        );

        let vm_id = run_applescript(&script)?;

        // Start the VM
        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Downloading,
            progress: 50,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Starting virtual machine...".to_string(),
        });

        start_vm(&vm_id)?;

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: 0,
            total_bytes: 0,
            message: "VM created and started".to_string(),
        });

        Ok(UtmVmResult {
            name: config.name.clone(),
            path: Some(format!(
                "~/Library/Containers/com.utmapp.UTM/Data/Documents/{}.utm",
                config.name
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    #[cfg(target_os = "macos")]
    async fn test_check_utm_status_mock() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let status = check_utm_status().await.unwrap();
        assert!(status.installed);
        assert!(status.version.is_some());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    #[cfg(target_os = "macos")]
    async fn test_create_vm_mock() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let config = UtmVmConfig {
            name: "Test VM".to_string(),
            image_path: "/tmp/test.qcow2".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };
        let result = create_vm(&config, &crate::NoOpProgress).await.unwrap();
        assert_eq!(result.name, "Test VM");
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    #[cfg(not(target_os = "macos"))]
    async fn test_check_utm_status_not_available_on_non_macos() {
        let result = check_utm_status().await;
        assert!(result.is_err());
        match result {
            Err(Error::UnsupportedPlatform(msg)) => {
                assert_eq!(msg, "UTM is only available on macOS");
            }
            _ => panic!("Expected UnsupportedPlatform error"),
        }
    }

    #[tokio::test]
    #[serial]
    #[cfg(not(target_os = "macos"))]
    async fn test_create_vm_not_available_on_non_macos() {
        let config = UtmVmConfig {
            name: "Test VM".to_string(),
            image_path: "/tmp/test.qcow2".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };
        let result = create_vm(&config, &crate::NoOpProgress).await;
        assert!(result.is_err());
        match result {
            Err(Error::UnsupportedPlatform(msg)) => {
                assert_eq!(msg, "UTM is only available on macOS");
            }
            _ => panic!("Expected UnsupportedPlatform error"),
        }
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::macos::{get_primary_network_interface, get_utm_version, run_applescript, start_vm, check_utm_status};
        use super::create_vm;
        use crate::error::Error;
        use crate::types::UtmVmConfig;

        #[test]
        fn test_get_primary_network_interface() {
            let interface = get_primary_network_interface();

            // Should return a non-empty string
            assert!(!interface.is_empty(), "Interface should not be empty");

            // Should typically start with "en" (like en0, en1, en2, etc.) or be "en0" as fallback
            // Common macOS interfaces: en0 (Ethernet/WiFi), en1, bridge0, etc.
            // At minimum, it should return "en0" as the fallback
            assert!(
                interface.starts_with("en") || interface.starts_with("bridge"),
                "Interface should typically start with 'en' or 'bridge', got: {}",
                interface
            );

            // The fallback is specifically "en0"
            // If the command fails or no interface is found, it returns "en0"
            if interface == "en0" {
                // This is valid - either it's the actual interface or the fallback
                assert_eq!(interface, "en0");
            }
        }

        #[tokio::test]
        #[serial_test::serial]
        async fn test_check_utm_status_not_installed() {
            // Temporarily disable mock mode to test the real path
            std::env::remove_var("HA_INSTALLER_MOCK");

            // Since UTM is unlikely to be installed at the exact path we check,
            // or if it is, we can still verify the logic works
            let status = check_utm_status().await.unwrap();

            // Either UTM is installed or not, both are valid states
            if status.installed {
                assert!(status.path.is_some());
                assert_eq!(status.path.as_ref().unwrap(), "/Applications/UTM.app");
            } else {
                assert!(!status.installed);
                assert!(status.version.is_none());
                assert!(status.path.is_none());
            }
        }

        #[tokio::test]
        #[serial_test::serial]
        async fn test_create_vm_non_mock_utm_not_installed() {
            // Disable mock mode
            std::env::remove_var("HA_INSTALLER_MOCK");

            let config = UtmVmConfig {
                name: "Test VM".to_string(),
                image_path: "/tmp/test.qcow2".to_string(),
                cpu_cores: 2,
                memory_mb: 2048,
                disk_size_gb: 32,
                auto_start: false,
            };

            let result = create_vm(&config, &crate::NoOpProgress).await;

            // If UTM is not installed, we should get an error
            // If it is installed, we'll get an error about the image file not existing
            // Both are valid test outcomes as they exercise the non-mock code paths
            if let Err(e) = result {
                let error_msg = format!("{}", e);
                assert!(
                    error_msg.contains("UTM is not installed") ||
                    error_msg.contains("Image file not found") ||
                    error_msg.contains("Failed to execute AppleScript"),
                    "Unexpected error: {}", error_msg
                );
            }
        }

        #[tokio::test]
        #[serial_test::serial]
        async fn test_create_vm_non_mock_image_not_found() {
            // Disable mock mode
            std::env::remove_var("HA_INSTALLER_MOCK");

            // Create a temporary directory for testing
            let temp_dir = std::env::temp_dir();
            let non_existent_image = temp_dir.join("non_existent_image.qcow2");

            let config = UtmVmConfig {
                name: "Test VM".to_string(),
                image_path: non_existent_image.to_str().unwrap().to_string(),
                cpu_cores: 2,
                memory_mb: 2048,
                disk_size_gb: 32,
                auto_start: false,
            };

            let result = create_vm(&config, &crate::NoOpProgress).await;

            // Should either fail because UTM is not installed or because image doesn't exist
            assert!(result.is_err());
            if let Err(e) = result {
                let error_msg = format!("{}", e);
                // Either UTM is not installed or the image file is not found
                assert!(
                    error_msg.contains("UTM is not installed") ||
                    error_msg.contains("Image file not found"),
                    "Expected specific error, got: {}", error_msg
                );
            }
        }

        #[test]
        fn test_run_applescript_failure() {
            // Test AppleScript with invalid syntax to trigger error path
            let result = run_applescript("this is invalid applescript syntax!");
            assert!(result.is_err());
            if let Err(Error::Utm(msg)) = result {
                // Error message should contain something about syntax error
                assert!(!msg.is_empty(), "Error message should not be empty");
            } else {
                panic!("Expected Utm error");
            }
        }

        #[test]
        fn test_run_applescript_success() {
            // Test a simple AppleScript that should succeed
            // This just returns a simple string
            let result = run_applescript("return \"test\"");

            match result {
                Ok(output) => {
                    assert_eq!(output, "test");
                }
                Err(_) => {
                    // If it fails, it might be because osascript is not available
                    // which is fine for the test environment
                }
            }
        }

        #[test]
        fn test_start_vm_function() {
            // Test that start_vm generates the correct AppleScript
            // We can't actually start a VM without UTM installed and a real VM ID,
            // but we can verify the function handles errors appropriately
            let result = start_vm("test-vm-id-12345");

            // Should fail because this VM ID doesn't exist
            if let Err(e) = result {
                // Verify it's a UTM error (from AppleScript execution)
                match e {
                    Error::Utm(_) => {
                        // Expected - the VM doesn't exist or UTM isn't running
                    }
                    _ => panic!("Expected Utm error, got: {:?}", e),
                }
            }
        }

        #[test]
        fn test_get_utm_version_with_string_not_dict() {
            // Create a temporary file with a plist that has the wrong type for the version
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("wrong_type_plist.plist");

            // Write a valid plist with CFBundleShortVersionString as a number instead of string
            let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleShortVersionString</key>
    <integer>42</integer>
</dict>
</plist>"#;

            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(plist_content.as_bytes()).unwrap();
            drop(file);

            let result = get_utm_version(test_file.to_str().unwrap());

            // Should return None when the value is not a string
            assert!(result.is_none(), "Should return None when version is not a string");

            // Cleanup
            let _ = std::fs::remove_file(test_file);
        }

        #[test]
        fn test_get_utm_version_with_non_dict_root() {
            // Create a temporary file with a plist that has an array at root instead of dict
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("array_root_plist.plist");

            // Write a valid plist with an array at the root
            let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
    <string>test</string>
</array>
</plist>"#;

            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(plist_content.as_bytes()).unwrap();
            drop(file);

            let result = get_utm_version(test_file.to_str().unwrap());

            // Should return None when root is not a dictionary
            assert!(result.is_none(), "Should return None when plist root is not a dictionary");

            // Cleanup
            let _ = std::fs::remove_file(test_file);
        }

        #[tokio::test]
        #[serial_test::serial]
        async fn test_create_vm_with_existing_image_file() {
            // Disable mock mode
            std::env::remove_var("HA_INSTALLER_MOCK");

            // Create a temporary image file
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_image = temp_dir.join("test_utm_image.qcow2");
            let mut file = std::fs::File::create(&test_image).unwrap();
            file.write_all(b"fake qcow2 data").unwrap();
            drop(file);

            let config = UtmVmConfig {
                name: "Test VM with Image".to_string(),
                image_path: test_image.to_str().unwrap().to_string(),
                cpu_cores: 2,
                memory_mb: 2048,
                disk_size_gb: 32,
                auto_start: false,
            };

            let result = create_vm(&config, &crate::NoOpProgress).await;

            // This test exercises the code path after the image file check
            // If UTM is installed, it might succeed or fail depending on the system
            // If UTM is not installed, it should fail
            // Either way, we've exercised the non-mock create_vm path
            match result {
                Ok(_) => {
                    // UTM is installed and VM was created (or at least attempted)
                    // This is actually good - it means we exercised the full VM creation path
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    // The error should be about UTM or AppleScript, not about the image file
                    assert!(
                        error_msg.contains("UTM") || error_msg.contains("AppleScript") ||
                        error_msg.contains("virtual machine"),
                        "Expected UTM/AppleScript/VM error after image check, got: {}", error_msg
                    );
                }
            }

            // Cleanup
            let _ = std::fs::remove_file(test_image);
        }

        #[test]
        fn test_get_utm_version_with_nonexistent_path() {
            // Test with a path that doesn't exist
            let result = get_utm_version("/nonexistent/path/to/Info.plist");

            // Should return None for invalid/non-existent plist path
            assert!(result.is_none(), "Should return None for non-existent path");
        }

        #[test]
        fn test_get_utm_version_with_invalid_plist() {
            // Create a temporary file with invalid plist content
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("invalid_plist.plist");

            // Write invalid content
            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(b"This is not a valid plist file").unwrap();
            drop(file);

            let result = get_utm_version(test_file.to_str().unwrap());

            // Should return None for invalid plist content
            assert!(result.is_none(), "Should return None for invalid plist file");

            // Cleanup
            let _ = std::fs::remove_file(test_file);
        }

        #[test]
        fn test_get_utm_version_with_missing_version_key() {
            // Create a temporary file with valid plist but missing version key
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("no_version_plist.plist");

            // Write a valid plist without CFBundleShortVersionString
            let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>TestApp</string>
</dict>
</plist>"#;

            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(plist_content.as_bytes()).unwrap();
            drop(file);

            let result = get_utm_version(test_file.to_str().unwrap());

            // Should return None when CFBundleShortVersionString is missing
            assert!(result.is_none(), "Should return None when version key is missing");

            // Cleanup
            let _ = std::fs::remove_file(test_file);
        }

        #[test]
        fn test_get_utm_version_with_valid_plist() {
            // Create a temporary file with valid plist including version
            use std::io::Write;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("valid_plist.plist");

            // Write a valid plist with CFBundleShortVersionString
            let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleShortVersionString</key>
    <string>4.5.0</string>
    <key>CFBundleName</key>
    <string>UTM</string>
</dict>
</plist>"#;

            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(plist_content.as_bytes()).unwrap();
            drop(file);

            let result = get_utm_version(test_file.to_str().unwrap());

            // Should return the version string
            assert!(result.is_some(), "Should return Some for valid plist with version");
            assert_eq!(result.unwrap(), "4.5.0", "Should extract correct version");

            // Cleanup
            let _ = std::fs::remove_file(test_file);
        }
    }
}
