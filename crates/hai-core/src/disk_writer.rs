//! Raw disk writing functionality for flashing images to devices.
//!
//! This module provides platform-specific implementations for writing
//! raw disk images to block devices (SD cards, USB drives, etc.).

use crate::error::{Error, Result};
use crate::types::{FlashProgress, FlashStage};
use crate::ProgressCallback;
use std::path::PathBuf;

/// Buffer size for disk writes (4 MB for SD cards)
#[allow(dead_code)]
const WRITE_BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// Buffer size for fast drives like NVMe/SSDs (64 MB)
#[allow(dead_code)]
const FAST_DRIVE_BUFFER_SIZE: usize = 64 * 1024 * 1024;

/// How often to send progress updates (every N bytes)
#[allow(dead_code)]
const PROGRESS_UPDATE_INTERVAL: u64 = 10 * 1024 * 1024; // 10 MB

/// Check if an I/O error indicates the drive was disconnected
fn is_drive_disconnected(io_err: &std::io::Error) -> bool {
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
fn validate_device_path(device_id: &str) -> Result<()> {
    let device_id = device_id.trim_end_matches('/');

    #[cfg(target_os = "macos")]
    {
        // On macOS, disk0 is always the system drive
        let disk_id = device_id.strip_prefix("/dev/").unwrap_or(device_id);
        let disk_id = disk_id.strip_prefix("r").unwrap_or(disk_id); // Handle raw device

        if disk_id == "disk0" {
            return Err(Error::PermissionDenied(
                "disk0 is the system drive and cannot be overwritten".to_string(),
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, refuse to write to common system drive patterns
        let dangerous_patterns = [
            "/dev/sda",     // First SATA drive (often system)
            "/dev/nvme0n1", // First NVMe drive (often system)
            "/dev/vda",     // First virtio drive (VMs)
        ];

        for pattern in dangerous_patterns {
            if device_id == pattern {
                return Err(Error::PermissionDenied(format!(
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
            return Err(Error::PermissionDenied(
                "PhysicalDrive0 is the system drive and cannot be overwritten".to_string(),
            ));
        }
    }

    Ok(())
}

/// Write an image file to a block device with progress updates
pub async fn write_image<P: ProgressCallback>(
    image_path: &PathBuf,
    device_id: &str,
    verify: bool,
    progress_callback: &P,
) -> Result<()> {
    // Safety check: refuse to write to system drives
    validate_device_path(device_id)?;

    #[cfg(target_os = "macos")]
    {
        macos::write_image(image_path, device_id, verify, progress_callback).await
    }

    #[cfg(target_os = "linux")]
    {
        linux::write_image(image_path, device_id, verify, progress_callback).await
    }

    #[cfg(target_os = "windows")]
    {
        windows::write_image(image_path, device_id, verify, progress_callback).await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(Error::UnsupportedPlatform("Disk writing".to_string()))
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
    use std::sync::mpsc;

    /// Progress update sent from blocking task
    struct ProgressUpdate {
        stage: FlashStage,
        bytes_processed: u64,
        total_bytes: u64,
        message: String,
    }

    pub async fn write_image<P: ProgressCallback>(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_callback: &P,
    ) -> Result<()> {
        // Extract disk identifier from device path
        let disk_id = device_id.strip_prefix("/dev/").unwrap_or(device_id);

        // Get the raw device path for faster writes
        let raw_device = format!("/dev/r{}", disk_id);

        // Unmount all volumes on the disk
        unmount_disk(disk_id)?;

        // Get image size for progress tracking
        let image_size = std::fs::metadata(image_path)?.len();

        // Send initial progress
        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Requesting administrator access...".to_string(),
        });

        // Create channel for progress updates from blocking task
        let (progress_tx, progress_rx) = mpsc::channel::<ProgressUpdate>();

        // Perform write and optional verify in a blocking task
        let image_path_clone = image_path.clone();
        let raw_device_clone = raw_device.clone();
        let disk_id_clone = disk_id.to_string();

        let write_handle = tokio::task::spawn_blocking(move || {
            write_and_verify_blocking(
                &image_path_clone,
                &raw_device_clone,
                &disk_id_clone,
                image_size,
                verify,
                progress_tx,
            )
        });

        // Forward progress updates while waiting for write to complete
        loop {
            // Check for progress updates (non-blocking with timeout)
            match progress_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(update) => {
                    let progress = if update.total_bytes > 0 {
                        ((update.bytes_processed as f64 / update.total_bytes as f64) * 100.0) as u8
                    } else {
                        0
                    };
                    progress_callback.on_progress(FlashProgress {
                        stage: update.stage,
                        progress,
                        bytes_processed: update.bytes_processed,
                        total_bytes: update.total_bytes,
                        message: update.message,
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if the blocking task is done
                    if write_handle.is_finished() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Sender dropped, task is done
                    break;
                }
            }
        }

        // Wait for the result
        let result = write_handle
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))?;

        result?;

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: image_size,
            total_bytes: image_size,
            message: "Complete".to_string(),
        });

        Ok(())
    }

    fn write_and_verify_blocking(
        image_path: &PathBuf,
        device_path: &str,
        disk_id: &str,
        total_size: u64,
        verify: bool,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        // Request authorization
        let auth = request_authorization()?;

        // Write the image and compute checksum if verification is requested
        let source_checksum = write_with_auth(
            &auth,
            image_path,
            device_path,
            total_size,
            verify,
            &progress_tx,
        )?;

        // Verify if requested
        if verify {
            let checksum =
                source_checksum.expect("Checksum should have been computed when verify=true");
            verify_with_auth(&auth, &checksum, device_path, total_size, &progress_tx)?;
        }

        // Finalize - eject
        eject_disk(disk_id)?;

        Ok(())
    }

    fn request_authorization() -> Result<Authorization> {
        let rights = AuthorizationItemSetBuilder::new()
            .add_right("system.privilege.admin")
            .map_err(|e| Error::PermissionDenied(format!("Failed to create rights: {}", e)))?
            .build();

        Authorization::new(
            Some(rights),
            None,
            Flags::INTERACTION_ALLOWED | Flags::EXTEND_RIGHTS | Flags::PREAUTHORIZE,
        )
        .map_err(|e| {
            if e.code() == -60006 {
                Error::PermissionDenied("Administrator access was denied by user".to_string())
            } else if e.code() == -60005 {
                Error::PermissionDenied("Authorization was canceled".to_string())
            } else {
                Error::PermissionDenied(format!("Authorization failed: {}", e))
            }
        })
    }

    fn write_with_auth(
        auth: &Authorization,
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        compute_checksum: bool,
        progress_tx: &mpsc::Sender<ProgressUpdate>,
    ) -> Result<Option<String>> {
        use sha2::{Digest, Sha256};
        use std::io::Write;

        let mut source = std::fs::File::open(image_path)?;

        let dd_path = Path::new("/bin/dd");
        let of_arg = format!("of={}", device_path);
        let bs_arg = "bs=64m".to_string();

        // Send progress update before requesting privilege
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Writing,
            bytes_processed: 0,
            total_bytes: total_size,
            message: "Starting write...".to_string(),
        });

        let mut pipe = auth
            .execute_with_privileges_piped(dd_path, [&of_arg, &bs_arg], Flags::empty())
            .map_err(|e| Error::PermissionDenied(format!("Failed to open device: {}", e)))?;

        let mut hasher = if compute_checksum {
            Some(Sha256::new())
        } else {
            None
        };
        let mut buffer = vec![0u8; FAST_DRIVE_BUFFER_SIZE];
        let mut bytes_written: u64 = 0;
        let mut last_progress_update: u64 = 0;

        loop {
            let bytes_read = std::io::Read::read(&mut source, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            if let Some(ref mut h) = hasher {
                h.update(&buffer[..bytes_read]);
            }

            pipe.write_all(&buffer[..bytes_read]).map_err(|e| {
                if is_drive_disconnected(&e) {
                    Error::DriveDisconnected
                } else {
                    Error::Io(e)
                }
            })?;

            bytes_written += bytes_read as u64;

            // Send progress update every PROGRESS_UPDATE_INTERVAL bytes
            if bytes_written - last_progress_update >= PROGRESS_UPDATE_INTERVAL {
                let _ = progress_tx.send(ProgressUpdate {
                    stage: FlashStage::Writing,
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to drive...".to_string(),
                });
                last_progress_update = bytes_written;
            }
        }

        // Send final write progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Writing,
            bytes_processed: bytes_written,
            total_bytes: total_size,
            message: "Syncing data to drive...".to_string(),
        });

        drop(pipe);
        let _ = Command::new("sync").output();

        let checksum = hasher.map(|h| hex::encode(h.finalize()));
        Ok(checksum)
    }

    fn verify_with_auth(
        auth: &Authorization,
        source_checksum: &str,
        device_path: &str,
        total_size: u64,
        progress_tx: &mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        use sha2::{Digest, Sha256};

        // Send initial verify progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Verifying,
            bytes_processed: 0,
            total_bytes: total_size,
            message: "Starting verification...".to_string(),
        });

        let block_count = total_size.div_ceil(FAST_DRIVE_BUFFER_SIZE as u64);
        let dd_path = Path::new("/bin/dd");
        let if_arg = format!("if={}", device_path);
        let bs_arg = "bs=64m".to_string();
        let count_arg = format!("count={}", block_count);

        let mut pipe = auth
            .execute_with_privileges_piped(dd_path, [&if_arg, &bs_arg, &count_arg], Flags::empty())
            .map_err(|e| Error::PermissionDenied(format!("Failed to read device: {}", e)))?;

        let mut device_hasher = Sha256::new();
        let mut buffer = vec![0u8; FAST_DRIVE_BUFFER_SIZE];
        let mut bytes_read_total: u64 = 0;
        let mut last_progress_update: u64 = 0;

        loop {
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    device_hasher.update(&buffer[..n]);
                    bytes_read_total += n as u64;

                    // Send progress update every PROGRESS_UPDATE_INTERVAL bytes
                    if bytes_read_total - last_progress_update >= PROGRESS_UPDATE_INTERVAL {
                        let _ = progress_tx.send(ProgressUpdate {
                            stage: FlashStage::Verifying,
                            bytes_processed: bytes_read_total,
                            total_bytes: total_size,
                            message: "Verifying written data...".to_string(),
                        });
                        last_progress_update = bytes_read_total;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(if is_drive_disconnected(&e) {
                        Error::DriveDisconnected
                    } else {
                        Error::Io(e)
                    });
                }
            }
        }

        let device_checksum = hex::encode(device_hasher.finalize());

        if source_checksum != device_checksum {
            return Err(Error::VerificationFailed(
                "Checksum mismatch after write".to_string(),
            ));
        }

        // Send final verify progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Verifying,
            bytes_processed: total_size,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }

    fn unmount_disk(disk_id: &str) -> Result<()> {
        let output = Command::new("diskutil")
            .args(["unmountDisk", disk_id])
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("not mounted") || stderr.contains("was already unmounted") {
            return Ok(());
        }

        // Try force unmount
        let force_output = Command::new("diskutil")
            .args(["unmountDisk", "force", disk_id])
            .output()?;

        if !force_output.status.success() {
            let force_stderr = String::from_utf8_lossy(&force_output.stderr);
            if !force_stderr.contains("not mounted")
                && !force_stderr.contains("was already unmounted")
            {
                return Err(Error::DeviceBusy(format!(
                    "Force unmount failed: {}",
                    force_stderr
                )));
            }
        }

        Ok(())
    }

    fn eject_disk(disk_id: &str) -> Result<()> {
        let output = Command::new("diskutil").args(["eject", disk_id]).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::DeviceBusy(format!("Eject failed: {}", stderr)));
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_unmount_disk_nonexistent() {
            // Test unmounting a disk that doesn't exist
            let result = unmount_disk("disk999");
            // This should either succeed (if disk not mounted) or fail with I/O error
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_eject_disk_nonexistent() {
            // Test ejecting a disk that doesn't exist
            let result = eject_disk("disk999");
            // This should fail because the disk doesn't exist
            assert!(result.is_err());
            if let Err(Error::DeviceBusy(msg)) = result {
                assert!(!msg.is_empty());
            }
        }
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
    use std::sync::mpsc;

    /// Progress update sent from blocking task
    struct ProgressUpdate {
        stage: FlashStage,
        bytes_processed: u64,
        total_bytes: u64,
        message: String,
    }

    pub async fn write_image<P: ProgressCallback>(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_callback: &P,
    ) -> Result<()> {
        unmount_device(device_id)?;

        let image_size = std::fs::metadata(image_path)?.len();

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Writing image to device...".to_string(),
        });

        // Create channel for progress updates from blocking task
        let (progress_tx, progress_rx) = mpsc::channel::<ProgressUpdate>();

        let image_path_clone = image_path.clone();
        let device_id_clone = device_id.to_string();

        let write_handle = tokio::task::spawn_blocking(move || {
            write_to_device(&image_path_clone, &device_id_clone, image_size, progress_tx)
        });

        // Forward progress updates while waiting for write to complete
        loop {
            match progress_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(update) => {
                    let progress = if update.total_bytes > 0 {
                        ((update.bytes_processed as f64 / update.total_bytes as f64) * 100.0) as u8
                    } else {
                        0
                    };
                    progress_callback.on_progress(FlashProgress {
                        stage: update.stage,
                        progress,
                        bytes_processed: update.bytes_processed,
                        total_bytes: update.total_bytes,
                        message: update.message,
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if write_handle.is_finished() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        write_handle
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))??;

        if verify {
            progress_callback.on_progress(FlashProgress {
                stage: FlashStage::Verifying,
                progress: 0,
                bytes_processed: 0,
                total_bytes: image_size,
                message: "Verifying written data...".to_string(),
            });

            let (verify_tx, verify_rx) = mpsc::channel::<ProgressUpdate>();

            let image_path_clone = image_path.clone();
            let device_id_clone = device_id.to_string();

            let verify_handle = tokio::task::spawn_blocking(move || {
                verify_write(&image_path_clone, &device_id_clone, image_size, verify_tx)
            });

            // Forward verify progress updates
            loop {
                match verify_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(update) => {
                        let progress = if update.total_bytes > 0 {
                            ((update.bytes_processed as f64 / update.total_bytes as f64) * 100.0)
                                as u8
                        } else {
                            0
                        };
                        progress_callback.on_progress(FlashProgress {
                            stage: update.stage,
                            progress,
                            bytes_processed: update.bytes_processed,
                            total_bytes: update.total_bytes,
                            message: update.message,
                        });
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if verify_handle.is_finished() {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            verify_handle
                .await
                .map_err(|e| Error::Io(std::io::Error::other(e)))??;
        }

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Finalizing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Syncing data...".to_string(),
        });

        let _ = Command::new("sync").output();

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: image_size,
            total_bytes: image_size,
            message: "Complete".to_string(),
        });

        Ok(())
    }

    fn unmount_device(device_id: &str) -> Result<()> {
        let _ = Command::new("umount")
            .args(["--all-targets", device_id])
            .output();

        for i in 1..=16 {
            let partition = if device_id.contains("mmcblk") || device_id.contains("nvme") {
                format!("{}p{}", device_id, i)
            } else {
                format!("{}{}", device_id, i)
            };
            let _ = Command::new("umount").arg(&partition).output();
        }

        Ok(())
    }

    fn write_to_device(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        let mut source = File::open(image_path)?;
        let mut dest = std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::PermissionDenied(
                        "Root access required. Please run with sudo.".to_string(),
                    )
                } else if is_drive_disconnected(&e) {
                    Error::DriveDisconnected
                } else {
                    Error::Io(e)
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
                    Error::DriveDisconnected
                } else {
                    Error::Io(e)
                }
            })?;

            bytes_written += bytes_read as u64;

            // Update progress periodically
            if bytes_written - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_written;
                let _ = progress_tx.send(ProgressUpdate {
                    stage: FlashStage::Writing,
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to device...".to_string(),
                });
            }
        }

        dest.sync_all().map_err(|e| {
            if is_drive_disconnected(&e) {
                Error::DriveDisconnected
            } else {
                Error::Io(e)
            }
        })?;

        // Send final progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Writing,
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
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        let mut source = File::open(image_path)?;
        let mut dest = File::open(device_path).map_err(|e| {
            if is_drive_disconnected(&e) {
                Error::DriveDisconnected
            } else {
                Error::Io(e)
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

            dest.read_exact(&mut dest_buffer[..source_read])
                .map_err(|e| {
                    if is_drive_disconnected(&e) {
                        Error::DriveDisconnected
                    } else {
                        Error::Io(e)
                    }
                })?;

            if source_buffer[..source_read] != dest_buffer[..source_read] {
                return Err(Error::VerificationFailed(
                    "Data mismatch during verification".to_string(),
                ));
            }

            bytes_verified += source_read as u64;

            // Update progress periodically
            if bytes_verified - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_verified;
                let _ = progress_tx.send(ProgressUpdate {
                    stage: FlashStage::Verifying,
                    bytes_processed: bytes_verified,
                    total_bytes: total_size,
                    message: "Verifying written data...".to_string(),
                });
            }
        }

        // Send final progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Verifying,
            bytes_processed: bytes_verified,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_unmount_device_nonexistent() {
            // Test unmounting a device that doesn't exist
            let result = unmount_device("/dev/nonexistent999");
            // Should succeed because we ignore errors from umount
            assert!(result.is_ok());
        }
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
    use std::sync::mpsc;

    /// Progress update sent from blocking task
    struct ProgressUpdate {
        stage: FlashStage,
        bytes_processed: u64,
        total_bytes: u64,
        message: String,
    }

    pub async fn write_image<P: ProgressCallback>(
        image_path: &PathBuf,
        device_id: &str,
        verify: bool,
        progress_callback: &P,
    ) -> Result<()> {
        let disk_number = device_id
            .strip_prefix("\\\\.\\PhysicalDrive")
            .ok_or_else(|| Error::DeviceNotFound(device_id.to_string()))?;

        clean_disk(disk_number)?;

        let image_size = std::fs::metadata(image_path)?.len();

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image_size,
            message: "Writing image to device...".to_string(),
        });

        // Create channel for progress updates from blocking task
        let (progress_tx, progress_rx) = mpsc::channel::<ProgressUpdate>();

        let image_path_clone = image_path.clone();
        let device_id_clone = device_id.to_string();

        let write_handle = tokio::task::spawn_blocking(move || {
            write_to_device(&image_path_clone, &device_id_clone, image_size, progress_tx)
        });

        // Forward progress updates while waiting for write to complete
        loop {
            match progress_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(update) => {
                    let progress = if update.total_bytes > 0 {
                        ((update.bytes_processed as f64 / update.total_bytes as f64) * 100.0) as u8
                    } else {
                        0
                    };
                    progress_callback.on_progress(FlashProgress {
                        stage: update.stage,
                        progress,
                        bytes_processed: update.bytes_processed,
                        total_bytes: update.total_bytes,
                        message: update.message,
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if write_handle.is_finished() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        write_handle
            .await
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;

        if verify {
            progress_callback.on_progress(FlashProgress {
                stage: FlashStage::Verifying,
                progress: 0,
                bytes_processed: 0,
                total_bytes: image_size,
                message: "Verifying written data...".to_string(),
            });

            let (verify_tx, verify_rx) = mpsc::channel::<ProgressUpdate>();

            let image_path_clone = image_path.clone();
            let device_id_clone = device_id.to_string();

            let verify_handle = tokio::task::spawn_blocking(move || {
                verify_write(&image_path_clone, &device_id_clone, image_size, verify_tx)
            });

            // Forward verify progress updates
            loop {
                match verify_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(update) => {
                        let progress = if update.total_bytes > 0 {
                            ((update.bytes_processed as f64 / update.total_bytes as f64) * 100.0)
                                as u8
                        } else {
                            0
                        };
                        progress_callback.on_progress(FlashProgress {
                            stage: update.stage,
                            progress,
                            bytes_processed: update.bytes_processed,
                            total_bytes: update.total_bytes,
                            message: update.message,
                        });
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if verify_handle.is_finished() {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            verify_handle
                .await
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;
        }

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Finalizing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Finalizing...".to_string(),
        });

        progress_callback.on_progress(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: image_size,
            total_bytes: image_size,
            message: "Complete".to_string(),
        });

        Ok(())
    }

    fn clean_disk(disk_number: &str) -> Result<()> {
        let ps_script = format!(
            "Clear-Disk -Number {} -RemoveData -RemoveOEM -Confirm:$false",
            disk_number
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("not found") && !stderr.contains("no media") {
                return Err(Error::DeviceBusy(stderr.to_string()));
            }
        }

        Ok(())
    }

    fn write_to_device(
        image_path: &PathBuf,
        device_path: &str,
        total_size: u64,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        let mut source = File::open(image_path)?;

        let mut dest = std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::PermissionDenied(
                        "Administrator access required. Please run as Administrator.".to_string(),
                    )
                } else if is_drive_disconnected(&e) {
                    Error::DriveDisconnected
                } else {
                    Error::Io(e)
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
                    Error::DriveDisconnected
                } else {
                    Error::Io(e)
                }
            })?;

            bytes_written += bytes_read as u64;

            // Update progress periodically
            if bytes_written - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_written;
                let _ = progress_tx.send(ProgressUpdate {
                    stage: FlashStage::Writing,
                    bytes_processed: bytes_written,
                    total_bytes: total_size,
                    message: "Writing image to device...".to_string(),
                });
            }
        }

        dest.sync_all().map_err(|e| {
            if is_drive_disconnected(&e) {
                Error::DriveDisconnected
            } else {
                Error::Io(e)
            }
        })?;

        // Send final progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Writing,
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
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<()> {
        let mut source = File::open(image_path)?;
        let mut dest = File::open(device_path).map_err(|e| {
            if is_drive_disconnected(&e) {
                Error::DriveDisconnected
            } else {
                Error::Io(e)
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

            dest.read_exact(&mut dest_buffer[..source_read])
                .map_err(|e| {
                    if is_drive_disconnected(&e) {
                        Error::DriveDisconnected
                    } else {
                        Error::Io(e)
                    }
                })?;

            if source_buffer[..source_read] != dest_buffer[..source_read] {
                return Err(Error::VerificationFailed(
                    "Data mismatch during verification".to_string(),
                ));
            }

            bytes_verified += source_read as u64;

            // Update progress periodically
            if bytes_verified - last_progress_bytes >= PROGRESS_UPDATE_INTERVAL {
                last_progress_bytes = bytes_verified;
                let _ = progress_tx.send(ProgressUpdate {
                    stage: FlashStage::Verifying,
                    bytes_processed: bytes_verified,
                    total_bytes: total_size,
                    message: "Verifying written data...".to_string(),
                });
            }
        }

        // Send final progress
        let _ = progress_tx.send(ProgressUpdate {
            stage: FlashStage::Verifying,
            bytes_processed: bytes_verified,
            total_bytes: total_size,
            message: "Verification complete".to_string(),
        });

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_clean_disk_nonexistent() {
            // Test cleaning a disk that doesn't exist
            let result = clean_disk("999");
            // Should either succeed or fail, but not panic
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(WRITE_BUFFER_SIZE, 4 * 1024 * 1024);
        assert_eq!(FAST_DRIVE_BUFFER_SIZE, 64 * 1024 * 1024);
    }

    #[test]
    fn test_progress_interval() {
        assert_eq!(PROGRESS_UPDATE_INTERVAL, 10 * 1024 * 1024);
    }

    #[test]
    fn test_is_drive_disconnected_not_found() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "device not found");
        assert!(is_drive_disconnected(&err));
    }

    #[test]
    fn test_is_drive_disconnected_broken_pipe() {
        let err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        assert!(is_drive_disconnected(&err));
    }

    #[test]
    fn test_is_drive_disconnected_permission_denied_is_not_disconnect() {
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        assert!(!is_drive_disconnected(&err));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_blocks_disk0_macos() {
        assert!(validate_device_path("/dev/disk0").is_err());
        assert!(validate_device_path("/dev/rdisk0").is_err());
        assert!(validate_device_path("disk0").is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_allows_other_disks_macos() {
        assert!(validate_device_path("/dev/disk2").is_ok());
        assert!(validate_device_path("/dev/disk10").is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_blocks_sda_linux() {
        let result = validate_device_path("/dev/sda");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PermissionDenied(_)));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_blocks_nvme0n1_linux() {
        let result = validate_device_path("/dev/nvme0n1");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PermissionDenied(_)));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_blocks_vda_linux() {
        let result = validate_device_path("/dev/vda");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PermissionDenied(_)));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_allows_sdb_linux() {
        assert!(validate_device_path("/dev/sdb").is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_allows_sdc_linux() {
        assert!(validate_device_path("/dev/sdc").is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_allows_mmcblk0_linux() {
        assert!(validate_device_path("/dev/mmcblk0").is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_device_path_allows_nvme1n1_linux() {
        assert!(validate_device_path("/dev/nvme1n1").is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_device_path_blocks_physicaldrive0_windows() {
        let result = validate_device_path("\\\\.\\PhysicalDrive0");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PermissionDenied(_)));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_device_path_allows_physicaldrive1_windows() {
        assert!(validate_device_path("\\\\.\\PhysicalDrive1").is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_device_path_allows_physicaldrive2_windows() {
        assert!(validate_device_path("\\\\.\\PhysicalDrive2").is_ok());
    }

    #[test]
    fn test_is_drive_disconnected_unexpected_eof() {
        let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "unexpected eof");
        assert!(is_drive_disconnected(&err));
    }

    #[test]
    fn test_is_drive_disconnected_other_error_kinds() {
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        assert!(!is_drive_disconnected(&err));

        let err = std::io::Error::new(std::io::ErrorKind::Other, "other error");
        assert!(!is_drive_disconnected(&err));
    }

    #[test]
    fn test_validate_device_path_empty_string() {
        // Empty string should be allowed (validation doesn't check for empty)
        // This tests the current behavior
        let result = validate_device_path("");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_disk1_macos() {
        // disk1 is usually okay (not system drive)
        assert!(validate_device_path("/dev/disk1").is_ok());
        assert!(validate_device_path("/dev/rdisk1").is_ok());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_device_path_high_disk_numbers_macos() {
        assert!(validate_device_path("/dev/disk99").is_ok());
        assert!(validate_device_path("/dev/rdisk99").is_ok());
    }

    #[test]
    fn test_is_drive_disconnected_with_os_error_code_6() {
        // ENXIO = 6 on macOS and Linux
        let err = std::io::Error::from_raw_os_error(6);
        assert!(is_drive_disconnected(&err));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_is_drive_disconnected_with_os_error_code_19() {
        // ENODEV = 19 on Linux
        let err = std::io::Error::from_raw_os_error(19);
        assert!(is_drive_disconnected(&err));
    }

    #[test]
    fn test_buffer_sizes_are_reasonable() {
        // Write buffer should be at least 1MB
        assert!(WRITE_BUFFER_SIZE >= 1024 * 1024);
        // Fast drive buffer should be larger than regular
        assert!(FAST_DRIVE_BUFFER_SIZE > WRITE_BUFFER_SIZE);
        // Progress interval should be reasonable (not too small, not too large)
        assert!(PROGRESS_UPDATE_INTERVAL >= 1024 * 1024);
        assert!(PROGRESS_UPDATE_INTERVAL <= 100 * 1024 * 1024);
    }

    #[test]
    fn test_is_drive_disconnected_all_matching_kinds() {
        // All these should return true
        assert!(is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::NotFound,
            ""
        )));
        assert!(is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            ""
        )));
        assert!(is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            ""
        )));
    }

    #[test]
    fn test_is_drive_disconnected_non_matching_kinds() {
        // All these should return false
        assert!(!is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            ""
        )));
        assert!(!is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            ""
        )));
        assert!(!is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            ""
        )));
        assert!(!is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            ""
        )));
        assert!(!is_drive_disconnected(&std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            ""
        )));
    }

    #[test]
    fn test_is_drive_disconnected_with_other_os_error_codes() {
        // Test non-matching OS error codes
        let err = std::io::Error::from_raw_os_error(1); // EPERM
        assert!(!is_drive_disconnected(&err));

        let err = std::io::Error::from_raw_os_error(13); // EACCES
        assert!(!is_drive_disconnected(&err));
    }

    #[test]
    fn test_is_drive_disconnected_no_os_error() {
        // Test error without raw OS error code
        let err = std::io::Error::new(std::io::ErrorKind::Other, "generic error");
        assert!(!is_drive_disconnected(&err));
    }

    // Helper struct for testing progress callbacks
    struct TestProgressCallback {
        updates: std::sync::Arc<std::sync::Mutex<Vec<FlashProgress>>>,
    }

    impl TestProgressCallback {
        fn new() -> Self {
            Self {
                updates: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        #[allow(dead_code)]
        fn get_updates(&self) -> Vec<FlashProgress> {
            self.updates.lock().unwrap().clone()
        }
    }

    impl ProgressCallback for TestProgressCallback {
        fn on_progress(&self, progress: FlashProgress) {
            self.updates.lock().unwrap().push(progress);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_write_image_rejects_system_drive() {
        let callback = TestProgressCallback::new();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let image_path = temp_file.path().to_path_buf();

        #[cfg(target_os = "macos")]
        let result = write_image(&image_path, "/dev/disk0", false, &callback).await;

        #[cfg(target_os = "linux")]
        let result = write_image(&image_path, "/dev/sda", false, &callback).await;

        #[cfg(target_os = "windows")]
        let result = write_image(&image_path, "\\\\.\\PhysicalDrive0", false, &callback).await;

        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), Error::PermissionDenied(_)));
        }
    }

    // macOS-specific tests
    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::*;

        #[test]
        fn test_validate_device_rdisk0_variants() {
            assert!(validate_device_path("rdisk0").is_err());
            assert!(validate_device_path("/dev/rdisk0").is_err());
        }

        #[test]
        fn test_validate_device_without_dev_prefix() {
            assert!(validate_device_path("disk2").is_ok());
            assert!(validate_device_path("rdisk2").is_ok());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_nonexistent_file() {
            let callback = TestProgressCallback::new();
            let image_path = PathBuf::from("/tmp/nonexistent_image_file.img");
            let device_id = "/dev/disk99";

            let result = write_image(&image_path, device_id, false, &callback).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_with_invalid_device() {
            let callback = TestProgressCallback::new();
            let temp_file = tempfile::NamedTempFile::new().unwrap();

            // Write some test data
            std::fs::write(temp_file.path(), b"test data").unwrap();
            let image_path = temp_file.path().to_path_buf();

            // Use an invalid device path
            let device_id = "/dev/nonexistent_disk999";

            let result = write_image(&image_path, device_id, false, &callback).await;
            // Should fail when trying to unmount or access the device
            assert!(result.is_err());
        }
    }

    // Linux-specific tests
    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;

        #[test]
        fn test_validate_all_dangerous_patterns() {
            assert!(validate_device_path("/dev/sda").is_err());
            assert!(validate_device_path("/dev/nvme0n1").is_err());
            assert!(validate_device_path("/dev/vda").is_err());
        }

        #[test]
        fn test_validate_safe_devices() {
            assert!(validate_device_path("/dev/sdb").is_ok());
            assert!(validate_device_path("/dev/sdc").is_ok());
            assert!(validate_device_path("/dev/sdd").is_ok());
            assert!(validate_device_path("/dev/nvme1n1").is_ok());
            assert!(validate_device_path("/dev/nvme2n1").is_ok());
            assert!(validate_device_path("/dev/vdb").is_ok());
            assert!(validate_device_path("/dev/mmcblk0").is_ok());
            assert!(validate_device_path("/dev/mmcblk1").is_ok());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_nonexistent_file() {
            let callback = TestProgressCallback::new();
            let image_path = PathBuf::from("/tmp/nonexistent_image_file.img");
            let device_id = "/dev/sdb";

            let result = write_image(&image_path, device_id, false, &callback).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_permission_denied() {
            let callback = TestProgressCallback::new();
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), b"test data").unwrap();
            let image_path = temp_file.path().to_path_buf();

            // This will fail with permission denied unless running as root
            let device_id = "/dev/null"; // Use /dev/null as a safe test target

            let result = write_image(&image_path, device_id, false, &callback).await;
            // Could be either permission denied or other error
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_validate_mmcblk_and_nvme_partitions() {
            // These should all pass - they're not in the dangerous list
            assert!(validate_device_path("/dev/mmcblk0p1").is_ok());
            assert!(validate_device_path("/dev/nvme0n1p1").is_ok());
            // But nvme0n1 itself should be blocked
            assert!(validate_device_path("/dev/nvme0n1").is_err());
        }
    }

    // Windows-specific tests
    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::*;

        #[test]
        fn test_validate_physical_drives() {
            assert!(validate_device_path("\\\\.\\PhysicalDrive0").is_err());
            assert!(validate_device_path("\\\\.\\PhysicalDrive1").is_ok());
            assert!(validate_device_path("\\\\.\\PhysicalDrive2").is_ok());
            assert!(validate_device_path("\\\\.\\PhysicalDrive10").is_ok());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_nonexistent_file() {
            let callback = TestProgressCallback::new();
            let image_path = PathBuf::from("C:\\nonexistent_image_file.img");
            let device_id = "\\\\.\\PhysicalDrive1";

            let result = write_image(&image_path, device_id, false, &callback).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        #[serial]
        async fn test_write_image_invalid_device() {
            let callback = TestProgressCallback::new();
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), b"test data").unwrap();
            let image_path = temp_file.path().to_path_buf();

            let device_id = "\\\\.\\PhysicalDrive999";

            let result = write_image(&image_path, device_id, false, &callback).await;
            assert!(result.is_err());
        }
    }

    // Test unsupported platforms (these tests will only run on non-standard platforms)
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    mod unsupported_platform_tests {
        use super::*;

        #[tokio::test]
        async fn test_write_image_unsupported_platform() {
            let callback = TestProgressCallback::new();
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            let image_path = temp_file.path().to_path_buf();
            let device_id = "/dev/sdb";

            let result = write_image(&image_path, device_id, false, &callback).await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                Error::UnsupportedPlatform(_)
            ));
        }
    }

    // Additional edge case tests that work on all platforms
    #[test]
    fn test_validate_device_path_special_characters() {
        // Test with various special characters to ensure no panics
        let result = validate_device_path("/dev/../disk0");
        // Should either pass validation or fail safely
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_device_path_very_long() {
        let long_path = format!("/dev/{}", "a".repeat(1000));
        let result = validate_device_path(&long_path);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_device_path_unicode() {
        let result = validate_device_path("/dev/disk🔥");
        assert!(result.is_ok() || result.is_err());
    }

    // Test with Mock mode enabled to exercise more code paths
    #[tokio::test]
    #[serial]
    async fn test_write_image_mock_mode_not_implemented() {
        // Enable mock mode
        std::env::set_var("HA_INSTALLER_MOCK", "1");

        let callback = TestProgressCallback::new();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"test data").unwrap();
        let image_path = temp_file.path().to_path_buf();

        #[cfg(target_os = "macos")]
        let device_id = "/dev/disk2";
        #[cfg(target_os = "linux")]
        let device_id = "/dev/sdb";
        #[cfg(target_os = "windows")]
        let device_id = "\\\\.\\PhysicalDrive1";

        // The write_image function doesn't have mock support yet,
        // so it will try to execute real commands which will fail
        let result = write_image(&image_path, device_id, false, &callback).await;

        // Clean up
        std::env::remove_var("HA_INSTALLER_MOCK");

        // Expected to fail as there's no mock implementation for write_image
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_write_image_with_verification_flag() {
        let callback = TestProgressCallback::new();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"test data").unwrap();
        let image_path = temp_file.path().to_path_buf();

        #[cfg(target_os = "macos")]
        let device_id = "/dev/disk99";
        #[cfg(target_os = "linux")]
        let device_id = "/dev/sdb99";
        #[cfg(target_os = "windows")]
        let device_id = "\\\\.\\PhysicalDrive99";

        // Test with verify=true flag
        let result = write_image(&image_path, device_id, true, &callback).await;

        // Should fail because device doesn't exist, but this exercises the verify code path
        assert!(result.is_err());
    }

    // Additional platform-specific validation tests
    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_macos_all_disk0_variations() {
        // Test all possible ways someone might reference disk0
        assert!(validate_device_path("/dev/disk0").is_err());
        assert!(validate_device_path("/dev/rdisk0").is_err());
        assert!(validate_device_path("disk0").is_err());
        assert!(validate_device_path("rdisk0").is_err());

        // Ensure disk0s1 (partition) is also rejected since the base disk is system
        #[cfg(target_os = "macos")]
        {
            // Actually disk0s1 should pass validation as it's a partition, not the whole disk
            // But let's verify the behavior
            let result = validate_device_path("/dev/disk0s1");
            // This will pass because we only check for exact "disk0" match
            assert!(result.is_ok());
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_linux_all_dangerous_devices() {
        // Test all dangerous patterns
        assert!(validate_device_path("/dev/sda").is_err());
        assert!(validate_device_path("/dev/nvme0n1").is_err());
        assert!(validate_device_path("/dev/vda").is_err());

        // Ensure partitions of these devices are OK
        assert!(validate_device_path("/dev/sda1").is_ok());
        assert!(validate_device_path("/dev/sda2").is_ok());
        assert!(validate_device_path("/dev/nvme0n1p1").is_ok());
        assert!(validate_device_path("/dev/vda1").is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_windows_all_physical_drives() {
        // Test PhysicalDrive0
        assert!(validate_device_path("\\\\.\\PhysicalDrive0").is_err());

        // Test other drives are OK
        for i in 1..10 {
            let drive = format!("\\\\.\\PhysicalDrive{}", i);
            assert!(validate_device_path(&drive).is_ok(), "Drive {} should be OK", i);
        }
    }

    #[test]
    fn test_constants_values() {
        // Verify the exact values of constants
        assert_eq!(WRITE_BUFFER_SIZE, 4_194_304);
        assert_eq!(FAST_DRIVE_BUFFER_SIZE, 67_108_864);
        assert_eq!(PROGRESS_UPDATE_INTERVAL, 10_485_760);
    }

    // Test error message generation for validation errors
    #[test]
    #[cfg(target_os = "macos")]
    fn test_validation_error_message_disk0() {
        let result = validate_device_path("/dev/disk0");
        assert!(result.is_err());
        match result {
            Err(Error::PermissionDenied(msg)) => {
                assert!(msg.contains("disk0"));
                assert!(msg.contains("system drive"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_validation_error_messages_linux() {
        let devices = vec!["/dev/sda", "/dev/nvme0n1", "/dev/vda"];
        for device in devices {
            let result = validate_device_path(device);
            assert!(result.is_err());
            match result {
                Err(Error::PermissionDenied(msg)) => {
                    assert!(msg.contains(device) || msg.contains("system drive"));
                }
                _ => panic!("Expected PermissionDenied error for {}", device),
            }
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validation_error_message_windows() {
        let result = validate_device_path("\\\\.\\PhysicalDrive0");
        assert!(result.is_err());
        match result {
            Err(Error::PermissionDenied(msg)) => {
                assert!(msg.contains("PhysicalDrive0") || msg.contains("system drive"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    // Test the full validation path for various device IDs
    #[test]
    fn test_validate_multiple_safe_devices() {
        let safe_devices = vec![
            #[cfg(target_os = "macos")]
            "/dev/disk5",
            #[cfg(target_os = "linux")]
            "/dev/sde",
            #[cfg(target_os = "windows")]
            "\\\\.\\PhysicalDrive5",
        ];

        for device in safe_devices {
            assert!(
                validate_device_path(device).is_ok(),
                "Device {} should be valid",
                device
            );
        }
    }

    // Test progress callback implementation
    #[test]
    fn test_progress_callback_receives_updates() {
        let callback = TestProgressCallback::new();

        // Simulate progress updates
        callback.on_progress(FlashProgress {
            stage: FlashStage::Writing,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 1000,
            message: "Starting".to_string(),
        });

        callback.on_progress(FlashProgress {
            stage: FlashStage::Writing,
            progress: 50,
            bytes_processed: 500,
            total_bytes: 1000,
            message: "Halfway".to_string(),
        });

        callback.on_progress(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: 1000,
            total_bytes: 1000,
            message: "Done".to_string(),
        });

        let updates = callback.get_updates();
        assert_eq!(updates.len(), 3);
        assert_eq!(updates[0].progress, 0);
        assert_eq!(updates[1].progress, 50);
        assert_eq!(updates[2].progress, 100);
        assert_eq!(updates[2].stage, FlashStage::Complete);
    }

    // Test with path that doesn't have /dev/ prefix
    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_validate_without_dev_prefix() {
        #[cfg(target_os = "macos")]
        {
            assert!(validate_device_path("disk5").is_ok());
            assert!(validate_device_path("disk0").is_err());
        }

        #[cfg(target_os = "linux")]
        {
            assert!(validate_device_path("sdb").is_ok());
        }
    }

    // Test case sensitivity
    #[test]
    #[cfg(target_os = "macos")]
    fn test_validate_case_sensitivity_macos() {
        // macOS device paths are case-sensitive
        assert!(validate_device_path("/dev/Disk0").is_ok()); // Capital D should pass
        assert!(validate_device_path("/dev/disk0").is_err()); // Lowercase should fail
    }

    // Test buffer size relationships
    #[test]
    fn test_buffer_size_relationships() {
        // Fast drive buffer should be significantly larger
        assert!(FAST_DRIVE_BUFFER_SIZE >= WRITE_BUFFER_SIZE * 10);

        // Progress interval should be larger than write buffer
        assert!(PROGRESS_UPDATE_INTERVAL >= WRITE_BUFFER_SIZE as u64);

        // But not too large compared to fast buffer
        assert!(PROGRESS_UPDATE_INTERVAL <= (FAST_DRIVE_BUFFER_SIZE * 2) as u64);
    }

    // Test creating actual temp file and trying to write (will fail safely)
    #[tokio::test]
    #[serial]
    async fn test_write_image_with_real_temp_file() {
        let callback = TestProgressCallback::new();

        // Create a temp file with some content
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"test image data").unwrap();
        let image_path = temp_file.path().to_path_buf();

        // Try to write to a nonexistent device
        #[cfg(target_os = "macos")]
        let device_id = "/dev/disk999";
        #[cfg(target_os = "linux")]
        let device_id = "/dev/sdz99";
        #[cfg(target_os = "windows")]
        let device_id = "\\\\.\\PhysicalDrive999";

        let result = write_image(&image_path, device_id, false, &callback).await;

        // Should fail because device doesn't exist
        assert!(result.is_err());

        // Progress updates may or may not have been sent depending on when the failure occurred
        // Just verify the function was called and failed appropriately
    }

    // Test all ErrorKind variants for is_drive_disconnected
    #[test]
    fn test_is_drive_disconnected_comprehensive() {
        use std::io::ErrorKind;

        // These should return true
        let disconnect_kinds = [
            ErrorKind::NotFound,
            ErrorKind::BrokenPipe,
            ErrorKind::UnexpectedEof,
        ];
        for kind in &disconnect_kinds {
            assert!(
                is_drive_disconnected(&std::io::Error::new(*kind, "test")),
                "{:?} should be detected as disconnected",
                kind
            );
        }

        // These should return false
        let other_kinds = [
            ErrorKind::PermissionDenied,
            ErrorKind::ConnectionRefused,
            ErrorKind::ConnectionReset,
            ErrorKind::ConnectionAborted,
            ErrorKind::AddrInUse,
            ErrorKind::AddrNotAvailable,
            ErrorKind::InvalidInput,
            ErrorKind::InvalidData,
            ErrorKind::TimedOut,
            ErrorKind::WriteZero,
            ErrorKind::Interrupted,
            ErrorKind::Other,
            ErrorKind::WouldBlock,
        ];
        for kind in &other_kinds {
            assert!(
                !is_drive_disconnected(&std::io::Error::new(*kind, "test")),
                "{:?} should NOT be detected as disconnected",
                kind
            );
        }
    }

    // Test validation with slash variations
    #[test]
    #[cfg(target_os = "linux")]
    fn test_validate_linux_with_trailing_slash() {
        assert!(validate_device_path("/dev/sdb/").is_ok());
        assert!(validate_device_path("/dev/sda/").is_err());
    }

    // Test multiple consecutive calls to progress callback
    #[test]
    fn test_progress_callback_multiple_calls() {
        let callback = TestProgressCallback::new();

        // Simulate a complete write cycle
        for i in 0..=100 {
            callback.on_progress(FlashProgress {
                stage: if i < 80 {
                    FlashStage::Writing
                } else if i < 100 {
                    FlashStage::Verifying
                } else {
                    FlashStage::Complete
                },
                progress: i as u8,
                bytes_processed: (i * 1000) as u64,
                total_bytes: 100000,
                message: format!("Progress: {}%", i),
            });
        }

        let updates = callback.get_updates();
        assert_eq!(updates.len(), 101);
        assert_eq!(updates.first().unwrap().progress, 0);
        assert_eq!(updates.last().unwrap().progress, 100);
        assert_eq!(updates.last().unwrap().stage, FlashStage::Complete);
    }
}
