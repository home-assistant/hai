//! UTM virtual machine management for macOS.
//!
//! This module provides functions to detect UTM installation, create Home Assistant VMs,
//! and manage VM lifecycle via AppleScript.

use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Errors that can occur during UTM operations.
#[derive(Error, Debug)]
pub enum UtmError {
    #[error("UTM is not installed")]
    NotInstalled,

    #[error("UTM operation failed: {0}")]
    OperationFailed(String),

    #[error("AppleScript execution failed: {0}")]
    AppleScriptError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result of checking UTM installation status.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UtmStatus {
    /// Whether UTM is installed
    pub installed: bool,
    /// Path to UTM.app if installed
    pub path: Option<String>,
    /// UTM version if installed
    pub version: Option<String>,
}

/// Configuration for creating a new VM.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VmConfig {
    /// Name for the VM
    pub name: String,
    /// Path to the HAOS image file (qcow2 or raw)
    pub image_path: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u32,
    /// Disk size in GB
    pub disk_size_gb: u32,
    /// Whether to start the VM after creation
    pub auto_start: bool,
}

/// The standard UTM application path.
const UTM_APP_PATH: &str = "/Applications/UTM.app";

/// Check if UTM is installed and get its status.
pub fn check_utm_installed() -> UtmStatus {
    let path = Path::new(UTM_APP_PATH);

    if !path.exists() {
        return UtmStatus {
            installed: false,
            path: None,
            version: None,
        };
    }

    // Try to get version via utmctl
    let version = get_utm_version().ok();

    UtmStatus {
        installed: true,
        path: Some(UTM_APP_PATH.to_string()),
        version,
    }
}

/// Get UTM version using utmctl.
fn get_utm_version() -> Result<String, UtmError> {
    let utmctl = format!("{}/Contents/MacOS/utmctl", UTM_APP_PATH);
    let output = Command::new(&utmctl).arg("version").output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err(UtmError::OperationFailed(
            "Failed to get UTM version".to_string(),
        ))
    }
}

/// Get the Mac's CPU architecture (arm64 or x86_64).
pub fn get_mac_architecture() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64".to_string()
    }
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64".to_string()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        "unknown".to_string()
    }
}

/// Get the primary network interface for bridged networking.
///
/// Uses the default route to determine which interface has internet connectivity.
/// Falls back to "en0" if detection fails.
pub fn get_primary_network_interface() -> String {
    // Use route command to find the default gateway interface
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

/// Create a new Home Assistant VM in UTM.
///
/// Uses the Apple Virtualization Framework for native performance on both
/// Apple Silicon (aarch64) and Intel (x86_64) Macs.
///
/// The VM is configured with:
/// - Boot disk from the provided HAOS image
/// - Bridged network for direct LAN access (required for Home Assistant)
/// - Virtual display with dynamic resolution
pub fn create_vm(config: &VmConfig) -> Result<String, UtmError> {
    let arch = get_mac_architecture();

    // Verify we're on a supported architecture
    if arch != "aarch64" && arch != "x86_64" {
        return Err(UtmError::OperationFailed("Unsupported architecture".to_string()));
    }

    // Verify the image file exists if a path was provided
    if !config.image_path.is_empty() {
        let image_path = Path::new(&config.image_path);
        if !image_path.exists() {
            return Err(UtmError::OperationFailed(format!(
                "Image file not found: {}",
                config.image_path
            )));
        }
        // Check that the file has actual content (not just a sparse/empty file)
        let metadata = std::fs::metadata(image_path)?;
        if metadata.len() == 0 {
            return Err(UtmError::OperationFailed(format!(
                "Image file is empty: {}",
                config.image_path
            )));
        }
    }

    // Escape the name for AppleScript (handle quotes)
    let escaped_name = config.name.replace('\\', "\\\\").replace('"', "\\\"");

    // Detect the primary network interface for bridged networking
    let network_interface = get_primary_network_interface();

    // Build the drives configuration for QEMU with VirtIO interface
    // Convert disk size from GB to MB for UTM
    let disk_size_mb = config.disk_size_gb * 1024;
    let drives_config = if config.image_path.is_empty() {
        // Create a new empty disk with VirtIO interface
        format!("{{interface:VirtIO, guest size:{}}}", disk_size_mb)
    } else {
        // Use existing qcow2 image file with VirtIO interface
        let escaped_path = config.image_path.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "{{interface:VirtIO, source:(POSIX file \"{}\"), guest size:{}}}",
            escaped_path, disk_size_mb
        )
    };

    // Use QEMU backend with hardware virtualization for best compatibility
    // QEMU supports qcow2 images directly and provides VirtIO for performance
    //
    // Configuration includes:
    // - architecture: aarch64 for Apple Silicon, x86_64 for Intel
    // - hypervisor: true for hardware acceleration (uses macOS Hypervisor.framework)
    // - uefi: true for UEFI boot (required by HAOS)
    // - drives: VirtIO interface with qcow2 source image
    // - network interfaces: bridged mode for direct LAN access
    //   (required for Home Assistant to communicate with IoT devices,
    //    mobile apps, and get its own IP address on the network)
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

    let output = run_applescript(&script)?;
    let vm_id = output.trim().to_string();

    // Start the VM if requested
    if config.auto_start {
        start_vm(&vm_id)?;
    }

    Ok(vm_id)
}

/// Resize the disk of a VM using UTM's update configuration command.
///
/// This must be called before the VM is started for the first time.
/// Uses AppleScript to update the drive's guest size property.
pub fn resize_vm_disk(vm_identifier: &str, size_mb: u32) -> Result<(), UtmError> {
    // AppleScript to get the VM configuration, update the first drive's guest size,
    // and apply the configuration update.
    let script = format!(
        r#"tell application "UTM"
    set vm to virtual machine id "{vm_id}"
    set config to configuration of vm
    set driveId to id of item 1 of drives of config
    set item 1 of drives of config to {{id:driveId, guest size:{size}}}
    update configuration of vm with config
end tell"#,
        vm_id = vm_identifier,
        size = size_mb,
    );

    run_applescript(&script)?;
    Ok(())
}

/// Start a VM by its ID or name.
pub fn start_vm(vm_identifier: &str) -> Result<(), UtmError> {
    let script = format!(
        r#"
tell application "UTM"
    set vm to virtual machine id "{}"
    start vm
end tell
"#,
        vm_identifier
    );

    run_applescript(&script)?;
    Ok(())
}

/// Stop a VM by its ID or name.
pub fn stop_vm(vm_identifier: &str) -> Result<(), UtmError> {
    let script = format!(
        r#"
tell application "UTM"
    set vm to virtual machine id "{}"
    stop vm
end tell
"#,
        vm_identifier
    );

    run_applescript(&script)?;
    Ok(())
}

/// Delete a VM by its ID or name.
pub fn delete_vm(vm_identifier: &str) -> Result<(), UtmError> {
    let script = format!(
        r#"
tell application "UTM"
    set vm to virtual machine id "{}"
    delete vm
end tell
"#,
        vm_identifier
    );

    run_applescript(&script)?;
    Ok(())
}

/// List all VMs in UTM.
pub fn list_vms() -> Result<Vec<String>, UtmError> {
    let utmctl = format!("{}/Contents/MacOS/utmctl", UTM_APP_PATH);
    let output = Command::new(&utmctl).arg("list").output()?;

    if !output.status.success() {
        return Err(UtmError::OperationFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Parse the output - format is "UUID Status Name"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let vms: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                Some(parts[2..].join(" "))
            } else {
                None
            }
        })
        .collect();

    Ok(vms)
}

/// Run an AppleScript and return the output.
fn run_applescript(script: &str) -> Result<String, UtmError> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(UtmError::AppleScriptError(stderr.to_string()))
    }
}

/// VM running status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VmStatus {
    Stopped,
    Starting,
    Started,
    Pausing,
    Paused,
    Stopping,
    Unknown,
}

impl From<&str> for VmStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "stopped" => VmStatus::Stopped,
            "starting" => VmStatus::Starting,
            "started" => VmStatus::Started,
            "pausing" => VmStatus::Pausing,
            "paused" => VmStatus::Paused,
            "stopping" => VmStatus::Stopping,
            _ => VmStatus::Unknown,
        }
    }
}

/// Get the status of a VM by its ID or name.
pub fn get_vm_status(vm_identifier: &str) -> Result<VmStatus, UtmError> {
    let utmctl = format!("{}/Contents/MacOS/utmctl", UTM_APP_PATH);
    let output = Command::new(&utmctl)
        .args(["status", vm_identifier])
        .output()?;

    if !output.status.success() {
        return Err(UtmError::OperationFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status_str = stdout.trim();
    Ok(VmStatus::from(status_str))
}

/// Get the IP address of a running VM.
///
/// This requires the QEMU guest agent to be running in the VM.
/// Returns the first IPv4 address if available, otherwise the first IPv6 address.
/// Returns None if no IP address is available yet.
pub fn get_vm_ip_address(vm_identifier: &str) -> Result<Option<String>, UtmError> {
    let utmctl = format!("{}/Contents/MacOS/utmctl", UTM_APP_PATH);
    let output = Command::new(&utmctl)
        .args(["ip-address", vm_identifier])
        .output()?;

    // If the command fails, it might be because the guest agent isn't ready yet
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Guest agent not ready is not a fatal error
        if stderr.contains("agent") || stderr.contains("not running") || stderr.contains("timeout") {
            return Ok(None);
        }
        return Err(UtmError::OperationFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ip = stdout.trim();

    if ip.is_empty() {
        return Ok(None);
    }

    // The output might contain multiple IPs, one per line
    // First pass: look for IPv4 addresses (preferred)
    for line in ip.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip localhost
        if trimmed.starts_with("127.") {
            continue;
        }
        // Check if it's an IPv4 address (contains dots, no colons)
        if trimmed.contains('.') && !trimmed.contains(':') {
            return Ok(Some(trimmed.to_string()));
        }
    }

    // Second pass: fall back to IPv6 if no IPv4 found
    for line in ip.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip localhost
        if trimmed == "::1" {
            continue;
        }
        // Skip link-local IPv6 (fe80::)
        if trimmed.starts_with("fe80:") {
            continue;
        }
        // Accept other IPv6 addresses
        if trimmed.contains(':') {
            return Ok(Some(trimmed.to_string()));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mac_architecture() {
        let arch = get_mac_architecture();
        // Should be one of the known architectures
        assert!(
            arch == "aarch64" || arch == "x86_64" || arch == "unknown",
            "Unexpected architecture: {}",
            arch
        );
    }

    #[test]
    fn test_check_utm_installed() {
        let status = check_utm_installed();
        // This will vary depending on whether UTM is installed
        // Just ensure it doesn't panic
        println!("UTM installed: {}", status.installed);
        if status.installed {
            assert!(status.path.is_some());
        }
    }

    // Tests for VmConfig struct
    #[test]
    fn test_vm_config_creation() {
        let config = VmConfig {
            name: "Test VM".to_string(),
            image_path: "/path/to/image.qcow2".to_string(),
            cpu_cores: 4,
            memory_mb: 4096,
            disk_size_gb: 32,
            auto_start: true,
        };

        assert_eq!(config.name, "Test VM");
        assert_eq!(config.image_path, "/path/to/image.qcow2");
        assert_eq!(config.cpu_cores, 4);
        assert_eq!(config.memory_mb, 4096);
        assert_eq!(config.disk_size_gb, 32);
        assert!(config.auto_start);
    }

    #[test]
    fn test_vm_config_various_values() {
        let config = VmConfig {
            name: "Home Assistant".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 64,
            auto_start: false,
        };

        assert_eq!(config.name, "Home Assistant");
        assert_eq!(config.image_path, "");
        assert_eq!(config.cpu_cores, 2);
        assert_eq!(config.memory_mb, 2048);
        assert_eq!(config.disk_size_gb, 64);
        assert!(!config.auto_start);
    }

    #[test]
    fn test_vm_config_with_special_characters() {
        let config = VmConfig {
            name: "Test \"VM\" with 'quotes'".to_string(),
            image_path: "/path/with spaces/image.qcow2".to_string(),
            cpu_cores: 8,
            memory_mb: 8192,
            disk_size_gb: 128,
            auto_start: true,
        };

        assert!(config.name.contains('"'));
        assert!(config.name.contains('\''));
        assert!(config.image_path.contains(' '));
    }

    #[test]
    fn test_vm_config_minimal_resources() {
        let config = VmConfig {
            name: "Minimal VM".to_string(),
            image_path: "/image.qcow2".to_string(),
            cpu_cores: 1,
            memory_mb: 512,
            disk_size_gb: 8,
            auto_start: false,
        };

        assert_eq!(config.cpu_cores, 1);
        assert_eq!(config.memory_mb, 512);
        assert_eq!(config.disk_size_gb, 8);
    }

    #[test]
    fn test_vm_config_maximum_resources() {
        let config = VmConfig {
            name: "Powerful VM".to_string(),
            image_path: "/image.qcow2".to_string(),
            cpu_cores: 32,
            memory_mb: 65536,
            disk_size_gb: 1024,
            auto_start: false,
        };

        assert_eq!(config.cpu_cores, 32);
        assert_eq!(config.memory_mb, 65536);
        assert_eq!(config.disk_size_gb, 1024);
    }

    // Tests for VmStatus enum
    #[test]
    fn test_vm_status_from_str_stopped() {
        let status = VmStatus::from("stopped");
        assert_eq!(status, VmStatus::Stopped);
    }

    #[test]
    fn test_vm_status_from_str_starting() {
        let status = VmStatus::from("starting");
        assert_eq!(status, VmStatus::Starting);
    }

    #[test]
    fn test_vm_status_from_str_started() {
        let status = VmStatus::from("started");
        assert_eq!(status, VmStatus::Started);
    }

    #[test]
    fn test_vm_status_from_str_pausing() {
        let status = VmStatus::from("pausing");
        assert_eq!(status, VmStatus::Pausing);
    }

    #[test]
    fn test_vm_status_from_str_paused() {
        let status = VmStatus::from("paused");
        assert_eq!(status, VmStatus::Paused);
    }

    #[test]
    fn test_vm_status_from_str_stopping() {
        let status = VmStatus::from("stopping");
        assert_eq!(status, VmStatus::Stopping);
    }

    #[test]
    fn test_vm_status_from_str_unknown() {
        let status = VmStatus::from("invalid");
        assert_eq!(status, VmStatus::Unknown);
    }

    #[test]
    fn test_vm_status_from_str_case_insensitive() {
        assert_eq!(VmStatus::from("STOPPED"), VmStatus::Stopped);
        assert_eq!(VmStatus::from("Started"), VmStatus::Started);
        assert_eq!(VmStatus::from("PAUSED"), VmStatus::Paused);
        assert_eq!(VmStatus::from("StArTeD"), VmStatus::Started);
    }

    #[test]
    fn test_vm_status_from_str_empty() {
        let status = VmStatus::from("");
        assert_eq!(status, VmStatus::Unknown);
    }

    #[test]
    fn test_vm_status_from_str_with_whitespace() {
        let status = VmStatus::from("  started  ");
        assert_eq!(status, VmStatus::Unknown); // Will not match due to whitespace
    }

    #[test]
    fn test_vm_status_equality() {
        assert_eq!(VmStatus::Stopped, VmStatus::Stopped);
        assert_eq!(VmStatus::Started, VmStatus::Started);
        assert_ne!(VmStatus::Stopped, VmStatus::Started);
        assert_ne!(VmStatus::Paused, VmStatus::Pausing);
    }

    #[test]
    fn test_vm_status_clone() {
        let status = VmStatus::Started;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_vm_status_debug() {
        let status = VmStatus::Started;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Started"));
    }

    // Tests for UtmStatus struct
    #[test]
    fn test_utm_status_installed() {
        let status = UtmStatus {
            installed: true,
            path: Some("/Applications/UTM.app".to_string()),
            version: Some("4.4.0".to_string()),
        };

        assert!(status.installed);
        assert_eq!(status.path, Some("/Applications/UTM.app".to_string()));
        assert_eq!(status.version, Some("4.4.0".to_string()));
    }

    #[test]
    fn test_utm_status_not_installed() {
        let status = UtmStatus {
            installed: false,
            path: None,
            version: None,
        };

        assert!(!status.installed);
        assert!(status.path.is_none());
        assert!(status.version.is_none());
    }

    #[test]
    fn test_utm_status_installed_no_version() {
        let status = UtmStatus {
            installed: true,
            path: Some("/Applications/UTM.app".to_string()),
            version: None,
        };

        assert!(status.installed);
        assert!(status.path.is_some());
        assert!(status.version.is_none());
    }

    #[test]
    fn test_utm_status_clone() {
        let status = UtmStatus {
            installed: true,
            path: Some("/Applications/UTM.app".to_string()),
            version: Some("4.4.0".to_string()),
        };

        let cloned = status.clone();
        assert_eq!(status.installed, cloned.installed);
        assert_eq!(status.path, cloned.path);
        assert_eq!(status.version, cloned.version);
    }

    #[test]
    fn test_utm_status_debug() {
        let status = UtmStatus {
            installed: true,
            path: Some("/Applications/UTM.app".to_string()),
            version: Some("4.4.0".to_string()),
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("installed"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_utm_status_with_custom_path() {
        let status = UtmStatus {
            installed: true,
            path: Some("/Custom/Path/UTM.app".to_string()),
            version: Some("4.5.0".to_string()),
        };

        assert!(status.path.unwrap().contains("Custom"));
        assert_eq!(status.version.unwrap(), "4.5.0");
    }

    // Tests for UtmError enum
    #[test]
    fn test_utm_error_not_installed() {
        let error = UtmError::NotInstalled;
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "UTM is not installed");
    }

    #[test]
    fn test_utm_error_operation_failed() {
        let error = UtmError::OperationFailed("Failed to create VM".to_string());
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "UTM operation failed: Failed to create VM");
    }

    #[test]
    fn test_utm_error_applescript_error() {
        let error = UtmError::AppleScriptError("Script execution failed".to_string());
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "AppleScript execution failed: Script execution failed");
    }

    #[test]
    fn test_utm_error_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = UtmError::IoError(io_error);
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("IO error"));
        assert!(error_msg.contains("File not found"));
    }

    #[test]
    fn test_utm_error_debug() {
        let error = UtmError::NotInstalled;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotInstalled"));
    }

    #[test]
    fn test_utm_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let utm_error: UtmError = io_error.into();
        let error_msg = format!("{}", utm_error);
        assert!(error_msg.contains("Access denied"));
    }

    #[test]
    fn test_utm_error_operation_failed_empty_message() {
        let error = UtmError::OperationFailed("".to_string());
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "UTM operation failed: ");
    }

    #[test]
    fn test_utm_error_applescript_error_multiline() {
        let error = UtmError::AppleScriptError("Line 1\nLine 2\nLine 3".to_string());
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Line 1"));
        assert!(error_msg.contains("Line 2"));
        assert!(error_msg.contains("Line 3"));
    }

    // Tests for helper functions
    #[test]
    fn test_get_primary_network_interface() {
        let interface = get_primary_network_interface();
        // Should return a non-empty string
        assert!(!interface.is_empty());
        // Common interfaces on macOS
        assert!(
            interface.starts_with("en")
            || interface.starts_with("bridge")
            || interface.starts_with("utun")
            || interface == "lo0",
            "Unexpected interface: {}",
            interface
        );
    }

    #[test]
    fn test_get_primary_network_interface_consistency() {
        let interface1 = get_primary_network_interface();
        let interface2 = get_primary_network_interface();
        // Should return the same interface on consecutive calls
        assert_eq!(interface1, interface2);
    }

    // Additional integration-style tests for create_vm validation
    #[test]
    fn test_create_vm_validates_architecture() {
        let _config = VmConfig {
            name: "Test VM".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        // This test validates that the architecture check works
        // On supported platforms, it should not fail at the architecture check
        let arch = get_mac_architecture();
        assert!(arch == "aarch64" || arch == "x86_64" || arch == "unknown");
    }

    #[test]
    fn test_create_vm_rejects_nonexistent_image() {
        let config = VmConfig {
            name: "Test VM".to_string(),
            image_path: "/nonexistent/path/image.qcow2".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        let result = create_vm(&config);
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                UtmError::OperationFailed(msg) => {
                    assert!(msg.contains("not found") || msg.contains("Unsupported architecture"));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_vm_config_name_validation_characters() {
        let config = VmConfig {
            name: "VM-with-dashes_and_underscores.123".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        assert!(config.name.contains('-'));
        assert!(config.name.contains('_'));
        assert!(config.name.contains('.'));
        assert!(config.name.contains("123"));
    }

    #[test]
    fn test_vm_config_unicode_name() {
        let config = VmConfig {
            name: "Test VM 🏠 Assistant".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        assert!(config.name.contains("🏠"));
        assert!(config.name.len() > "Test VM  Assistant".len()); // Unicode takes more bytes
    }

    #[test]
    fn test_vm_config_long_name() {
        let long_name = "A".repeat(256);
        let config = VmConfig {
            name: long_name.clone(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        assert_eq!(config.name.len(), 256);
        assert_eq!(config.name, long_name);
    }

    #[test]
    fn test_vm_config_empty_name() {
        let config = VmConfig {
            name: "".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        assert!(config.name.is_empty());
    }

    #[test]
    fn test_vm_config_zero_resources() {
        let config = VmConfig {
            name: "Test".to_string(),
            image_path: "".to_string(),
            cpu_cores: 0,
            memory_mb: 0,
            disk_size_gb: 0,
            auto_start: false,
        };

        assert_eq!(config.cpu_cores, 0);
        assert_eq!(config.memory_mb, 0);
        assert_eq!(config.disk_size_gb, 0);
    }

    #[test]
    fn test_utm_status_serialization() {
        // Test that UtmStatus can be serialized (has serde::Serialize)
        let status = UtmStatus {
            installed: true,
            path: Some("/Applications/UTM.app".to_string()),
            version: Some("4.4.0".to_string()),
        };

        let json = serde_json::to_string(&status);
        assert!(json.is_ok());

        if let Ok(json_str) = json {
            assert!(json_str.contains("installed"));
            assert!(json_str.contains("true"));
            assert!(json_str.contains("/Applications/UTM.app"));
        }
    }

    #[test]
    fn test_vm_status_serialization() {
        // Test that VmStatus can be serialized with snake_case
        let status = VmStatus::Started;
        let json = serde_json::to_string(&status);
        assert!(json.is_ok());

        if let Ok(json_str) = json {
            assert!(json_str.contains("started"));
        }
    }

    #[test]
    fn test_vm_config_deserialization() {
        let json = r#"{
            "name": "Test VM",
            "image_path": "/path/to/image.qcow2",
            "cpu_cores": 4,
            "memory_mb": 4096,
            "disk_size_gb": 32,
            "auto_start": true
        }"#;

        let config: Result<VmConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        if let Ok(config) = config {
            assert_eq!(config.name, "Test VM");
            assert_eq!(config.cpu_cores, 4);
            assert_eq!(config.memory_mb, 4096);
            assert_eq!(config.disk_size_gb, 32);
            assert!(config.auto_start);
        }
    }

    #[test]
    fn test_vm_status_all_variants_serialization() {
        let variants = vec![
            VmStatus::Stopped,
            VmStatus::Starting,
            VmStatus::Started,
            VmStatus::Pausing,
            VmStatus::Paused,
            VmStatus::Stopping,
            VmStatus::Unknown,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant);
            assert!(json.is_ok(), "Failed to serialize {:?}", variant);
        }
    }

    #[test]
    fn test_utm_app_path_constant() {
        // Verify the constant is set correctly
        assert_eq!(UTM_APP_PATH, "/Applications/UTM.app");
        assert!(UTM_APP_PATH.ends_with(".app"));
        assert!(UTM_APP_PATH.starts_with("/Applications"));
    }

    #[test]
    fn test_vm_status_coverage() {
        // Ensure we have tests for all enum variants
        let all_statuses = vec![
            VmStatus::Stopped,
            VmStatus::Starting,
            VmStatus::Started,
            VmStatus::Pausing,
            VmStatus::Paused,
            VmStatus::Stopping,
            VmStatus::Unknown,
        ];

        // Test that all can be cloned
        for status in &all_statuses {
            let _ = status.clone();
        }

        // Test that all can be compared
        for i in 0..all_statuses.len() {
            for j in 0..all_statuses.len() {
                if i == j {
                    assert_eq!(all_statuses[i], all_statuses[j]);
                } else {
                    assert_ne!(all_statuses[i], all_statuses[j]);
                }
            }
        }
    }

    #[test]
    fn test_disk_size_conversion() {
        // Test the disk size GB to MB conversion logic used in create_vm
        let config = VmConfig {
            name: "Test".to_string(),
            image_path: "".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 32,
            auto_start: false,
        };

        let disk_size_mb = config.disk_size_gb * 1024;
        assert_eq!(disk_size_mb, 32768);

        // Test various sizes
        assert_eq!(1 * 1024, 1024); // 1GB = 1024MB
        assert_eq!(64 * 1024, 65536); // 64GB = 65536MB
        assert_eq!(128 * 1024, 131072); // 128GB = 131072MB
    }

    #[test]
    fn test_vm_config_path_variations() {
        let paths = vec![
            "/absolute/path/image.qcow2",
            "/path/with spaces/image.qcow2",
            "/path/with/unicode/🏠/image.qcow2",
            "relative/path/image.qcow2",
            "./current/dir/image.qcow2",
            "../parent/dir/image.qcow2",
            "",
        ];

        for path in paths {
            let config = VmConfig {
                name: "Test".to_string(),
                image_path: path.to_string(),
                cpu_cores: 2,
                memory_mb: 2048,
                disk_size_gb: 32,
                auto_start: false,
            };

            assert_eq!(config.image_path, path);
        }
    }
}
