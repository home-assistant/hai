//! Home Assistant Installer - Desktop Application
//!
//! This crate provides the Tauri desktop application for HAI.
//! It uses hai-core for business logic and provides Tauri command wrappers.

mod commands;

use commands::{
    check_for_updates, check_ha_ready, check_ha_updated, check_utm_status, create_utm_vm,
    download_utm_image, flash_image, get_haos_release, get_mac_architecture, get_manifest,
    get_system_info, get_utm_vm_status, is_mock_mode, list_block_devices, list_utm_vms,
    proxmox_connect, proxmox_create_vm, proxmox_get_next_vm_id, proxmox_list_nodes,
    proxmox_list_storage, resize_utm_vm_disk, start_utm_vm,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            is_mock_mode,
            list_block_devices,
            flash_image,
            check_for_updates,
            get_manifest,
            get_haos_release,
            get_system_info,
            // UTM commands (macOS only, stubs on other platforms)
            check_utm_status,
            get_mac_architecture,
            download_utm_image,
            create_utm_vm,
            start_utm_vm,
            resize_utm_vm_disk,
            list_utm_vms,
            get_utm_vm_status,
            check_ha_ready,
            check_ha_updated,
            // Proxmox commands
            proxmox_connect,
            proxmox_list_nodes,
            proxmox_list_storage,
            proxmox_get_next_vm_id,
            proxmox_create_vm
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
