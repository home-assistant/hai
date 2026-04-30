//! hai-core - Core library for Home Assistant Installer
//!
//! This library provides the shared business logic for the Home Assistant
//! Installer, including device enumeration, image downloading, disk writing,
//! and VM provisioning for Proxmox and UTM.
//!
//! The library is designed to be frontend-agnostic, supporting both the
//! Tauri desktop application and potential TUI implementations.

pub mod devices;
pub mod disk_writer;
pub mod download;
pub mod error;
pub mod types;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "proxmox")]
pub mod proxmox;

#[cfg(feature = "utm")]
pub mod utm;

pub use error::{Error, Result};
pub use types::*;

/// Trait for receiving progress updates during long-running operations.
///
/// This trait abstracts the progress reporting mechanism, allowing hai-core
/// to work with different frontends (Tauri desktop, TUI, etc.) without
/// coupling to any specific implementation.
///
/// # Example
///
/// ```ignore
/// use hai_core::{FlashProgress, ProgressCallback};
///
/// struct MyProgressHandler;
///
/// impl ProgressCallback for MyProgressHandler {
///     fn on_progress(&self, progress: FlashProgress) {
///         println!("Progress: {}%", progress.progress);
///     }
/// }
/// ```
pub trait ProgressCallback: Send + Sync {
    /// Called when progress is updated during an operation.
    ///
    /// Implementations should handle this method being called frequently
    /// during long-running operations like downloads and disk writes.
    fn on_progress(&self, progress: FlashProgress);
}

/// A no-op progress callback for use when progress reporting is not needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpProgress;

impl ProgressCallback for NoOpProgress {
    fn on_progress(&self, _progress: FlashProgress) {
        // Intentionally empty - used when progress is not needed
    }
}

/// Check if mock mode is enabled via environment variable.
///
/// Mock mode is enabled when the `HA_INSTALLER_MOCK` environment variable
/// is set to "1" or "true". This is useful for testing and development.
pub fn is_mock_enabled() -> bool {
    match std::env::var("HA_INSTALLER_MOCK") {
        Ok(val) => val == "1" || val.to_lowercase() == "true",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_progress_callback() {
        let callback = NoOpProgress;
        let progress = FlashProgress {
            stage: FlashStage::Downloading,
            progress: 50,
            bytes_processed: 1000,
            total_bytes: 2000,
            message: "Test".to_string(),
        };

        // Should not panic
        callback.on_progress(progress);
    }

    #[test]
    fn test_mock_mode_default_disabled() {
        // Remove the env var if it exists
        std::env::remove_var("HA_INSTALLER_MOCK");
        assert!(!is_mock_enabled());
    }
}
