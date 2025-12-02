//! Proxmox VE integration module
//!
//! This module provides functionality to connect to Proxmox VE servers,
//! list nodes and storage, and create Home Assistant VMs.
//!
//! ## HAOS Installation Workflow
//!
//! The correct procedure for installing HAOS on Proxmox via API:
//! 1. Download the qcow2.xz image locally
//! 2. Extract to qcow2
//! 3. Upload qcow2 to Proxmox "local" storage (content=import)
//! 4. Wait for upload task to complete
//! 5. Create VM with UEFI/OVMF, EFI disk, and import-from to import the disk
//! 6. Wait for VM creation task to complete
//! 7. Start VM and wait for IP via QEMU guest agent
//!
//! References:
//! - https://forum.proxmox.com/threads/api-equivalent-of-qm-importdisk.157457/
//! - https://forum.proxmox.com/threads/guide-install-home-assistant-os-in-a-vm.143251/

use crate::types::{
    FlashProgress, FlashStage, ProxmoxCredentials, ProxmoxNode, ProxmoxSession, ProxmoxStorage,
    ProxmoxVmConfig, ProxmoxVmResult,
};
use tauri::ipc::Channel;

/// Create a configured HTTP client for Proxmox API calls.
/// Accepts self-signed certificates (common for Proxmox installations).
fn create_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Minimum required Proxmox VE version for disk image import via API.
/// Version 8.4.1 added support for uploading qcow2/raw/img/vmdk files with content=import.
const MIN_PROXMOX_VERSION: (u32, u32, u32) = (8, 4, 1);

/// Parse a Proxmox version string like "8.4.1" into (major, minor, patch).
fn parse_version(version_str: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
        Some((major, minor, patch))
    } else {
        None
    }
}

/// Check if version meets minimum requirements.
fn version_meets_minimum(version: (u32, u32, u32), minimum: (u32, u32, u32)) -> bool {
    version.0 > minimum.0
        || (version.0 == minimum.0 && version.1 > minimum.1)
        || (version.0 == minimum.0 && version.1 == minimum.1 && version.2 >= minimum.2)
}

/// Connect to a Proxmox VE server and authenticate.
///
/// This function also verifies the Proxmox version is at least 8.4.1,
/// which is required for disk image import via the API.
///
/// # Arguments
/// * `credentials` - Server URL, username, and password
///
/// # Returns
/// * `Ok(ProxmoxSession)` - Authentication session with ticket and CSRF token
/// * `Err(String)` - Error message if connection fails or version is too old
pub async fn connect(credentials: &ProxmoxCredentials) -> Result<ProxmoxSession, String> {
    // Validate URL format
    if !credentials.server_url.starts_with("https://") {
        return Err("Server URL must start with https://".to_string());
    }

    let base_url = credentials.server_url.trim_end_matches('/');
    let client = create_client(30)?;

    // Step 1: Authenticate
    let auth_url = format!("{}/api2/json/access/ticket", base_url);

    let response = client
        .post(&auth_url)
        .form(&[
            ("username", credentials.username.as_str()),
            ("password", credentials.password.as_str()),
        ])
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Connection timed out. Please check the server URL and network connectivity."
                    .to_string()
            } else if e.is_connect() {
                "Failed to connect to Proxmox server. Please verify the URL is correct.".to_string()
            } else {
                format!("Connection error: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        if status.as_u16() == 401 {
            return Err("Authentication failed. Please check your username and password.".to_string());
        } else if status.as_u16() == 403 {
            return Err("Access denied. The user may not have sufficient permissions.".to_string());
        }
        return Err(format!("Server returned error: {}", status));
    }

    // Parse the authentication response
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse server response: {}", e))?;

    let data = json
        .get("data")
        .ok_or("Invalid response from server: missing 'data' field")?;

    let ticket = data
        .get("ticket")
        .and_then(|v| v.as_str())
        .ok_or("Invalid response from server: missing 'ticket' field")?
        .to_string();

    let csrf_token = data
        .get("CSRFPreventionToken")
        .and_then(|v| v.as_str())
        .ok_or("Invalid response from server: missing 'CSRFPreventionToken' field")?
        .to_string();

    // Step 2: Check Proxmox version
    let version_url = format!("{}/api2/json/version", base_url);

    let version_response = client
        .get(&version_url)
        .header("Cookie", format!("PVEAuthCookie={}", ticket))
        .send()
        .await
        .map_err(|e| format!("Failed to get Proxmox version: {}", e))?;

    if !version_response.status().is_success() {
        return Err(format!(
            "Failed to get Proxmox version: {}",
            version_response.status()
        ));
    }

    let version_json: serde_json::Value = version_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse version response: {}", e))?;

    let version_str = version_json
        .get("data")
        .and_then(|d| d.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("Failed to get Proxmox version from response")?;

    let version = parse_version(version_str).ok_or_else(|| {
        format!("Failed to parse Proxmox version: {}", version_str)
    })?;

    if !version_meets_minimum(version, MIN_PROXMOX_VERSION) {
        return Err(format!(
            "Proxmox VE version {} is not supported. \
             This installer requires Proxmox VE {}.{}.{} or later for disk image import. \
             Please upgrade your Proxmox installation.",
            version_str,
            MIN_PROXMOX_VERSION.0,
            MIN_PROXMOX_VERSION.1,
            MIN_PROXMOX_VERSION.2
        ));
    }

    Ok(ProxmoxSession {
        server_url: credentials.server_url.clone(),
        ticket,
        csrf_token,
    })
}

/// List available nodes on the Proxmox server.
///
/// # Arguments
/// * `session` - The authentication session
///
/// # Returns
/// * `Ok(Vec<ProxmoxNode>)` - List of available nodes
/// * `Err(String)` - Error message if request fails
pub async fn list_nodes(session: &ProxmoxSession) -> Result<Vec<ProxmoxNode>, String> {
    let url = format!(
        "{}/api2/json/nodes",
        session.server_url.trim_end_matches('/')
    );

    let client = create_client(30)?;

    let response = client
        .get(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Connection timed out while listing nodes. Please check network connectivity.".to_string()
            } else if e.is_connect() {
                format!("Failed to connect to Proxmox server at {}", session.server_url)
            } else {
                format!("Network error while listing nodes: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        if status.as_u16() == 401 {
            return Err("Authentication expired or invalid. Please reconnect to Proxmox.".to_string());
        } else if status.as_u16() == 403 {
            return Err("Access denied. Your user may not have permission to list nodes.".to_string());
        }
        return Err(format!("Proxmox server returned error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Proxmox response: {}", e))?;

    let data = json
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("Unexpected response from Proxmox: missing node data")?;

    let nodes: Vec<ProxmoxNode> = data
        .iter()
        .filter_map(|node| {
            Some(ProxmoxNode {
                name: node.get("node")?.as_str()?.to_string(),
                status: node
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                cpu_usage: node.get("cpu").and_then(|v| v.as_f64()),
                memory_used: node.get("mem").and_then(|v| v.as_u64()),
                memory_total: node.get("maxmem").and_then(|v| v.as_u64()),
            })
        })
        .collect();

    Ok(nodes)
}

/// List available storage on a Proxmox node.
///
/// # Arguments
/// * `session` - The authentication session
/// * `node` - The node name
///
/// # Returns
/// * `Ok(Vec<ProxmoxStorage>)` - List of available storage locations
/// * `Err(String)` - Error message if request fails
pub async fn list_storage(
    session: &ProxmoxSession,
    node: &str,
) -> Result<Vec<ProxmoxStorage>, String> {
    let url = format!(
        "{}/api2/json/nodes/{}/storage",
        session.server_url.trim_end_matches('/'),
        node
    );

    let client = create_client(30)?;

    let response = client
        .get(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .send()
        .await
        .map_err(|e| format!("Failed to list storage: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to list storage: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse storage list: {}", e))?;

    let data = json
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("Invalid response: missing 'data' array")?;

    let storage: Vec<ProxmoxStorage> = data
        .iter()
        .filter_map(|s| {
            // Parse content types from comma-separated string
            let content_str = s.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let content: Vec<String> = content_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Some(ProxmoxStorage {
                name: s.get("storage")?.as_str()?.to_string(),
                storage_type: s
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                content,
                available: s.get("avail").and_then(|v| v.as_u64()).unwrap_or(0),
                total: s.get("total").and_then(|v| v.as_u64()).unwrap_or(0),
                active: s.get("active").and_then(|v| v.as_u64()).unwrap_or(0) == 1,
            })
        })
        .collect();

    Ok(storage)
}

/// Get the next available VM ID on the Proxmox server.
///
/// # Arguments
/// * `session` - The authentication session
///
/// # Returns
/// * `Ok(u32)` - Next available VM ID
/// * `Err(String)` - Error message if request fails
pub async fn get_next_vm_id(session: &ProxmoxSession) -> Result<u32, String> {
    let url = format!(
        "{}/api2/json/cluster/nextid",
        session.server_url.trim_end_matches('/')
    );

    let client = create_client(30)?;

    let response = client
        .get(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .send()
        .await
        .map_err(|e| format!("Failed to get next VM ID: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to get next VM ID: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse VM ID response: {}", e))?;

    let data = json
        .get("data")
        .ok_or("Invalid response: missing 'data' field")?;

    // Proxmox returns the VM ID as a string (e.g., "100"), not a number
    let vm_id = if let Some(n) = data.as_u64() {
        n as u32
    } else if let Some(s) = data.as_str() {
        s.parse::<u32>()
            .map_err(|_| format!("Invalid VM ID format: {}", s))?
    } else {
        return Err(format!("Unexpected VM ID type: {:?}", data));
    };

    Ok(vm_id)
}

/// Wait for a Proxmox task to complete.
///
/// # Arguments
/// * `session` - The authentication session
/// * `node` - The node where the task is running
/// * `upid` - The task UPID (unique process ID)
/// * `timeout_secs` - Maximum time to wait in seconds
///
/// # Returns
/// * `Ok(())` - Task completed successfully
/// * `Err(String)` - Task failed or timed out
async fn wait_for_task(
    session: &ProxmoxSession,
    node: &str,
    upid: &str,
    timeout_secs: u64,
) -> Result<(), String> {
    let url = format!(
        "{}/api2/json/nodes/{}/tasks/{}/status",
        session.server_url.trim_end_matches('/'),
        node,
        urlencoding::encode(upid)
    );

    let client = create_client(30)?;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Err(format!("Task timed out after {} seconds", timeout_secs));
        }

        let response = client
            .get(&url)
            .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
            .send()
            .await
            .map_err(|e| format!("Failed to check task status: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to check task status: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse task status: {}", e))?;

        let data = json.get("data").ok_or("Invalid task status response")?;

        let status = data.get("status").and_then(|v| v.as_str()).unwrap_or("");

        if status == "stopped" {
            // Task is complete, check if it succeeded
            let exitstatus = data.get("exitstatus").and_then(|v| v.as_str()).unwrap_or("");
            if exitstatus == "OK" {
                return Ok(());
            } else {
                return Err(format!("Task failed with status: {}", exitstatus));
            }
        }

        // Task still running, wait a bit before checking again
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

/// Upload a disk image to Proxmox storage with progress reporting.
///
/// Uploads to "local" storage with content type "import".
/// The storage must have the "import" content type enabled.
///
/// # Arguments
/// * `session` - The authentication session
/// * `node` - The target node
/// * `local_path` - Path to the local qcow2 file
/// * `progress_channel` - Channel for progress updates
///
/// # Returns
/// * `Ok(String)` - The filename on Proxmox storage
/// * `Err(String)` - Error message if upload fails
async fn upload_image_to_proxmox(
    session: &ProxmoxSession,
    node: &str,
    local_path: &std::path::PathBuf,
    progress_channel: &Channel<FlashProgress>,
) -> Result<String, String> {
    use futures_util::stream;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    // Get the filename from the path
    // Proxmox 8.4.1+ accepts .qcow2, .vmdk, .raw for import
    let filename = local_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?
        .to_string();

    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: format!("Reading {} for upload...", filename),
    });

    // Read the file into memory
    let mut file = File::open(local_path)
        .await
        .map_err(|e| format!("Failed to open image file: {}", e))?;

    let file_size = file
        .metadata()
        .await
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();

    let mut file_contents = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut file_contents)
        .await
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 5,
        bytes_processed: 0,
        total_bytes: file_size,
        message: format!(
            "Uploading {} ({:.1} MB) to Proxmox...",
            filename,
            file_size as f64 / 1_000_000.0
        ),
    });

    // Upload to Proxmox using multipart form
    // POST /api2/json/nodes/{node}/storage/{storage}/upload
    let url = format!(
        "{}/api2/json/nodes/{}/storage/local/upload",
        session.server_url.trim_end_matches('/'),
        node
    );

    // Create client with longer timeout for large uploads
    let client = create_client(1800)?; // 30 minutes

    // Create a chunked stream that reports upload progress
    // Use 256KB chunks for smooth progress updates
    let chunk_size = 256 * 1024;
    let bytes_sent = Arc::new(AtomicU64::new(0));
    let progress_channel_clone = progress_channel.clone();
    let filename_for_progress = filename.clone();

    // Convert file contents to owned chunks for streaming
    let chunks: Vec<Vec<u8>> = file_contents.chunks(chunk_size).map(|c| c.to_vec()).collect();
    let total_chunks = chunks.len();

    let progress_stream = stream::iter(chunks.into_iter().enumerate().map(
        move |(chunk_idx, chunk)| {
            let chunk_len = chunk.len() as u64;
            let sent = bytes_sent.fetch_add(chunk_len, Ordering::SeqCst) + chunk_len;
            let progress_pct = (sent as f64 / file_size as f64 * 100.0).min(100.0);

            // Send progress update every ~16MB (every 64 chunks of 256KB)
            if chunk_idx % 64 == 0 || chunk_idx == total_chunks - 1 {
                let _ = progress_channel_clone.send(FlashProgress {
                    stage: FlashStage::Extracting,
                    // Progress 0-90% during upload (leave room for processing)
                    progress: (progress_pct * 0.9) as u8,
                    bytes_processed: sent,
                    total_bytes: file_size,
                    message: format!(
                        "Uploading {} to Proxmox... {:.0}%",
                        filename_for_progress, progress_pct
                    ),
                });
            }

            Ok::<_, std::io::Error>(chunk)
        },
    ));

    // Create the multipart part with streaming body
    let body = reqwest::Body::wrap_stream(progress_stream);
    let file_part = reqwest::multipart::Part::stream_with_length(body, file_size)
        .file_name(filename.clone())
        .mime_str("application/octet-stream")
        .map_err(|e| format!("Failed to create file part: {}", e))?;

    let form = reqwest::multipart::Form::new()
        .text("content", "import")
        .part("filename", file_part);

    let response = client
        .post(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .header("CSRFPreventionToken", &session.csrf_token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to upload image: {}", e))?;

    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!(
            "Failed to upload image to Proxmox ({}): {}",
            status, response_text
        ));
    }

    // Parse the response to get the task UPID
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse upload response: {}", e))?;

    let upid = json
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or("Upload response missing task UPID")?;

    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 95,
        bytes_processed: file_size,
        total_bytes: file_size,
        message: "Waiting for Proxmox to process upload...".to_string(),
    });

    // Wait for the upload task to complete
    wait_for_task(session, node, upid, 1800).await?;

    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 100,
        bytes_processed: file_size,
        total_bytes: file_size,
        message: "Upload complete".to_string(),
    });

    Ok(filename)
}

/// Create a Home Assistant VM on Proxmox.
///
/// This function:
/// 1. Downloads the HAOS qcow2 image locally
/// 2. Extracts the compressed image
/// 3. Uploads it to Proxmox "local" storage
/// 4. Creates the VM with UEFI boot and imports the disk
/// 5. Starts the VM and waits for an IP address
///
/// # Arguments
/// * `session` - The authentication session
/// * `config` - VM configuration
/// * `progress_channel` - Channel for progress updates
///
/// # Returns
/// * `Ok(ProxmoxVmResult)` - Result with VM ID and IP address
/// * `Err(String)` - Error message if creation fails
pub async fn create_vm(
    session: &ProxmoxSession,
    config: &ProxmoxVmConfig,
    progress_channel: &Channel<FlashProgress>,
) -> Result<ProxmoxVmResult, String> {
    // Step 1: Get HAOS release info
    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Downloading,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Fetching release info...".to_string(),
    });

    // Get the latest HAOS release for "ova" which provides qcow2 images for x86-64 virtualization
    let release = crate::download::fetch_latest_release()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    // Use "ova" board which provides qcow2 images for virtualization platforms like Proxmox
    let qcow2_image = crate::download::find_image_for_board(&release, "ova")
        .ok_or("No qcow2 image found for virtualization (ova board)")?;

    // Step 2: Download the compressed image locally
    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Downloading,
        progress: 5,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Downloading HAOS image...".to_string(),
    });

    let compressed_path = crate::download::download_image(&qcow2_image, progress_channel)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    // Step 3: Extract the compressed image
    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Extracting image...".to_string(),
    });

    let extracted_path = crate::download::extract_image(&compressed_path, progress_channel)
        .await
        .map_err(|e| format!("Failed to extract image: {}", e))?;

    // Step 4: Upload the extracted image to Proxmox "local" storage
    // Note: upload_image_to_proxmox handles its own progress updates

    let image_filename = upload_image_to_proxmox(
        session,
        &config.node,
        &extracted_path,
        progress_channel,
    )
    .await?;

    // Step 5: Create the VM with disk import
    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Writing,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Creating virtual machine...".to_string(),
    });

    create_vm_with_disk(session, config, &image_filename).await?;

    // Step 6: Start the VM if requested
    if config.auto_start {
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Verifying,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Starting virtual machine...".to_string(),
        });

        start_vm(session, &config.node, config.vm_id).await?;
    }

    // Step 7: Wait for IP address
    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Finalizing,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Waiting for network connection...".to_string(),
    });

    let ip_address = if config.auto_start {
        wait_for_vm_ip(session, &config.node, config.vm_id).await
    } else {
        None
    };

    // Step 8: Wait for Home Assistant webserver to be ready
    if let Some(ref ip) = ip_address {
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Ready,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Waiting for Home Assistant to start...".to_string(),
        });

        // Wait for webserver (don't fail if it times out - HA might just be slow)
        wait_for_ha_webserver(ip).await;

        // Step 9: Wait for Home Assistant to finish updating
        let _ = progress_channel.send(FlashProgress {
            stage: FlashStage::Updating,
            progress: 0,
            bytes_processed: 0,
            total_bytes: 0,
            message: "Updating to the latest version...".to_string(),
        });

        // Wait for manifest.json (don't fail if it times out)
        wait_for_ha_updated(ip).await;
    }

    let _ = progress_channel.send(FlashProgress {
        stage: FlashStage::Complete,
        progress: 100,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Installation complete!".to_string(),
    });

    Ok(ProxmoxVmResult {
        vm_id: config.vm_id,
        node: config.node.clone(),
        ip_address,
    })
}

/// Create a VM with the disk imported during creation.
///
/// This uses the `import-from` parameter on scsi0 to import the uploaded
/// disk image during VM creation. This is the recommended approach for
/// Proxmox 7.1-5 and later.
///
/// The VM is configured with:
/// - OVMF (UEFI) BIOS for modern boot support
/// - Q35 machine type (modern PCIe chipset)
/// - VirtIO-SCSI controller for optimal disk performance
/// - EFI disk for UEFI boot
/// - VirtIO network adapter
/// - QEMU guest agent enabled
async fn create_vm_with_disk(
    session: &ProxmoxSession,
    config: &ProxmoxVmConfig,
    image_filename: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/api2/json/nodes/{}/qemu",
        session.server_url.trim_end_matches('/'),
        config.node
    );

    let client = create_client(300)?; // 5 minutes for VM creation with disk import

    // Build the disk import specification
    // Format: storage:0,import-from=local:import/filename.qcow2
    // The :0 is a placeholder that tells Proxmox to allocate a new disk
    let scsi0_spec = format!(
        "{}:0,import-from=local:import/{}",
        config.storage,
        image_filename
    );

    // EFI disk specification for UEFI boot
    // pre-enrolled-keys=0 disables Secure Boot (required for HAOS)
    let efidisk0_spec = format!(
        "{}:1,efitype=4m,pre-enrolled-keys=0",
        config.storage
    );

    let response = client
        .post(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .header("CSRFPreventionToken", &session.csrf_token)
        .form(&[
            ("vmid", config.vm_id.to_string()),
            ("name", config.name.clone()),
            ("cores", config.cpu_cores.to_string()),
            ("memory", config.memory_mb.to_string()),
            ("bios", "ovmf".to_string()),           // UEFI boot (required for HAOS)
            ("machine", "q35".to_string()),         // Modern PCIe chipset
            ("cpu", "host".to_string()),            // Best CPU performance
            ("scsihw", "virtio-scsi-pci".to_string()), // VirtIO SCSI controller
            ("ostype", "l26".to_string()),          // Linux 2.6/3.x/4.x/5.x/6.x kernel
            ("efidisk0", efidisk0_spec),            // EFI disk for UEFI
            ("scsi0", scsi0_spec),                  // Main disk with import
            ("net0", "virtio,bridge=vmbr0".to_string()), // VirtIO network
            ("agent", "enabled=1".to_string()),     // QEMU guest agent
            ("boot", "order=scsi0".to_string()),    // Boot from main disk
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to create VM: {}", e))?;

    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Failed to create VM ({}): {}", status, response_text));
    }

    // Parse the response to get the task UPID
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse VM creation response: {}", e))?;

    // VM creation returns a task UPID since it involves disk import
    if let Some(upid) = json.get("data").and_then(|v| v.as_str()) {
        // Wait for the VM creation task to complete
        wait_for_task(session, &config.node, upid, 600).await?;
    }

    Ok(())
}

/// Start the VM.
async fn start_vm(
    session: &ProxmoxSession,
    node: &str,
    vm_id: u32,
) -> Result<(), String> {
    let url = format!(
        "{}/api2/json/nodes/{}/qemu/{}/status/start",
        session.server_url.trim_end_matches('/'),
        node,
        vm_id
    );

    let client = create_client(60)?;

    let response = client
        .post(&url)
        .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
        .header("CSRFPreventionToken", &session.csrf_token)
        .send()
        .await
        .map_err(|e| format!("Failed to start VM: {}", e))?;

    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Failed to start VM ({}): {}", status, response_text));
    }

    // Parse the response to get the task UPID
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse start VM response: {}", e))?;

    // VM start returns a task UPID
    if let Some(upid) = json.get("data").and_then(|v| v.as_str()) {
        // Wait for the VM start task to complete
        wait_for_task(session, node, upid, 120).await?;
    }

    Ok(())
}

/// Wait for the Home Assistant webserver to be ready on port 8123.
///
/// This polls the webserver every 2 seconds until it responds,
/// or until the timeout is reached.
async fn wait_for_ha_webserver(ip: &str) -> bool {
    let url = format!("http://{}:8123", ip);
    let client = match create_client(10) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try for up to 5 minutes (150 attempts * 2 seconds)
    for _ in 0..150 {
        match client.get(&url).send().await {
            Ok(response) => {
                // Any response means the webserver is up
                if response.status().is_success() || response.status().as_u16() < 500 {
                    return true;
                }
            }
            Err(_) => {
                // Connection refused or other error, keep trying
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    false
}

/// Wait for Home Assistant to finish updating to the latest version.
///
/// This checks for the manifest.json endpoint which only becomes available
/// after the initial setup and updates are complete. This typically takes
/// around 20 minutes on first boot, but we allow up to 1 hour.
async fn wait_for_ha_updated(ip: &str) -> bool {
    let url = format!("http://{}:8123/manifest.json", ip);
    let client = match create_client(10) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try for up to 1 hour (1800 attempts * 2 seconds)
    for _ in 0..1800 {
        match client.get(&url).send().await {
            Ok(response) => {
                // 200 OK means Home Assistant is fully ready
                if response.status().is_success() {
                    return true;
                }
            }
            Err(_) => {
                // Connection refused or other error, keep trying
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    false
}

/// Wait for the VM to get an IP address via QEMU guest agent.
///
/// The QEMU guest agent must be running inside the VM for this to work.
/// HAOS includes the guest agent, but it takes time to boot and start.
async fn wait_for_vm_ip(
    session: &ProxmoxSession,
    node: &str,
    vm_id: u32,
) -> Option<String> {
    let url = format!(
        "{}/api2/json/nodes/{}/qemu/{}/agent/network-get-interfaces",
        session.server_url.trim_end_matches('/'),
        node,
        vm_id
    );

    let client = create_client(10).ok()?;

    // Try for up to 5 minutes (150 attempts * 2 seconds)
    for _ in 0..150 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let response = client
            .get(&url)
            .header("Cookie", format!("PVEAuthCookie={}", session.ticket))
            .send()
            .await
            .ok()?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await.ok()?;

            // Look for an IPv4 address on a non-loopback interface
            if let Some(interfaces) = json.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
                for iface in interfaces {
                    let name = iface.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    if name == "lo" {
                        continue;
                    }

                    if let Some(ip_addresses) = iface.get("ip-addresses").and_then(|a| a.as_array()) {
                        for addr in ip_addresses {
                            if addr.get("ip-address-type").and_then(|t| t.as_str()) == Some("ipv4") {
                                if let Some(ip) = addr.get("ip-address").and_then(|i| i.as_str()) {
                                    if !ip.starts_with("127.") {
                                        return Some(ip.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxmox_credentials_creation() {
        let creds = ProxmoxCredentials {
            server_url: "https://192.168.1.100:8006".to_string(),
            username: "root@pam".to_string(),
            password: "secret".to_string(),
        };

        assert_eq!(creds.server_url, "https://192.168.1.100:8006");
        assert_eq!(creds.username, "root@pam");
        assert_eq!(creds.password, "secret");
    }

    #[test]
    fn test_proxmox_session_creation() {
        let session = ProxmoxSession {
            server_url: "https://192.168.1.100:8006".to_string(),
            ticket: "test-ticket".to_string(),
            csrf_token: "test-csrf".to_string(),
        };

        assert_eq!(session.server_url, "https://192.168.1.100:8006");
        assert_eq!(session.ticket, "test-ticket");
        assert_eq!(session.csrf_token, "test-csrf");
    }

    #[test]
    fn test_proxmox_vm_config_creation() {
        let config = ProxmoxVmConfig {
            node: "pve".to_string(),
            storage: "local-lvm".to_string(),
            vm_id: 100,
            name: "Home Assistant".to_string(),
            cpu_cores: 4,
            memory_mb: 4096,
            disk_size_gb: 32,
            auto_start: true,
        };

        assert_eq!(config.node, "pve");
        assert_eq!(config.storage, "local-lvm");
        assert_eq!(config.vm_id, 100);
        assert_eq!(config.name, "Home Assistant");
        assert_eq!(config.cpu_cores, 4);
        assert_eq!(config.memory_mb, 4096);
        assert_eq!(config.disk_size_gb, 32);
        assert!(config.auto_start);
    }

    // Version parsing tests
    #[test]
    fn test_parse_version_valid_standard() {
        assert_eq!(parse_version("8.4.1"), Some((8, 4, 1)));
        assert_eq!(parse_version("8.0.0"), Some((8, 0, 0)));
        assert_eq!(parse_version("10.2.3"), Some((10, 2, 3)));
    }

    #[test]
    fn test_parse_version_valid_two_parts() {
        // Should parse versions with only major.minor (patch defaults to 0)
        assert_eq!(parse_version("8.4"), Some((8, 4, 0)));
        assert_eq!(parse_version("10.2"), Some((10, 2, 0)));
        assert_eq!(parse_version("1.0"), Some((1, 0, 0)));
    }

    #[test]
    fn test_parse_version_valid_large_numbers() {
        assert_eq!(parse_version("99.99.99"), Some((99, 99, 99)));
        assert_eq!(parse_version("100.200.300"), Some((100, 200, 300)));
    }

    #[test]
    fn test_parse_version_invalid_single_part() {
        // Single number is not a valid version (needs at least major.minor)
        assert_eq!(parse_version("8"), None);
        assert_eq!(parse_version("100"), None);
    }

    #[test]
    fn test_parse_version_invalid_empty() {
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_parse_version_invalid_format() {
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("abc.def.ghi"), None);
        // Note: "8.4.x" parses as (8, 4, 0) because patch parsing is lenient
        assert_eq!(parse_version("8.4.x"), Some((8, 4, 0)));
    }

    #[test]
    fn test_parse_version_invalid_negative() {
        assert_eq!(parse_version("-1.0.0"), None);
        assert_eq!(parse_version("8.-4.1"), None);
    }

    #[test]
    fn test_parse_version_with_extra_parts() {
        // Should parse versions with extra parts (ignoring them)
        assert_eq!(parse_version("8.4.1.2"), Some((8, 4, 1)));
        assert_eq!(parse_version("10.2.3.4.5"), Some((10, 2, 3)));
    }

    #[test]
    fn test_parse_version_with_whitespace() {
        // Version strings with whitespace in major/minor should fail
        // Whitespace in patch is tolerated (defaults to 0)
        assert_eq!(parse_version(" 8.4.1"), None);
        assert_eq!(parse_version("8.4.1 "), Some((8, 4, 0))); // "1 " fails to parse
        assert_eq!(parse_version("8. 4.1"), None);
        assert_eq!(parse_version("8.4. 1"), Some((8, 4, 0))); // " 1" fails to parse
    }

    // Version comparison tests
    #[test]
    fn test_version_meets_minimum_equal() {
        // Exact match should meet minimum
        assert!(version_meets_minimum((8, 4, 1), (8, 4, 1)));
        assert!(version_meets_minimum((10, 0, 0), (10, 0, 0)));
    }

    #[test]
    fn test_version_meets_minimum_higher_major() {
        // Higher major version should always meet minimum
        assert!(version_meets_minimum((9, 0, 0), (8, 4, 1)));
        assert!(version_meets_minimum((10, 0, 0), (8, 4, 1)));
        assert!(version_meets_minimum((100, 0, 0), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_higher_minor() {
        // Same major, higher minor should meet minimum
        assert!(version_meets_minimum((8, 5, 0), (8, 4, 1)));
        assert!(version_meets_minimum((8, 10, 0), (8, 4, 1)));
        assert!(version_meets_minimum((8, 4, 2), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_higher_patch() {
        // Same major and minor, higher patch should meet minimum
        assert!(version_meets_minimum((8, 4, 2), (8, 4, 1)));
        assert!(version_meets_minimum((8, 4, 10), (8, 4, 1)));
        assert!(version_meets_minimum((8, 4, 100), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_lower_major() {
        // Lower major version should not meet minimum
        assert!(!version_meets_minimum((7, 9, 9), (8, 4, 1)));
        assert!(!version_meets_minimum((6, 10, 10), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_lower_minor() {
        // Same major, lower minor should not meet minimum
        assert!(!version_meets_minimum((8, 3, 9), (8, 4, 1)));
        assert!(!version_meets_minimum((8, 0, 0), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_lower_patch() {
        // Same major and minor, lower patch should not meet minimum
        assert!(!version_meets_minimum((8, 4, 0), (8, 4, 1)));
    }

    #[test]
    fn test_version_meets_minimum_mixed_comparisons() {
        // Test various edge cases
        let min = (8, 4, 1);

        // Just below minimum in different ways
        assert!(!version_meets_minimum((8, 4, 0), min));
        assert!(!version_meets_minimum((8, 3, 100), min));
        assert!(!version_meets_minimum((7, 100, 100), min));

        // Just above minimum in different ways
        assert!(version_meets_minimum((8, 4, 2), min));
        assert!(version_meets_minimum((8, 5, 0), min));
        assert!(version_meets_minimum((9, 0, 0), min));
    }

    #[test]
    fn test_version_meets_minimum_zero_versions() {
        assert!(version_meets_minimum((0, 0, 1), (0, 0, 0)));
        assert!(version_meets_minimum((1, 0, 0), (0, 0, 0)));
        assert!(!version_meets_minimum((0, 0, 0), (0, 0, 1)));
    }

    // ProxmoxSession struct tests
    #[test]
    fn test_proxmox_session_with_trailing_slash() {
        let session = ProxmoxSession {
            server_url: "https://192.168.1.100:8006/".to_string(),
            ticket: "ticket123".to_string(),
            csrf_token: "csrf123".to_string(),
        };

        assert!(session.server_url.ends_with('/'));
        assert_eq!(session.ticket, "ticket123");
        assert_eq!(session.csrf_token, "csrf123");
    }

    #[test]
    fn test_proxmox_session_with_empty_strings() {
        let session = ProxmoxSession {
            server_url: String::new(),
            ticket: String::new(),
            csrf_token: String::new(),
        };

        assert!(session.server_url.is_empty());
        assert!(session.ticket.is_empty());
        assert!(session.csrf_token.is_empty());
    }

    // ProxmoxVmConfig struct tests
    #[test]
    fn test_proxmox_vm_config_minimal_resources() {
        let config = ProxmoxVmConfig {
            node: "node1".to_string(),
            storage: "local".to_string(),
            vm_id: 100,
            name: "TestVM".to_string(),
            cpu_cores: 1,
            memory_mb: 512,
            disk_size_gb: 8,
            auto_start: false,
        };

        assert_eq!(config.cpu_cores, 1);
        assert_eq!(config.memory_mb, 512);
        assert_eq!(config.disk_size_gb, 8);
        assert!(!config.auto_start);
    }

    #[test]
    fn test_proxmox_vm_config_large_resources() {
        let config = ProxmoxVmConfig {
            node: "node1".to_string(),
            storage: "local-lvm".to_string(),
            vm_id: 999,
            name: "Large VM".to_string(),
            cpu_cores: 32,
            memory_mb: 65536,
            disk_size_gb: 500,
            auto_start: true,
        };

        assert_eq!(config.cpu_cores, 32);
        assert_eq!(config.memory_mb, 65536);
        assert_eq!(config.disk_size_gb, 500);
        assert!(config.auto_start);
    }

    #[test]
    fn test_proxmox_vm_config_special_characters_in_name() {
        let config = ProxmoxVmConfig {
            node: "pve-node-01".to_string(),
            storage: "storage-01".to_string(),
            vm_id: 200,
            name: "Test-VM_123".to_string(),
            cpu_cores: 2,
            memory_mb: 2048,
            disk_size_gb: 16,
            auto_start: false,
        };

        assert_eq!(config.name, "Test-VM_123");
        assert_eq!(config.node, "pve-node-01");
    }

    #[test]
    fn test_proxmox_vm_config_different_storage_types() {
        let storage_types = vec!["local", "local-lvm", "local-zfs"];

        for (i, storage) in storage_types.iter().enumerate() {
            let config = ProxmoxVmConfig {
                node: "pve".to_string(),
                storage: storage.to_string(),
                vm_id: 100 + i as u32,
                name: format!("VM-{}", i),
                cpu_cores: 2,
                memory_mb: 2048,
                disk_size_gb: 32,
                auto_start: false,
            };

            assert_eq!(config.storage, *storage);
        }
    }

    // ProxmoxVmResult struct tests
    #[test]
    fn test_proxmox_vm_result_with_ip() {
        let result = ProxmoxVmResult {
            vm_id: 100,
            node: "pve".to_string(),
            ip_address: Some("192.168.1.50".to_string()),
        };

        assert_eq!(result.vm_id, 100);
        assert_eq!(result.node, "pve");
        assert_eq!(result.ip_address, Some("192.168.1.50".to_string()));
        assert!(result.ip_address.is_some());
    }

    #[test]
    fn test_proxmox_vm_result_without_ip() {
        let result = ProxmoxVmResult {
            vm_id: 100,
            node: "pve".to_string(),
            ip_address: None,
        };

        assert_eq!(result.vm_id, 100);
        assert_eq!(result.node, "pve");
        assert!(result.ip_address.is_none());
    }

    #[test]
    fn test_proxmox_vm_result_with_ipv6() {
        let result = ProxmoxVmResult {
            vm_id: 200,
            node: "node2".to_string(),
            ip_address: Some("fe80::1".to_string()),
        };

        assert_eq!(result.ip_address, Some("fe80::1".to_string()));
    }

    #[test]
    fn test_proxmox_vm_result_different_vm_ids() {
        for vm_id in [100, 999, 10000] {
            let result = ProxmoxVmResult {
                vm_id,
                node: "test-node".to_string(),
                ip_address: Some("10.0.0.1".to_string()),
            };

            assert_eq!(result.vm_id, vm_id);
        }
    }

    // ProxmoxCredentials validation tests
    #[test]
    fn test_proxmox_credentials_with_different_auth_realms() {
        let realms = vec!["root@pam", "user@pve", "admin@custom"];

        for username in realms {
            let creds = ProxmoxCredentials {
                server_url: "https://proxmox.local:8006".to_string(),
                username: username.to_string(),
                password: "password123".to_string(),
            };

            assert_eq!(creds.username, username);
        }
    }

    #[test]
    fn test_proxmox_credentials_with_different_urls() {
        let urls = vec![
            "https://192.168.1.1:8006",
            "https://proxmox.example.com:8006",
            "https://10.0.0.1:8006",
        ];

        for url in urls {
            let creds = ProxmoxCredentials {
                server_url: url.to_string(),
                username: "root@pam".to_string(),
                password: "secret".to_string(),
            };

            assert_eq!(creds.server_url, url);
            assert!(creds.server_url.starts_with("https://"));
        }
    }

    #[test]
    fn test_proxmox_credentials_empty_password() {
        let creds = ProxmoxCredentials {
            server_url: "https://192.168.1.100:8006".to_string(),
            username: "root@pam".to_string(),
            password: String::new(),
        };

        assert!(creds.password.is_empty());
    }

    // ProxmoxNode struct tests
    #[test]
    fn test_proxmox_node_creation() {
        let node = ProxmoxNode {
            name: "pve-node1".to_string(),
            status: "online".to_string(),
            cpu_usage: Some(0.45),
            memory_used: Some(4294967296),
            memory_total: Some(8589934592),
        };

        assert_eq!(node.name, "pve-node1");
        assert_eq!(node.status, "online");
        assert_eq!(node.cpu_usage, Some(0.45));
        assert_eq!(node.memory_used, Some(4294967296));
        assert_eq!(node.memory_total, Some(8589934592));
    }

    #[test]
    fn test_proxmox_node_with_no_metrics() {
        let node = ProxmoxNode {
            name: "offline-node".to_string(),
            status: "offline".to_string(),
            cpu_usage: None,
            memory_used: None,
            memory_total: None,
        };

        assert_eq!(node.status, "offline");
        assert!(node.cpu_usage.is_none());
        assert!(node.memory_used.is_none());
        assert!(node.memory_total.is_none());
    }

    #[test]
    fn test_proxmox_node_different_statuses() {
        let statuses = vec!["online", "offline", "unknown"];

        for status in statuses {
            let node = ProxmoxNode {
                name: "test-node".to_string(),
                status: status.to_string(),
                cpu_usage: Some(0.1),
                memory_used: Some(1000000),
                memory_total: Some(2000000),
            };

            assert_eq!(node.status, status);
        }
    }

    // ProxmoxStorage struct tests
    #[test]
    fn test_proxmox_storage_creation() {
        let storage = ProxmoxStorage {
            name: "local".to_string(),
            storage_type: "dir".to_string(),
            content: vec!["iso".to_string(), "vztmpl".to_string()],
            available: 100000000000,
            total: 500000000000,
            active: true,
        };

        assert_eq!(storage.name, "local");
        assert_eq!(storage.storage_type, "dir");
        assert_eq!(storage.content.len(), 2);
        assert!(storage.content.contains(&"iso".to_string()));
        assert!(storage.active);
    }

    #[test]
    fn test_proxmox_storage_with_multiple_content_types() {
        let storage = ProxmoxStorage {
            name: "local-lvm".to_string(),
            storage_type: "lvm".to_string(),
            content: vec![
                "images".to_string(),
                "rootdir".to_string(),
                "import".to_string(),
            ],
            available: 50000000000,
            total: 100000000000,
            active: true,
        };

        assert_eq!(storage.content.len(), 3);
        assert!(storage.content.contains(&"import".to_string()));
        assert!(storage.content.contains(&"images".to_string()));
    }

    #[test]
    fn test_proxmox_storage_inactive() {
        let storage = ProxmoxStorage {
            name: "backup-storage".to_string(),
            storage_type: "nfs".to_string(),
            content: vec!["backup".to_string()],
            available: 0,
            total: 1000000000000,
            active: false,
        };

        assert!(!storage.active);
        assert_eq!(storage.available, 0);
    }

    #[test]
    fn test_proxmox_storage_different_types() {
        let types = vec!["dir", "lvm", "lvmthin", "zfs", "nfs", "cifs"];

        for storage_type in types {
            let storage = ProxmoxStorage {
                name: format!("{}-storage", storage_type),
                storage_type: storage_type.to_string(),
                content: vec!["images".to_string()],
                available: 1000000,
                total: 2000000,
                active: true,
            };

            assert_eq!(storage.storage_type, storage_type);
        }
    }

    #[test]
    fn test_proxmox_storage_empty_content() {
        let storage = ProxmoxStorage {
            name: "empty-storage".to_string(),
            storage_type: "dir".to_string(),
            content: vec![],
            available: 0,
            total: 0,
            active: false,
        };

        assert!(storage.content.is_empty());
    }

    // Integration tests for minimum version constant
    #[test]
    fn test_min_proxmox_version_constant() {
        // Ensure the minimum version constant is correctly defined
        assert_eq!(MIN_PROXMOX_VERSION, (8, 4, 1));
    }

    #[test]
    fn test_version_against_min_constant() {
        // Test versions against the actual minimum version constant
        assert!(version_meets_minimum((8, 4, 1), MIN_PROXMOX_VERSION));
        assert!(version_meets_minimum((8, 4, 2), MIN_PROXMOX_VERSION));
        assert!(version_meets_minimum((8, 5, 0), MIN_PROXMOX_VERSION));
        assert!(version_meets_minimum((9, 0, 0), MIN_PROXMOX_VERSION));
        assert!(!version_meets_minimum((8, 4, 0), MIN_PROXMOX_VERSION));
        assert!(!version_meets_minimum((8, 3, 9), MIN_PROXMOX_VERSION));
        assert!(!version_meets_minimum((7, 9, 9), MIN_PROXMOX_VERSION));
    }

    #[test]
    fn test_parse_and_check_version_integration() {
        // Integration test: parse version strings and check against minimum
        let test_cases = vec![
            ("8.4.1", true),
            ("8.4.2", true),
            ("8.5.0", true),
            ("9.0.0", true),
            ("10.0.0", true),
            ("8.4.0", false),
            ("8.3.9", false),
            ("7.9.9", false),
            ("8.4", false), // Defaults to 8.4.0 which is below minimum (8.4.1)
        ];

        for (version_str, should_meet_min) in test_cases {
            if let Some(version) = parse_version(version_str) {
                let meets_min = version_meets_minimum(version, MIN_PROXMOX_VERSION);
                assert_eq!(
                    meets_min, should_meet_min,
                    "Version {} should{} meet minimum",
                    version_str,
                    if should_meet_min { "" } else { " not" }
                );
            }
        }
    }
}
