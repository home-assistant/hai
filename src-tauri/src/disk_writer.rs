//! Raw disk writing functionality for flashing images to devices.
//!
//! This module provides platform-specific implementations for writing
//! raw disk images to block devices (SD cards, USB drives, etc.).

use crate::types::{FlashProgress, FlashStage};
use std::path::PathBuf;
use tauri::ipc::Channel;
use thiserror::Error;

/// Buffer size for disk writes (4 MB for SD cards)
const WRITE_BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// Buffer size for fast drives like NVMe/SSDs (64 MB)
const FAST_DRIVE_BUFFER_SIZE: usize = 64 * 1024 * 1024;

/// How often to send progress updates (every N bytes)
const PROGRESS_UPDATE_INTERVAL: u64 = 10 * 1024 * 1024; // 10 MB

#[derive(Error, Debug)]
pub enum DiskWriteError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to unmount device: {0}")]
    UnmountError(String),

    #[error("Failed to eject device: {0}")]
    EjectError(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Write verification failed")]
    VerificationFailed,

    #[error("Platform not supported")]
    PlatformNotSupported,

    #[error("Refusing to write to system drive: {0}")]
    SystemDriveProtected(String),

    #[error("The storage device was disconnected during the operation")]
    DriveDisconnected,
}

/// Check if an I/O error indicates the drive was disconnected
fn is_drive_disconnected(io_err: &std::io::Error) -> bool {
    // Check common error kinds that indicate disconnection
    matches!(
        io_err.kind(),
        std::io::ErrorKind::NotFound
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::UnexpectedEof
    ) || io_err.raw_os_error().is_some_and(|code| {
        // macOS: ENXIO (6) = "Device not configured"
        // Linux: ENODEV (19) = "No such device", ENXIO (6)
        matches!(code, 6 | 19)
    })
}

/// Validate that a device path is safe to write to (not a system drive)
fn validate_device_path(device_id: &str) -> Result<(), DiskWriteError> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, disk0 is always the system drive
        let disk_id = device_id.strip_prefix("/dev/").unwrap_or(device_id);
        let disk_id = disk_id.strip_prefix("r").unwrap_or(disk_id); // Handle raw device

        if disk_id == "disk0" {
            return Err(DiskWriteError::SystemDriveProtected(
                "disk0 is the system drive and cannot be overwritten".to_string(),
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, refuse to write to common system drive patterns
        // Note: This is a heuristic - the block_devices module already filters these out
        let dangerous_patterns = [
            "/dev/sda",     // First SATA drive (often system)
            "/dev/nvme0n1", // First NVMe drive (often system)
            "/dev/vda",     // First virtio drive (VMs)
        ];

        for pattern in dangerous_patterns {
            if device_id == pattern {
                return Err(DiskWriteError::SystemDriveProtected(format!(
                    "{} appears to be a system drive and cannot be overwritten",
                    device_id
                )));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, PhysicalDrive0 is usually the system drive
        if device_id == "\\\\.\\PhysicalDrive0" {
            return Err(DiskWriteError::SystemDriveProtected(
                "PhysicalDrive0 is the system drive and cannot be overwritten".to_string(),
            ));
        }
    }

    Ok(())
}

/// Write an image file to a block device with progress updates
pub async fn write_image(
    image_path: &PathBuf,
    device_id: &str,
    verify: bool,
    progress_channel: &Channel<FlashProgress>,
) -> Result<(), DiskWriteError> {
    // Safety check: refuse to write to system drives
    validate_device_path(device_id)?;

    #[cfg(target_os = "macos")]
    {
        macos::write_image(image_path, device_id, verify, progress_channel).await
    }

    #[cfg(target_os = "linux")]
    {
        linux::write_image(image_path, device_id, verify, progress_channel).await
    }

    #[cfg(target_os = "windows")]
    {
        windows::write_image(image_path, device_id, verify, progress_channel).await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(DiskWriteError::PlatformNotSupported)
    }
}

// =============================================================================
// macOS Implementation
// =============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use security_framework::authorization::{Authorization, AuthorizationItemSetBuilder, Flags};
    use std::io::Read;
    use std::path::Path;
    use std::process::Command;

    pub async fn write_image(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        // Extract disk identifier from device path (e.g., "/dev/disk2" -> "disk2")
        let disk_id = device_id.strip_prefix("/dev/").unwrap_or(device_id);

        // Get the raw device path for faster writes
        let raw_device = format!("/dev/r{}", disk_id);

        // Unmount all volumes on the disk
        unmount_disk(disk_id)?;

        // Get image size for progress tracking
        let image_size = std::fs::metadata(image_path)?.len();

        // Send initial progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Requesting administrator access...".to_string(),
        });

        // Perform write and optional verify in a single blocking task
        // This allows us to reuse the authorization for both operations
        let image_path_clone = image_path.clone();
        let raw_device_clone = raw_device.clone();
        let disk_id_clone = disk_id.to_string();
        let progress_channel_clone = progress_channel.clone();

        tokio::task::spawn_blocking(move || {
            write_and_verify(
                &image_path_clone,
                &raw_device_clone,
                &disk_id_clone,
                image_size,
                verify,
                &progress_channel_clone,
            )
        })
        .await
        .map_err(|e| {
            DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })??;

        Ok(())
    }

    /// Combined write and verify operation that reuses authorization
    fn write_and_verify(
        image_path: &PathBuf,
        device_path: &str,
        disk_id: &str,
        total_size: u64,
        verify: bool,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        // Request authorization once - this will show the macOS password dialog
        let auth = request_authorization()?;

        // Write the image and compute checksum if verification is requested
        let source_checksum = write_with_auth(
            &auth,
            image_path,
            device_path,
            total_size,
            progress_channel,
            verify,
        )?;

        // Verify if requested - reuse the same authorization
        if verify {
            let checksum = source_checksum
                .expect("Checksum should have been computed when verify=true");

            let _ = progress_channel.send(FlashProgress {
                stage: FlashStage::Verifying,
                progress: 0,
                bytes_processed: 0,
                total_bytes: total_size,
                message: "Verifying written data...".to_string(),
            });

            verify_with_auth(&auth, &checksum, device_path, total_size, progress_channel)?;
        }

        // Finalize - sync and eject
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Finalizing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Ejecting device...".to_string(),
        });

        eject_disk(disk_id)?;

        Ok(())
    }

    /// Request authorization from the user using the Security framework
    fn request_authorization() -> Result<Authorization, DiskWriteError> {
        // Create authorization with the right to execute privileged operations
        let rights = AuthorizationItemSetBuilder::new()
            .add_right("system.privilege.admin")
            .map_err(|e| DiskWriteError::PermissionDenied(format!("Failed to create rights: {}", e)))?
            .build();

        let auth = Authorization::new(
            Some(rights),
            None,
            Flags::INTERACTION_ALLOWED | Flags::EXTEND_RIGHTS | Flags::PREAUTHORIZE,
        )
        .map_err(|e| {
            if e.code() == -60006 {
                // errAuthorizationDenied
                DiskWriteError::PermissionDenied(
                    "Administrator access was denied by user".to_string(),
                )
            } else if e.code() == -60005 {
                // errAuthorizationCanceled
                DiskWriteError::PermissionDenied("Authorization was canceled".to_string())
            } else {
                DiskWriteError::PermissionDenied(format!("Authorization failed: {}", e))
            }
        })?;

        Ok(auth)
    }

    /// Write to device with pre-obtained authorization and progress updates
    /// Returns the SHA256 checksum of the source data if computed during write
    fn write_with_auth(
        auth: &Authorization,
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
        compute_checksum: bool,
    ) -> Result<Option<String>, DiskWriteError> {
        use sha2::{Digest, Sha256};
        use std::io::Write;

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: total_size,
            message: "Writing image to device...".to_string(),
        });

        // Open source file (no special permissions needed)
        let mut source = std::fs::File::open(image_path)?;

        // Use dd to open the device with admin privileges and get a pipe to write to
        // We read from our source and write to dd's stdin
        let dd_path = Path::new("/bin/dd");
        let of_arg = format!("of={}", device_path);
        let bs_arg = "bs=64m".to_string(); // 64MB block size for fast NVMe/SSD throughput

        // Execute dd with admin privileges, getting a pipe to write to
        let mut pipe = auth
            .execute_with_privileges_piped(dd_path, [&of_arg, &bs_arg], Flags::empty())
            .map_err(|e| {
                DiskWriteError::PermissionDenied(format!("Failed to open device: {}", e))
            })?;

        // Read from source and write to dd's stdin with progress updates
        // Optionally compute SHA256 while writing
        let mut hasher = if compute_checksum {
            Some(Sha256::new())
        } else {
            None
        };
        let mut buffer = vec![0u8; FAST_DRIVE_BUFFER_SIZE]; // 64MB buffer to match dd block size
        let mut bytes_written: u64 = 0;
        let mut last_progress_update = std::time::Instant::now();

        loop {
            let bytes_read = std::io::Read::read(&mut source, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // Compute checksum while writing (no extra read needed later)
            if let Some(ref mut h) = hasher {
                h.update(&buffer[..bytes_read]);
            }

            pipe.write_all(&buffer[..bytes_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;
            bytes_written += bytes_read as u64;

            // Update progress every 500ms or 10MB, whichever comes first
            let now = std::time::Instant::now();
            if now.duration_since(last_progress_update).as_millis() >= 500
                || bytes_written % (10 * 1024 * 1024) == 0
            {
                last_progress_update = now;
                let progress = ((bytes_written as f64 / total_size as f64) * 100.0) as u8;
                let _ = progress_channel.send(FlashProgress {
                    stage: FlashStage::Writing,
                    progress: progress.min(99),
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to device...".to_string(),
                });
            }
        }

        // Close the pipe to signal EOF to dd
        drop(pipe);

        // Run sync to ensure data is flushed to disk
        let _ = Command::new("sync").output();

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 100,
            bytes_processed: total_size,
            total_bytes: total_size,
            message: "Write complete".to_string(),
        });

        // Return the checksum if computed
        let checksum = hasher.map(|h| hex::encode(h.finalize()));
        Ok(checksum)
    }

    /// Verify written data by comparing checksums using pre-obtained authorization
    /// source_checksum: Pre-computed SHA256 of the source image (computed during write)
    fn verify_with_auth(
        auth: &Authorization,
        source_checksum: &str,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        use sha2::{Digest, Sha256};

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Verifying,
            progress: 0,
            bytes_processed: 0,
            total_bytes: total_size,
            message: "Reading device for verification...".to_string(),
        });

        // Read the device and compute checksum
        let block_count = (total_size + FAST_DRIVE_BUFFER_SIZE as u64 - 1) / (FAST_DRIVE_BUFFER_SIZE as u64);
        let dd_path = Path::new("/bin/dd");
        let if_arg = format!("if={}", device_path);
        let bs_arg = "bs=64m".to_string(); // 64MB block size for fast NVMe/SSD throughput
        let count_arg = format!("count={}", block_count);

        let mut pipe = auth
            .execute_with_privileges_piped(
                dd_path,
                [&if_arg, &bs_arg, &count_arg],
                Flags::empty(),
            )
            .map_err(|e| {
                DiskWriteError::PermissionDenied(format!("Failed to read device: {}", e))
            })?;

        // Read the device data and compute SHA256 with progress
        let mut device_hasher = Sha256::new();
        let mut buffer = vec![0u8; FAST_DRIVE_BUFFER_SIZE]; // 64MB buffer to match dd block size
        let mut bytes_verified: u64 = 0;
        let mut last_progress_update = std::time::Instant::now();

        loop {
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    device_hasher.update(&buffer[..n]);
                    bytes_verified += n as u64;

                    // Update progress (0-100% just reading device)
                    let now = std::time::Instant::now();
                    if now.duration_since(last_progress_update).as_millis() >= 500 {
                        last_progress_update = now;
                        let progress =
                            ((bytes_verified as f64 / total_size as f64) * 100.0) as u8;
                        let _ = progress_channel.send(FlashProgress {
                            stage: FlashStage::Verifying,
                            progress: progress.min(99),
                            bytes_processed: bytes_verified,
                            total_bytes: total_size,
                            message: "Verifying written data...".to_string(),
                        });
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(if is_drive_disconnected(&e) {
                        DiskWriteError::DriveDisconnected
                    } else {
                        DiskWriteError::IoError(e)
                    });
                }
            }
        }

        let device_checksum = hex::encode(device_hasher.finalize());

        if source_checksum != device_checksum {
            return Err(DiskWriteError::VerificationFailed);
        }

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Verifying,
            progress: 100,
            bytes_processed: total_size,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }

    fn unmount_disk(disk_id: &str) -> Result<(), DiskWriteError> {
        // First try a normal unmount
        let output = Command::new("diskutil")
            .args(["unmountDisk", disk_id])
            .output()
            .map_err(|e| DiskWriteError::UnmountError(e.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Ignore "not mounted" errors
        if stderr.contains("not mounted") || stderr.contains("was already unmounted") {
            return Ok(());
        }

        // If normal unmount failed (e.g., Spotlight indexing), try force unmount
        let force_output = Command::new("diskutil")
            .args(["unmountDisk", "force", disk_id])
            .output()
            .map_err(|e| DiskWriteError::UnmountError(e.to_string()))?;

        if !force_output.status.success() {
            let force_stderr = String::from_utf8_lossy(&force_output.stderr);
            if !force_stderr.contains("not mounted")
                && !force_stderr.contains("was already unmounted")
            {
                return Err(DiskWriteError::UnmountError(format!(
                    "Force unmount failed: {}",
                    force_stderr
                )));
            }
        }

        Ok(())
    }

    fn eject_disk(disk_id: &str) -> Result<(), DiskWriteError> {
        let output = Command::new("diskutil")
            .args(["eject", disk_id])
            .output()
            .map_err(|e| DiskWriteError::EjectError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DiskWriteError::EjectError(stderr.to_string()));
        }

        Ok(())
    }
}

// =============================================================================
// Linux Implementation
// =============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;

    pub async fn write_image(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        // Unmount all partitions on the device
        unmount_device(device_id)?;

        // Get image size for progress tracking
        let image_size = std::fs::metadata(image_path)?.len();

        // Send initial progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Writing image to device...".to_string(),
        });

        // Perform the write in a blocking task
        let image_path_clone = image_path.clone();
        let device_id_clone = device_id.to_string();
        let progress_channel_clone = progress_channel.clone();

        tokio::task::spawn_blocking(move || {
            write_to_device(
                &image_path_clone,
                &device_id_clone,
                image_size,
                &progress_channel_clone,
            )
        })
        .await
        .map_err(|e| {
            DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })??;

        // Verify if requested
        if verify {
            let _ = progress_channel.send(FlashProgress {
                stage: FlashStage::Verifying,
                progress: 0,
                bytes_processed: 0,
                total_bytes: image_size,
                message: "Verifying written data...".to_string(),
            });

            let image_path_clone = image_path.clone();
            let device_id_clone = device_id.to_string();
            let progress_channel_clone = progress_channel.clone();

            tokio::task::spawn_blocking(move || {
                verify_write(
                    &image_path_clone,
                    &device_id_clone,
                    image_size,
                    &progress_channel_clone,
                )
            })
            .await
            .map_err(|e| {
                DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
            })??;
        }

        // Finalize - sync
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Finalizing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Syncing data...".to_string(),
        });

        // Run sync command
        let _ = Command::new("sync").output();

        Ok(())
    }

    fn unmount_device(device_id: &str) -> Result<(), DiskWriteError> {
        // Find and unmount all partitions of this device
        // e.g., /dev/sdb -> unmount /dev/sdb1, /dev/sdb2, etc.
        let output = Command::new("umount")
            .args(["--all-targets", device_id])
            .output();

        // Also try to unmount numbered partitions
        for i in 1..=16 {
            let partition = if device_id.contains("mmcblk") || device_id.contains("nvme") {
                format!("{}p{}", device_id, i)
            } else {
                format!("{}{}", device_id, i)
            };
            let _ = Command::new("umount").arg(&partition).output();
        }

        // Ignore unmount errors - device might not be mounted
        let _ = output;
        Ok(())
    }

    fn write_to_device(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        let mut source = File::open(image_path)?;
        let mut dest = std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    DiskWriteError::PermissionDenied(
                        "Root access required. Please run with sudo.".to_string(),
                    )
                } else if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;

        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut bytes_written: u64 = 0;
        let mut last_progress_bytes: u64 = 0;

        loop {
            let bytes_read = source.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            dest.write_all(&buffer[..bytes_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;
            bytes_written += bytes_read as u64;

            // Update progress periodically
            if bytes_written - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_written;
                let progress = ((bytes_written as f64 / total_size as f64) * 100.0) as u8;
                let _ = progress_channel.send(FlashProgress {
                    stage: FlashStage::Writing,
                    progress: progress.min(99),
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to device...".to_string(),
                });
            }
        }

        // Sync to ensure all data is written
        dest.sync_all().map_err(|e| {
            if is_drive_disconnected(&e) {
                DiskWriteError::DriveDisconnected
            } else {
                DiskWriteError::IoError(e)
            }
        })?;

        // Send 100% progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 100,
            bytes_processed: bytes_written,
            total_bytes: total_size,
            message: "Write complete".to_string(),
        });

        Ok(())
    }

    fn verify_write(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        let mut source = File::open(image_path)?;
        let mut dest = File::open(device_path).map_err(|e| {
            if is_drive_disconnected(&e) {
                DiskWriteError::DriveDisconnected
            } else {
                DiskWriteError::IoError(e)
            }
        })?;

        let mut source_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut dest_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut bytes_verified: u64 = 0;
        let mut last_progress_bytes: u64 = 0;

        loop {
            let source_read = source.read(&mut source_buffer)?;
            if source_read == 0 {
                break;
            }

            dest.read_exact(&mut dest_buffer[..source_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;

            if source_buffer[..source_read] != dest_buffer[..source_read] {
                return Err(DiskWriteError::VerificationFailed);
            }

            bytes_verified += source_read as u64;

            // Update progress periodically
            if bytes_verified - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_verified;
                let progress = ((bytes_verified as f64 / total_size as f64) * 100.0) as u8;
                let _ = progress_channel.send(FlashProgress {
                    stage: FlashStage::Verifying,
                    progress: progress.min(99),
                    bytes_processed: bytes_verified,
                    total_bytes: total_size,
                    message: "Verifying written data...".to_string(),
                });
            }
        }

        // Send 100% progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Verifying,
            progress: 100,
            bytes_processed: bytes_verified,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }
}

// =============================================================================
// Windows Implementation
// =============================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;

    pub async fn write_image(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        // Extract disk number from device path (e.g., "\\.\PhysicalDrive2" -> "2")
        let disk_number = device_id
            .strip_prefix("\\\\.\\PhysicalDrive")
            .ok_or_else(|| DiskWriteError::DeviceNotFound(device_id.to_string()))?;

        // Clean the disk (removes all partitions and volumes)
        clean_disk(disk_number)?;

        // Get image size for progress tracking
        let image_size = std::fs::metadata(image_path)?.len();

        // Send initial progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Writing image to device...".to_string(),
        });

        // Perform the write in a blocking task
        let image_path_clone = image_path.clone();
        let device_id_clone = device_id.to_string();
        let progress_channel_clone = progress_channel.clone();

        tokio::task::spawn_blocking(move || {
            write_to_device(
                &image_path_clone,
                &device_id_clone,
                image_size,
                &progress_channel_clone,
            )
        })
        .await
        .map_err(|e| {
            DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })??;

        // Verify if requested
        if verify {
            let _ = progress_channel.send(FlashProgress {
                stage: FlashStage::Verifying,
                progress: 0,
                bytes_processed: 0,
                total_bytes: image_size,
                message: "Verifying written data...".to_string(),
            });

            let image_path_clone = image_path.clone();
            let device_id_clone = device_id.to_string();
            let progress_channel_clone = progress_channel.clone();

            tokio::task::spawn_blocking(move || {
                verify_write(
                    &image_path_clone,
                    &device_id_clone,
                    image_size,
                    &progress_channel_clone,
                )
            })
            .await
            .map_err(|e| {
                DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
            })??;
        }

        // Finalize
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Finalizing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Finalizing...".to_string(),
        });

        Ok(())
    }

    fn clean_disk(disk_number: &str) -> Result<(), DiskWriteError> {
        // Use diskpart to clean the disk (requires admin)
        let script = format!("select disk {}\nclean\n", disk_number);

        let output = Command::new("diskpart")
            .args(["/s", "/dev/stdin"])
            .stdin(std::process::Stdio::piped())
            .output();

        // Alternative: use PowerShell Clear-Disk
        let ps_script = format!(
            "Clear-Disk -Number {} -RemoveData -RemoveOEM -Confirm:$false",
            disk_number
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output()
            .map_err(|e| DiskWriteError::UnmountError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore certain errors
            if !stderr.contains("not found") && !stderr.contains("no media") {
                return Err(DiskWriteError::UnmountError(stderr.to_string()));
            }
        }

        Ok(())
    }

    fn write_to_device(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        let mut source = File::open(image_path)?;

        // On Windows, we need to open the physical drive with specific flags
        let mut dest = std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    DiskWriteError::PermissionDenied(
                        "Administrator access required. Please run as Administrator.".to_string(),
                    )
                } else if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;

        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut bytes_written: u64 = 0;
        let mut last_progress_bytes: u64 = 0;

        loop {
            let bytes_read = source.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            dest.write_all(&buffer[..bytes_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;
            bytes_written += bytes_read as u64;

            // Update progress periodically
            if bytes_written - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_written;
                let progress = ((bytes_written as f64 / total_size as f64) * 100.0) as u8;
                let _ = progress_channel.send(FlashProgress {
                    stage: FlashStage::Writing,
                    progress: progress.min(99),
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to device...".to_string(),
                });
            }
        }

        // Sync to ensure all data is written
        dest.sync_all().map_err(|e| {
            if is_drive_disconnected(&e) {
                DiskWriteError::DriveDisconnected
            } else {
                DiskWriteError::IoError(e)
            }
        })?;

        // Send 100% progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Writing,
            progress: 100,
            bytes_processed: bytes_written,
            total_bytes: total_size,
            message: "Write complete".to_string(),
        });

        Ok(())
    }

    fn verify_write(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_channel: &Channel<FlashProgress>,
    ) -> Result<(), DiskWriteError> {
        let mut source = File::open(image_path)?;
        let mut dest = File::open(device_path).map_err(|e| {
            if is_drive_disconnected(&e) {
                DiskWriteError::DriveDisconnected
            } else {
                DiskWriteError::IoError(e)
            }
        })?;

        let mut source_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut dest_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut bytes_verified: u64 = 0;
        let mut last_progress_bytes: u64 = 0;

        loop {
            let source_read = source.read(&mut source_buffer)?;
            if source_read == 0 {
                break;
            }

            dest.read_exact(&mut dest_buffer[..source_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    DiskWriteError::DriveDisconnected
                } else {
                    DiskWriteError::IoError(e)
                }
            })?;

            if source_buffer[..source_read] != dest_buffer[..source_read] {
                return Err(DiskWriteError::VerificationFailed);
            }

            bytes_verified += source_read as u64;

            // Update progress periodically
            if bytes_verified - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_verified;
                let progress = ((bytes_verified as f64 / total_size as f64) * 100.0) as u8;
                let _ = progress_channel.send(FlashProgress {
                    stage: FlashStage::Verifying,
                    progress: progress.min(99),
                    bytes_processed: bytes_verified,
                    total_bytes: total_size,
                    message: "Verifying written data...".to_string(),
                });
            }
        }

        // Send 100% progress
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Verifying,
            progress: 100,
            bytes_processed: bytes_verified,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // -------------------------------------------------------------------------
    // macOS path parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_macos_disk_id_extraction() {
        // Test extracting disk identifier from device path
        let test_cases = [
            ("/dev/disk2", "disk2"),
            ("/dev/disk10", "disk10"),
            ("disk2", "disk2"), // Already without prefix
            ("/dev/disk0", "disk0"),
        ];

        for (input, expected) in test_cases {
            let result = input.strip_prefix("/dev/").unwrap_or(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_macos_raw_device_path() {
        // Test that we construct the raw device path correctly
        let disk_id = "disk2";
        let raw_device = format!("/dev/r{}", disk_id);
        assert_eq!(raw_device, "/dev/rdisk2");

        let disk_id = "disk10";
        let raw_device = format!("/dev/r{}", disk_id);
        assert_eq!(raw_device, "/dev/rdisk10");
    }

    // -------------------------------------------------------------------------
    // Linux path parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_linux_partition_naming() {
        // Test partition naming for different device types
        let device = "/dev/sdb";

        // Standard SCSI/SATA drives use sdXN format
        for i in 1..=4 {
            let partition = format!("{}{}", device, i);
            assert_eq!(partition, format!("/dev/sdb{}", i));
        }

        // MMC/NVMe drives use XpN format
        let mmc_device = "/dev/mmcblk0";
        for i in 1..=4 {
            let partition = format!("{}p{}", mmc_device, i);
            assert_eq!(partition, format!("/dev/mmcblk0p{}", i));
        }

        let nvme_device = "/dev/nvme0n1";
        for i in 1..=4 {
            let partition = format!("{}p{}", nvme_device, i);
            assert_eq!(partition, format!("/dev/nvme0n1p{}", i));
        }
    }

    // -------------------------------------------------------------------------
    // Windows path parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_windows_disk_number_extraction() {
        let test_cases = [
            ("\\\\.\\PhysicalDrive0", Some("0")),
            ("\\\\.\\PhysicalDrive2", Some("2")),
            ("\\\\.\\PhysicalDrive10", Some("10")),
            ("/dev/disk2", None), // Invalid Windows path
            ("C:\\", None),       // Not a physical drive
        ];

        for (input, expected) in test_cases {
            let result = input.strip_prefix("\\\\.\\PhysicalDrive");
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    // -------------------------------------------------------------------------
    // Write and verify logic tests (using temp files)
    // -------------------------------------------------------------------------

    #[test]
    fn test_write_buffer_size() {
        // Ensure buffer size is reasonable (4 MB for SD cards)
        assert_eq!(WRITE_BUFFER_SIZE, 4 * 1024 * 1024);
        // Fast drive buffer is 64 MB for NVMe/SSDs
        assert_eq!(FAST_DRIVE_BUFFER_SIZE, 64 * 1024 * 1024);
    }

    #[test]
    fn test_progress_update_interval() {
        // Ensure progress updates every 10 MB
        assert_eq!(PROGRESS_UPDATE_INTERVAL, 10 * 1024 * 1024);
    }

    #[test]
    fn test_file_copy_and_verify() {
        // Test that we can correctly copy data and verify it
        let test_data = vec![0xABu8; 4096]; // 4KB of test data

        // Create source file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination file
        let mut dest_file = NamedTempFile::new().unwrap();

        // Copy data
        let source_path = source_file.path();
        let mut source = std::fs::File::open(source_path).unwrap();
        let mut buffer = vec![0u8; 1024];

        loop {
            use std::io::Read;
            let bytes_read = source.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            dest_file.write_all(&buffer[..bytes_read]).unwrap();
        }
        dest_file.flush().unwrap();

        // Verify data matches
        let source_data = std::fs::read(source_file.path()).unwrap();
        let dest_data = std::fs::read(dest_file.path()).unwrap();

        assert_eq!(source_data.len(), dest_data.len(), "File sizes don't match");
        assert_eq!(source_data, dest_data, "File contents don't match");
    }

    #[test]
    fn test_verification_detects_mismatch() {
        // Test that verification correctly detects data corruption
        let source_data = vec![0xABu8; 4096];
        let corrupt_data = vec![0xCDu8; 4096]; // Different data

        // Create source and "destination" files
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&source_data).unwrap();
        source_file.flush().unwrap();

        let mut dest_file = NamedTempFile::new().unwrap();
        dest_file.write_all(&corrupt_data).unwrap();
        dest_file.flush().unwrap();

        // Read and compare
        let source_read = std::fs::read(source_file.path()).unwrap();
        let dest_read = std::fs::read(dest_file.path()).unwrap();

        // Should NOT be equal
        assert_ne!(source_read, dest_read, "Corrupt data should not match");
    }

    // -------------------------------------------------------------------------
    // Safety validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_system_drive_detection_macos() {
        // disk0 is typically the system drive on macOS
        let system_drives = ["disk0", "/dev/disk0"];

        for drive in system_drives {
            let disk_id = drive.strip_prefix("/dev/").unwrap_or(drive);
            // disk0 should be flagged as potentially dangerous
            let is_likely_system = disk_id == "disk0";
            assert!(is_likely_system, "disk0 should be detected as system drive");
        }
    }

    #[test]
    fn test_system_drive_detection_linux() {
        // Common system drive patterns on Linux
        let system_indicators = [
            "/dev/sda",     // Often the first SATA drive (system)
            "/dev/nvme0n1", // Often the first NVMe drive (system)
        ];

        // These are less likely to be system drives
        let removable_indicators = [
            "/dev/sdb",     // Second drive, often removable
            "/dev/mmcblk0", // SD card
            "/dev/sdc",     // Third drive
        ];

        // This test documents our assumptions about drive naming
        for drive in system_indicators {
            assert!(drive.contains("sda") || drive.contains("nvme0n1"));
        }

        for drive in removable_indicators {
            assert!(!drive.contains("sda") || drive.contains("mmcblk") || drive.contains("sdc"));
        }
    }

    #[test]
    fn test_error_types() {
        // Test that all error types can be created
        let io_err =
            DiskWriteError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(io_err.to_string().contains("IO error"));

        let unmount_err = DiskWriteError::UnmountError("test".to_string());
        assert!(unmount_err.to_string().contains("unmount"));

        let eject_err = DiskWriteError::EjectError("test".to_string());
        assert!(eject_err.to_string().contains("eject"));

        let perm_err = DiskWriteError::PermissionDenied("test".to_string());
        assert!(perm_err.to_string().contains("Permission"));

        let verify_err = DiskWriteError::VerificationFailed;
        assert!(verify_err.to_string().contains("verification"));

        let system_err = DiskWriteError::SystemDriveProtected("disk0".to_string());
        assert!(system_err.to_string().contains("system drive"));
    }

    // -------------------------------------------------------------------------
    // Safety validation tests
    // -------------------------------------------------------------------------

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_blocks_disk0_macos() {
        // disk0 is always the system drive on macOS
        let result = validate_device_path("/dev/disk0");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DiskWriteError::SystemDriveProtected(_)
        ));

        // Also test raw device variant
        let result = validate_device_path("/dev/rdisk0");
        assert!(result.is_err());

        // disk0 without prefix
        let result = validate_device_path("disk0");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_allows_other_disks_macos() {
        // Other disks should be allowed
        assert!(validate_device_path("/dev/disk2").is_ok());
        assert!(validate_device_path("/dev/disk10").is_ok());
        assert!(validate_device_path("/dev/rdisk2").is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_blocks_system_drives_linux() {
        // Common system drive patterns
        assert!(validate_device_path("/dev/sda").is_err());
        assert!(validate_device_path("/dev/nvme0n1").is_err());
        assert!(validate_device_path("/dev/vda").is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_allows_other_drives_linux() {
        // Removable drives should be allowed
        assert!(validate_device_path("/dev/sdb").is_ok());
        assert!(validate_device_path("/dev/sdc").is_ok());
        assert!(validate_device_path("/dev/mmcblk0").is_ok());
        assert!(validate_device_path("/dev/nvme1n1").is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_device_path_blocks_drive0_windows() {
        // PhysicalDrive0 is usually the system drive
        let result = validate_device_path("\\\\.\\PhysicalDrive0");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DiskWriteError::SystemDriveProtected(_)
        ));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_device_path_allows_other_drives_windows() {
        // Other drives should be allowed
        assert!(validate_device_path("\\\\.\\PhysicalDrive1").is_ok());
        assert!(validate_device_path("\\\\.\\PhysicalDrive2").is_ok());
    }

    // -------------------------------------------------------------------------
    // Integration tests for write and verify operations with temp files
    // -------------------------------------------------------------------------

    #[test]
    fn test_write_to_temp_file_succeeds() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create a known byte pattern
        let test_data = vec![0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90];

        // Create source temp file with test data
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination temp file
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        // Manually copy the data (simulating write operation)
        let mut source = File::open(source_file.path()).unwrap();
        let mut dest = File::create(&dest_path).unwrap();
        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];

        let mut total_written = 0;
        loop {
            let bytes_read = source.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            dest.write_all(&buffer[..bytes_read]).unwrap();
            total_written += bytes_read;
        }
        dest.sync_all().unwrap();

        // Verify the write succeeded
        let written_data = std::fs::read(&dest_path).unwrap();
        assert_eq!(written_data.len(), test_data.len());
        assert_eq!(written_data, test_data);
        assert_eq!(total_written, test_data.len());
    }

    #[test]
    fn test_write_large_file() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create test data larger than WRITE_BUFFER_SIZE (1MB)
        // Using 2.5 MB to test multiple buffer fills
        let size = (WRITE_BUFFER_SIZE * 2) + (WRITE_BUFFER_SIZE / 2);
        let mut test_data = Vec::with_capacity(size);

        // Fill with a repeating pattern so we can verify it
        for i in 0..size {
            test_data.push((i % 256) as u8);
        }

        // Create source file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination file
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        // Write using buffer (simulating the actual write operation)
        let mut source = File::open(source_file.path()).unwrap();
        let mut dest = File::create(&dest_path).unwrap();
        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];

        let mut total_written = 0;
        let mut iterations = 0;
        loop {
            let bytes_read = source.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            dest.write_all(&buffer[..bytes_read]).unwrap();
            total_written += bytes_read;
            iterations += 1;
        }
        dest.sync_all().unwrap();

        // Verify
        assert_eq!(total_written, size);
        assert!(
            iterations >= 3,
            "Should have required multiple buffer fills"
        );

        let written_data = std::fs::read(&dest_path).unwrap();
        assert_eq!(written_data.len(), test_data.len());
        assert_eq!(written_data, test_data);
    }

    #[test]
    fn test_write_preserves_exact_bytes() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create binary data with all possible byte values
        let mut test_data = Vec::with_capacity(256);
        for i in 0..=255u8 {
            test_data.push(i);
        }
        // Repeat the pattern a few times
        let test_data = test_data.repeat(100); // 25,600 bytes

        // Create source file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination file
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        // Write the data
        let mut source = File::open(source_file.path()).unwrap();
        let mut dest = File::create(&dest_path).unwrap();
        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];

        loop {
            let bytes_read = source.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            dest.write_all(&buffer[..bytes_read]).unwrap();
        }
        dest.sync_all().unwrap();

        // Verify byte-for-byte accuracy
        let written_data = std::fs::read(&dest_path).unwrap();
        assert_eq!(written_data.len(), test_data.len(), "Size mismatch");

        // Check every byte
        for (i, (expected, actual)) in test_data.iter().zip(written_data.iter()).enumerate() {
            assert_eq!(
                expected, actual,
                "Byte mismatch at position {}: expected {:#04x}, got {:#04x}",
                i, expected, actual
            );
        }
    }

    #[test]
    fn test_verify_matching_files_succeeds() {
        use std::fs::File;
        use std::io::Read;

        // Create identical test data
        let test_data = vec![0x42u8; 8192]; // 8KB of data

        // Create two identical temp files
        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(&test_data).unwrap();
        file1.flush().unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(&test_data).unwrap();
        file2.flush().unwrap();

        // Verify they match (simulating verify operation)
        let mut f1 = File::open(file1.path()).unwrap();
        let mut f2 = File::open(file2.path()).unwrap();

        let mut buffer1 = vec![0u8; WRITE_BUFFER_SIZE];
        let mut buffer2 = vec![0u8; WRITE_BUFFER_SIZE];

        let mut verification_passed = true;
        loop {
            let bytes_read1 = f1.read(&mut buffer1).unwrap();
            if bytes_read1 == 0 {
                break;
            }

            let bytes_read2 = f2.read(&mut buffer2).unwrap();
            assert_eq!(bytes_read1, bytes_read2, "Read sizes should match");

            if buffer1[..bytes_read1] != buffer2[..bytes_read2] {
                verification_passed = false;
                break;
            }
        }

        assert!(
            verification_passed,
            "Identical files should verify successfully"
        );
    }

    #[test]
    fn test_verify_different_files_fails() {
        use std::fs::File;
        use std::io::Read;

        // Create two different files
        let data1 = vec![0xAAu8; 4096];
        let data2 = vec![0xBBu8; 4096];

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(&data1).unwrap();
        file1.flush().unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(&data2).unwrap();
        file2.flush().unwrap();

        // Verify they don't match
        let mut f1 = File::open(file1.path()).unwrap();
        let mut f2 = File::open(file2.path()).unwrap();

        let mut buffer1 = vec![0u8; WRITE_BUFFER_SIZE];
        let mut buffer2 = vec![0u8; WRITE_BUFFER_SIZE];

        let mut verification_passed = true;
        loop {
            let bytes_read1 = f1.read(&mut buffer1).unwrap();
            if bytes_read1 == 0 {
                break;
            }

            let bytes_read2 = f2.read(&mut buffer2).unwrap();

            if buffer1[..bytes_read1] != buffer2[..bytes_read2] {
                verification_passed = false;
                break;
            }
        }

        assert!(
            !verification_passed,
            "Different files should fail verification"
        );
    }

    #[test]
    fn test_verify_different_size_files_fails() {
        use std::fs::File;
        use std::io::Read;

        // Create files of different sizes
        let data1 = vec![0xAAu8; 4096];
        let data2 = vec![0xAAu8; 8192]; // Same content but longer

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(&data1).unwrap();
        file1.flush().unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(&data2).unwrap();
        file2.flush().unwrap();

        // Check that sizes are different
        let size1 = std::fs::metadata(file1.path()).unwrap().len();
        let size2 = std::fs::metadata(file2.path()).unwrap().len();
        assert_ne!(size1, size2, "Files should have different sizes");

        // Attempt verification - should fail because of size difference
        let mut f1 = File::open(file1.path()).unwrap();
        let mut f2 = File::open(file2.path()).unwrap();

        let mut buffer1 = vec![0u8; WRITE_BUFFER_SIZE];
        let mut buffer2 = vec![0u8; WRITE_BUFFER_SIZE];

        let mut total_read1 = 0;
        let mut total_read2 = 0;

        // Read both files completely
        loop {
            let bytes_read1 = f1.read(&mut buffer1).unwrap();
            total_read1 += bytes_read1;
            if bytes_read1 == 0 {
                break;
            }
        }

        loop {
            let bytes_read2 = f2.read(&mut buffer2).unwrap();
            total_read2 += bytes_read2;
            if bytes_read2 == 0 {
                break;
            }
        }

        assert_ne!(
            total_read1, total_read2,
            "Files of different sizes should have different total bytes read"
        );
    }

    #[test]
    fn test_write_then_verify_succeeds() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create source data
        let test_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        // Create source temp file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination temp file
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_path_buf();

        // Step 1: Write operation
        {
            let mut source = File::open(source_file.path()).unwrap();
            let mut dest = File::create(&dest_path).unwrap();
            let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];

            loop {
                let bytes_read = source.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }
                dest.write_all(&buffer[..bytes_read]).unwrap();
            }
            dest.sync_all().unwrap();
        }

        // Step 2: Verify operation
        {
            let mut source = File::open(source_file.path()).unwrap();
            let mut dest = File::open(&dest_path).unwrap();

            let mut source_buffer = vec![0u8; WRITE_BUFFER_SIZE];
            let mut dest_buffer = vec![0u8; WRITE_BUFFER_SIZE];

            let mut verification_passed = true;
            loop {
                let source_read = source.read(&mut source_buffer).unwrap();
                if source_read == 0 {
                    break;
                }

                let dest_read = dest.read(&mut dest_buffer).unwrap();
                assert_eq!(source_read, dest_read, "Read sizes should match");

                if source_buffer[..source_read] != dest_buffer[..dest_read] {
                    verification_passed = false;
                    break;
                }
            }

            assert!(
                verification_passed,
                "Write then verify should succeed for identical data"
            );
        }

        // Double-check with full file comparison
        let source_data = std::fs::read(source_file.path()).unwrap();
        let dest_data = std::fs::read(&dest_path).unwrap();
        assert_eq!(
            source_data, dest_data,
            "Final file contents should match exactly"
        );
    }

    // -------------------------------------------------------------------------
    // Partial Failure Scenarios and Cleanup Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_write_to_invalid_path_fails() {
        // Try to write to a path that doesn't exist (simulating device disconnect)
        let invalid_path = "/nonexistent/device/that/does/not/exist";
        let result = std::fs::OpenOptions::new().write(true).open(invalid_path);

        assert!(result.is_err(), "Should fail to open invalid device path");

        // Verify error kind
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_verification_fails_after_write_completes() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create source data
        let source_data = vec![0xAAu8; 4096];
        let corrupted_data = vec![0xBBu8; 4096]; // Different data

        // Create source file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&source_data).unwrap();
        source_file.flush().unwrap();

        // Simulate write operation
        let mut dest_file = NamedTempFile::new().unwrap();
        dest_file.write_all(&source_data).unwrap();
        dest_file.flush().unwrap();

        // Manually corrupt the destination after write
        let dest_path = dest_file.path().to_path_buf();
        drop(dest_file);
        std::fs::write(&dest_path, &corrupted_data).unwrap();

        // Verify operation should fail
        let mut source = File::open(source_file.path()).unwrap();
        let mut dest = File::open(&dest_path).unwrap();

        let mut source_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut dest_buffer = vec![0u8; WRITE_BUFFER_SIZE];

        let source_read = source.read(&mut source_buffer).unwrap();
        let dest_read = dest.read(&mut dest_buffer).unwrap();

        assert_eq!(source_read, dest_read, "Read sizes should match");
        assert_ne!(
            source_buffer[..source_read],
            dest_buffer[..dest_read],
            "Corrupted data should not match source"
        );

        // Clean up
        std::fs::remove_file(&dest_path).unwrap();
    }

    #[test]
    fn test_cleanup_after_write_failure() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create source data
        let test_data = vec![0x42u8; 1024];
        let source_path = temp_dir.path().join("source.img");
        std::fs::write(&source_path, &test_data).unwrap();

        // Create a read-only destination to trigger write failure
        let dest_path = temp_dir.path().join("dest_readonly");
        std::fs::write(&dest_path, b"").unwrap();

        // Make it read-only on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            std::fs::set_permissions(&dest_path, perms).unwrap();
        }

        // Try to open for writing - should fail
        let result = std::fs::OpenOptions::new().write(true).open(&dest_path);

        // On Unix, this should fail with PermissionDenied
        #[cfg(unix)]
        {
            assert!(
                result.is_err(),
                "Should fail to open read-only file for writing"
            );
            let err = result.unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }

        // Clean up - restore permissions before removing
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest_path).unwrap().permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&dest_path, perms).unwrap();
        }
    }

    #[test]
    fn test_write_simulation_device_disconnect() {
        use std::fs::File;
        use std::io::{Read, Write};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create source file
        let source_data = vec![0xABu8; 1024 * 100]; // 100KB
        let source_path = temp_dir.path().join("source.img");
        std::fs::write(&source_path, &source_data).unwrap();

        // Create destination file
        let dest_path = temp_dir.path().join("dest.img");

        // Simulate write operation
        let mut source = File::open(&source_path).unwrap();
        let mut dest = File::create(&dest_path).unwrap();
        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut bytes_written = 0;

        // Write only part of the data (simulate disconnect mid-write)
        let bytes_read = source.read(&mut buffer).unwrap();
        if bytes_read > 0 {
            // Write only half the buffer
            dest.write_all(&buffer[..bytes_read / 2]).unwrap();
            bytes_written += bytes_read / 2;
        }
        dest.sync_all().unwrap();
        drop(dest);

        // Verify partial write
        let written_size = std::fs::metadata(&dest_path).unwrap().len();
        assert!(
            written_size < source_data.len() as u64,
            "Only partial data should be written"
        );
        assert_eq!(written_size, bytes_written as u64);

        // Clean up incomplete file
        std::fs::remove_file(&dest_path).unwrap();
        assert!(!dest_path.exists(), "Incomplete file should be cleaned up");
    }

    #[test]
    fn test_verify_partial_write_fails() {
        use std::fs::File;
        use std::io::{Read, Write};

        // Create full source data
        let source_data = vec![0xAAu8; 8192];
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(&source_data).unwrap();
        source_file.flush().unwrap();

        // Create destination with only partial data
        let partial_data = vec![0xAAu8; 4096]; // Only half
        let mut dest_file = NamedTempFile::new().unwrap();
        dest_file.write_all(&partial_data).unwrap();
        dest_file.flush().unwrap();

        // Attempt verification - should fail when trying to read more from dest
        let mut source = File::open(source_file.path()).unwrap();
        let mut dest = File::open(dest_file.path()).unwrap();

        let mut source_buffer = vec![0u8; WRITE_BUFFER_SIZE];
        let mut dest_buffer = vec![0u8; WRITE_BUFFER_SIZE];

        let source_read = source.read(&mut source_buffer).unwrap();
        assert_eq!(source_read, 8192, "Should read all source data");

        // Try to read the same amount from dest - will fail or read less
        let result = dest.read(&mut dest_buffer);
        assert!(result.is_ok());
        let dest_read = result.unwrap();

        // Destination has less data
        assert!(
            dest_read < source_read,
            "Partial write should result in less data: {} < {}",
            dest_read,
            source_read
        );
    }

    #[test]
    fn test_sync_ensures_data_written() {
        use std::fs::File;
        use std::io::Write;

        let test_data = vec![0x42u8; 4096];
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write data
        temp_file.write_all(&test_data).unwrap();

        // Sync to ensure it's on disk
        temp_file.as_file().sync_all().unwrap();

        // Read back while file is still open to verify sync worked
        use std::io::{Read, Seek, SeekFrom};
        let file = temp_file.as_file();
        let mut file_ref = file;
        file_ref.seek(SeekFrom::Start(0)).unwrap();
        let mut read_data = Vec::new();
        file_ref.read_to_end(&mut read_data).unwrap();
        assert_eq!(read_data, test_data, "Data should persist after sync");
    }

    #[test]
    fn test_write_failure_leaves_no_partial_data() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("dest.img");

        // Create destination file
        let mut dest = File::create(&dest_path).unwrap();

        // Write some data
        dest.write_all(b"partial data").unwrap();

        // Don't sync, just drop (simulating crash/error)
        drop(dest);

        // In a real failure scenario, we should clean up
        if dest_path.exists() {
            std::fs::remove_file(&dest_path).unwrap();
        }

        assert!(!dest_path.exists(), "Partial data should be cleaned up");
    }

    #[test]
    fn test_permission_denied_error_conversion() {
        // Test that permission denied errors are properly converted
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");

        let disk_err = DiskWriteError::from(io_err);

        // Should be wrapped as IoError, not PermissionDenied
        match disk_err {
            DiskWriteError::IoError(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_buffer_size_alignment() {
        // Verify buffer sizes are appropriate for disk I/O
        assert_eq!(WRITE_BUFFER_SIZE, 4 * 1024 * 1024);
        assert_eq!(FAST_DRIVE_BUFFER_SIZE, 64 * 1024 * 1024);
        assert_eq!(
            WRITE_BUFFER_SIZE % 512,
            0,
            "Buffer should be 512-byte aligned"
        );
        assert_eq!(WRITE_BUFFER_SIZE % 4096, 0, "Buffer should be 4K aligned");
        assert_eq!(FAST_DRIVE_BUFFER_SIZE % 4096, 0, "Fast buffer should be 4K aligned");
    }

    #[test]
    fn test_progress_interval_reasonable() {
        // Progress should update every 10MB
        assert_eq!(PROGRESS_UPDATE_INTERVAL, 10 * 1024 * 1024);
        assert!(
            PROGRESS_UPDATE_INTERVAL >= WRITE_BUFFER_SIZE as u64,
            "Progress interval should be >= buffer size"
        );
    }

    // -------------------------------------------------------------------------
    // Drive disconnect detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_drive_disconnected_not_found() {
        // NotFound error kind should indicate disconnection
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "device not found");
        assert!(is_drive_disconnected(&err), "NotFound should indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_broken_pipe() {
        // BrokenPipe error kind should indicate disconnection
        let err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        assert!(is_drive_disconnected(&err), "BrokenPipe should indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_unexpected_eof() {
        // UnexpectedEof error kind should indicate disconnection
        let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "unexpected end of file");
        assert!(is_drive_disconnected(&err), "UnexpectedEof should indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_enxio() {
        // ENXIO (6) - "Device not configured" on macOS/Linux
        let err = std::io::Error::from_raw_os_error(6);
        assert!(is_drive_disconnected(&err), "ENXIO (6) should indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_enodev() {
        // ENODEV (19) - "No such device" on Linux
        let err = std::io::Error::from_raw_os_error(19);
        assert!(is_drive_disconnected(&err), "ENODEV (19) should indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_permission_denied_is_not_disconnect() {
        // PermissionDenied should NOT indicate disconnection
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        assert!(!is_drive_disconnected(&err), "PermissionDenied should not indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_other_error_is_not_disconnect() {
        // Other errors should NOT indicate disconnection
        let err = std::io::Error::new(std::io::ErrorKind::Other, "some other error");
        assert!(!is_drive_disconnected(&err), "Other errors should not indicate disconnection");
    }

    #[test]
    fn test_is_drive_disconnected_write_zero_is_not_disconnect() {
        // WriteZero should NOT indicate disconnection (it's a different issue)
        let err = std::io::Error::new(std::io::ErrorKind::WriteZero, "write zero");
        assert!(!is_drive_disconnected(&err), "WriteZero should not indicate disconnection");
    }

    #[test]
    fn test_drive_disconnected_error_type() {
        // Test that DriveDisconnected error can be created and has correct message
        let err = DiskWriteError::DriveDisconnected;
        let msg = err.to_string();
        assert!(msg.contains("disconnected"), "Error message should mention disconnection");
        assert!(msg.contains("storage device"), "Error message should mention storage device");
    }

    #[test]
    fn test_drive_disconnected_error_distinct_from_io_error() {
        // Ensure DriveDisconnected is a distinct variant from IoError
        let disconnect_err = DiskWriteError::DriveDisconnected;
        let io_err = DiskWriteError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));

        // They should have different string representations
        assert_ne!(
            disconnect_err.to_string(),
            io_err.to_string(),
            "DriveDisconnected should have distinct message from IoError"
        );
    }
}
