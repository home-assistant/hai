import type { BlockDevice, DeviceManifest, HaosRelease, UpdateInfo } from "./types.js";

/**
 * Mock block devices for testing
 */
export const MOCK_BLOCK_DEVICES: BlockDevice[] = [
  {
    id: "mock-sd-card-32gb",
    name: "SD Card 32GB",
    size: 32 * 1024 * 1024 * 1024,
    device_type: "sd_card",
    removable: true,
    model: "SanDisk Ultra",
    vendor: "SanDisk",
  },
  {
    id: "mock-sd-card-64gb",
    name: "SD Card 64GB",
    size: 64 * 1024 * 1024 * 1024,
    device_type: "sd_card",
    removable: true,
    model: "Samsung EVO Plus",
    vendor: "Samsung",
  },
  {
    id: "mock-usb-drive-128gb",
    name: "USB Drive 128GB",
    size: 128 * 1024 * 1024 * 1024,
    device_type: "usb_drive",
    removable: true,
    model: "USB Flash Drive",
    vendor: "Kingston",
  },
];

/**
 * Mock device manifest for testing
 */
export const MOCK_MANIFEST: DeviceManifest = {
  version: 1,
  devices: [
    // Raspberry Pi devices
    {
      id: "rpi5",
      name: "Raspberry Pi 5",
      category: "raspberry_pi",
      image_url: "/assets/devices/raspberry_pi_5.png",
      haos: {
        board: "rpi5-64",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi5-64-{version}.img.xz",
      },
    },
    {
      id: "rpi4",
      name: "Raspberry Pi 4",
      category: "raspberry_pi",
      image_url: "/assets/devices/raspberry_pi_4.png",
      haos: {
        board: "rpi4-64",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi4-64-{version}.img.xz",
      },
    },
    {
      id: "rpi3",
      name: "Raspberry Pi 3",
      category: "raspberry_pi",
      image_url: "/assets/devices/raspberry_pi_3.png",
      haos: {
        board: "rpi3-64",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_rpi3-64-{version}.img.xz",
      },
    },
    // ODROID devices
    {
      id: "odroid-n2",
      name: "ODROID-N2/N2+",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-n2.png",
      haos: {
        board: "odroid-n2",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-n2-{version}.img.xz",
      },
    },
    {
      id: "odroid-c2",
      name: "ODROID-C2",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-c2.png",
      haos: {
        board: "odroid-c2",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-c2-{version}.img.xz",
      },
    },
    {
      id: "odroid-c4",
      name: "ODROID-C4",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-c4.png",
      haos: {
        board: "odroid-c4",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-c4-{version}.img.xz",
      },
    },
    {
      id: "odroid-m1",
      name: "ODROID-M1",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-m1.png",
      haos: {
        board: "odroid-m1",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-m1-{version}.img.xz",
      },
    },
    {
      id: "odroid-m1s",
      name: "ODROID-M1S",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-m1s.png",
      haos: {
        board: "odroid-m1s",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-m1s-{version}.img.xz",
      },
    },
    {
      id: "odroid-xu4",
      name: "ODROID-XU4",
      category: "odroid",
      image_url: "/assets/devices/hardkernel_odroid-xu4.png",
      haos: {
        board: "odroid-xu",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_odroid-xu-{version}.img.xz",
      },
    },
    // Khadas devices
    {
      id: "khadas-vim3",
      name: "Khadas VIM3",
      category: "khadas",
      image_url: "/assets/devices/khadas_vim3.png",
      haos: {
        board: "khadas-vim3",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_khadas-vim3-{version}.img.xz",
      },
    },
    // ASUS devices
    {
      id: "asus-tinker",
      name: "ASUS Tinker Board",
      category: "asus",
      image_url: "/assets/devices/asus_tinker.png",
      haos: {
        board: "tinker",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_tinker-{version}.img.xz",
      },
    },
    // Home Assistant Hardware
    {
      id: "ha-green",
      name: "Home Assistant Green",
      category: "home_assistant_hardware",
      image_url: "/assets/devices/homeassistant_green.png",
      haos: {
        board: "green",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_green-{version}.img.xz",
      },
    },
    {
      id: "ha-yellow",
      name: "Home Assistant Yellow",
      category: "home_assistant_hardware",
      image_url: "/assets/devices/homeassistant_yellow.png",
      haos: {
        board: "yellow",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_yellow-{version}.img.xz",
      },
    },
    // Generic x86-64
    {
      id: "generic-x86-64",
      name: "Intel/AMD (x86-64)",
      category: "generic_x86",
      image_url: "/assets/icons/cpu-64-bit.svg",
      haos: {
        board: "generic-x86-64",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_generic-x86-64-{version}.img.xz",
      },
    },
    // Generic ARM64
    {
      id: "generic-aarch64",
      name: "ARM (aarch64)",
      category: "generic_arm64",
      image_url: "/assets/icons/chip.svg",
      haos: {
        board: "generic-aarch64",
        download_url:
          "https://github.com/home-assistant/operating-system/releases/download/{version}/haos_generic-aarch64-{version}.img.xz",
      },
    },
  ],
};

/**
 * Mock update info for testing
 */
export const MOCK_UPDATE_INFO: UpdateInfo = {
  update_available: false,
  current_version: "0.1.0",
  latest_version: "0.1.0",
  download_url:
    "https://github.com/home-assistant/home-assistant-installer/releases",
  release_notes_url:
    "https://github.com/home-assistant/home-assistant-installer/releases",
  is_beta: false,
};

/**
 * Mock HAOS release info for testing (based on real 16.3 release)
 */
export const MOCK_HAOS_RELEASE: HaosRelease = {
  version: "16.3",
  images: [
    {
      board: "rpi5-64",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi5-64-16.3.img.xz",
      size: 331_899_792,
      sha256: "5ade653232aa1c4504e52b56347b389fb0b24d9edc69134a860edb84f41ea9e9",
    },
    {
      board: "rpi4-64",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi4-64-16.3.img.xz",
      size: 322_239_272,
      sha256: "3ebed523708dc1dad5b5399707ee74d0a54b9604b7d4cae5d591d75c85b35013",
    },
    {
      board: "rpi3-64",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_rpi3-64-16.3.img.xz",
      size: 311_438_560,
      sha256: "f21d5da83a94a5045d4d36822da77d2bee3539ab5150a7074c562d922f81e0de",
    },
    {
      board: "odroid-n2",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_odroid-n2-16.3.img.xz",
      size: 298_412_092,
      sha256: "f97b188d9fd2c239269c886e53031ad8bc38828296f1eaede2e89fd4b89207b7",
    },
    {
      board: "green",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_green-16.3.img.xz",
      size: 336_860_104,
      sha256: "fd41fb3432fb5d64d916b04f6ab18c39824b128fd996d55ea207e393fc65c943",
    },
    {
      board: "yellow",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_yellow-16.3.img.xz",
      size: 322_261_788,
      sha256: "145f252403a00a50391ed4074242e5b770c59477b66f2a2ea33927f68bef0e98",
    },
    {
      board: "generic-x86-64",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_generic-x86-64-16.3.img.xz",
      size: 396_451_208,
      sha256: "afe591a859a068eb25dcef15be9e7b2236f9c06f515cac3706681db900cb02df",
    },
    {
      board: "generic-aarch64",
      download_url: "https://github.com/home-assistant/operating-system/releases/download/16.3/haos_generic-aarch64-16.3.img.xz",
      size: 341_537_340,
      sha256: "4769532f71886f8b41c4520b3c0c8f974f5bbf583782a2dc7b16a8e2743315ed",
    },
  ],
};
