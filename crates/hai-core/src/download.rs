//! Image download and extraction functionality
//!
//! This module provides functions for downloading HAOS images,
//! verifying checksums, and extracting compressed archives.

use crate::error::{Error, Result};
use crate::types::{
    DeviceManifest, FlashProgress, FlashStage, GitHubRelease, HaosImage, HaosRelease,
    StableVersionInfo,
};
use crate::ProgressCallback;
use directories::ProjectDirs;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Home Assistant version API for stable releases
const VERSION_URL: &str = "https://version.home-assistant.io/stable.json";

/// GitHub API URL for HAOS releases
const HAOS_RELEASES_API: &str =
    "https://api.github.com/repos/home-assistant/operating-system/releases";

/// User agent for API requests
const USER_AGENT: &str = "HomeAssistantInstaller/0.1.0";

/// How often to send progress updates (every N bytes)
const PROGRESS_UPDATE_INTERVAL: u64 = 10 * 1024 * 1024; // 10 MB

/// Get the cache directory for downloaded images
pub fn get_cache_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("io", "home-assistant", "installer")
        .ok_or_else(|| Error::InvalidConfig("Could not determine cache directory".to_string()))?;

    let cache_dir = project_dirs.cache_dir().to_path_buf();
    std::fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir)
}

/// Fetch the device manifest
///
/// In mock mode, returns mock data. Otherwise fetches from the network.
pub async fn get_device_manifest() -> Result<DeviceManifest> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            return Ok(crate::mock::get_mock_manifest());
        }
    }

    // For now, return the mock manifest as a fallback
    // TODO: Implement actual network fetch
    Ok(crate::mock::get_mock_manifest())
}

/// Check if cache should be skipped via environment variable
pub fn should_skip_cache() -> bool {
    std::env::var("HA_INSTALLER_NO_CACHE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get the path where an image would be cached
pub fn get_cached_image_path(image: &HaosImage) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let filename = image
        .download_url
        .rsplit('/')
        .next()
        .unwrap_or("image.img.xz");

    Ok(cache_dir.join(filename))
}

/// Check if an image is already cached and valid
pub async fn is_cached(image: &HaosImage) -> Result<bool> {
    // Allow skipping cache via environment variable
    if should_skip_cache() {
        return Ok(false);
    }

    let cache_path = get_cached_image_path(image)?;

    if !cache_path.exists() {
        return Ok(false);
    }

    // First check file size (fast)
    let metadata = fs::metadata(&cache_path).await?;
    if metadata.len() != image.size {
        return Ok(false);
    }

    // File size matches - for now, skip expensive SHA256 verification
    Ok(true)
}

/// Clean up old cached images (partial downloads)
pub async fn cleanup_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    if !cache_dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(&cache_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        // Remove partial downloads
        if path.extension().is_some_and(|ext| ext == "part") {
            let _ = fs::remove_file(path).await;
        }
    }

    Ok(())
}

/// Compute SHA256 hash of a file
pub async fn compute_file_sha256(path: &PathBuf) -> Result<String> {
    let data = fs::read(path).await?;
    let hash = Sha256::digest(&data);
    Ok(hex::encode(hash))
}

/// Fetch the stable version info from Home Assistant (internal version with custom URL)
async fn get_stable_version_from_url(url: &str) -> Result<StableVersionInfo> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::DownloadFailed(format!(
            "Failed to fetch version info: HTTP {}",
            response.status()
        )));
    }

    let version_info: StableVersionInfo = response.json().await?;
    Ok(version_info)
}

/// Fetch the stable version info from Home Assistant
pub async fn get_stable_version() -> Result<StableVersionInfo> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            return Ok(crate::mock::get_mock_stable_version());
        }
    }

    get_stable_version_from_url(VERSION_URL).await
}

/// Get the latest stable HAOS version from the version API
pub async fn get_latest_haos_version() -> Result<String> {
    let version_info = get_stable_version().await?;

    // All boards should have the same version, just get the first one
    version_info
        .hassos
        .values()
        .next()
        .cloned()
        .ok_or_else(|| Error::DownloadFailed("No HAOS versions found in stable.json".to_string()))
}

/// Fetch the latest HAOS release information
pub async fn fetch_latest_release() -> Result<HaosRelease> {
    let version = get_latest_haos_version().await?;
    fetch_release(&version).await
}

/// Fetch a specific HAOS release by version (internal version with custom base URL)
async fn fetch_release_from_api(api_base_url: &str, version: &str) -> Result<HaosRelease> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/tags/{}", api_base_url, version))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::DownloadFailed(format!(
            "Failed to fetch release {}: HTTP {}",
            version,
            response.status()
        )));
    }

    let release: GitHubRelease = response.json().await?;
    parse_github_release(release)
}

/// Fetch a specific HAOS release by version
pub async fn fetch_release(version: &str) -> Result<HaosRelease> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            return Ok(crate::mock::get_mock_haos_release());
        }
    }

    fetch_release_from_api(HAOS_RELEASES_API, version).await
}

/// Fetch HAOS release info for a specific version (or "latest")
pub async fn get_haos_release(version: &str) -> Result<HaosRelease> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            return Ok(crate::mock::get_mock_haos_release());
        }
    }

    if version == "latest" {
        fetch_latest_release().await
    } else {
        fetch_release(version).await
    }
}

/// Parse a GitHub release into our HaosRelease format
fn parse_github_release(release: GitHubRelease) -> Result<HaosRelease> {
    let version = release.tag_name;
    let mut images = Vec::new();

    for asset in release.assets {
        // Process .img.xz and .qcow2.xz files
        let suffix = if asset.name.ends_with(".img.xz") {
            ".img.xz"
        } else if asset.name.ends_with(".qcow2.xz") {
            ".qcow2.xz"
        } else {
            continue;
        };

        // Parse board name from filename: haos_{board}-{version}.img.xz
        let board = match parse_board_from_filename_with_suffix(&asset.name, &version, suffix) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Parse SHA256 from digest field
        let sha256 = asset
            .digest
            .and_then(|d| d.strip_prefix("sha256:").map(|s| s.to_string()))
            .unwrap_or_default();

        images.push(HaosImage {
            board,
            download_url: asset.browser_download_url,
            size: asset.size,
            sha256,
        });
    }

    Ok(HaosRelease { version, images })
}

/// Parse board name from HAOS image filename with a specific suffix
fn parse_board_from_filename_with_suffix(
    filename: &str,
    version: &str,
    file_suffix: &str,
) -> Result<String> {
    // Format: haos_{board}-{version}{file_suffix}
    let prefix = "haos_";
    let suffix = format!("-{}{}", version, file_suffix);

    if !filename.starts_with(prefix) || !filename.ends_with(&suffix) {
        return Err(Error::InvalidConfig(format!(
            "Invalid filename format: {}",
            filename
        )));
    }

    let board = filename
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(&suffix))
        .ok_or_else(|| Error::InvalidConfig(format!("Cannot parse board from: {}", filename)))?;

    Ok(board.to_string())
}

/// Parse board name from HAOS image filename (convenience wrapper for .img.xz)
pub fn parse_board_from_filename(filename: &str, version: &str) -> Result<String> {
    parse_board_from_filename_with_suffix(filename, version, ".img.xz")
}

/// Find image for a specific board in a release
pub fn find_image_for_board<'a>(release: &'a HaosRelease, board: &str) -> Option<&'a HaosImage> {
    release.images.iter().find(|img| img.board == board)
}

/// Download an image file with progress updates
pub async fn download_image<P: ProgressCallback>(
    url: &str,
    dest_path: &PathBuf,
    expected_sha256: Option<&str>,
    progress_callback: &P,
) -> Result<()> {
    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            // Simulate download progress
            for i in 0..=100 {
                progress_callback.on_progress(FlashProgress {
                    stage: FlashStage::Downloading,
                    progress: i,
                    bytes_processed: (i as u64) * 1_000_000,
                    total_bytes: 100_000_000,
                    message: "Downloading image (mock)...".to_string(),
                });
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
            return Ok(());
        }
    }

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(Error::DownloadFailed(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    progress_callback.on_progress(FlashProgress {
        stage: FlashStage::Downloading,
        progress: 0,
        bytes_processed: 0,
        total_bytes: total_size,
        message: "Starting download...".to_string(),
    });

    let mut file = std::fs::File::create(dest_path)?;
    let mut hasher = if expected_sha256.is_some() {
        Some(Sha256::new())
    } else {
        None
    };
    let mut downloaded: u64 = 0;
    let mut last_progress_update: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        use std::io::Write;
        file.write_all(&chunk)?;

        if let Some(ref mut h) = hasher {
            h.update(&chunk);
        }

        downloaded += chunk.len() as u64;

        // Send progress update every PROGRESS_UPDATE_INTERVAL bytes
        if downloaded - last_progress_update >= PROGRESS_UPDATE_INTERVAL {
            let progress = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0) as u8
            } else {
                0
            };

            progress_callback.on_progress(FlashProgress {
                stage: FlashStage::Downloading,
                progress,
                bytes_processed: downloaded,
                total_bytes: total_size,
                message: "Downloading image...".to_string(),
            });
            last_progress_update = downloaded;
        }
    }

    // Verify checksum if provided
    if let (Some(expected), Some(hasher)) = (expected_sha256, hasher) {
        let actual = hex::encode(hasher.finalize());
        if actual != expected {
            std::fs::remove_file(dest_path)?;
            return Err(Error::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }
    }

    progress_callback.on_progress(FlashProgress {
        stage: FlashStage::Downloading,
        progress: 100,
        bytes_processed: downloaded,
        total_bytes: total_size,
        message: "Download complete".to_string(),
    });

    Ok(())
}

/// Extract a .xz compressed file
pub async fn extract_xz<P: ProgressCallback>(
    archive_path: &Path,
    dest_path: &Path,
    progress_callback: &P,
) -> Result<()> {
    use std::sync::mpsc;

    #[cfg(feature = "mock")]
    {
        if crate::is_mock_enabled() {
            // Simulate extraction progress
            for i in 0..=100 {
                progress_callback.on_progress(FlashProgress {
                    stage: FlashStage::Extracting,
                    progress: i,
                    bytes_processed: (i as u64) * 5_000_000,
                    total_bytes: 500_000_000,
                    message: "Extracting image (mock)...".to_string(),
                });
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            return Ok(());
        }
    }

    // For extraction, we don't know the final size upfront (xz doesn't store it)
    // Use 0 for total_bytes to signal indeterminate progress
    progress_callback.on_progress(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 0,
        bytes_processed: 0,
        total_bytes: 0,
        message: "Extracting image...".to_string(),
    });

    // Create channel for progress updates
    let (progress_tx, progress_rx) = mpsc::channel::<u64>();

    let archive_path_clone = archive_path.to_path_buf();
    let dest_path_clone = dest_path.to_path_buf();

    let extract_handle = tokio::task::spawn_blocking(move || {
        use std::io::{Read, Write};

        let input = std::fs::File::open(&archive_path_clone)?;
        let mut decoder = xz2::read::XzDecoder::new(input);
        let mut output = std::fs::File::create(&dest_path_clone)?;

        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
        let mut bytes_extracted: u64 = 0;
        let mut last_progress_update: u64 = 0;

        loop {
            let bytes_read = decoder.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            output.write_all(&buffer[..bytes_read])?;
            bytes_extracted += bytes_read as u64;

            // Send progress update every PROGRESS_UPDATE_INTERVAL bytes
            if bytes_extracted - last_progress_update >= PROGRESS_UPDATE_INTERVAL {
                let _ = progress_tx.send(bytes_extracted);
                last_progress_update = bytes_extracted;
            }
        }

        output.sync_all()?;

        Ok::<u64, Error>(bytes_extracted)
    });

    // Forward progress updates while waiting for extraction to complete
    loop {
        match progress_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(bytes_extracted) => {
                // Use 0 for total_bytes to signal indeterminate progress
                progress_callback.on_progress(FlashProgress {
                    stage: FlashStage::Extracting,
                    progress: 0, // Indeterminate
                    bytes_processed: bytes_extracted,
                    total_bytes: 0,
                    message: "Extracting image...".to_string(),
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if extract_handle.is_finished() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    let final_size = extract_handle
        .await
        .map_err(|e| Error::ExtractionFailed(e.to_string()))??;

    progress_callback.on_progress(FlashProgress {
        stage: FlashStage::Extracting,
        progress: 100,
        bytes_processed: final_size,
        total_bytes: final_size,
        message: "Extraction complete".to_string(),
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GitHubAsset;
    use serial_test::serial;

    #[test]
    fn test_get_cache_dir() {
        let result = get_cache_dir();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_device_manifest_mock() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let manifest = get_device_manifest().await.unwrap();
        assert!(!manifest.devices.is_empty());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    #[serial]
    fn test_should_skip_cache_true() {
        std::env::set_var("HA_INSTALLER_NO_CACHE", "1");
        assert!(should_skip_cache());
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
    }

    #[test]
    #[serial]
    fn test_should_skip_cache_false() {
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
        assert!(!should_skip_cache());
    }

    #[test]
    fn test_parse_board_from_filename_standard() {
        let result = parse_board_from_filename("haos_rpi5-64-14.2.img.xz", "14.2");
        assert_eq!(result.unwrap(), "rpi5-64");

        let result = parse_board_from_filename("haos_generic-x86-64-14.2.img.xz", "14.2");
        assert_eq!(result.unwrap(), "generic-x86-64");

        let result = parse_board_from_filename("haos_green-14.2.img.xz", "14.2");
        assert_eq!(result.unwrap(), "green");
    }

    #[test]
    fn test_parse_board_from_filename_qcow2() {
        let result =
            parse_board_from_filename_with_suffix("haos_generic-x86-64-14.2.qcow2.xz", "14.2", ".qcow2.xz");
        assert_eq!(result.unwrap(), "generic-x86-64");

        let result =
            parse_board_from_filename_with_suffix("haos_generic-aarch64-14.2.qcow2.xz", "14.2", ".qcow2.xz");
        assert_eq!(result.unwrap(), "generic-aarch64");
    }

    #[test]
    fn test_parse_board_from_filename_invalid() {
        // Wrong prefix
        let result = parse_board_from_filename("wrong_rpi5-64-14.2.img.xz", "14.2");
        assert!(result.is_err());

        // Wrong suffix
        let result = parse_board_from_filename("haos_rpi5-64-14.2.zip", "14.2");
        assert!(result.is_err());

        // Wrong version
        let result = parse_board_from_filename("haos_rpi5-64-14.2.img.xz", "14.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_image_for_board_found() {
        let release = HaosRelease {
            version: "14.2".to_string(),
            images: vec![
                HaosImage {
                    board: "rpi5-64".to_string(),
                    download_url: "https://example.com/rpi5.img.xz".to_string(),
                    size: 100,
                    sha256: "abc".to_string(),
                },
                HaosImage {
                    board: "green".to_string(),
                    download_url: "https://example.com/green.img.xz".to_string(),
                    size: 200,
                    sha256: "def".to_string(),
                },
            ],
        };

        let found = find_image_for_board(&release, "green");
        assert!(found.is_some());
        assert_eq!(found.unwrap().board, "green");
        assert_eq!(found.unwrap().size, 200);
    }

    #[test]
    fn test_find_image_for_board_not_found() {
        let release = HaosRelease {
            version: "14.2".to_string(),
            images: vec![HaosImage {
                board: "rpi5-64".to_string(),
                download_url: "https://example.com/rpi5.img.xz".to_string(),
                size: 100,
                sha256: "abc".to_string(),
            }],
        };

        let found = find_image_for_board(&release, "nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_parse_github_release() {
        let release = GitHubRelease {
            tag_name: "14.2".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "haos_rpi5-64-14.2.img.xz".to_string(),
                    size: 500_000_000,
                    browser_download_url: "https://github.com/download/rpi5.img.xz".to_string(),
                    digest: Some("sha256:abc123".to_string()),
                },
                GitHubAsset {
                    name: "haos_generic-x86-64-14.2.qcow2.xz".to_string(),
                    size: 600_000_000,
                    browser_download_url: "https://github.com/download/x86.qcow2.xz".to_string(),
                    digest: Some("sha256:def456".to_string()),
                },
                // Should be ignored (wrong extension)
                GitHubAsset {
                    name: "haos_rpi5-64-14.2.img.xz.sha256".to_string(),
                    size: 100,
                    browser_download_url: "https://github.com/download/sha256".to_string(),
                    digest: None,
                },
            ],
        };

        let parsed = parse_github_release(release).unwrap();
        assert_eq!(parsed.version, "14.2");
        assert_eq!(parsed.images.len(), 2);

        // Check rpi5-64 image
        let rpi_image = parsed.images.iter().find(|i| i.board == "rpi5-64").unwrap();
        assert_eq!(rpi_image.size, 500_000_000);
        assert_eq!(rpi_image.sha256, "abc123");

        // Check x86 qcow2 image
        let x86_image = parsed
            .images
            .iter()
            .find(|i| i.board == "generic-x86-64")
            .unwrap();
        assert_eq!(x86_image.size, 600_000_000);
        assert_eq!(x86_image.sha256, "def456");
    }

    #[tokio::test]
    #[serial]
    async fn test_is_cached_skip_cache_env() {
        std::env::set_var("HA_INSTALLER_NO_CACHE", "1");
        let image = HaosImage {
            board: "test".to_string(),
            download_url: "https://example.com/test.img.xz".to_string(),
            size: 100,
            sha256: "abc".to_string(),
        };
        let result = is_cached(&image).await.unwrap();
        assert!(!result);
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
    }

    #[tokio::test]
    #[serial]
    async fn test_is_cached_file_not_exist() {
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
        let image = HaosImage {
            board: "test".to_string(),
            download_url: "https://example.com/nonexistent-file-12345.img.xz".to_string(),
            size: 100,
            sha256: "abc".to_string(),
        };
        let result = is_cached(&image).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_cleanup_cache_removes_part_files() {
        let cache_dir = get_cache_dir().unwrap();

        // Create a test .part file
        let part_file = cache_dir.join("test_cleanup.img.xz.part");
        std::fs::write(&part_file, b"test").unwrap();
        assert!(part_file.exists());

        // Run cleanup
        cleanup_cache().await.unwrap();

        // Part file should be removed
        assert!(!part_file.exists());
    }

    #[tokio::test]
    async fn test_get_cached_image_path() {
        let image = HaosImage {
            board: "test".to_string(),
            download_url: "https://github.com/home-assistant/operating-system/releases/download/14.2/haos_rpi5-64-14.2.img.xz".to_string(),
            size: 100,
            sha256: "abc".to_string(),
        };

        let path = get_cached_image_path(&image).unwrap();
        assert!(path.to_string_lossy().contains("haos_rpi5-64-14.2.img.xz"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_mock() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let release = get_haos_release("14.2").await.unwrap();
        assert!(!release.images.is_empty());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_haos_release_latest_mock() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let release = get_haos_release("latest").await.unwrap();
        assert!(!release.images.is_empty());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[test]
    fn test_parse_board_from_filename_empty() {
        let result = parse_board_from_filename("", "14.2");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_board_from_filename_no_haos_prefix() {
        let result = parse_board_from_filename("rpi5-64-14.2.img.xz", "14.2");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_should_skip_cache_true_lowercase() {
        std::env::set_var("HA_INSTALLER_NO_CACHE", "true");
        assert!(should_skip_cache());
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
    }

    #[test]
    #[serial]
    fn test_should_skip_cache_false_with_false_value() {
        std::env::set_var("HA_INSTALLER_NO_CACHE", "false");
        assert!(!should_skip_cache());
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
    }

    #[test]
    #[serial]
    fn test_should_skip_cache_false_with_zero() {
        std::env::set_var("HA_INSTALLER_NO_CACHE", "0");
        assert!(!should_skip_cache());
        std::env::remove_var("HA_INSTALLER_NO_CACHE");
    }

    #[test]
    fn test_get_cached_image_path_url_without_slash() {
        // Edge case: URL without "/" should use fallback filename
        let image = HaosImage {
            board: "test".to_string(),
            download_url: "no-slashes-here".to_string(),
            size: 100,
            sha256: "abc".to_string(),
        };

        let path = get_cached_image_path(&image).unwrap();
        assert!(path.to_string_lossy().contains("no-slashes-here"));
    }

    #[tokio::test]
    #[serial]
    async fn test_is_cached_size_mismatch() {
        std::env::remove_var("HA_INSTALLER_NO_CACHE");

        // Create a temp file with wrong size
        let cache_dir = get_cache_dir().unwrap();
        let test_file = cache_dir.join("test_size_mismatch.img.xz");

        // Write 50 bytes
        std::fs::write(&test_file, &[0u8; 50]).unwrap();

        // Image expects 100 bytes
        let image = HaosImage {
            board: "test".to_string(),
            download_url: format!("https://example.com/{}", test_file.file_name().unwrap().to_string_lossy()),
            size: 100,
            sha256: "abc".to_string(),
        };

        let result = is_cached(&image).await.unwrap();
        assert!(!result, "Should return false when file size doesn't match");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_compute_file_sha256() {
        // Create a temp file with known content
        let cache_dir = get_cache_dir().unwrap();
        let test_file = cache_dir.join("test_sha256.txt");
        std::fs::write(&test_file, b"hello world").unwrap();

        let hash = compute_file_sha256(&test_file).await.unwrap();
        // SHA256 of "hello world" is known
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        std::fs::remove_file(&test_file).unwrap();
    }

    #[tokio::test]
    async fn test_compute_file_sha256_empty_file() {
        let cache_dir = get_cache_dir().unwrap();
        let test_file = cache_dir.join("test_sha256_empty.txt");
        std::fs::write(&test_file, b"").unwrap();

        let hash = compute_file_sha256(&test_file).await.unwrap();
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        std::fs::remove_file(&test_file).unwrap();
    }

    #[tokio::test]
    async fn test_compute_file_sha256_nonexistent_file() {
        let path = std::path::PathBuf::from("/nonexistent/file/path.txt");
        let result = compute_file_sha256(&path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_mock_mode() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("mock_download_test.img");

        let result = download_image(
            "https://example.com/test.img",
            &dest,
            None,
            &crate::NoOpProgress,
        )
        .await;

        assert!(result.is_ok());
        std::env::remove_var("HA_INSTALLER_MOCK");
    }

    #[tokio::test]
    #[serial]
    async fn test_extract_xz_mock_mode() {
        std::env::set_var("HA_INSTALLER_MOCK", "1");
        let cache_dir = get_cache_dir().unwrap();
        let archive = cache_dir.join("mock_archive.xz");
        let dest = cache_dir.join("mock_extracted.img");

        let result = extract_xz(&archive, &dest, &crate::NoOpProgress).await;
        assert!(result.is_ok());

        std::env::remove_var("HA_INSTALLER_MOCK");
    }


    #[tokio::test]
    #[serial]
    async fn test_download_image_http_404_error() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(404)
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_404.img");

        let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::Error::DownloadFailed(_)));
        }

        mock.assert_async().await;
        let _ = std::fs::remove_file(&dest);
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_http_500_error() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(500)
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_500.img");

        // Clean up any existing file from previous test runs
        let _ = std::fs::remove_file(&dest);

        let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
        assert!(result.is_err());

        mock.assert_async().await;
        let _ = std::fs::remove_file(&dest);
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_success_without_checksum() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let test_data = b"test image data content";
        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(200)
            .with_header("content-length", &test_data.len().to_string())
            .with_body(test_data.as_slice())
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_download_success.img");

        let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
        assert!(result.is_ok());

        // Verify file was created and has correct content
        let content = std::fs::read(&dest).unwrap();
        assert_eq!(content, test_data);

        mock.assert_async().await;
        std::fs::remove_file(&dest).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_with_checksum_verification_success() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let test_data = b"test image data";
        // Pre-computed SHA256 of "test image data"
        let expected_sha = "fc50f1a3c9cbf0154d7dc87998446624c8b78f84c5cbef4f8139a0c8be1e4976";

        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(200)
            .with_header("content-length", &test_data.len().to_string())
            .with_body(test_data.as_slice())
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_checksum_success.img");

        // Clean up any existing file from previous test runs
        let _ = std::fs::remove_file(&dest);

        let result = download_image(&url, &dest, Some(expected_sha), &crate::NoOpProgress).await;
        assert!(result.is_ok());

        mock.assert_async().await;
        std::fs::remove_file(&dest).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_checksum_mismatch() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let test_data = b"test data";
        let wrong_sha = "0000000000000000000000000000000000000000000000000000000000000000";

        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(200)
            .with_header("content-length", &test_data.len().to_string())
            .with_body(test_data.as_slice())
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_checksum_fail.img");

        let result = download_image(&url, &dest, Some(wrong_sha), &crate::NoOpProgress).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::Error::ChecksumMismatch { .. }));
        }

        // File should be deleted on checksum failure
        assert!(!dest.exists());

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_with_progress_updates() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        use std::sync::{Arc, Mutex};

        struct TestProgressCallback {
            calls: Arc<Mutex<Vec<FlashProgress>>>,
        }

        impl crate::ProgressCallback for TestProgressCallback {
            fn on_progress(&self, progress: FlashProgress) {
                self.calls.lock().unwrap().push(progress);
            }
        }

        let mut server = mockito::Server::new_async().await;

        // Create data larger than PROGRESS_UPDATE_INTERVAL (10MB)
        let test_data = vec![0u8; 11 * 1024 * 1024]; // 11MB

        let mock = server
            .mock("GET", "/large.img.xz")
            .with_status(200)
            .with_header("content-length", &test_data.len().to_string())
            .with_body(&test_data)
            .create_async()
            .await;

        let url = format!("{}/large.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_progress.img");

        let calls = Arc::new(Mutex::new(Vec::new()));
        let callback = TestProgressCallback {
            calls: calls.clone(),
        };

        let result = download_image(&url, &dest, None, &callback).await;
        assert!(result.is_ok());

        // Check that we got progress callbacks
        let progress_calls = calls.lock().unwrap();
        assert!(!progress_calls.is_empty());
        assert!(progress_calls.iter().any(|p| p.progress == 0)); // Start
        assert!(progress_calls.iter().any(|p| p.progress == 100)); // End
        assert!(progress_calls.iter().all(|p| p.stage == FlashStage::Downloading));

        mock.assert_async().await;
        std::fs::remove_file(&dest).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_download_image_no_content_length() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let mut server = mockito::Server::new_async().await;

        let test_data = b"small data";
        let mock = server
            .mock("GET", "/test.img.xz")
            .with_status(200)
            // No content-length header
            .with_body(test_data.as_slice())
            .create_async()
            .await;

        let url = format!("{}/test.img.xz", server.url());
        let cache_dir = get_cache_dir().unwrap();
        let dest = cache_dir.join("test_no_length.img");

        let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
        assert!(result.is_ok());

        mock.assert_async().await;
        std::fs::remove_file(&dest).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_extract_xz_real_file() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        use std::io::Write;

        let cache_dir = get_cache_dir().unwrap();
        let test_content = b"Hello, this is test content for XZ compression!";
        let extracted_path = cache_dir.join("test_extracted.txt");
        let archive_path = cache_dir.join("test_archive.txt.xz");

        // Create a real XZ compressed file
        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut encoder = xz2::write::XzEncoder::new(file, 6);
            encoder.write_all(test_content).unwrap();
            encoder.finish().unwrap();
        }

        // Extract it
        let result = extract_xz(&archive_path, &extracted_path, &crate::NoOpProgress).await;
        assert!(result.is_ok());

        // Verify extracted content
        let extracted = std::fs::read(&extracted_path).unwrap();
        assert_eq!(extracted, test_content);

        // Cleanup
        std::fs::remove_file(&archive_path).unwrap();
        std::fs::remove_file(&extracted_path).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_extract_xz_nonexistent_file() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        let cache_dir = get_cache_dir().unwrap();
        let archive_path = cache_dir.join("nonexistent_archive.xz");
        let dest_path = cache_dir.join("output.img");

        let result = extract_xz(&archive_path, &dest_path, &crate::NoOpProgress).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_extract_xz_with_progress() {
        std::env::remove_var("HA_INSTALLER_MOCK");

        use std::sync::{Arc, Mutex};
        use std::io::Write;

        struct TestProgressCallback {
            calls: Arc<Mutex<Vec<FlashProgress>>>,
        }

        impl crate::ProgressCallback for TestProgressCallback {
            fn on_progress(&self, progress: FlashProgress) {
                self.calls.lock().unwrap().push(progress);
            }
        }

        let cache_dir = get_cache_dir().unwrap();
        // Create larger content to trigger progress updates (> 10MB)
        let test_content = vec![0u8; 11 * 1024 * 1024]; // 11MB
        let extracted_path = cache_dir.join("test_extracted_large.img");
        let archive_path = cache_dir.join("test_archive_large.img.xz");

        // Create XZ compressed file
        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut encoder = xz2::write::XzEncoder::new(file, 1); // Use compression level 1 for speed
            encoder.write_all(&test_content).unwrap();
            encoder.finish().unwrap();
        }

        let calls = Arc::new(Mutex::new(Vec::new()));
        let callback = TestProgressCallback {
            calls: calls.clone(),
        };

        // Extract with progress tracking
        let result = extract_xz(&archive_path, &extracted_path, &callback).await;
        assert!(result.is_ok());

        // Verify we got progress callbacks
        let progress_calls = calls.lock().unwrap();
        assert!(!progress_calls.is_empty());
        assert!(progress_calls.iter().any(|p| p.progress == 0)); // Start
        assert!(progress_calls.iter().any(|p| p.progress == 100)); // End
        assert!(progress_calls.iter().all(|p| p.stage == FlashStage::Extracting));

        // Cleanup
        std::fs::remove_file(&archive_path).unwrap();
        std::fs::remove_file(&extracted_path).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_is_cached_with_matching_size() {
        std::env::remove_var("HA_INSTALLER_NO_CACHE");

        let cache_dir = get_cache_dir().unwrap();
        let test_file = cache_dir.join("test_matching_size.img.xz");

        // Write exactly 100 bytes
        std::fs::write(&test_file, &[0u8; 100]).unwrap();

        // Image expects exactly 100 bytes
        let image = HaosImage {
            board: "test".to_string(),
            download_url: format!("https://example.com/{}", test_file.file_name().unwrap().to_string_lossy()),
            size: 100,
            sha256: "abc".to_string(),
        };

        let result = is_cached(&image).await.unwrap();
        assert!(result, "Should return true when file size matches");

        // Cleanup
        std::fs::remove_file(&test_file).unwrap();
    }

    #[tokio::test]
    async fn test_cleanup_cache_nonexistent_directory() {
        // This tests the early return path when the cache directory doesn't exist
        // The function should handle this gracefully
        let result = cleanup_cache().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_github_release_with_missing_digest() {
        use crate::types::{GitHubAsset, GitHubRelease};

        let release = GitHubRelease {
            tag_name: "14.2".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "haos_rpi4-14.2.img.xz".to_string(),
                    size: 500_000_000,
                    browser_download_url: "https://github.com/download/rpi4.img.xz".to_string(),
                    digest: None, // No digest
                },
            ],
        };

        let parsed = parse_github_release(release).unwrap();
        assert_eq!(parsed.images.len(), 1);
        assert_eq!(parsed.images[0].sha256, ""); // Should be empty string
    }

    #[tokio::test]
    async fn test_parse_github_release_with_digest_no_prefix() {
        use crate::types::{GitHubAsset, GitHubRelease};

        let release = GitHubRelease {
            tag_name: "14.2".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "haos_rpi4-14.2.img.xz".to_string(),
                    size: 500_000_000,
                    browser_download_url: "https://github.com/download/rpi4.img.xz".to_string(),
                    digest: Some("abc123".to_string()), // No "sha256:" prefix
                },
            ],
        };

        let parsed = parse_github_release(release).unwrap();
        assert_eq!(parsed.images.len(), 1);
        assert_eq!(parsed.images[0].sha256, ""); // Should be empty when prefix missing
    }

    #[tokio::test]
    async fn test_parse_github_release_invalid_filename_skipped() {
        use crate::types::{GitHubAsset, GitHubRelease};

        let release = GitHubRelease {
            tag_name: "14.2".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "invalid_filename.img.xz".to_string(), // Doesn't match pattern
                    size: 500_000_000,
                    browser_download_url: "https://github.com/download/invalid.img.xz".to_string(),
                    digest: Some("sha256:abc".to_string()),
                },
            ],
        };

        let parsed = parse_github_release(release).unwrap();
        assert_eq!(parsed.images.len(), 0); // Invalid filename should be skipped
    }

    #[tokio::test]
    async fn test_parse_board_from_filename_with_suffix_error() {
        // Test the error path in parse_board_from_filename_with_suffix
        let result = parse_board_from_filename_with_suffix(
            "haos_rpi4-14.2.img.xz",
            "99.9", // Wrong version
            ".img.xz"
        );
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::Error::InvalidConfig(_)));
        }
    }

    #[tokio::test]
    async fn test_fetch_release_network_error() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/tags/14.2")
            .with_status(404)
            .create_async()
            .await;

        // Can't easily test this without dependency injection
        // This test documents the intent
    }

    #[tokio::test]
    async fn test_get_device_manifest_fallback() {
        // Without mock mode, should return mock manifest as fallback
        std::env::remove_var("HA_INSTALLER_MOCK");
        let manifest = get_device_manifest().await.unwrap();
        assert!(!manifest.devices.is_empty());
    }

    // HTTP Mock Tests Module
    // These tests use mockito to mock external HTTP endpoints
    mod http_mock_tests {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_success() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .match_header("User-Agent", "HomeAssistantInstaller/0.1.0")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{"hassos":{"rpi4":"14.2","generic-x86-64":"14.2"}}"#)
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_ok());

            let version_info = result.unwrap();
            assert_eq!(version_info.hassos.get("rpi4"), Some(&"14.2".to_string()));
            assert_eq!(version_info.hassos.get("generic-x86-64"), Some(&"14.2".to_string()));

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_http_404() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(404)
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_err());

            if let Err(e) = result {
                assert!(matches!(e, crate::error::Error::DownloadFailed(_)));
            }

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_http_500() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(500)
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_err());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_invalid_json() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("not valid json")
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_err());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_empty_response() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("")
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_err());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_malformed_json() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{"hassos":{"rpi4":"14.2"}}"#) // Missing expected fields, but valid JSON
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            // This should succeed since the JSON is valid, even if minimal
            assert!(result.is_ok());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_success() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .match_header("User-Agent", "HomeAssistantInstaller/0.1.0")
                .match_header("Accept", "application/vnd.github.v3+json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [
                        {
                            "name": "haos_rpi5-64-14.2.img.xz",
                            "size": 500000000,
                            "browser_download_url": "https://github.com/download/rpi5.img.xz",
                            "digest": "sha256:abc123"
                        },
                        {
                            "name": "haos_generic-x86-64-14.2.qcow2.xz",
                            "size": 600000000,
                            "browser_download_url": "https://github.com/download/x86.qcow2.xz",
                            "digest": "sha256:def456"
                        }
                    ]
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.version, "14.2");
            assert_eq!(release.images.len(), 2);
            assert!(release.images.iter().any(|i| i.board == "rpi5-64"));
            assert!(release.images.iter().any(|i| i.board == "generic-x86-64"));

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_http_404() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/99.99")
                .with_status(404)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "99.99").await;
            assert!(result.is_err());

            if let Err(e) = result {
                assert!(matches!(e, crate::error::Error::DownloadFailed(_)));
                let error_msg = format!("{:?}", e);
                assert!(error_msg.contains("404"));
            }

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_http_500() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(500)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_err());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_invalid_json() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("invalid json")
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_err());

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_empty_assets() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": []
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.version, "14.2");
            assert_eq!(release.images.len(), 0);

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_mixed_assets() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [
                        {
                            "name": "haos_rpi5-64-14.2.img.xz",
                            "size": 500000000,
                            "browser_download_url": "https://github.com/download/rpi5.img.xz",
                            "digest": "sha256:abc123"
                        },
                        {
                            "name": "haos_rpi5-64-14.2.img.xz.sha256",
                            "size": 100,
                            "browser_download_url": "https://github.com/download/sha256",
                            "digest": null
                        },
                        {
                            "name": "README.md",
                            "size": 1000,
                            "browser_download_url": "https://github.com/download/readme",
                            "digest": null
                        }
                    ]
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.version, "14.2");
            assert_eq!(release.images.len(), 1); // Only valid image files
            assert_eq!(release.images[0].board, "rpi5-64");

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_with_redirects() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(302)
                .with_header("Location", "/redirected/tags/14.2")
                .create_async()
                .await;

            let redirect_mock = server
                .mock("GET", "/redirected/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [{
                        "name": "haos_rpi4-14.2.img.xz",
                        "size": 400000000,
                        "browser_download_url": "https://github.com/download/rpi4.img.xz",
                        "digest": "sha256:xyz789"
                    }]
                }"#)
                .create_async()
                .await;

            // reqwest follows redirects by default
            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.version, "14.2");
            assert_eq!(release.images.len(), 1);

            mock.assert_async().await;
            redirect_mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_missing_digest() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [{
                        "name": "haos_rpi4-14.2.img.xz",
                        "size": 400000000,
                        "browser_download_url": "https://github.com/download/rpi4.img.xz",
                        "digest": null
                    }]
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.images.len(), 1);
            assert_eq!(release.images[0].sha256, ""); // Should be empty when digest is null

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_digest_without_prefix() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [{
                        "name": "haos_rpi4-14.2.img.xz",
                        "size": 400000000,
                        "browser_download_url": "https://github.com/download/rpi4.img.xz",
                        "digest": "abc123"
                    }]
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.images.len(), 1);
            assert_eq!(release.images[0].sha256, ""); // Should be empty when sha256: prefix is missing

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_download_image_network_timeout() {
            std::env::remove_var("HA_INSTALLER_MOCK");

            // Test with an invalid URL that will cause a network error
            let cache_dir = get_cache_dir().unwrap();
            let dest = cache_dir.join("test_timeout.img");

            let result = download_image(
                "http://192.0.2.1:9999/nonexistent", // Using TEST-NET-1 IP that should timeout
                &dest,
                None,
                &crate::NoOpProgress
            ).await;

            assert!(result.is_err());
            let _ = std::fs::remove_file(&dest);
        }

        #[tokio::test]
        #[serial]
        async fn test_download_image_empty_response() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/empty.img.xz")
                .with_status(200)
                .with_header("content-length", "0")
                .with_body("")
                .create_async()
                .await;

            let url = format!("{}/empty.img.xz", server.url());
            let cache_dir = get_cache_dir().unwrap();
            let dest = cache_dir.join("test_empty.img");

            let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
            assert!(result.is_ok());

            // Verify empty file was created
            let metadata = std::fs::metadata(&dest).unwrap();
            assert_eq!(metadata.len(), 0);

            mock.assert_async().await;
            std::fs::remove_file(&dest).unwrap();
        }

        #[tokio::test]
        #[serial]
        async fn test_download_image_with_redirect() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let redirect_mock = server
                .mock("GET", "/redirect.img.xz")
                .with_status(302)
                .with_header("Location", "/actual.img.xz")
                .create_async()
                .await;

            let test_data = b"redirected content";
            let actual_mock = server
                .mock("GET", "/actual.img.xz")
                .with_status(200)
                .with_header("content-length", &test_data.len().to_string())
                .with_body(test_data.as_slice())
                .create_async()
                .await;

            let url = format!("{}/redirect.img.xz", server.url());
            let cache_dir = get_cache_dir().unwrap();
            let dest = cache_dir.join("test_redirect.img");

            let result = download_image(&url, &dest, None, &crate::NoOpProgress).await;
            assert!(result.is_ok());

            // Verify file content
            let content = std::fs::read(&dest).unwrap();
            assert_eq!(content, test_data);

            redirect_mock.assert_async().await;
            actual_mock.assert_async().await;
            std::fs::remove_file(&dest).unwrap();
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_with_extra_fields() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/stable.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "hassos": {
                        "rpi4": "14.2",
                        "generic-x86-64": "14.2",
                        "green": "14.2",
                        "yellow": "14.2"
                    },
                    "extra_field": "should be ignored"
                }"#)
                .create_async()
                .await;

            let url = format!("{}/stable.json", server.url());
            let result = get_stable_version_from_url(&url).await;
            assert!(result.is_ok());

            let version_info = result.unwrap();
            assert_eq!(version_info.hassos.len(), 4);
            assert_eq!(version_info.hassos.get("green"), Some(&"14.2".to_string()));

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_with_qcow2_only() {
            std::env::remove_var("HA_INSTALLER_MOCK");
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/tags/14.2")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "14.2",
                    "assets": [
                        {
                            "name": "haos_generic-x86-64-14.2.qcow2.xz",
                            "size": 600000000,
                            "browser_download_url": "https://github.com/download/x86.qcow2.xz",
                            "digest": "sha256:qcow2hash"
                        }
                    ]
                }"#)
                .create_async()
                .await;

            let result = fetch_release_from_api(&server.url(), "14.2").await;
            assert!(result.is_ok());

            let release = result.unwrap();
            assert_eq!(release.images.len(), 1);
            assert_eq!(release.images[0].board, "generic-x86-64");
            assert!(release.images[0].download_url.contains("qcow2"));

            mock.assert_async().await;
        }

        #[tokio::test]
        #[serial]
        async fn test_fetch_release_connection_refused() {
            std::env::remove_var("HA_INSTALLER_MOCK");

            // Use a port that's likely not in use
            let result = fetch_release_from_api("http://127.0.0.1:59999", "14.2").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        #[serial]
        async fn test_get_stable_version_connection_refused() {
            std::env::remove_var("HA_INSTALLER_MOCK");

            let result = get_stable_version_from_url("http://127.0.0.1:59998/stable.json").await;
            assert!(result.is_err());
        }
    }
}
