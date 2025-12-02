/** Represents a block device (SD card, USB drive, etc.) */
export interface BlockDevice {
  /** Unique identifier (e.g., "/dev/sda" on Linux, "disk2" on macOS) */
  id: string;
  /** Human-readable name */
  name: string;
  /** Size in bytes */
  size: number;
  /** Device type */
  device_type: DeviceType;
  /** Whether this is a removable device */
  removable: boolean;
  /** Model name if available */
  model?: string;
  /** Vendor name if available */
  vendor?: string;
}

export type DeviceType =
  | "sd_card"
  | "usb_drive"
  | "ssd"
  | "hdd"
  | "nvme"
  | "unknown";

/** Progress event sent during flashing */
export interface FlashProgress {
  /** Current stage of the process */
  stage: FlashStage;
  /** Progress percentage (0-100) */
  progress: number;
  /** Bytes processed so far */
  bytes_processed: number;
  /** Total bytes to process */
  total_bytes: number;
  /** Human-readable message */
  message: string;
}

export type FlashStage =
  | "downloading"
  | "extracting"
  | "writing"
  | "verifying"
  | "finalizing"
  | "complete"
  | "error";

/** Update information */
export interface UpdateInfo {
  /** Whether an update is available */
  update_available: boolean;
  /** Current version */
  current_version: string;
  /** Latest available version */
  latest_version: string;
  /** Download URL for the latest version */
  download_url?: string;
  /** Release notes URL */
  release_notes_url?: string;
  /** Whether this is a beta release */
  is_beta: boolean;
}

/** Device manifest for supported devices */
export interface DeviceManifest {
  /** Version of the manifest format */
  version: number;
  /** List of supported devices */
  devices: Device[];
}

/** A supported device in the manifest */
export interface Device {
  /** Unique device identifier */
  id: string;
  /** Human-readable name */
  name: string;
  /** Device category */
  category: DeviceCategory;
  /** Image URL for the device photo */
  image_url?: string;
  /** HAOS image configuration */
  haos: HaosConfig;
}

export type DeviceCategory =
  | "raspberry_pi"
  | "odroid"
  | "khadas"
  | "asus"
  | "home_assistant_hardware"
  | "generic_x86"
  | "generic_arm64";

export interface HaosConfig {
  /** Board identifier for the HAOS image */
  board: string;
  /** Download URL template */
  download_url: string;
}

/** Flash request parameters */
export interface FlashRequest {
  /** Target device ID (block device path) */
  device_id: string;
  /** Board identifier (e.g., "rpi5-64", "green") */
  board: string;
  /** Whether to verify after writing */
  verify: boolean;
}

/** Result of a flash operation */
export interface FlashResult {
  /** Whether the operation was successful */
  success: boolean;
  /** Error message if failed */
  error?: string;
  /** Duration in seconds */
  duration_secs: number;
}

/** HAOS release information */
export interface HaosRelease {
  /** Version string (e.g., "16.3") */
  version: string;
  /** List of available images */
  images: HaosImage[];
}

/** A single HAOS image file */
export interface HaosImage {
  /** Board name (e.g., "rpi5-64", "green", "generic-x86-64") */
  board: string;
  /** Download URL */
  download_url: string;
  /** File size in bytes */
  size: number;
  /** SHA256 checksum (hex string) */
  sha256: string;
}

// ============================================================================
// System Info Types
// ============================================================================

/** System information for VM configuration limits */
export interface SystemInfo {
  /** Number of CPU cores available */
  cpu_cores: number;
  /** Total memory in MB */
  memory_mb: number;
}

// ============================================================================
// UTM Types (macOS only)
// ============================================================================

/** UTM installation status */
export interface UtmStatus {
  /** Whether UTM is installed */
  installed: boolean;
  /** Path to UTM.app if installed */
  path?: string;
  /** UTM version if installed */
  version?: string;
}

/** Configuration for creating a UTM VM */
export interface UtmVmConfig {
  /** Name for the VM */
  name: string;
  /** Path to the HAOS image file */
  image_path: string;
  /** Number of CPU cores */
  cpu_cores: number;
  /** Memory in MB */
  memory_mb: number;
  /** Disk size in GB */
  disk_size_gb: number;
  /** Whether to start the VM after creation */
  auto_start: boolean;
}

// ============================================================================
// Proxmox VE Types
// ============================================================================

/** Proxmox connection credentials */
export interface ProxmoxCredentials {
  /** Proxmox server URL (e.g., https://192.168.1.100:8006) */
  server_url: string;
  /** Username (e.g., root@pam) */
  username: string;
  /** Password */
  password: string;
}

/** Proxmox session (authentication result) */
export interface ProxmoxSession {
  /** Server URL for the session */
  server_url: string;
  /** Authentication ticket */
  ticket: string;
  /** CSRF prevention token */
  csrf_token: string;
}

/** Proxmox node information */
export interface ProxmoxNode {
  /** Node name */
  name: string;
  /** Node status (online/offline) */
  status: string;
  /** CPU usage percentage */
  cpu_usage?: number;
  /** Memory usage in bytes */
  memory_used?: number;
  /** Total memory in bytes */
  memory_total?: number;
}

/** Proxmox storage information */
export interface ProxmoxStorage {
  /** Storage name */
  name: string;
  /** Storage type (local, nfs, cifs, etc.) */
  storage_type: string;
  /** Content types (images, rootdir, iso, etc.) */
  content: string[];
  /** Available space in bytes */
  available: number;
  /** Total space in bytes */
  total: number;
  /** Whether storage is active */
  active: boolean;
}

/** Configuration for creating a Proxmox VM */
export interface ProxmoxVmConfig {
  /** Target node name */
  node: string;
  /** Target storage name */
  storage: string;
  /** VM ID (e.g., 100) */
  vm_id: number;
  /** VM name */
  name: string;
  /** Number of CPU cores */
  cpu_cores: number;
  /** Memory in MB */
  memory_mb: number;
  /** Disk size in GB */
  disk_size_gb: number;
  /** Whether to start VM after creation */
  auto_start: boolean;
}

/** Proxmox VM creation result */
export interface ProxmoxVmResult {
  /** The created VM ID */
  vm_id: number;
  /** Node where VM was created */
  node: string;
  /** IP address if available */
  ip_address?: string;
}
