import { test, expect } from "@playwright/test";

// Mini PC flow order: method → architecture → drive → confirm → flash → success

test.describe("Mini PC Flow - Setup Method Selection", () => {
  test.beforeEach(async ({ page }) => {
    // Use mock mode to avoid real API calls
    await page.goto("/?mock=true");
    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("navigates to Mini PC setup method when clicking the option", async ({
    page,
  }) => {
    // Click Generic (mini) PC option
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    // Should now see wizard shell
    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Should see step indicator
    await expect(wizardShell.locator("step-indicator")).toBeVisible();
  });

  test("shows setup method view with correct title", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupMethodView = page.locator("minipc-setup-method-view");
    await expect(setupMethodView).toBeVisible();

    // Use first() to avoid matching the dialog h2
    const title = setupMethodView.locator("h2").first();
    await expect(title).toContainText("How will you install?");
  });

  test("shows subtitle about installation method", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const subtitle = page
      .locator("minipc-setup-method-view")
      .locator(".subtitle");
    await expect(subtitle).toContainText("install Home Assistant");
  });

  test("displays connect drive option", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const connectDriveOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I can connect the drive" });
    await expect(connectDriveOption).toBeVisible();
    await expect(connectDriveOption).toContainText("SSD");
    await expect(connectDriveOption).toContainText("NVMe");
  });

  test("displays USB boot option", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const usbBootOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I need to boot from USB" });
    await expect(usbBootOption).toBeVisible();
    await expect(usbBootOption).toContainText("bootable USB");
  });

  test("clicking USB boot shows info dialog", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const usbBootOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I need to boot from USB" });
    await usbBootOption.click();

    // Should show info dialog
    const infoDialog = page.locator("info-dialog");
    await expect(infoDialog).toBeVisible();
    await expect(infoDialog).toContainText("USB boot installation");
    await expect(infoDialog).toContainText("not supported");
  });

  test("USB boot dialog has View Instructions button", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const usbBootOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I need to boot from USB" });
    await usbBootOption.click();

    const infoDialog = page.locator("info-dialog");
    await expect(infoDialog).toBeVisible();

    // Check for primary button (View Instructions)
    const primaryButton = infoDialog.locator(
      'wa-button[variant="brand"], button:has-text("View Instructions")'
    );
    await expect(primaryButton).toBeVisible();
  });

  test("USB boot dialog can be closed with Go Back", async ({ page }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const usbBootOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I need to boot from USB" });
    await usbBootOption.click();

    const infoDialog = page.locator("info-dialog");
    await expect(infoDialog).toBeVisible();

    // Click secondary button (Go Back)
    const secondaryButton = infoDialog.locator(
      'wa-button[variant="neutral"], button:has-text("Go Back")'
    );
    await secondaryButton.click();

    // Dialog should close, setup method view should still be visible
    await expect(infoDialog).not.toBeVisible();
    await expect(setupView).toBeVisible();
  });

  test("clicking connect drive navigates to architecture selection", async ({
    page,
  }) => {
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    const setupView = page.locator("minipc-setup-method-view");
    const connectDriveOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I can connect the drive" });
    await connectDriveOption.click();

    // Should navigate to architecture selection
    await expect(
      page.locator("minipc-architecture-selection-view")
    ).toBeVisible();
  });
});

test.describe("Mini PC Flow - Architecture Selection", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    // Select "connect drive" to get to architecture selection
    const setupView = page.locator("minipc-setup-method-view");
    await expect(setupView).toBeVisible();
    const connectDriveOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I can connect the drive" });
    await connectDriveOption.click();
  });

  test("shows architecture selection view with correct title", async ({
    page,
  }) => {
    const archSelectionView = page.locator(
      "minipc-architecture-selection-view"
    );
    await expect(archSelectionView).toBeVisible();

    const title = archSelectionView.locator("h2");
    await expect(title).toContainText("Select your architecture");
  });

  test("shows subtitle with CPU architecture mention", async ({ page }) => {
    const subtitle = page
      .locator("minipc-architecture-selection-view")
      .locator(".subtitle");
    await expect(subtitle).toContainText("CPU architecture");
  });

  test("displays x86-64 architecture option", async ({ page }) => {
    const archView = page.locator("minipc-architecture-selection-view");
    await expect(archView).toBeVisible();

    const options = archView.locator(".options");
    await expect(options).toBeVisible();

    const x86Option = options.locator(".option-card").filter({
      hasText: "Intel/AMD",
    });
    await expect(x86Option).toBeVisible();
    await expect(x86Option).toContainText("x86-64");
  });

  test("displays ARM64 architecture option", async ({ page }) => {
    const archView = page.locator("minipc-architecture-selection-view");
    const options = archView.locator(".options");
    await expect(options).toBeVisible();

    const armOption = options.locator(".option-card").filter({
      hasText: "ARM",
    });
    await expect(armOption).toBeVisible();
    await expect(armOption).toContainText("aarch64");
  });

  test("x86-64 option shows example devices", async ({ page }) => {
    const archView = page.locator("minipc-architecture-selection-view");
    const x86Option = archView.locator(".option-card").filter({
      hasText: "Intel/AMD",
    });
    await expect(x86Option).toBeVisible();

    await expect(x86Option.locator(".option-examples")).toContainText(
      "Intel NUC"
    );
    await expect(x86Option.locator(".option-examples")).toContainText(
      "Beelink"
    );
  });

  test("ARM64 option shows example devices", async ({ page }) => {
    const archView = page.locator("minipc-architecture-selection-view");
    const armOption = archView.locator(".option-card").filter({
      hasText: "ARM",
    });
    await expect(armOption).toBeVisible();

    await expect(armOption.locator(".option-examples")).toContainText(
      "Apple Silicon"
    );
  });

  test("selecting an architecture and clicking Next navigates to drive selection", async ({
    page,
  }) => {
    const archView = page.locator("minipc-architecture-selection-view");
    const options = archView.locator(".options");
    await expect(options).toBeVisible();

    // Click x86-64 option to select it
    const x86Option = options.locator(".option-card").filter({
      hasText: "Intel/AMD",
    });
    await x86Option.click();

    // Click Next button to advance to drive selection
    const nextButton = page.locator("wizard-shell").locator(".footer-button.primary");
    await nextButton.click();

    // Should navigate to drive selection
    await expect(page.locator("drive-selection-view")).toBeVisible();
  });
});

test.describe("Mini PC Flow - Navigation", () => {
  test("can navigate through Mini PC flow steps up to drive selection", async ({
    page,
  }) => {
    await page.goto("/?mock=true");

    // Step 1: Welcome
    await page.locator("welcome-view").locator(".lets-go-button").click();

    // Step 2: Path selection - Mini PC
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    // Step 3: Setup method - Connect drive
    const setupView = page.locator("minipc-setup-method-view");
    await expect(setupView).toBeVisible();

    const connectDriveOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I can connect the drive" });
    await connectDriveOption.click();

    // Step 4: Architecture selection - x86-64
    const archView = page.locator("minipc-architecture-selection-view");
    const options = archView.locator(".options");
    await expect(options).toBeVisible();

    const x86Option = options.locator(".option-card").filter({
      hasText: "Intel/AMD",
    });
    await x86Option.click();

    // Click Next to proceed to drive selection
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await nextButton.click();

    // Step 5: Drive selection
    const driveSelection = page.locator("drive-selection-view");
    await expect(driveSelection).toBeVisible();

    // Drive selection view should show title
    await expect(driveSelection.locator("h2")).toContainText("Select");
  });

  test("can navigate through Mini PC flow with ARM64 architecture", async ({
    page,
  }) => {
    await page.goto("/?mock=true");

    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await page.locator('option-card[title="Generic (mini) PC"]').click();

    // Setup method - Connect drive
    const setupView = page.locator("minipc-setup-method-view");
    await expect(setupView).toBeVisible();
    const connectDriveOption = setupView
      .locator(".option-card")
      .filter({ hasText: "I can connect the drive" });
    await connectDriveOption.click();

    // Select ARM64 architecture
    const archView = page.locator("minipc-architecture-selection-view");
    const options = archView.locator(".options");
    await expect(options).toBeVisible();

    const armOption = options.locator(".option-card").filter({
      hasText: "ARM",
    });
    await armOption.click();

    // Click Next to proceed to drive selection
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await nextButton.click();

    // Should navigate to drive selection
    await expect(page.locator("drive-selection-view")).toBeVisible();
  });
});
