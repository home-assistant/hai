# Testing Strategy

## Overview

Testing is split into three layers: Rust unit tests, frontend component tests, and Playwright E2E tests. A mock mode enables testing flows without real hardware.

---

## 1. Rust Unit Tests

Test core backend logic in isolation.

```rust
// src-tauri/src/manifest.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{"devices": [{"id": "rpi5", "name": "Raspberry Pi 5"}]}"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.devices[0].id, "rpi5");
    }

    #[test]
    fn test_checksum_validation_success() {
        let data = b"test data";
        let hash = "sha256:...";
        assert!(validate_checksum(data, hash).is_ok());
    }

    #[test]
    fn test_checksum_validation_failure() {
        let data = b"test data";
        let wrong_hash = "sha256:wrong";
        assert!(validate_checksum(data, wrong_hash).is_err());
    }

    #[tokio::test]
    async fn test_manifest_cache_fallback() {
        // Test that cached manifest is used when fetch fails
    }
}
```

**Run:** `cargo test` in `src-tauri/`

---

## 2. Frontend Component Tests (Web Test Runner)

Test Lit components in isolation.

```typescript
// test/unit/option-card.test.ts
import { fixture, html, expect } from '@open-wc/testing';
import '../../src/components/option-card.js';

describe('OptionCard', () => {
  it('renders title and description', async () => {
    const el = await fixture(html`
      <option-card 
        title="Raspberry Pi" 
        description="Flash an SD card">
      </option-card>
    `);
    
    expect(el.shadowRoot?.textContent).to.include('Raspberry Pi');
    expect(el.shadowRoot?.textContent).to.include('Flash an SD card');
  });

  it('dispatches select event on click', async () => {
    const el = await fixture(html`<option-card></option-card>`);
    
    let fired = false;
    el.addEventListener('select', () => fired = true);
    el.click();
    
    expect(fired).to.be.true;
  });

  it('applies selected state visually', async () => {
    const el = await fixture(html`<option-card selected></option-card>`);
    const card = el.shadowRoot?.querySelector('wa-card');
    
    expect(card?.hasAttribute('data-selected')).to.be.true;
  });
});
```

**Config:** `web-test-runner.config.js`
```javascript
export default {
  files: 'test/unit/**/*.test.ts',
  nodeResolve: true,
  plugins: [esbuildPlugin({ ts: true })],
};
```

**Run:** `npm run test:unit`

---

## 3. E2E Tests (Playwright)

Test complete user flows through the app.

### Welcome Screen Test

```typescript
// test/e2e/welcome.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Welcome Screen', () => {
  test('displays welcome message and proceeds on button click', async ({ page }) => {
    await page.goto('/');
    
    // Verify welcome screen content
    await expect(page.getByText('Welcome to your privacy-first')).toBeVisible();
    await expect(page.getByRole('button', { name: "Let's go" })).toBeVisible();
    await expect(page.locator('[alt="Open Home Foundation"]')).toBeVisible();
    
    // Click through
    await page.getByRole('button', { name: "Let's go" }).click();
    
    // Verify navigation to path selection
    await expect(page.getByText('What are you installing on?')).toBeVisible();
  });
});
```

### SBC Flow Test

```typescript
// test/e2e/sbc-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Single Board Computer Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Enable mock mode
    await page.goto('/?mock=true');
    await page.getByRole('button', { name: "Let's go" }).click();
  });

  test('complete flow from path selection to confirmation', async ({ page }) => {
    // Select SBC path
    await page.getByRole('button', { name: /Raspberry Pi/i }).click();
    
    // Select specific device
    await page.getByRole('button', { name: 'Raspberry Pi 5' }).click();
    
    // Should show drive selection
    await expect(page.getByText('Select SD card')).toBeVisible();
    
    // Mock device should be available
    await page.getByText('Mock SD Card').click();
    await page.getByRole('button', { name: 'Next' }).click();
    
    // Should show confirmation
    await expect(page.getByText('Ready to install')).toBeVisible();
    await expect(page.getByText('Raspberry Pi 5')).toBeVisible();
    await expect(page.getByText('Mock SD Card')).toBeVisible();
  });

  test('back navigation works correctly', async ({ page }) => {
    await page.getByRole('button', { name: /Raspberry Pi/i }).click();
    await page.getByRole('button', { name: 'Raspberry Pi 5' }).click();
    
    // Go back
    await page.getByRole('button', { name: /back/i }).click();
    
    // Should be on device selection
    await expect(page.getByRole('button', { name: 'Raspberry Pi 5' })).toBeVisible();
  });
});
```

### Proxmox Flow Test

```typescript
// test/e2e/proxmox-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Proxmox Flow', () => {
  test('shows error on invalid credentials', async ({ page }) => {
    await page.goto('/?mock=true');
    await page.getByRole('button', { name: "Let's go" }).click();
    await page.getByRole('button', { name: /Proxmox/i }).click();
    
    // Fill invalid credentials
    await page.getByLabel('Server URL').fill('https://proxmox.local:8006');
    await page.getByLabel('Username').fill('root@pam');
    await page.getByLabel('Password').fill('wrong');
    await page.getByRole('button', { name: 'Connect' }).click();
    
    // Should show error
    await expect(page.getByText(/authentication failed/i)).toBeVisible();
  });
});
```

### Toolbox Test

```typescript
// test/e2e/toolbox.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Toolbox Integration', () => {
  test('opens toolbox iframe and returns', async ({ page }) => {
    await page.goto('/');
    
    // Click toolbox button
    await page.getByRole('button', { name: /toolbox/i }).click();
    
    // Should show iframe with toolbox
    const iframe = page.frameLocator('iframe');
    await expect(iframe.locator('body')).toBeVisible();
    
    // Return to installer
    await page.getByRole('button', { name: /back to installer/i }).click();
    
    // Should be back on welcome
    await expect(page.getByText('Welcome to your privacy-first')).toBeVisible();
  });
});
```

### Playwright Config

```typescript
// playwright.config.ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./test/e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
    },
  ],
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

**Run:** `npm run test:e2e`

---

## 4. Mock Mode

Enable testing without real hardware.

```rust
// src-tauri/src/commands/devices.rs

#[tauri::command]
pub async fn list_block_devices() -> Result<Vec<BlockDevice>, String> {
    if std::env::var("HA_INSTALLER_MOCK").is_ok() {
        return Ok(mock_devices());
    }
    real_list_block_devices().await
}

fn mock_devices() -> Vec<BlockDevice> {
    vec![
        BlockDevice {
            id: "mock-sd-32".into(),
            name: "Mock SD Card".into(),
            size: 32_000_000_000,
            device_type: DeviceType::SdCard,
        },
        BlockDevice {
            id: "mock-usb-64".into(),
            name: "Mock USB Drive".into(),
            size: 64_000_000_000,
            device_type: DeviceType::Usb,
        },
    ]
}

#[tauri::command]
pub async fn flash_image(
    image_path: String,
    target_device: String,
    window: tauri::Window,
) -> Result<(), String> {
    if std::env::var("HA_INSTALLER_MOCK").is_ok() {
        return mock_flash(window).await;
    }
    real_flash_image(image_path, target_device, window).await
}

async fn mock_flash(window: tauri::Window) -> Result<(), String> {
    // Simulate download progress
    for i in 0..=100 {
        window.emit("flash-progress", FlashProgress {
            stage: if i < 50 { "downloading" } else { "writing" },
            percent: i,
        }).ok();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}
```

---

## Test Directory Structure

```
home-assistant-installer/
├── src-tauri/src/
│   └── commands.rs             # Contains inline Rust unit tests
├── test/
│   └── e2e/
│       ├── navigation.spec.ts  # Basic navigation tests
│       ├── proxmox-flow.spec.ts # Proxmox VE installation flow
│       └── utm-flow.spec.ts    # UTM VM creation flow (macOS)
├── playwright.config.ts
└── package.json
```

---

## npm Scripts

```json
{
  "scripts": {
    "test": "npm run test:unit",
    "test:unit": "web-test-runner",
    "test:e2e": "playwright test"
  }
}
```

### Running Rust Tests

```bash
cd src-tauri
cargo test
```
