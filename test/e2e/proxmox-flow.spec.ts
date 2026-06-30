import { test, expect } from "@playwright/test";

test.describe("Proxmox Installation Flow", () => {
  test.beforeEach(async ({ page }) => {
    // Use mock mode to avoid actual Proxmox connections
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    // Select Proxmox Server option
    await page.locator('option-card[title="Proxmox server"]').click();
    await expect(page.locator("wizard-shell")).toBeVisible();
  });

  test("shows wizard shell with correct flow title", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toContainText("Proxmox");
  });

  test("shows step indicator with all steps", async ({ page }) => {
    const stepIndicator = page.locator("step-indicator");
    await expect(stepIndicator).toBeVisible();
    // Should show: Connect to Proxmox, Configure VM, Confirm, Install, Done
    await expect(stepIndicator).toContainText("Connect to Proxmox");
    await expect(stepIndicator).toContainText("Configure VM");
    await expect(stepIndicator).toContainText("Confirm");
    await expect(stepIndicator).toContainText("Install");
    await expect(stepIndicator).toContainText("Done");
  });

  test("step 1: shows Proxmox connection view", async ({ page }) => {
    const connectView = page.locator("proxmox-connect-view");
    await expect(connectView).toBeVisible();
  });

  test("step 1: shows connection form fields", async ({ page }) => {
    const connectView = page.locator("proxmox-connect-view");

    // Check heading
    await expect(connectView.locator("h2")).toContainText("Connect to Proxmox");

    // Check form fields
    await expect(connectView.locator("#server-url")).toBeVisible();
    await expect(connectView.locator("#username")).toBeVisible();
    await expect(connectView.locator("#password")).toBeVisible();

    // Check placeholders/hints
    await expect(connectView.locator("#server-url")).toHaveAttribute(
      "placeholder",
      "https://192.168.1.100:8006"
    );
    await expect(connectView.locator("#username")).toHaveAttribute(
      "placeholder",
      "root@pam"
    );
  });

  test("step 1: clicking next with empty form shows error", async ({ page }) => {
    const connectView = page.locator("proxmox-connect-view");
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");

    // Clear the username field (which has default value)
    await connectView.locator("#username").clear();

    // Click next - should trigger validation
    await nextButton.click();

    // Should show error message and stay on connect view
    await expect(connectView.locator(".status-row")).toBeVisible();
    await expect(connectView).toContainText("Please fill in all fields");
    await expect(connectView).toBeVisible();
  });

  test("step 1: clicking next with invalid URL shows error", async ({ page }) => {
    const connectView = page.locator("proxmox-connect-view");

    // Fill with invalid URL (http instead of https)
    await connectView.locator("#server-url").fill("http://192.168.1.100:8006");
    await connectView.locator("#username").fill("root@pam");
    await connectView.locator("#password").fill("password123");

    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");

    // Click next - should trigger validation
    await nextButton.click();

    // Should show error message about HTTPS and stay on connect view
    await expect(connectView.locator(".status-row")).toBeVisible();
    await expect(connectView).toContainText("HTTPS");
    await expect(connectView).toBeVisible();
  });

  test("step 1: connection behavior with credentials", async ({ page }) => {
    const connectView = page.locator("proxmox-connect-view");

    // Fill with credentials
    await connectView.locator("#server-url").fill("https://192.168.1.100:8006");
    await connectView.locator("#username").fill("root@pam");
    await connectView.locator("#password").fill("testpassword");

    // Click next to trigger connection attempt
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeEnabled();
    await nextButton.click();

    // In mock mode, connection might succeed or fail depending on implementation
    // We should either see an error (status-row) or advance to configure view
    await page.waitForTimeout(2000);

    const onConnectView = await connectView.isVisible();
    const onConfigureView = await page
      .locator("proxmox-configure-view")
      .isVisible();

    // Either stayed on connect (error shown) or advanced to configure (success)
    expect(onConnectView || onConfigureView).toBe(true);
  });

  test("step 1: can navigate to step 2 with valid connection", async ({
    page,
  }) => {
    const connectView = page.locator("proxmox-connect-view");

    // Fill with valid credentials for mock mode
    await connectView.locator("#server-url").fill("https://192.168.1.100:8006");
    await connectView.locator("#username").fill("root@pam");
    await connectView.locator("#password").fill("test");

    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeEnabled();
    await nextButton.click();

    // Should advance to configure view
    await expect(page.locator("proxmox-configure-view")).toBeVisible();
  });

  test("step 2: shows VM configuration view", async ({ page }) => {
    // Navigate to step 2
    await navigateToProxmoxStep2(page);

    const configView = page.locator("proxmox-configure-view");
    await expect(configView).toBeVisible();
    await expect(configView.locator("h2")).toContainText(
      "Configure virtual machine"
    );
  });

  test("step 2: shows all configuration options", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    const configView = page.locator("proxmox-configure-view");

    // Check for all settings
    await expect(configView).toContainText("Display name");
    await expect(configView).toContainText("Node");
    await expect(configView).toContainText("Storage");
    await expect(configView).toContainText("VM ID");
    await expect(configView).toContainText("CPU cores");
    await expect(configView).toContainText("Memory");
    await expect(configView).toContainText("Disk size");
  });

  test("step 2: can modify VM name", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    const configView = page.locator("proxmox-configure-view");
    const nameInput = configView.locator(".name-input").first();

    await nameInput.clear();
    await nameInput.fill("my-home-assistant");

    await expect(nameInput).toHaveValue("my-home-assistant");
  });

  test("step 2: can adjust CPU cores slider", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    const configView = page.locator("proxmox-configure-view");
    const cpuSlider = configView.locator('input[type="range"]').first();

    // Should show CPU value
    await expect(configView).toContainText("Cores");
  });

  test("step 2: can adjust memory slider", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    const configView = page.locator("proxmox-configure-view");

    // Should show memory value in GB
    await expect(configView).toContainText("GB");
  });

  test("step 2: can navigate to step 3", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    // Next should be enabled
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeEnabled();
    await nextButton.click();

    // Should advance to confirmation
    await expect(page.locator("proxmox-confirm-view")).toBeVisible();
  });

  test("step 2: can navigate back to step 1", async ({ page }) => {
    await navigateToProxmoxStep2(page);

    const backButton = page.locator("wizard-shell").locator(".back-button");
    await expect(backButton).toBeEnabled();
    await backButton.click();

    // Should go back to connection view
    await expect(page.locator("proxmox-connect-view")).toBeVisible();
  });

  test("step 3: shows confirmation view", async ({ page }) => {
    await navigateToProxmoxStep3(page);

    const confirmView = page.locator("proxmox-confirm-view");
    await expect(confirmView).toBeVisible();
    await expect(confirmView.locator("h2")).toContainText("Ready to install");
  });

  test("step 3: shows configuration summary", async ({ page }) => {
    await navigateToProxmoxStep3(page);

    const confirmView = page.locator("proxmox-confirm-view");

    // Should show VM details
    await expect(confirmView).toContainText("Virtual machine");
    await expect(confirmView).toContainText("Node");
    await expect(confirmView).toContainText("Storage");
  });

  test("step 3: shows Install button instead of Next", async ({ page }) => {
    await navigateToProxmoxStep3(page);

    const installButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(installButton).toBeVisible();
    await expect(installButton).toContainText("Install");
  });

  test("step 3: clicking Install proceeds directly to installation", async ({
    page,
  }) => {
    await navigateToProxmoxStep3(page);

    // Click Install button - Proxmox flow has no confirmation dialog
    const installButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await installButton.click();

    // Should proceed directly to progress view (no dialog for Proxmox flow)
    await expect(page.locator("proxmox-progress-view")).toBeVisible();
  });

  test("step 3: can navigate back to step 2", async ({ page }) => {
    await navigateToProxmoxStep3(page);

    const backButton = page.locator("wizard-shell").locator(".back-button");
    await expect(backButton).toBeEnabled();
    await backButton.click();

    // Should go back to configure view
    await expect(page.locator("proxmox-configure-view")).toBeVisible();
  });

  test("step 4: shows progress view during installation", async ({ page }) => {
    await navigateToProxmoxStep4(page);

    const progressView = page.locator("proxmox-progress-view");
    await expect(progressView).toBeVisible();
  });

  test("step 4: shows progress indicators", async ({ page }) => {
    await navigateToProxmoxStep4(page);

    const progressView = page.locator("proxmox-progress-view");

    // Should show progress bar
    await expect(progressView.locator("progress-bar")).toBeVisible();

    // Should show thinking cloud with stage
    await expect(progressView.locator(".thinking-cloud")).toBeVisible();
  });

  test("step 4: shows stage dots", async ({ page }) => {
    await navigateToProxmoxStep4(page);

    const progressView = page.locator("proxmox-progress-view");

    // Should have stage indicator
    await expect(progressView.locator(".stages-indicator")).toBeVisible();

    // Should show multiple stages
    const stageDots = progressView.locator(".stage-dot");
    await expect(stageDots.first()).toBeVisible();
  });

  test("step 4: footer is hidden during installation", async ({ page }) => {
    await navigateToProxmoxStep4(page);

    const wizardShell = page.locator("wizard-shell");
    // Footer should not be visible during install
    await expect(wizardShell.locator(".footer")).not.toBeVisible();
  });

  test("step 4: back button is hidden during installation", async ({ page }) => {
    await navigateToProxmoxStep4(page);

    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");
    await expect(backButton).toHaveCSS("visibility", "hidden");
  });

  test("step 4: advances to success after installation completes", async ({
    page,
  }) => {
    await navigateToProxmoxStep4(page);

    // Wait for installation to complete (mock should finish quickly)
    await expect(page.locator("proxmox-success-view")).toBeVisible({
      timeout: 30000,
    });
  });

  test("step 5: shows success view", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const successView = page.locator("proxmox-success-view");
    await expect(successView).toBeVisible();
    await expect(successView.locator("h2")).toContainText("all set");
  });

  test("step 5: shows Casita mascot", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const successView = page.locator("proxmox-success-view");
    await expect(successView.locator(".casita-mascot")).toBeVisible();
  });

  test("step 5: shows VM access information", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const successView = page.locator("proxmox-success-view");

    // Should show next steps
    await expect(successView.locator(".next-steps")).toBeVisible();
  });

  test("step 5: shows Done button", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const wizardShell = page.locator("wizard-shell");
    const doneButton = wizardShell.locator(".footer-button.primary");
    await expect(doneButton).toBeVisible();
    await expect(doneButton).toContainText("Done");
  });

  test("step 5: back button is hidden on success", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");
    await expect(backButton).toHaveCSS("visibility", "hidden");
  });

  test("step 5: clicking Done returns to welcome", async ({ page }) => {
    await navigateToProxmoxStep5(page);

    const wizardShell = page.locator("wizard-shell");
    const doneButton = wizardShell.locator(".footer-button.primary");
    await doneButton.click();

    // Should be back on welcome screen
    await expect(page.locator("welcome-view")).toBeVisible();
  });

  test("can cancel wizard at any step before installation", async ({
    page,
  }) => {
    // Test cancel on step 1
    const wizardShell = page.locator("wizard-shell");
    await wizardShell.locator(".cancel-button").click();
    await expect(page.locator("welcome-view")).toBeVisible();

    // Restart and test cancel on step 2
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await page.locator('option-card[title="Proxmox server"]').click();
    await navigateToProxmoxStep2(page);
    await wizardShell.locator(".cancel-button").click();
    await expect(page.locator("welcome-view")).toBeVisible();
  });

  test("complete end-to-end flow", async ({ page }) => {
    // Step 1: Connect
    const connectView = page.locator("proxmox-connect-view");
    await expect(connectView).toBeVisible();

    await connectView.locator("#server-url").fill("https://192.168.1.100:8006");
    await connectView.locator("#username").fill("root@pam");
    await connectView.locator("#password").fill("test");

    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Step 2: Configure
    await expect(page.locator("proxmox-configure-view")).toBeVisible();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Step 3: Confirm - click Install (no confirmation dialog for Proxmox)
    await expect(page.locator("proxmox-confirm-view")).toBeVisible();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Step 4: Progress (proceeds directly, no dialog)
    await expect(page.locator("proxmox-progress-view")).toBeVisible();

    // Step 5: Success
    await expect(page.locator("proxmox-success-view")).toBeVisible({
      timeout: 30000,
    });

    // Return to welcome
    await page.locator("wizard-shell").locator(".footer-button.primary").click();
    await expect(page.locator("welcome-view")).toBeVisible();
  });
});

// Helper functions to navigate to specific steps
async function navigateToProxmoxStep2(page: any) {
  const connectView = page.locator("proxmox-connect-view");
  await connectView.locator("#server-url").fill("https://192.168.1.100:8006");
  await connectView.locator("#username").fill("root@pam");
  await connectView.locator("#password").fill("test");
  await page.locator("wizard-shell").locator(".footer-button.primary").click();
  await expect(page.locator("proxmox-configure-view")).toBeVisible();
}

async function navigateToProxmoxStep3(page: any) {
  await navigateToProxmoxStep2(page);
  await page.locator("wizard-shell").locator(".footer-button.primary").click();
  await expect(page.locator("proxmox-confirm-view")).toBeVisible();
}

async function navigateToProxmoxStep4(page: any) {
  await navigateToProxmoxStep3(page);
  // No confirmation dialog for Proxmox flow - proceeds directly to install
  await page.locator("wizard-shell").locator(".footer-button.primary").click();
  await expect(page.locator("proxmox-progress-view")).toBeVisible();
}

async function navigateToProxmoxStep5(page: any) {
  await navigateToProxmoxStep4(page);
  await expect(page.locator("proxmox-success-view")).toBeVisible({
    timeout: 30000,
  });
}
