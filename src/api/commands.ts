import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  BlockDevice,
  DeviceManifest,
  FlashProgress,
  FlashRequest,
  FlashResult,
  HaosRelease,
  ProxmoxCredentials,
  ProxmoxNode,
  ProxmoxSession,
  ProxmoxStorage,
  ProxmoxVmConfig,
  ProxmoxVmResult,
  SystemInfo,
  UpdateInfo,
  UtmStatus,
  UtmVmConfig,
} from "./types.js";

/**
 * Check if we're running in a browser without Tauri (e.g., Playwright tests)
 */
function isBrowserOnly(): boolean {
  return typeof window !== "undefined" && !("__TAURI__" in window);
}

/**
 * Check if URL mock mode is enabled
 */
function isUrlMockMode(): boolean {
  const urlParams = new URLSearchParams(window.location.search);
  return urlParams.get("mock") === "true";
}

/**
 * Check if the app is running in mock mode.
 * Mock mode is enabled via the HA_INSTALLER_MOCK environment variable
 * or the ?mock=true URL parameter.
 */
export async function isMockMode(): Promise<boolean> {
  // Check URL parameter first (for development)
  if (isUrlMockMode()) {
    return true;
  }

  // In browser-only mode (no Tauri), always use mock
  if (isBrowserOnly()) {
    return true;
  }

  // Check backend mock mode
  return invoke<boolean>("is_mock_mode");
}

// Import mock data for browser-only mode
import {
  MOCK_BLOCK_DEVICES,
  MOCK_HAOS_RELEASE,
  MOCK_MANIFEST,
  MOCK_UPDATE_INFO,
} from "./mock-data.js";

/**
 * List available block devices (SD cards, USB drives, etc.)
 */
export async function listBlockDevices(): Promise<BlockDevice[]> {
  if (isBrowserOnly()) {
    return MOCK_BLOCK_DEVICES;
  }
  return invoke<BlockDevice[]>("list_block_devices");
}

/**
 * Flash an image to a device with progress updates.
 * @param request The flash request parameters
 * @param onProgress Callback for progress updates
 */
export async function flashImage(
  request: FlashRequest,
  onProgress: (progress: FlashProgress) => void
): Promise<FlashResult> {
  if (isBrowserOnly()) {
    // Simulate flash progress in browser-only mode
    return simulateFlashProgress(onProgress);
  }

  const channel = new Channel<FlashProgress>();

  channel.onmessage = (progress) => {
    onProgress(progress);
  };

  return invoke<FlashResult>("flash_image", {
    request,
    progressChannel: channel,
  });
}

/**
 * Simulate flash progress for browser-only mode
 */
async function simulateFlashProgress(
  onProgress: (progress: FlashProgress) => void
): Promise<FlashResult> {
  const compressedSize = 400 * 1024 * 1024; // 400 MB compressed
  const extractedSize = 2 * 1024 * 1024 * 1024; // 2 GB extracted

  const stages: Array<{
    stage: FlashProgress["stage"];
    message: string;
    weight: number;
    totalBytes: number;
    showBytes: boolean;
    steps: number;
    delay: number;
  }> = [
    { stage: "downloading", message: "Downloading image...", weight: 30, totalBytes: compressedSize, showBytes: true, steps: 30, delay: 150 },
    { stage: "extracting", message: "Extracting image...", weight: 10, totalBytes: 0, showBytes: false, steps: 10, delay: 100 },
    { stage: "writing", message: "Writing to device...", weight: 35, totalBytes: extractedSize, showBytes: true, steps: 35, delay: 175 },
    { stage: "verifying", message: "Verifying written data...", weight: 15, totalBytes: extractedSize, showBytes: true, steps: 15, delay: 150 },
    { stage: "finalizing", message: "Finalizing...", weight: 10, totalBytes: 0, showBytes: false, steps: 10, delay: 100 },
  ];

  let overallProgress = 0;

  for (const { stage, message, weight, totalBytes, showBytes, steps, delay } of stages) {
    for (let step = 0; step <= steps; step++) {
      const stageProgress = (step * 100) / steps;
      const progress = overallProgress + (stageProgress * weight) / 100;

      onProgress({
        stage,
        progress: Math.round(progress),
        bytes_processed: showBytes ? Math.round((totalBytes * stageProgress) / 100) : 0,
        total_bytes: showBytes ? totalBytes : 0,
        message,
      });

      await new Promise((resolve) => setTimeout(resolve, delay));
    }
    overallProgress += weight;
  }

  onProgress({
    stage: "complete",
    progress: 100,
    bytes_processed: extractedSize,
    total_bytes: extractedSize,
    message: "Installation complete!",
  });

  return {
    success: true,
    duration_secs: 45,
  };
}

/**
 * Check for application updates.
 */
export async function checkForUpdates(): Promise<UpdateInfo> {
  if (isBrowserOnly()) {
    return MOCK_UPDATE_INFO;
  }
  return invoke<UpdateInfo>("check_for_updates");
}

/**
 * Get the device manifest.
 */
export async function getManifest(): Promise<DeviceManifest> {
  if (isBrowserOnly()) {
    return MOCK_MANIFEST;
  }
  return invoke<DeviceManifest>("get_manifest");
}

/**
 * Format bytes to a human-readable string.
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";

  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

/**
 * Get HAOS release information.
 * @param version Optional specific version to fetch (defaults to latest stable)
 */
export async function getHaosRelease(version?: string): Promise<HaosRelease> {
  if (isBrowserOnly()) {
    return MOCK_HAOS_RELEASE;
  }
  return invoke<HaosRelease>("get_haos_release", { version });
}

// ============================================================================
// System Info Commands
// ============================================================================

/**
 * Get system information (CPU cores and memory) for VM configuration limits.
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  if (isBrowserOnly()) {
    return {
      cpu_cores: 10,
      memory_mb: 32768, // 32 GB
    };
  }
  return invoke<SystemInfo>("get_system_info");
}

// ============================================================================
// UTM Commands (macOS only)
// ============================================================================

/**
 * Check if UTM is installed and get its status.
 */
export async function checkUtmStatus(): Promise<UtmStatus> {
  if (isBrowserOnly()) {
    // Mock: UTM is installed
    return {
      installed: true,
      path: "/Applications/UTM.app",
      version: "4.5.0",
    };
  }
  return invoke<UtmStatus>("check_utm_status");
}

/**
 * Download the HAOS qcow2 image for UTM.
 * @param onProgress Callback for progress updates
 * @returns Path to the downloaded qcow2 file
 */
export async function downloadUtmImage(
  onProgress: (progress: FlashProgress) => void
): Promise<string> {
  if (isBrowserOnly()) {
    // Simulate download progress in browser-only mode
    return simulateUtmDownload(onProgress);
  }

  const channel = new Channel<FlashProgress>();

  channel.onmessage = (progress) => {
    onProgress(progress);
  };

  return invoke<string>("download_utm_image", {
    progressChannel: channel,
  });
}

/**
 * Simulate UTM download for browser-only mode
 */
async function simulateUtmDownload(
  onProgress: (progress: FlashProgress) => void
): Promise<string> {
  const stages: Array<{
    stage: FlashProgress["stage"];
    message: string;
    steps: number;
    delay: number;
  }> = [
    { stage: "downloading", message: "Downloading HAOS image...", steps: 20, delay: 100 },
    { stage: "extracting", message: "Extracting image...", steps: 10, delay: 100 },
  ];

  let overallProgress = 0;
  const stageWeight = 100 / stages.length;

  for (const { stage, message, steps, delay } of stages) {
    for (let step = 0; step <= steps; step++) {
      const stageProgress = (step * 100) / steps;
      const progress = overallProgress + (stageProgress * stageWeight) / 100;

      onProgress({
        stage,
        progress: Math.round(progress),
        bytes_processed: 0,
        total_bytes: 0,
        message,
      });

      await new Promise((resolve) => setTimeout(resolve, delay));
    }
    overallProgress += stageWeight;
  }

  onProgress({
    stage: "complete",
    progress: 100,
    bytes_processed: 0,
    total_bytes: 0,
    message: "Download complete!",
  });

  return "/tmp/mock-haos.qcow2";
}

/**
 * Get the Mac's CPU architecture.
 * Returns "aarch64" for Apple Silicon, "x86_64" for Intel, or "unsupported".
 */
export async function getMacArchitecture(): Promise<string> {
  if (isBrowserOnly()) {
    return "aarch64"; // Mock as Apple Silicon
  }
  return invoke<string>("get_mac_architecture");
}

/**
 * Create a Home Assistant VM in UTM.
 * @param config The VM configuration
 * @returns The VM ID if successful
 */
export async function createUtmVm(config: UtmVmConfig): Promise<string> {
  if (isBrowserOnly()) {
    // Simulate VM creation
    await new Promise((resolve) => setTimeout(resolve, 2000));
    return "mock-vm-id-12345";
  }
  return invoke<string>("create_utm_vm", { config });
}

/**
 * Start a UTM VM.
 * @param vmId The VM ID to start
 */
export async function startUtmVm(vmId: string): Promise<void> {
  if (isBrowserOnly()) {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return;
  }
  return invoke<void>("start_utm_vm", { vmId });
}

/**
 * Resize a UTM VM's disk before first start.
 * @param vmId The VM ID
 * @param sizeGb The target disk size in GB
 */
export async function resizeUtmVmDisk(vmId: string, sizeGb: number): Promise<void> {
  if (isBrowserOnly()) {
    await new Promise((resolve) => setTimeout(resolve, 200));
    return;
  }
  return invoke<void>("resize_utm_vm_disk", { vmId, sizeGb });
}

/**
 * List all UTM VMs.
 * @returns Array of VM names
 */
export async function listUtmVms(): Promise<string[]> {
  if (isBrowserOnly()) {
    return ["Home Assistant"];
  }
  return invoke<string[]>("list_utm_vms");
}

/**
 * VM status info from backend.
 */
export interface VmStatusInfo {
  status: string;
  ip_address: string | null;
}

/**
 * Get the status of a UTM VM including its IP address if available.
 * @param vmId The VM ID to check
 * @returns VM status and IP address
 */
export async function getUtmVmStatus(vmId: string): Promise<VmStatusInfo> {
  if (isBrowserOnly()) {
    return {
      status: "started",
      ip_address: "192.168.1.100",
    };
  }
  return invoke<VmStatusInfo>("get_utm_vm_status", { vmId });
}

/**
 * Check if Home Assistant webserver is ready at the given IP address.
 * @param ipAddress The IP address to check
 * @returns True if the webserver is reachable on port 8123
 */
export async function checkHaReady(ipAddress: string): Promise<boolean> {
  if (isBrowserOnly()) {
    return true;
  }
  return invoke<boolean>("check_ha_ready", { ipAddress });
}

/**
 * Check if Home Assistant has finished updating by checking the manifest.json endpoint.
 * @param ipAddress The IP address to check
 * @returns True if manifest.json returns 200 OK
 */
export async function checkHaUpdated(ipAddress: string): Promise<boolean> {
  if (isBrowserOnly()) {
    return true;
  }
  return invoke<boolean>("check_ha_updated", { ipAddress });
}

// ============================================================================
// Proxmox VE Commands
// ============================================================================

/** Store for the current Proxmox session (browser-only mock) */
let mockProxmoxSession: ProxmoxSession | null = null;

/**
 * Connect to a Proxmox VE server and authenticate.
 * @param credentials Server URL, username, and password
 * @returns Session with authentication ticket and CSRF token
 */
export async function proxmoxConnect(
  credentials: ProxmoxCredentials
): Promise<ProxmoxSession> {
  if (isBrowserOnly()) {
    // Simulate connection delay
    await new Promise((resolve) => setTimeout(resolve, 1500));
    mockProxmoxSession = {
      server_url: credentials.server_url,
      ticket: "mock-ticket-" + Date.now(),
      csrf_token: "mock-csrf-" + Date.now(),
    };
    return mockProxmoxSession;
  }
  return invoke<ProxmoxSession>("proxmox_connect", { credentials });
}

/**
 * List available nodes on the Proxmox server.
 * @param session The authentication session
 * @returns List of available nodes
 */
export async function proxmoxListNodes(
  session: ProxmoxSession
): Promise<ProxmoxNode[]> {
  if (isBrowserOnly()) {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return [
      {
        name: "pve",
        status: "online",
        cpu_usage: 12.5,
        memory_used: 8 * 1024 * 1024 * 1024,
        memory_total: 32 * 1024 * 1024 * 1024,
      },
      {
        name: "pve2",
        status: "online",
        cpu_usage: 8.2,
        memory_used: 4 * 1024 * 1024 * 1024,
        memory_total: 16 * 1024 * 1024 * 1024,
      },
    ];
  }
  return invoke<ProxmoxNode[]>("proxmox_list_nodes", { session });
}

/**
 * List available storage on a Proxmox node.
 * @param session The authentication session
 * @param node The node name
 * @returns List of available storage locations
 */
export async function proxmoxListStorage(
  session: ProxmoxSession,
  node: string
): Promise<ProxmoxStorage[]> {
  if (isBrowserOnly()) {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return [
      {
        name: "local",
        storage_type: "dir",
        content: ["images", "rootdir", "vztmpl", "backup", "iso", "snippets"],
        available: 200 * 1024 * 1024 * 1024,
        total: 500 * 1024 * 1024 * 1024,
        active: true,
      },
      {
        name: "local-lvm",
        storage_type: "lvmthin",
        content: ["images", "rootdir"],
        available: 400 * 1024 * 1024 * 1024,
        total: 1024 * 1024 * 1024 * 1024,
        active: true,
      },
    ];
  }
  return invoke<ProxmoxStorage[]>("proxmox_list_storage", { session, node });
}

/**
 * Get the next available VM ID on the Proxmox server.
 * @param session The authentication session
 * @returns Next available VM ID
 */
export async function proxmoxGetNextVmId(
  session: ProxmoxSession
): Promise<number> {
  if (isBrowserOnly()) {
    await new Promise((resolve) => setTimeout(resolve, 200));
    return 100;
  }
  return invoke<number>("proxmox_get_next_vm_id", { session });
}

/**
 * Create a Home Assistant VM on Proxmox.
 * @param session The authentication session
 * @param config VM configuration
 * @param onProgress Callback for progress updates
 * @returns Result with VM ID and IP address
 */
export async function proxmoxCreateVm(
  session: ProxmoxSession,
  config: ProxmoxVmConfig,
  onProgress: (progress: FlashProgress) => void
): Promise<ProxmoxVmResult> {
  if (isBrowserOnly()) {
    return simulateProxmoxInstall(config, onProgress);
  }

  const channel = new Channel<FlashProgress>();
  channel.onmessage = (progress) => {
    onProgress(progress);
  };

  return invoke<ProxmoxVmResult>("proxmox_create_vm", {
    session,
    config,
    progressChannel: channel,
  });
}

/**
 * Simulate Proxmox VM installation for browser-only mode
 */
async function simulateProxmoxInstall(
  config: ProxmoxVmConfig,
  onProgress: (progress: FlashProgress) => void
): Promise<ProxmoxVmResult> {
  const stages: Array<{
    stage: FlashProgress["stage"];
    message: string;
    weight: number;
    steps: number;
    delay: number;
  }> = [
    { stage: "downloading", message: "Downloading HAOS image...", weight: 40, steps: 40, delay: 100 },
    { stage: "extracting", message: "Uploading to Proxmox...", weight: 25, steps: 25, delay: 80 },
    { stage: "writing", message: "Creating virtual machine...", weight: 20, steps: 20, delay: 100 },
    { stage: "verifying", message: "Starting Home Assistant OS...", weight: 10, steps: 10, delay: 150 },
    { stage: "finalizing", message: "Waiting for network...", weight: 5, steps: 10, delay: 200 },
  ];

  let overallProgress = 0;

  for (const { stage, message, weight, steps, delay } of stages) {
    for (let step = 0; step <= steps; step++) {
      const stageProgress = (step * 100) / steps;
      const progress = overallProgress + (stageProgress * weight) / 100;

      onProgress({
        stage,
        progress: Math.round(progress),
        bytes_processed: 0,
        total_bytes: 0,
        message,
      });

      await new Promise((resolve) => setTimeout(resolve, delay));
    }
    overallProgress += weight;
  }

  onProgress({
    stage: "complete",
    progress: 100,
    bytes_processed: 0,
    total_bytes: 0,
    message: "Installation complete!",
  });

  return {
    vm_id: config.vm_id,
    node: config.node,
    ip_address: "192.168.1.150",
  };
}
