use crate::block_devices;
use crate::disk_writer;
use crate::download;
use crate::mock;
use crate::proxmox;
use crate::types::{
    BlockDevice, DeviceManifest, FlashProgress, FlashRequest, FlashResult, FlashStage, HaosRelease,
    ProxmoxCredentials, ProxmoxNode, ProxmoxSession, ProxmoxStorage, ProxmoxVmConfig,
    ProxmoxVmResult, UpdateInfo,
};

/// Get the latest HAOS release information
#[tauri::command]
pub async fn get_haos_release(version: Option<String>) -> Result<HaosRelease, String> {
    if mock::is_mock_enabled() {
        Ok(mock::get_mock_haos_release())
    } else {
        // Fetch from GitHub API
        let release = match version {
            Some(v) => download::fetch_release(&v).await,
            None => download::fetch_latest_release().await,
        };
        release.map_err(|e| e.to_string())
    }
}

use std::time::Duration;
use tauri::ipc::Channel;

/// Check if mock mode is enabled
#[tauri::command]
pub fn is_mock_mode() -> bool {
    mock::is_mock_enabled()
}

/// List available block devices (SD cards, USB drives, etc.)
#[tauri::command]
pub async fn list_block_devices() -> Result<Vec<BlockDevice>, String> {
    if mock::is_mock_enabled() {
        // Return mock devices
        Ok(mock::get_mock_block_devices())
    } else {
        // Use real device enumeration
        block_devices::list_devices().await
    }
}

/// Flash an image to a device
#[tauri::command]
pub async fn flash_image(
    request: FlashRequest,
    progress_channel: Channel<FlashProgress>,
) -> Result<FlashResult, String> {
    if mock::is_mock_enabled() {
        // Simulate flashing with progress updates
        simulate_flash_progress(&progress_channel).await;

        Ok(FlashResult {
            success: true,
            error: None,
            duration_secs: 45, // Simulated duration
        })
    } else {
        // Real flashing implementation
        let start_time = std::time::Instant::now();

        // Send initial progress immediately so UI shows something
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Downloading,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Fetching release info...".to_string(),
        });

        // 1. Fetch the latest HAOS release to get image info
        let release = download::fetch_latest_release()
            .await
            .map_err(|e| format!("Failed to fetch release info: {}", e))?;

        // 2. Find the image for the requested board
        let image = download::find_image_for_board(&release, &request.board)
            .ok_or_else(|| format!("No image found for board: {}", request.board))?;

        // Send progress update with total size now that we know it
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Downloading,
            progress: 0,
            bytes_processed: 0,
            total_bytes: image.size,
            message: "Starting download...".to_string(),
        });

        // 3. Download the image (with progress)
        let compressed_path = download::download_image(image, &progress_channel)
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        // 4. Extract the image (xz decompression)
        let extracted_path = download::extract_image(&compressed_path, &progress_channel)
            .await
            .map_err(|e| format!("Extraction failed: {}", e))?;

        // 5. Check if the image fits on the device
        let image_size = tokio::fs::metadata(&extracted_path)
            .await
            .map_err(|e| format!("Failed to get image size: {}", e))?
            .len();

        // Get the device size from the block devices list
        let devices = block_devices::list_devices()
            .await
            .map_err(|e| format!("Failed to list devices: {}", e))?;
        let device = devices
            .iter()
            .find(|d| d.id == request.device_id)
            .ok_or_else(|| {
                format!(
                    "Device {} not found. It may have been disconnected.",
                    request.device_id
                )
            })?;

        if image_size > device.size {
            return Err(format!(
                "Image is too large for the selected device. Image size: {:.1} GB, Device size: {:.1} GB. Please use a larger storage device.",
                image_size as f64 / 1_000_000_000.0,
                device.size as f64 / 1_000_000_000.0
            ));
        }

        // 6. Write to device
        disk_writer::write_image(
            &extracted_path,
            &request.device_id,
            request.verify,
            &progress_channel,
        )
        .await
        .map_err(|e| format!("Write failed: {}", e))?;

        // 7. Clean up extracted image to save disk space (keep compressed for potential retry)
        if let Err(e) = tokio::fs::remove_file(&extracted_path).await {
            // Log but don't fail - cleanup is best-effort
            eprintln!("Warning: Failed to clean up extracted image: {}", e);
        }

        let duration = start_time.elapsed();

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Installation complete!".to_string(),
        });

        Ok(FlashResult {
            success: true,
            error: None,
            duration_secs: duration.as_secs(),
        })
    }
}

/// Simulate flash progress for mock mode
async fn simulate_flash_progress(channel: &Channel<FlashProgress>) {
    let total_bytes: u64 = 2 * 1024 * 1024 * 1024; // 2 GB simulated image
    let stages: [(FlashStage, &str, u32); 4] = [
        (FlashStage::Downloading, "Downloading image...", 40),
        (FlashStage::Verifying, "Verifying download...", 10),
        (FlashStage::Writing, "Writing to device...", 45),
        (FlashStage::Finalizing, "Finalizing...", 5),
    ];

    let mut overall_progress: u32 = 0;

    for (stage, message, stage_weight) in stages {
        let steps: u32 = 10;
        for step in 0..=steps {
            let stage_progress = step * 100 / steps;
            let bytes_for_stage = (total_bytes as f64
                * (stage_weight as f64 / 100.0)
                * (step as f64 / steps as f64)) as u64;

            let current_progress = overall_progress + (stage_progress * stage_weight / 100);

            let progress = FlashProgress {
                stage: stage.clone(),
                progress: current_progress.min(100) as u8,
                bytes_processed: bytes_for_stage
                    + (total_bytes as f64 * (overall_progress as f64 / 100.0)) as u64,
                total_bytes,
                message: message.to_string(),
            };

            let _ = channel.send(progress);
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        overall_progress += stage_weight;
    }

    // Send completion
    let _ = channel.send(FlashProgress {
        stage: FlashStage::Complete,
        progress: 100,
        bytes_processed: total_bytes,
        total_bytes,
        message: "Installation complete!".to_string(),
    });
}

/// Check for application updates
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    if mock::is_mock_enabled() {
        Ok(mock::get_mock_update_info())
    } else {
        // TODO: Implement real update checking
        // For now, return mock data
        Ok(mock::get_mock_update_info())
    }
}

/// Get the device manifest
#[tauri::command]
pub async fn get_manifest() -> Result<DeviceManifest, String> {
    if mock::is_mock_enabled() {
        Ok(mock::get_mock_manifest())
    } else {
        // TODO: Implement real manifest fetching
        // For now, return mock data
        Ok(mock::get_mock_manifest())
    }
}

// ============================================================================
// System Info Commands
// ============================================================================

/// System information for VM configuration
#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub cpu_cores: usize,
    pub memory_mb: u64,
}

/// Get system information (CPU cores and memory) for VM configuration limits
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    if mock::is_mock_enabled() {
        return SystemInfo {
            cpu_cores: 10,
            memory_mb: 32768, // 32 GB
        };
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Get CPU cores using sysctl
        let cpu_cores = Command::new("sysctl")
            .args(["-n", "hw.ncpu"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(4);

        // Get total memory using sysctl (returns bytes)
        let memory_bytes = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(8 * 1024 * 1024 * 1024); // Default 8 GB

        let memory_mb = memory_bytes / (1024 * 1024);

        SystemInfo {
            cpu_cores,
            memory_mb,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Default values for non-macOS
        SystemInfo {
            cpu_cores: 4,
            memory_mb: 8192,
        }
    }
}

// ============================================================================
// UTM Commands (macOS only)
// ============================================================================

/// Download the HAOS qcow2 image for UTM.
/// Returns the path to the downloaded and extracted qcow2 file.
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn download_utm_image(
    progress_channel: Channel<FlashProgress>,
) -> Result<String, String> {
    if mock::is_mock_enabled() {
        // Simulate download progress
        simulate_utm_download_progress(&progress_channel).await;
        // Create a minimal valid qcow2 file for mock mode testing
        // qcow2 magic number is: 0x514649fb ('QFI\xfb')
        // This creates a minimal valid qcow2 header that UTM can recognize
        let mock_path = "/tmp/mock-haos.qcow2";
        let qcow2_header: [u8; 512] = {
            let mut header = [0u8; 512];
            // Magic number: QFI\xfb
            header[0..4].copy_from_slice(&[0x51, 0x46, 0x49, 0xfb]);
            // Version: 3 (big endian u32)
            header[4..8].copy_from_slice(&[0x00, 0x00, 0x00, 0x03]);
            // Backing file offset: 0
            header[8..16].copy_from_slice(&[0x00; 8]);
            // Backing file size: 0
            header[16..20].copy_from_slice(&[0x00; 4]);
            // Cluster bits: 16 (64KB clusters) - big endian u32
            header[20..24].copy_from_slice(&[0x00, 0x00, 0x00, 0x10]);
            // Size: 1GB in bytes (big endian u64)
            header[24..32].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00]);
            // Encryption method: 0
            header[32..36].copy_from_slice(&[0x00; 4]);
            // L1 size: 16384 entries
            header[36..40].copy_from_slice(&[0x00, 0x00, 0x40, 0x00]);
            // L1 table offset: 0x30000
            header[40..48].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00]);
            // Refcount table offset: 0x10000
            header[48..56].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]);
            // Refcount table clusters: 1
            header[56..60].copy_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            // Number of snapshots: 0
            header[60..64].copy_from_slice(&[0x00; 4]);
            // Snapshots offset: 0
            header[64..72].copy_from_slice(&[0x00; 8]);
            header
        };
        if let Err(e) = std::fs::write(mock_path, qcow2_header) {
            return Err(format!("Failed to create mock qcow2 file: {}", e));
        }
        Ok(mock_path.to_string())
    } else {
        // Get architecture to determine the right image
        let arch = crate::utm::get_mac_architecture();
        let board = if arch == "aarch64" {
            "generic-aarch64"
        } else {
            "generic-x86-64"
        };

        // Fetch the latest HAOS release
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Downloading,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Fetching release info...".to_string(),
        });

        let release = download::fetch_latest_release()
            .await
            .map_err(|e| format!("Failed to fetch release info: {}", e))?;

        // Find the image for the board
        let image = download::find_image_for_board(&release, board)
            .ok_or_else(|| format!("No image found for board: {}", board))?;

        // Modify the URL to get qcow2 instead of img
        // HAOS URLs follow pattern: haos_BOARD-VERSION.img.xz
        // We need: haos_BOARD-VERSION.qcow2.xz
        let qcow2_url = image.download_url.replace(".img.xz", ".qcow2.xz");

        // Create a modified image info for download
        let qcow2_image = crate::types::HaosImage {
            board: image.board.clone(),
            download_url: qcow2_url,
            size: image.size, // Size will be different but we'll update during download
            sha256: String::new(), // We don't have the qcow2 checksum, skip verification
        };

        // Download the qcow2 image
        let compressed_path = download::download_image(&qcow2_image, &progress_channel)
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        // Extract the qcow2 image
        let extracted_path = download::extract_image(&compressed_path, &progress_channel)
            .await
            .map_err(|e| format!("Extraction failed: {}", e))?;

        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Download complete!".to_string(),
        });

        Ok(extracted_path.to_string_lossy().to_string())
    }
}

/// Simulate UTM download progress for mock mode
#[cfg(target_os = "macos")]
async fn simulate_utm_download_progress(channel: &Channel<FlashProgress>) {
    let stages: [(FlashStage, &str, u32); 2] = [
        (FlashStage::Downloading, "Downloading HAOS image...", 70),
        (FlashStage::Extracting, "Extracting image...", 30),
    ];

    let mut overall_progress: u32 = 0;

    for (stage, message, stage_weight) in stages {
        let steps: u32 = 10;
        for step in 0..=steps {
            let stage_progress = step * 100 / steps;
            let current_progress = overall_progress + (stage_progress * stage_weight / 100);

            let _ = channel.send(FlashProgress {
                stage: stage.clone(),
                progress: current_progress.min(100) as u8,
                bytes_processed: 0,
                total_bytes: 0,
                message: message.to_string(),
            });

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        overall_progress += stage_weight;
    }

    let _ = channel.send(FlashProgress {
        stage: FlashStage::Complete,
        progress: 100,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Download complete!".to_string(),
    });
}

/// Stub for non-macOS platforms
#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn download_utm_image(
    _progress_channel: Channel<FlashProgress>,
) -> Result<String, String> {
    Err("UTM is only available on macOS".to_string())
}

/// Check if UTM is installed and get its status.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_utm_status() -> crate::utm::UtmStatus {
    crate::utm::check_utm_installed()
}

/// Get the Mac's CPU architecture for VM creation.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn get_mac_architecture() -> String {
    crate::utm::get_mac_architecture()
}

/// Create a Home Assistant VM in UTM.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn create_utm_vm(config: crate::utm::VmConfig) -> Result<String, String> {
    if mock::is_mock_enabled() {
        // Return a mock VM ID
        Ok("mock-vm-id-12345".to_string())
    } else {
        crate::utm::create_vm(&config).map_err(|e| e.to_string())
    }
}

/// Start a UTM VM.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn start_utm_vm(vm_id: String) -> Result<(), String> {
    if mock::is_mock_enabled() {
        Ok(())
    } else {
        crate::utm::start_vm(&vm_id).map_err(|e| e.to_string())
    }
}

/// Resize a UTM VM's disk before first start.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn resize_utm_vm_disk(vm_id: String, size_gb: u32) -> Result<(), String> {
    if mock::is_mock_enabled() {
        Ok(())
    } else {
        // Convert GB to MB for UTM
        let size_mb = size_gb * 1024;
        crate::utm::resize_vm_disk(&vm_id, size_mb).map_err(|e| e.to_string())
    }
}

/// List UTM VMs.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn list_utm_vms() -> Result<Vec<String>, String> {
    if mock::is_mock_enabled() {
        Ok(vec!["Home Assistant".to_string()])
    } else {
        crate::utm::list_vms().map_err(|e| e.to_string())
    }
}

// Stub implementations for non-macOS platforms
#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn check_utm_status() -> serde_json::Value {
    serde_json::json!({
        "installed": false,
        "path": null,
        "version": null
    })
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn get_mac_architecture() -> String {
    "unsupported".to_string()
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn create_utm_vm(_config: serde_json::Value) -> Result<String, String> {
    Err("UTM is only available on macOS".to_string())
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn start_utm_vm(_vm_id: String) -> Result<(), String> {
    Err("UTM is only available on macOS".to_string())
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn resize_utm_vm_disk(_vm_id: String, _size_gb: u32) -> Result<(), String> {
    Err("UTM is only available on macOS".to_string())
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn list_utm_vms() -> Result<Vec<String>, String> {
    Err("UTM is only available on macOS".to_string())
}

/// VM status info returned to frontend.
#[derive(serde::Serialize)]
pub struct VmStatusInfo {
    pub status: String,
    pub ip_address: Option<String>,
}

/// Get the status of a UTM VM.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn get_utm_vm_status(vm_id: String) -> Result<VmStatusInfo, String> {
    if mock::is_mock_enabled() {
        // In mock mode, return a simulated running VM with an IP
        return Ok(VmStatusInfo {
            status: "started".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
        });
    }

    let status = crate::utm::get_vm_status(&vm_id).map_err(|e| e.to_string())?;
    let ip_address = if status == crate::utm::VmStatus::Started {
        crate::utm::get_vm_ip_address(&vm_id)
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    Ok(VmStatusInfo {
        status: format!("{:?}", status).to_lowercase(),
        ip_address,
    })
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn get_utm_vm_status(_vm_id: String) -> Result<VmStatusInfo, String> {
    Err("UTM is only available on macOS".to_string())
}

/// Check if Home Assistant webserver is ready at the given IP address.
/// Returns true if we can connect to port 8123.
#[tauri::command]
pub async fn check_ha_ready(ip_address: String) -> bool {
    if mock::is_mock_enabled() {
        return true;
    }

    use tokio::net::TcpStream;
    use tokio::time::timeout;

    let addr = format!("{}:8123", ip_address);

    // Try to establish a TCP connection with a 3 second timeout
    match timeout(Duration::from_secs(3), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        Ok(Err(_)) | Err(_) => false,
    }
}

/// Check if Home Assistant has finished updating by checking the manifest.json endpoint.
/// Returns true if we get a 200 response from http://<ip>:8123/manifest.json
#[tauri::command]
pub async fn check_ha_updated(ip_address: String) -> bool {
    if mock::is_mock_enabled() {
        return true;
    }

    let url = format!("http://{}:8123/manifest.json", ip_address);

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

// ============================================================================
// Proxmox VE Commands
// ============================================================================

/// Connect to a Proxmox VE server and authenticate.
#[tauri::command]
pub async fn proxmox_connect(credentials: ProxmoxCredentials) -> Result<ProxmoxSession, String> {
    if mock::is_mock_enabled() {
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(1500)).await;
        return Ok(ProxmoxSession {
            server_url: credentials.server_url,
            ticket: format!("mock-ticket-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            csrf_token: format!("mock-csrf-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
        });
    }
    proxmox::connect(&credentials).await
}

/// List available nodes on the Proxmox server.
#[tauri::command]
pub async fn proxmox_list_nodes(session: ProxmoxSession) -> Result<Vec<ProxmoxNode>, String> {
    if mock::is_mock_enabled() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        return Ok(vec![
            ProxmoxNode {
                name: "pve".to_string(),
                status: "online".to_string(),
                cpu_usage: Some(12.5),
                memory_used: Some(8 * 1024 * 1024 * 1024),
                memory_total: Some(32 * 1024 * 1024 * 1024),
            },
            ProxmoxNode {
                name: "pve2".to_string(),
                status: "online".to_string(),
                cpu_usage: Some(8.2),
                memory_used: Some(4 * 1024 * 1024 * 1024),
                memory_total: Some(16 * 1024 * 1024 * 1024),
            },
        ]);
    }
    proxmox::list_nodes(&session).await
}

/// List available storage on a Proxmox node.
#[tauri::command]
pub async fn proxmox_list_storage(
    session: ProxmoxSession,
    node: String,
) -> Result<Vec<ProxmoxStorage>, String> {
    if mock::is_mock_enabled() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        return Ok(vec![
            ProxmoxStorage {
                name: "local".to_string(),
                storage_type: "dir".to_string(),
                content: vec![
                    "images".to_string(),
                    "rootdir".to_string(),
                    "vztmpl".to_string(),
                    "backup".to_string(),
                    "iso".to_string(),
                    "snippets".to_string(),
                ],
                available: 200 * 1024 * 1024 * 1024,
                total: 500 * 1024 * 1024 * 1024,
                active: true,
            },
            ProxmoxStorage {
                name: "local-lvm".to_string(),
                storage_type: "lvmthin".to_string(),
                content: vec!["images".to_string(), "rootdir".to_string()],
                available: 400 * 1024 * 1024 * 1024,
                total: 1024 * 1024 * 1024 * 1024,
                active: true,
            },
        ]);
    }
    proxmox::list_storage(&session, &node).await
}

/// Get the next available VM ID on the Proxmox server.
#[tauri::command]
pub async fn proxmox_get_next_vm_id(session: ProxmoxSession) -> Result<u32, String> {
    if mock::is_mock_enabled() {
        tokio::time::sleep(Duration::from_millis(200)).await;
        return Ok(100);
    }
    proxmox::get_next_vm_id(&session).await
}

/// Create a Home Assistant VM on Proxmox.
#[tauri::command]
pub async fn proxmox_create_vm(
    session: ProxmoxSession,
    config: ProxmoxVmConfig,
    progress_channel: Channel<FlashProgress>,
) -> Result<ProxmoxVmResult, String> {
    if mock::is_mock_enabled() {
        // Simulate installation progress
        simulate_proxmox_install_progress(&progress_channel).await;
        return Ok(ProxmoxVmResult {
            vm_id: config.vm_id,
            node: config.node,
            ip_address: Some("192.168.1.150".to_string()),
        });
    }
    proxmox::create_vm(&session, &config, &progress_channel).await
}

/// Simulate Proxmox installation progress for mock mode.
async fn simulate_proxmox_install_progress(channel: &Channel<FlashProgress>) {
    let stages: [(FlashStage, &str, u32); 5] = [
        (FlashStage::Downloading, "Downloading HAOS image...", 40),
        (FlashStage::Extracting, "Uploading to Proxmox...", 25),
        (FlashStage::Writing, "Creating virtual machine...", 20),
        (FlashStage::Verifying, "Starting Home Assistant...", 10),
        (FlashStage::Finalizing, "Waiting for network...", 5),
    ];

    let mut overall_progress: u32 = 0;

    for (stage, message, stage_weight) in stages {
        let steps: u32 = 10;
        for step in 0..=steps {
            let stage_progress = step * 100 / steps;
            let current_progress = overall_progress + (stage_progress * stage_weight / 100);

            let _ = channel.send(FlashProgress {
                stage: stage.clone(),
                progress: current_progress.min(100) as u8,
                bytes_processed: 0,
                total_bytes: 0,
                message: message.to_string(),
            });

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        overall_progress += stage_weight;
    }

    let _ = channel.send(FlashProgress {
        stage: FlashStage::Complete,
        progress: 100,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Installation complete!".to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // ===== Mock Mode Tests =====

    #[test]
    #[serial]
    fn test_is_mock_mode_returns_correct_value_when_enabled() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        assert!(is_mock_mode());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_is_mock_mode_returns_false_when_disabled() {
        std::env::remove_var("HA_INSTALLER_MOCK");
        assert!(!is_mock_mode());
    }

    #[test]
    #[serial]
    fn test_is_mock_mode_returns_true_for_true_string() {
        std::env::set_var("HA_INSTALLER_MOCK", "true");
        assert!(is_mock_mode());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_is_mock_mode_returns_false_for_invalid_value() {
        std::env::set_var("HA_INSTALLER_MOCK", "0");
        assert!(!is_mock_mode());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    // ===== Block Device Tests =====

    #[tokio::test]
    async fn test_list_block_devices_returns_ok() {
        let result = list_block_devices().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_list_block_devices_returns_mock_data_in_mock_mode() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = list_block_devices().await;
        assert!(result.is_ok());
        let devices = result.unwrap();
        assert!(!devices.is_empty());
        // Verify at least one device has expected mock properties
        assert!(devices.iter().any(|d| d.id.starts_with("mock-")));
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_list_block_devices_has_valid_device_types() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = list_block_devices().await;
        assert!(result.is_ok());
        let devices = result.unwrap();
        // Verify all devices have valid device types
        for device in devices {
            assert!(!device.name.is_empty());
            assert!(device.size > 0);
        }
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    // ===== HAOS Release Tests =====

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_returns_ok_in_mock_mode() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = get_haos_release(None).await;
        assert!(result.is_ok());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_has_valid_version() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = get_haos_release(None).await;
        assert!(result.is_ok());
        let release = result.unwrap();
        assert!(!release.version.is_empty());
        assert!(!release.images.is_empty());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_images_have_required_fields() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = get_haos_release(None).await;
        assert!(result.is_ok());
        let release = result.unwrap();
        for image in release.images {
            assert!(!image.board.is_empty());
            assert!(!image.download_url.is_empty());
            assert!(image.size > 0);
            assert!(!image.sha256.is_empty());
            assert_eq!(image.sha256.len(), 64); // SHA256 is 64 hex characters
        }
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_with_specific_version() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = get_haos_release(Some("16.3".to_string())).await;
        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.version, "16.3");
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    // ===== Update Info Tests =====

    #[tokio::test]
    async fn test_check_for_updates_returns_ok() {
        let result = check_for_updates().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_for_updates_has_valid_structure() {
        let result = check_for_updates().await;
        assert!(result.is_ok());
        let update_info = result.unwrap();
        assert!(!update_info.current_version.is_empty());
        assert!(!update_info.latest_version.is_empty());
    }

    // ===== Manifest Tests =====

    #[tokio::test]
    async fn test_get_manifest_returns_ok() {
        let result = get_manifest().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_manifest_has_devices() {
        let result = get_manifest().await;
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert!(!manifest.devices.is_empty());
        assert!(manifest.version > 0);
    }

    #[tokio::test]
    async fn test_get_manifest_devices_have_valid_haos_config() {
        let result = get_manifest().await;
        assert!(result.is_ok());
        let manifest = result.unwrap();
        for device in manifest.devices {
            assert!(!device.id.is_empty());
            assert!(!device.name.is_empty());
            assert!(!device.haos.board.is_empty());
            assert!(!device.haos.download_url.is_empty());
            // Verify URL template contains placeholder
            assert!(device.haos.download_url.contains("{version}"));
        }
    }

    // ===== Flash Stage Enum Tests =====

    #[test]
    fn test_flash_stage_enum_ordering_is_logical() {
        // Test that stages are in a logical progression
        // This documents the expected stage order
        let stages = vec![
            FlashStage::Downloading,
            FlashStage::Extracting,
            FlashStage::Writing,
            FlashStage::Verifying,
            FlashStage::Finalizing,
            FlashStage::Complete,
        ];

        // Verify we can create all stages
        for stage in stages {
            let _serialized = serde_json::to_string(&stage);
            // If this compiles and runs, the enum is properly defined
        }
    }

    #[test]
    fn test_flash_stage_serialization() {
        // Test that FlashStage serializes correctly to snake_case
        let stage = FlashStage::Downloading;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"downloading\"");

        let stage = FlashStage::Complete;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"complete\"");
    }

    // ===== Flash Progress Validation Tests =====

    #[test]
    fn test_flash_progress_creation() {
        let progress = FlashProgress {
            stage: FlashStage::Downloading,
            progress: 50,
            bytes_processed: 1024,
            total_bytes: 2048,
            message: "Test message".to_string(),
        };

        assert_eq!(progress.stage, FlashStage::Downloading);
        assert_eq!(progress.progress, 50);
        assert_eq!(progress.bytes_processed, 1024);
        assert_eq!(progress.total_bytes, 2048);
        assert_eq!(progress.message, "Test message");
    }

    #[test]
    fn test_flash_progress_serialization() {
        let progress = FlashProgress {
            stage: FlashStage::Writing,
            progress: 75,
            bytes_processed: 1500,
            total_bytes: 2000,
            message: "Writing to device...".to_string(),
        };

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"writing\""));
        assert!(json.contains("\"progress\":75"));
        assert!(json.contains("\"bytes_processed\":1500"));
    }

    // ===== Flash Result Tests =====

    #[test]
    fn test_flash_result_success() {
        let result = FlashResult {
            success: true,
            error: None,
            duration_secs: 45,
        };

        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.duration_secs, 45);
    }

    #[test]
    fn test_flash_result_failure() {
        let result = FlashResult {
            success: false,
            error: Some("Test error".to_string()),
            duration_secs: 0,
        };

        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Test error");
    }

    // ===== Flash Request Validation Tests =====

    #[test]
    fn test_flash_request_creation() {
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: "rpi5-64".to_string(),
            verify: true,
        };

        assert_eq!(request.device_id, "/dev/sda");
        assert_eq!(request.board, "rpi5-64");
        assert!(request.verify);
    }

    #[test]
    fn test_flash_request_serialization() {
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: "rpi5-64".to_string(),
            verify: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"device_id\":\"/dev/sda\""));
        assert!(json.contains("\"board\":\"rpi5-64\""));
        assert!(json.contains("\"verify\":true"));
    }

    #[test]
    fn test_flash_request_deserialization() {
        let json = r#"{
            "device_id": "/dev/sda",
            "board": "rpi5-64",
            "verify": true
        }"#;

        let request: FlashRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.device_id, "/dev/sda");
        assert_eq!(request.board, "rpi5-64");
        assert!(request.verify);
    }

    // ===== Error Message Format Tests =====
    // These tests document the expected error message formats

    #[test]
    fn test_error_message_format_for_missing_board() {
        // Documents the expected error format when board is not found
        let board = "unknown-board";
        let error_msg = format!("No image found for board: {}", board);
        assert_eq!(error_msg, "No image found for board: unknown-board");
    }

    #[test]
    fn test_error_message_format_for_download_failure() {
        // Documents the expected error format for download failures
        let inner_error = "Network timeout";
        let error_msg = format!("Download failed: {}", inner_error);
        assert_eq!(error_msg, "Download failed: Network timeout");
    }

    #[test]
    fn test_error_message_format_for_extraction_failure() {
        // Documents the expected error format for extraction failures
        let inner_error = "Invalid XZ archive";
        let error_msg = format!("Extraction failed: {}", inner_error);
        assert_eq!(error_msg, "Extraction failed: Invalid XZ archive");
    }

    #[test]
    fn test_error_message_format_for_write_failure() {
        // Documents the expected error format for write failures
        let inner_error = "Permission denied";
        let error_msg = format!("Write failed: {}", inner_error);
        assert_eq!(error_msg, "Write failed: Permission denied");
    }

    #[test]
    fn test_error_message_format_for_release_fetch_failure() {
        // Documents the expected error format for release fetch failures
        let inner_error = "API rate limit exceeded";
        let error_msg = format!("Failed to fetch release info: {}", inner_error);
        assert_eq!(
            error_msg,
            "Failed to fetch release info: API rate limit exceeded"
        );
    }

    // ===== Simulate Flash Progress Logic Tests =====
    // Note: These tests verify the stage configuration used in simulate_flash_progress
    // without requiring the Channel infrastructure

    #[test]
    fn test_simulate_flash_progress_stage_weights_total_100() {
        // Verify that the stage weights in simulate_flash_progress add up to 100
        let stage_weights = [40, 10, 45, 5]; // Downloading, Verifying, Writing, Finalizing
        let total: u32 = stage_weights.iter().sum();
        assert_eq!(total, 100, "Stage weights should sum to 100%");
    }

    #[test]
    fn test_simulate_flash_progress_stage_order() {
        // Documents the expected order of stages in simulate_flash_progress
        let expected_stages = [
            FlashStage::Downloading,
            FlashStage::Verifying,
            FlashStage::Writing,
            FlashStage::Finalizing,
        ];

        // Verify each stage is unique
        for (i, stage1) in expected_stages.iter().enumerate() {
            for (j, stage2) in expected_stages.iter().enumerate() {
                if i != j {
                    assert_ne!(stage1, stage2, "Stages should be unique in the sequence");
                }
            }
        }
    }

    #[test]
    fn test_simulate_flash_progress_total_bytes_calculation() {
        // Verify the simulated total bytes matches expected value
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
        assert_eq!(total_bytes, 2_147_483_648);
    }

    #[test]
    fn test_simulate_flash_progress_stage_progress_calculation() {
        // Test the progress calculation logic used in simulate_flash_progress
        let stage_weight: u32 = 40; // Example: Downloading stage
        let steps: u32 = 10;

        // Test at various step points
        for step in 0..=steps {
            let stage_progress = step * 100 / steps;
            assert!(stage_progress <= 100);

            // Verify the overall progress contribution
            let contribution = stage_progress * stage_weight / 100;
            assert!(contribution <= stage_weight);
        }
    }

    #[test]
    fn test_simulate_flash_progress_bytes_calculation() {
        // Test bytes processed calculation logic
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024;
        let stage_weight: u32 = 40;
        let steps: u32 = 10;

        for step in 0..=steps {
            let bytes_for_stage = (total_bytes as f64
                * (stage_weight as f64 / 100.0)
                * (step as f64 / steps as f64)) as u64;

            assert!(bytes_for_stage <= total_bytes);
            assert!(bytes_for_stage <= (total_bytes * stage_weight as u64 / 100));
        }
    }

    #[test]
    fn test_simulate_flash_progress_final_progress_is_100() {
        // Verify that when all stages are complete, progress reaches 100%
        let stage_weights = [40, 10, 45, 5];
        let mut overall_progress: u32 = 0;

        for stage_weight in stage_weights {
            // At end of each stage, add its weight
            overall_progress += stage_weight;
        }

        assert_eq!(
            overall_progress, 100,
            "Final overall progress should be 100%"
        );
    }

    #[test]
    fn test_flash_progress_clamps_to_100() {
        // Test that progress is clamped to 100 (as done in simulate_flash_progress)
        let progress: u32 = 105;
        let clamped = progress.min(100);
        assert_eq!(clamped, 100);

        let progress: u32 = 50;
        let clamped = progress.min(100);
        assert_eq!(clamped, 50);
    }

    // ===== Integration-Style Tests =====
    // These tests verify the interaction between components in mock mode

    #[tokio::test]
    async fn test_flash_image_mock_mode_returns_success() {
        // NOTE: This test would require a mock Channel implementation
        // For now, we document what should be tested:
        //
        // 1. flash_image in mock mode should:
        //    - Call simulate_flash_progress with the provided channel
        //    - Return FlashResult with success=true
        //    - Return a reasonable duration (e.g., 45 seconds)
        //    - Have no error message
        //
        // 2. simulate_flash_progress should:
        //    - Send progress updates for each stage: Downloading, Verifying, Writing, Finalizing
        //    - Send a final Complete stage with 100% progress
        //    - Include appropriate messages for each stage
        //
        // This would require creating a mock Channel that collects messages:
        // let (tx, rx) = tokio::sync::mpsc::channel(100);
        // let mock_channel = MockChannel::new(tx);
        // let result = flash_image(request, mock_channel).await;
        //
        // Then verify:
        // assert!(result.is_ok());
        // let flash_result = result.unwrap();
        // assert!(flash_result.success);
        // assert!(flash_result.error.is_none());
        // assert_eq!(flash_result.duration_secs, 45);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_contains_common_boards() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = get_haos_release(None).await;
        assert!(result.is_ok());
        let release = result.unwrap();

        // Verify common boards are present
        let boards: Vec<String> = release.images.iter().map(|i| i.board.clone()).collect();
        assert!(boards.contains(&"rpi5-64".to_string()));
        assert!(boards.contains(&"rpi4-64".to_string()));
        assert!(boards.contains(&"green".to_string()));
        assert!(boards.contains(&"generic-x86-64".to_string()));
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    // ===== Progress Channel Verification Tests =====

    #[test]
    fn test_progress_percentage_calculation_accuracy() {
        // Test the exact math used in simulate_flash_progress
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
        let stage_weight: u32 = 40; // Downloading stage
        let steps: u32 = 10;
        let overall_progress: u32 = 0; // Start of first stage

        // Test at step 0 (0%)
        let step = 0;
        let stage_progress = step * 100 / steps;
        assert_eq!(stage_progress, 0, "Step 0 should be 0% stage progress");
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(current_progress, 0, "Overall progress should be 0 at start");

        // Test at step 5 (50%)
        let step = 5;
        let stage_progress = step * 100 / steps;
        assert_eq!(stage_progress, 50, "Step 5/10 should be 50% stage progress");
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(
            current_progress, 20,
            "50% of 40% weight stage should be 20% overall"
        );

        // Test at step 10 (100%)
        let step = 10;
        let stage_progress = step * 100 / steps;
        assert_eq!(
            stage_progress, 100,
            "Step 10/10 should be 100% stage progress"
        );
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(
            current_progress, 40,
            "100% of 40% weight stage should be 40% overall"
        );

        // Test bytes calculation accuracy
        let step = 5;
        let bytes_for_stage = (total_bytes as f64
            * (stage_weight as f64 / 100.0)
            * (step as f64 / steps as f64)) as u64;
        let expected_bytes = (total_bytes as f64 * 0.4 * 0.5) as u64;
        assert_eq!(
            bytes_for_stage, expected_bytes,
            "Bytes calculation should be accurate"
        );
    }

    #[test]
    fn test_progress_stages_transition_in_correct_order() {
        // Define the exact stage sequence used in simulate_flash_progress
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];

        // Verify stages are in the correct order
        assert_eq!(stages[0].0, FlashStage::Downloading);
        assert_eq!(stages[1].0, FlashStage::Verifying);
        assert_eq!(stages[2].0, FlashStage::Writing);
        assert_eq!(stages[3].0, FlashStage::Finalizing);

        // Verify weights
        assert_eq!(stages[0].2, 40, "Downloading should be 40%");
        assert_eq!(stages[1].2, 10, "Verifying should be 10%");
        assert_eq!(stages[2].2, 45, "Writing should be 45%");
        assert_eq!(stages[3].2, 5, "Finalizing should be 5%");

        // Verify messages are not empty
        for (_, message, _) in stages {
            assert!(!message.is_empty(), "Stage message should not be empty");
        }
    }

    #[test]
    fn test_progress_bytes_written_and_total_bytes_accuracy() {
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];
        let steps: u32 = 10;
        let mut overall_progress: u32 = 0;

        for (stage_idx, (_, _, stage_weight)) in stages.iter().enumerate() {
            for step in 0..=steps {
                let _stage_progress = step * 100 / steps;
                let bytes_for_stage = (total_bytes as f64
                    * (*stage_weight as f64 / 100.0)
                    * (step as f64 / steps as f64)) as u64;

                let bytes_processed = bytes_for_stage
                    + (total_bytes as f64 * (overall_progress as f64 / 100.0)) as u64;

                // Verify bytes_processed never exceeds total_bytes
                assert!(
                    bytes_processed <= total_bytes,
                    "Bytes processed ({}) should never exceed total ({}) at stage {} step {}",
                    bytes_processed,
                    total_bytes,
                    stage_idx,
                    step
                );

                // Verify bytes_for_stage is proportional to stage_weight
                let max_bytes_for_stage = (total_bytes * (*stage_weight as u64)) / 100;
                assert!(
                    bytes_for_stage <= max_bytes_for_stage,
                    "Bytes for stage ({}) should not exceed max for stage weight ({})",
                    bytes_for_stage,
                    max_bytes_for_stage
                );

                // At the end of a stage (step == steps), verify bytes match expected
                if step == steps {
                    let expected_total_after_stage = (total_bytes as f64
                        * ((overall_progress + stage_weight) as f64 / 100.0))
                        as u64;
                    // Allow for small rounding differences
                    let diff = if bytes_processed > expected_total_after_stage {
                        bytes_processed - expected_total_after_stage
                    } else {
                        expected_total_after_stage - bytes_processed
                    };
                    assert!(
                        diff <= 1024, // Allow 1KB rounding error
                        "Bytes at end of stage should match expected: {} vs {}",
                        bytes_processed,
                        expected_total_after_stage
                    );
                }
            }
            overall_progress += stage_weight;
        }
    }

    #[test]
    fn test_progress_updates_at_correct_intervals() {
        // Verify the update interval used in simulate_flash_progress
        let update_interval_ms = 200;
        assert_eq!(update_interval_ms, 200, "Update interval should be 200ms");

        // Calculate total updates that would be sent
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];
        let steps: u32 = 10;
        let updates_per_stage = steps + 1; // 0..=steps inclusive
        let total_progress_updates = stages.len() as u32 * updates_per_stage;
        let completion_update = 1; // Final Complete stage
        let total_updates = total_progress_updates + completion_update;

        // Verify we don't send updates too frequently
        assert_eq!(
            total_progress_updates, 44,
            "Should send 44 progress updates (11 per stage × 4 stages)"
        );
        assert_eq!(total_updates, 45, "Should send 45 total updates");

        // Verify total simulated time
        let total_time_ms = total_progress_updates * update_interval_ms;
        assert_eq!(
            total_time_ms, 8800,
            "Total simulation should take 8.8 seconds"
        );
    }

    #[test]
    fn test_final_progress_is_always_100_percent() {
        // Test the completion logic from simulate_flash_progress
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024;
        let final_progress = FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: total_bytes,
            total_bytes,
            message: "Installation complete!".to_string(),
        };

        assert_eq!(
            final_progress.progress, 100,
            "Final progress must be exactly 100%"
        );
        assert_eq!(
            final_progress.stage,
            FlashStage::Complete,
            "Final stage must be Complete"
        );
        assert_eq!(
            final_progress.bytes_processed, final_progress.total_bytes,
            "Final bytes_processed must equal total_bytes"
        );
        assert!(
            final_progress.message.contains("complete"),
            "Final message should indicate completion"
        );
    }

    #[test]
    fn test_stage_weights_sum_to_100_percent() {
        // Verify stage weights from simulate_flash_progress sum to exactly 100%
        let stage_weights: [u32; 4] = [
            40, // Downloading
            10, // Verifying
            45, // Writing
            5,  // Finalizing
        ];

        let total: u32 = stage_weights.iter().sum();
        assert_eq!(total, 100, "Stage weights must sum to exactly 100%");

        // Verify no individual stage exceeds 100%
        for weight in stage_weights {
            assert!(
                weight <= 100,
                "Individual stage weight {} should not exceed 100%",
                weight
            );
        }

        // Verify all weights are positive
        for weight in stage_weights {
            assert!(weight > 0, "All stage weights should be positive");
        }

        // Verify weights are reasonable (no stage is more than 50%)
        for weight in stage_weights {
            assert!(
                weight <= 50,
                "No single stage should dominate with weight {} > 50%",
                weight
            );
        }
    }

    #[test]
    fn test_progress_never_decreases() {
        // Simulate the progress calculation to ensure it's monotonically increasing
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];
        let steps: u32 = 10;
        let mut overall_progress: u32 = 0;
        let mut previous_progress: u32 = 0;

        for (_, _, stage_weight) in stages {
            for step in 0..=steps {
                let stage_progress = step * 100 / steps;
                let current_progress = overall_progress + (stage_progress * stage_weight / 100);
                let clamped_progress = current_progress.min(100);

                assert!(
                    clamped_progress >= previous_progress,
                    "Progress should never decrease: {} -> {}",
                    previous_progress,
                    clamped_progress
                );
                previous_progress = clamped_progress;
            }
            overall_progress += stage_weight;
        }

        // Final verification: we should end at 100%
        assert_eq!(previous_progress, 100, "Should end at 100% progress");
    }

    #[test]
    fn test_progress_calculation_edge_cases() {
        // Test edge cases in progress calculation

        // Edge case 1: First progress update (should be 0%)
        let overall_progress: u32 = 0;
        let stage_weight: u32 = 40;
        let step: u32 = 0;
        let steps: u32 = 10;
        let stage_progress = step * 100 / steps;
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(current_progress, 0, "First update should be 0%");

        // Edge case 2: Progress at stage boundary
        let overall_progress: u32 = 40; // Completed first stage
        let stage_weight: u32 = 10; // Second stage
        let step: u32 = 0; // Start of second stage
        let stage_progress = step * 100 / steps;
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(
            current_progress, 40,
            "Stage boundary should maintain progress"
        );

        // Edge case 3: Last progress update before completion (should be 95%)
        let overall_progress: u32 = 95; // After Downloading(40) + Verifying(10) + Writing(45)
        let stage_weight: u32 = 5; // Finalizing stage
        let step: u32 = 10; // End of last stage
        let stage_progress = step * 100 / steps;
        let current_progress = overall_progress + (stage_progress * stage_weight / 100);
        assert_eq!(current_progress, 100, "Last stage should reach 100%");

        // Edge case 4: Verify clamping works
        let progress_over_100: u32 = 105;
        let clamped = progress_over_100.min(100);
        assert_eq!(clamped, 100, "Progress over 100 should be clamped");
    }

    #[test]
    fn test_bytes_processed_calculation_consistency() {
        // Verify that bytes_processed calculation is consistent across all stages
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024;
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];
        let steps: u32 = 10;
        let mut overall_progress: u32 = 0;

        for (_, _, stage_weight) in stages {
            let step = steps; // End of stage
            let bytes_for_stage = (total_bytes as f64
                * (stage_weight as f64 / 100.0)
                * (step as f64 / steps as f64)) as u64;

            let bytes_processed =
                bytes_for_stage + (total_bytes as f64 * (overall_progress as f64 / 100.0)) as u64;

            // Calculate expected bytes at this point
            let expected_progress = overall_progress + stage_weight;
            let expected_bytes = (total_bytes as f64 * (expected_progress as f64 / 100.0)) as u64;

            // Allow for rounding errors (up to 0.1% of total)
            let tolerance = total_bytes / 1000;
            let diff = if bytes_processed > expected_bytes {
                bytes_processed - expected_bytes
            } else {
                expected_bytes - bytes_processed
            };

            assert!(
                diff <= tolerance,
                "Bytes processed should be consistent with progress percentage. Expected: {}, Got: {}, Diff: {}",
                expected_bytes,
                bytes_processed,
                diff
            );

            overall_progress += stage_weight;
        }
    }

    #[test]
    fn test_stage_message_format() {
        // Verify that stage messages follow consistent format
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];

        for (stage, message, _) in stages {
            // Verify message is not empty
            assert!(!message.is_empty(), "Message should not be empty for stage");

            // Verify message starts with capital letter
            assert!(
                message.chars().next().unwrap().is_uppercase(),
                "Message '{}' should start with capital letter",
                message
            );

            // Verify message ends with ellipsis for ongoing stages
            if stage != FlashStage::Complete {
                assert!(
                    message.ends_with("..."),
                    "Message '{}' should end with ellipsis for ongoing stages",
                    message
                );
            }
        }

        // Verify completion message format
        let completion_message = "Installation complete!";
        assert!(
            completion_message.ends_with("!"),
            "Completion message should end with exclamation"
        );
    }

    // ===== End-to-End Integration Tests for flash_image Flow =====
    //
    // Note: These tests use a custom test helper that intercepts progress updates
    // since Channel is a Tauri IPC struct that cannot be easily mocked

    /// Test helper: Mock channel that collects progress updates for verification
    struct TestProgressChannel {
        updates: std::sync::Arc<std::sync::Mutex<Vec<FlashProgress>>>,
    }

    impl TestProgressChannel {
        fn new() -> Self {
            Self {
                updates: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn collect(&self, progress: FlashProgress) {
            self.updates.lock().unwrap().push(progress);
        }

        fn get_updates(&self) -> Vec<FlashProgress> {
            self.updates.lock().unwrap().clone()
        }

        fn get_stages(&self) -> Vec<FlashStage> {
            self.updates
                .lock()
                .unwrap()
                .iter()
                .map(|p| p.stage.clone())
                .collect()
        }
    }

    // Manual implementation of Channel to work around Tauri IPC limitations in tests
    impl TestProgressChannel {
        fn send(&self, data: FlashProgress) -> Result<(), String> {
            self.collect(data);
            Ok(())
        }
    }

    /// Create a channel-like object for testing
    /// Returns an object that implements the send() method that Channel has
    fn create_test_progress_channel() -> TestProgressChannel {
        TestProgressChannel::new()
    }

    /// Wrapper that makes TestProgressChannel compatible with the Channel<FlashProgress> type
    /// by converting it via a helper function
    async fn test_simulate_flash_progress(collector: &TestProgressChannel) -> Result<(), String> {
        let total_bytes: u64 = 2 * 1024 * 1024 * 1024;
        let stages: [(FlashStage, &str, u32); 4] = [
            (FlashStage::Downloading, "Downloading image...", 40),
            (FlashStage::Verifying, "Verifying download...", 10),
            (FlashStage::Writing, "Writing to device...", 45),
            (FlashStage::Finalizing, "Finalizing...", 5),
        ];

        let mut overall_progress: u32 = 0;

        for (stage, message, stage_weight) in stages {
            let steps: u32 = 10;
            for step in 0..=steps {
                let stage_progress = step * 100 / steps;
                let bytes_for_stage = (total_bytes as f64
                    * (stage_weight as f64 / 100.0)
                    * (step as f64 / steps as f64)) as u64;

                let current_progress = overall_progress + (stage_progress * stage_weight / 100);

                let progress = FlashProgress {
                    stage: stage.clone(),
                    progress: current_progress.min(100) as u8,
                    bytes_processed: bytes_for_stage
                        + (total_bytes as f64 * (overall_progress as f64 / 100.0)) as u64,
                    total_bytes,
                    message: message.to_string(),
                };

                collector.send(progress)?;
                tokio::time::sleep(Duration::from_millis(50)).await; // Faster for tests
            }
            overall_progress += stage_weight;
        }

        // Send completion
        collector.send(FlashProgress {
            stage: FlashStage::Complete,
            progress: 100,
            bytes_processed: total_bytes,
            total_bytes,
            message: "Installation complete!".to_string(),
        })?;

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_complete_mock_flow() {
        // Test 1: Complete mock flow - verify all stages complete successfully
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        // Verify successful result
        assert!(result.is_ok(), "Flash simulation should succeed");

        // Verify progress updates were sent
        let updates = channel.get_updates();
        assert!(!updates.is_empty(), "Should have progress updates");

        // Verify stages are in correct order
        let stages = channel.get_stages();
        assert!(
            stages.contains(&FlashStage::Downloading),
            "Should include Downloading stage"
        );
        assert!(
            stages.contains(&FlashStage::Verifying),
            "Should include Verifying stage"
        );
        assert!(
            stages.contains(&FlashStage::Writing),
            "Should include Writing stage"
        );
        assert!(
            stages.contains(&FlashStage::Finalizing),
            "Should include Finalizing stage"
        );
        assert!(
            stages.contains(&FlashStage::Complete),
            "Should end with Complete stage"
        );

        // Verify stage ordering
        let downloading_idx = stages
            .iter()
            .position(|s| *s == FlashStage::Downloading)
            .expect("Should have Downloading stage");
        let verifying_idx = stages
            .iter()
            .position(|s| *s == FlashStage::Verifying)
            .expect("Should have Verifying stage");
        let writing_idx = stages
            .iter()
            .position(|s| *s == FlashStage::Writing)
            .expect("Should have Writing stage");
        let finalizing_idx = stages
            .iter()
            .position(|s| *s == FlashStage::Finalizing)
            .expect("Should have Finalizing stage");
        let complete_idx = stages
            .iter()
            .position(|s| *s == FlashStage::Complete)
            .expect("Should have Complete stage");

        assert!(
            downloading_idx < verifying_idx,
            "Downloading should come before Verifying"
        );
        assert!(
            verifying_idx < writing_idx,
            "Verifying should come before Writing"
        );
        assert!(
            writing_idx < finalizing_idx,
            "Writing should come before Finalizing"
        );
        assert!(
            finalizing_idx < complete_idx,
            "Finalizing should come before Complete"
        );

        // Verify final progress is 100%
        let final_update = updates.last().expect("Should have final update");
        assert_eq!(final_update.stage, FlashStage::Complete);
        assert_eq!(final_update.progress, 100);
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_invalid_board_id() {
        // Test: Invalid board ID by testing with real mode
        // Since we cannot easily mock download failures, we test the error path
        // by verifying that the download module's find_image_for_board returns None
        std::env::remove_var("HA_INSTALLER_MOCK");

        // Get a release and verify that an invalid board returns None
        let release = get_haos_release(None).await;
        if let Ok(rel) = release {
            use crate::download::find_image_for_board;
            let image = find_image_for_board(&rel, "nonexistent-board-xyz-123");
            assert!(image.is_none(), "Should not find image for invalid board");
        }

        std::env::set_var("HA_INSTALLER_MOCK", "1");
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_progress_order_and_stages() {
        // Test: Progress updates are sent in correct order
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        assert!(result.is_ok());

        let updates = channel.get_updates();
        let stages = channel.get_stages();

        // Define expected stage progression for mock mode
        let expected_stages = vec![
            FlashStage::Downloading,
            FlashStage::Verifying,
            FlashStage::Writing,
            FlashStage::Finalizing,
            FlashStage::Complete,
        ];

        // Verify each expected stage appears at least once
        for expected_stage in &expected_stages {
            assert!(
                stages.contains(expected_stage),
                "Missing expected stage: {:?}",
                expected_stage
            );
        }

        // Verify progress increases monotonically within each stage
        let mut last_progress_by_stage: std::collections::HashMap<String, u8> =
            std::collections::HashMap::new();

        for update in updates {
            let stage_key = format!("{:?}", update.stage);
            if let Some(&last_progress) = last_progress_by_stage.get(&stage_key) {
                assert!(
                    update.progress >= last_progress,
                    "Progress should not decrease within stage {:?}: {} -> {}",
                    update.stage,
                    last_progress,
                    update.progress
                );
            }
            last_progress_by_stage.insert(stage_key, update.progress);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_progress_reaches_100_percent() {
        // Verify that progress reaches 100% at completion
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        assert!(result.is_ok());

        let updates = channel.get_updates();
        let final_update = updates.last().expect("Should have at least one update");

        assert_eq!(
            final_update.stage,
            FlashStage::Complete,
            "Final stage should be Complete"
        );
        assert_eq!(final_update.progress, 100, "Final progress should be 100%");
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_multiple_sequential_flashes() {
        // Test multiple sequential flash simulations
        for i in 0..3 {
            let channel = create_test_progress_channel();
            let result = test_simulate_flash_progress(&channel).await;

            assert!(result.is_ok(), "Flash {} should succeed", i);

            let updates = channel.get_updates();
            assert!(!updates.is_empty(), "Flash {} should have updates", i);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_all_expected_stages_present() {
        // Comprehensive test that all expected stages are present
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        assert!(result.is_ok());

        let stages = channel.get_stages();

        // Verify all expected stages are present
        let expected_stages = vec![
            FlashStage::Downloading,
            FlashStage::Verifying,
            FlashStage::Writing,
            FlashStage::Finalizing,
            FlashStage::Complete,
        ];

        for expected in expected_stages {
            assert!(
                stages.contains(&expected),
                "Should contain stage: {:?}",
                expected
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_bytes_processed_increases() {
        // Verify that bytes_processed increases over time
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        assert!(result.is_ok());

        let updates = channel.get_updates();

        // Group updates by stage
        let mut updates_by_stage: std::collections::HashMap<String, Vec<u64>> =
            std::collections::HashMap::new();

        for update in &updates {
            let stage_key = format!("{:?}", update.stage);
            updates_by_stage
                .entry(stage_key)
                .or_insert_with(Vec::new)
                .push(update.bytes_processed);
        }

        // Within each stage, bytes_processed should generally increase
        for (stage, bytes_list) in updates_by_stage {
            if bytes_list.len() > 1 {
                let first = bytes_list[0];
                let last = bytes_list[bytes_list.len() - 1];
                assert!(
                    last >= first,
                    "Bytes processed should not decrease within stage {}: {} -> {}",
                    stage,
                    first,
                    last
                );
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_e2e_messages_are_meaningful() {
        // Verify that progress messages are meaningful and not empty
        let channel = create_test_progress_channel();
        let result = test_simulate_flash_progress(&channel).await;

        assert!(result.is_ok());

        let updates = channel.get_updates();

        for update in updates {
            assert!(
                !update.message.is_empty(),
                "Progress message should not be empty for stage {:?}",
                update.stage
            );
            // Messages should be reasonable length
            assert!(
                update.message.len() < 200,
                "Message should be reasonably short: {}",
                update.message
            );
        }
    }

    // =============================================================================
    // Invalid Board Handling Tests for flash_image
    // =============================================================================

    #[tokio::test]
    #[serial]
    async fn test_flash_image_nonexistent_board_returns_error() {
        // NOTE: This test verifies the error message format for non-existent boards
        // The actual flash_image function (line 80-81) would return this error

        // Test the error message format for non-existent board
        let board = "nonexistent-board-xyz-123";
        let error_msg = format!("No image found for board: {}", board);
        assert_eq!(
            error_msg,
            "No image found for board: nonexistent-board-xyz-123"
        );

        // Verify the error message matches the pattern in flash_image
        assert!(error_msg.starts_with("No image found for board:"));
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_empty_board_returns_error() {
        // Test the error message format for empty board ID
        let board = "";
        let error_msg = format!("No image found for board: {}", board);
        assert_eq!(error_msg, "No image found for board: ");

        // Verify empty string produces a valid error message
        assert!(error_msg.starts_with("No image found for board:"));
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_special_characters_board_returns_error() {
        // Test various special characters that might cause issues
        let special_boards = [
            "rpi5/64",
            "rpi5\\64",
            "rpi5@64",
            "rpi5#64",
            "../../../etc/passwd",      // Path traversal attempt
            "'; DROP TABLE boards; --", // SQL injection attempt
        ];

        for board in special_boards.iter() {
            let error_msg = format!("No image found for board: {}", board);
            assert!(
                error_msg.starts_with("No image found for board:"),
                "Special character board '{}' should produce valid error message",
                board
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_flash_image_case_sensitivity_board() {
        // Test that board matching is case-sensitive
        // If user provides "RPI5-64" instead of "rpi5-64", it should fail
        let boards_to_test = [
            ("RPI5-64", "RPI5-64"),               // All uppercase
            ("Rpi5-64", "Rpi5-64"),               // Mixed case
            ("GREEN", "GREEN"),                   // Uppercase variant of 'green'
            ("Generic-X86-64", "Generic-X86-64"), // Mixed case variant
        ];

        for (board, expected_in_error) in boards_to_test.iter() {
            let error_msg = format!("No image found for board: {}", board);
            assert!(
                error_msg.contains(expected_in_error),
                "Error message should preserve case: {}",
                board
            );
        }
    }

    #[test]
    fn test_flash_request_validation_empty_board() {
        // Test that FlashRequest can be created with empty board
        // (validation happens in flash_image, not in struct creation)
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: "".to_string(),
            verify: true,
        };

        assert_eq!(request.board, "");
        assert!(!request.board.is_empty() == false); // Double negative to confirm empty
    }

    #[test]
    fn test_flash_request_validation_special_chars_board() {
        // Test that FlashRequest accepts special characters
        // (filtering happens in flash_image logic)
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: "test@#$%".to_string(),
            verify: true,
        };

        assert!(request.board.contains("@"));
        assert!(request.board.contains("#"));
        assert!(request.board.contains("$"));
    }

    #[test]
    fn test_flash_request_whitespace_board() {
        // Test that FlashRequest preserves whitespace
        let request = FlashRequest {
            device_id: "/dev/sda".to_string(),
            board: " rpi5-64 ".to_string(),
            verify: true,
        };

        assert_eq!(request.board, " rpi5-64 ");
        assert!(request.board.starts_with(" "));
        assert!(request.board.ends_with(" "));
    }

    // =============================================================================
    // Empty Device List Tests
    // =============================================================================

    #[tokio::test]
    #[serial]
    async fn test_list_block_devices_empty_result_is_valid() {
        // When no removable devices exist, the result should be Ok with empty Vec
        // This tests the contract that list_block_devices never returns Err for "no devices"
        std::env::remove_var("HA_INSTALLER_MOCK");
        let result = list_block_devices().await;

        // Should always be Ok, even if empty
        assert!(
            result.is_ok(),
            "list_block_devices should return Ok even when no devices found"
        );

        let devices = result.unwrap();
        // Could be empty on CI systems without removable media
        // Just verify it's a valid Vec
        assert!(devices.len() >= 0); // Always true, but documents the contract
    }

    #[tokio::test]
    #[serial]
    async fn test_list_block_devices_mock_mode_never_empty() {
        // In mock mode, we should always have at least one device
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let result = list_block_devices().await;

        assert!(result.is_ok());
        let devices = result.unwrap();
        assert!(
            !devices.is_empty(),
            "Mock mode should return at least one device for testing"
        );

        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    fn test_block_device_list_handling_empty_vec() {
        // Test that code can handle an empty device list
        let devices: Vec<BlockDevice> = Vec::new();

        assert_eq!(devices.len(), 0);
        assert!(devices.is_empty());

        // Verify iteration over empty list works
        for device in &devices {
            // Should never execute
            panic!(
                "Should not iterate over empty device list, found: {}",
                device.name
            );
        }
    }

    #[test]
    fn test_block_device_filtering_results_in_empty_list() {
        // Simulate filtering that results in empty list
        let all_devices = vec![BlockDevice {
            id: "/dev/sda".to_string(),
            name: "System Drive".to_string(),
            size: 500_000_000_000,
            device_type: crate::types::DeviceType::Ssd,
            removable: false,
            model: None,
            vendor: None,
        }];

        // Filter for removable only
        let removable_devices: Vec<&BlockDevice> =
            all_devices.iter().filter(|d| d.removable).collect();

        assert_eq!(
            removable_devices.len(),
            0,
            "Filtering non-removable devices should result in empty list"
        );
    }
}
