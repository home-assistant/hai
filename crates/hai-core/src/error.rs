//! Unified error types for hai-core

use thiserror::Error;

/// Error type for all hai-core operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device is busy: {0}")]
    DeviceBusy(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Disk service unavailable: {0}")]
    DiskServiceUnavailable(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Proxmox API error: {0}")]
    ProxmoxApi(String),

    #[error("UTM error: {0}")]
    Utm(String),

    #[error("Drive disconnected")]
    DriveDisconnected,

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// Result type alias for hai-core operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_network_error() {
        // Create a network error by attempting to connect to an invalid address
        let reqwest_err = reqwest::get("http://0.0.0.0:0/test")
            .await
            .expect_err("Should produce network error");
        let error = Error::Network(reqwest_err);
        let msg = error.to_string();
        assert!(msg.starts_with("Network error:"));
    }

    #[test]
    fn test_display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::Io(io_err);
        let msg = error.to_string();
        assert!(msg.starts_with("IO error:"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_display_device_not_found() {
        let error = Error::DeviceNotFound("disk2".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Device not found: disk2");
        assert!(msg.contains("disk2"));
    }

    #[test]
    fn test_display_device_busy() {
        let error = Error::DeviceBusy("disk2".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Device is busy: disk2");
        assert!(msg.contains("disk2"));
    }

    #[test]
    fn test_display_checksum_mismatch() {
        let error = Error::ChecksumMismatch {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let msg = error.to_string();
        assert_eq!(msg, "Checksum mismatch: expected abc123, got def456");
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

    #[test]
    fn test_display_permission_denied() {
        let error = Error::PermissionDenied("Need root access".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Permission denied: Need root access");
        assert!(msg.contains("Need root access"));
    }

    #[test]
    fn test_display_disk_service_unavailable() {
        let error = Error::DiskServiceUnavailable("udisks2 is not available".to_string());
        assert_eq!(
            error.to_string(),
            "Disk service unavailable: udisks2 is not available"
        );
    }

    #[test]
    fn test_display_cancelled() {
        let error = Error::Cancelled;
        let msg = error.to_string();
        assert_eq!(msg, "Operation cancelled");
    }

    #[test]
    fn test_display_proxmox_api() {
        let error = Error::ProxmoxApi("Connection timeout".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Proxmox API error: Connection timeout");
        assert!(msg.contains("Connection timeout"));
    }

    #[test]
    fn test_display_utm() {
        let error = Error::Utm("VM creation failed".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "UTM error: VM creation failed");
        assert!(msg.contains("VM creation failed"));
    }

    #[test]
    fn test_display_drive_disconnected() {
        let error = Error::DriveDisconnected;
        let msg = error.to_string();
        assert_eq!(msg, "Drive disconnected");
    }

    #[test]
    fn test_display_unsupported_platform() {
        let error = Error::UnsupportedPlatform("Windows XP".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Platform not supported: Windows XP");
        assert!(msg.contains("Windows XP"));
    }

    #[test]
    fn test_display_json_error() {
        // Create a JSON parse error
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .expect_err("Should produce error");
        let error = Error::Json(json_err);
        let msg = error.to_string();
        assert!(msg.starts_with("JSON serialization error:"));
    }

    #[test]
    fn test_display_invalid_config() {
        let error = Error::InvalidConfig("Missing required field".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Invalid configuration: Missing required field");
        assert!(msg.contains("Missing required field"));
    }

    #[test]
    fn test_display_download_failed() {
        let error = Error::DownloadFailed("404 Not Found".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Download failed: 404 Not Found");
        assert!(msg.contains("404 Not Found"));
    }

    #[test]
    fn test_display_extraction_failed() {
        let error = Error::ExtractionFailed("Archive corrupted".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Extraction failed: Archive corrupted");
        assert!(msg.contains("Archive corrupted"));
    }

    #[test]
    fn test_display_verification_failed() {
        let error = Error::VerificationFailed("Signature invalid".to_string());
        let msg = error.to_string();
        assert_eq!(msg, "Verification failed: Signature invalid");
        assert!(msg.contains("Signature invalid"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error: Error = io_err.into();
        match error {
            Error::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
                assert_eq!(e.to_string(), "access denied");
            }
            _ => panic!("Expected Error::Io variant"),
        }
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid}")
            .expect_err("Should produce error");
        let error: Error = json_err.into();
        match error {
            Error::Json(_) => {
                // Successfully converted
            }
            _ => panic!("Expected Error::Json variant"),
        }
    }

    #[tokio::test]
    async fn test_from_reqwest_error() {
        // Create a reqwest error by attempting to connect to an invalid address
        let reqwest_err = reqwest::get("http://0.0.0.0:0/test")
            .await
            .expect_err("Should produce network error");
        let error: Error = reqwest_err.into();
        match error {
            Error::Network(_) => {
                // Successfully converted
            }
            _ => panic!("Expected Error::Network variant"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = Error::Cancelled;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Cancelled"));
    }

    #[test]
    fn test_checksum_mismatch_fields() {
        let error = Error::ChecksumMismatch {
            expected: "expected_hash".to_string(),
            actual: "actual_hash".to_string(),
        };
        if let Error::ChecksumMismatch { expected, actual } = error {
            assert_eq!(expected, "expected_hash");
            assert_eq!(actual, "actual_hash");
        } else {
            panic!("Expected ChecksumMismatch variant");
        }
    }

    #[test]
    fn test_all_error_variants_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // This will fail to compile if Error is not Send + Sync
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(Error::Cancelled)
        }

        assert_eq!(returns_result().unwrap(), 42);
        assert!(returns_error().is_err());
    }
}
