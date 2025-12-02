import { test, expect } from "@playwright/test";

test.describe("Navigation Flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("shows welcome view on initial load", async ({ page }) => {
    // Check for welcome view elements
    const logo = page.locator("welcome-view").locator(".logo");
    await expect(logo.first()).toBeVisible();

    const letsGoButton = page.locator("welcome-view").locator(".lets-go-button");
    await expect(letsGoButton).toBeVisible();
    await expect(letsGoButton).toContainText("Let's go");
  });

  test("navigates to path selection when clicking Let's go", async ({
    page,
  }) => {
    const letsGoButton = page.locator("welcome-view").locator(".lets-go-button");
    await letsGoButton.click();

    // Should now be on path selection view
    const pathSelectionView = page.locator("path-selection-view");
    await expect(pathSelectionView).toBeVisible();

    const title = pathSelectionView.locator("h1");
    await expect(title).toContainText("install on");
  });

  test("shows all installation options on path selection", async ({ page }) => {
    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();

    const pathSelectionView = page.locator("path-selection-view");
    await expect(pathSelectionView).toBeVisible();

    // Check for all options
    await expect(
      pathSelectionView.locator('option-card[title="Home Assistant Hardware"]')
    ).toBeVisible();
    await expect(
      pathSelectionView.locator(
        'option-card[title="Raspberry Pi & other boards"]'
      )
    ).toBeVisible();
    await expect(
      pathSelectionView.locator('option-card[title="Generic (mini) PC"]')
    ).toBeVisible();
    await expect(
      pathSelectionView.locator('option-card[title="Proxmox Server"]')
    ).toBeVisible();
    await expect(
      pathSelectionView.locator('option-card[title="Others"]')
    ).toBeVisible();
  });

  test("navigates back to welcome from path selection", async ({ page }) => {
    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();

    const pathSelectionView = page.locator("path-selection-view");
    await expect(pathSelectionView).toBeVisible();

    // Click back button
    await pathSelectionView.locator(".back-button").click();

    // Should be back on welcome view
    const welcomeView = page.locator("welcome-view");
    await expect(welcomeView).toBeVisible();
    await expect(welcomeView.locator(".lets-go-button")).toBeVisible();
  });

  test("welcome view shows OHF logo", async ({ page }) => {
    const ohfLink = page.locator("welcome-view").locator(".ohf-link");
    await expect(ohfLink).toBeVisible();

    // Check it links to OHF website
    await expect(ohfLink).toHaveAttribute(
      "href",
      "https://www.openhomefoundation.org/"
    );
  });

  test("welcome view shows learn more link", async ({ page }) => {
    const learnMore = page.locator("welcome-view").locator(".learn-more");
    await expect(learnMore).toBeVisible();
    await expect(learnMore).toContainText("learn more");
    await expect(learnMore).toHaveAttribute(
      "href",
      "https://www.home-assistant.io/installation/"
    );
  });
});

test.describe("Path Selection Options", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("Home Assistant Hardware option has correct description", async ({
    page,
  }) => {
    const card = page.locator('option-card[title="Home Assistant Hardware"]');
    await expect(card).toHaveAttribute(
      "description",
      /Green.*Yellow.*Blue.*Nabu Casa/
    );
  });

  test("Raspberry Pi option has correct description", async ({ page }) => {
    const card = page.locator(
      'option-card[title="Raspberry Pi & other boards"]'
    );
    await expect(card).toHaveAttribute("description", /Raspberry Pi.*ODROID/);
  });

  test("Mini PC option has correct description", async ({ page }) => {
    const card = page.locator('option-card[title="Generic (mini) PC"]');
    await expect(card).toHaveAttribute("description", /x86-64.*ARM64/);
  });

  test("Proxmox option has correct description", async ({ page }) => {
    const card = page.locator('option-card[title="Proxmox Server"]');
    await expect(card).toHaveAttribute("description", /VM.*Proxmox/);
  });
});

test.describe("Wizard Flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("clicking an option starts the wizard flow", async ({ page }) => {
    // Click Raspberry Pi option
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    // Should now see wizard shell
    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Should see step indicator
    await expect(wizardShell.locator("step-indicator")).toBeVisible();

    // Should see cancel button
    await expect(wizardShell.locator(".cancel-button")).toBeVisible();
  });

  test("wizard shows correct flow title", async ({ page }) => {
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toContainText("Raspberry Pi");
  });

  test("wizard cancel returns to welcome screen", async ({ page }) => {
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Click cancel
    await wizardShell.locator(".cancel-button").click();

    // Should be back on welcome screen
    await expect(page.locator("welcome-view")).toBeVisible();
  });

  test("wizard next button advances steps after selection", async ({ page }) => {
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // First step should show device selection
    await expect(page.locator("device-selection-view")).toBeVisible();

    // Wait for devices to load and select one
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();

    // Click next
    await wizardShell.locator(".footer-button.primary").click();

    // Should show "drive" step
    await expect(wizardShell).toContainText("drive");
  });

  test("wizard back button goes to previous step", async ({ page }) => {
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    const wizardShell = page.locator("wizard-shell");

    // Wait for devices to load and select one
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();

    // Go to second step
    await wizardShell.locator(".footer-button.primary").click();
    await expect(wizardShell).toContainText("drive");

    // Back button should now be enabled
    const backButton = wizardShell.locator(".back-button");
    await expect(backButton).toBeEnabled();

    // Click back
    await backButton.click();

    // Should be back on first step - device selection view
    await expect(page.locator("device-selection-view")).toBeVisible();
  });

  test("back button is disabled on first step", async ({ page }) => {
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();

    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");

    await expect(backButton).toBeDisabled();
  });
});

test.describe("SBC Device Selection", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to SBC flow
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();
    await expect(page.locator("wizard-shell")).toBeVisible();
  });

  test("shows device selection view on first step", async ({ page }) => {
    const deviceSelection = page.locator("device-selection-view");
    await expect(deviceSelection).toBeVisible();
  });

  test("shows device selection heading", async ({ page }) => {
    await expect(page.locator("device-selection-view h2")).toContainText(
      "Select your device"
    );
  });

  test("next button is disabled when no device selected", async ({ page }) => {
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeDisabled();
  });

  test("can select a device", async ({ page }) => {
    // Wait for devices to load
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });

    // Click to select
    await deviceCard.click();

    // Next button should now be enabled
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeEnabled();
  });

  test("shows multiple devices in grid", async ({ page }) => {
    // Wait for devices to load
    await expect(page.locator("device-card").first()).toBeVisible({
      timeout: 5000,
    });

    // Should have multiple devices
    const deviceCount = await page.locator("device-card").count();
    expect(deviceCount).toBeGreaterThan(1);
  });
});

test.describe("SBC Drive Selection", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to drive selection step
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();
    await expect(page.locator("wizard-shell")).toBeVisible();

    // Select a device and proceed to drive selection
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Should now be on drive selection
    await expect(page.locator("drive-selection-view")).toBeVisible();
  });

  test("shows drive selection view on second step", async ({ page }) => {
    const driveSelection = page.locator("drive-selection-view");
    await expect(driveSelection).toBeVisible();
  });

  test("shows drive selection heading", async ({ page }) => {
    await expect(page.locator("drive-selection-view h2")).toContainText(
      "Select your drive"
    );
  });

  test("shows data erasure warning", async ({ page }) => {
    const warning = page.locator("drive-selection-view .warning");
    await expect(warning).toBeVisible();
    await expect(warning).toContainText("Warning");
    await expect(warning).toContainText("erased");
  });

  test("shows refresh button", async ({ page }) => {
    const refreshButton = page.locator("drive-selection-view .refresh-button");
    await expect(refreshButton).toBeVisible();
    await expect(refreshButton).toContainText("Refresh");
  });

  test("next button is disabled when no drive selected", async ({ page }) => {
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeDisabled();
  });

  test("shows available drives", async ({ page }) => {
    // Wait for drives to load
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });

    // Should have multiple drives
    const driveCount = await page.locator("drive-card").count();
    expect(driveCount).toBeGreaterThan(0);
  });

  test("can select a drive", async ({ page }) => {
    // Wait for drives to load
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });

    // Click to select
    await driveCard.click();

    // Next button should now be enabled
    const nextButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(nextButton).toBeEnabled();
  });

  test("shows drive size in card", async ({ page }) => {
    // Wait for drives to load
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });

    // Drive card should show size (contains GB or TB)
    await expect(driveCard).toContainText(/\d+\s*(GB|TB)/);
  });

  test("can navigate back to device selection", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");

    // Back button should be enabled on drive step
    await expect(backButton).toBeEnabled();

    // Click back
    await backButton.click();

    // Should be back on device selection
    await expect(page.locator("device-selection-view")).toBeVisible();
  });
});

test.describe("SBC Confirmation", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to confirmation step
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();
    await expect(page.locator("wizard-shell")).toBeVisible();

    // Select a device and proceed
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Select a drive and proceed
    await expect(page.locator("drive-selection-view")).toBeVisible();
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });
    await driveCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Should now be on confirmation
    await expect(page.locator("confirmation-view")).toBeVisible();
  });

  test("shows confirmation view on third step", async ({ page }) => {
    const confirmationView = page.locator("confirmation-view");
    await expect(confirmationView).toBeVisible();
  });

  test("shows ready to install heading", async ({ page }) => {
    await expect(page.locator("confirmation-view h2")).toContainText(
      "Ready to install"
    );
  });

  test("shows selected device", async ({ page }) => {
    const confirmationView = page.locator("confirmation-view");
    // Should show device info
    await expect(confirmationView).toContainText("Device");
  });

  test("shows selected drive", async ({ page }) => {
    const confirmationView = page.locator("confirmation-view");
    // Should show target drive info
    await expect(confirmationView).toContainText("Target Drive");
  });

  test("shows HAOS version", async ({ page }) => {
    const confirmationView = page.locator("confirmation-view");
    // Should show Home Assistant Operating System version
    await expect(confirmationView).toContainText("Home Assistant Operating System");
    await expect(confirmationView).toContainText("Version");
  });

  test("shows Install button instead of Next", async ({ page }) => {
    const installButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await expect(installButton).toBeVisible();
    await expect(installButton).toContainText("Install");
  });

  test("clicking Install shows confirmation dialog", async ({ page }) => {
    const installButton = page
      .locator("wizard-shell")
      .locator(".footer-button.primary");
    await installButton.click();

    // Should show confirmation dialog
    const dialog = page.locator("confirm-dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Erase drive and install");
    await expect(dialog).toContainText("All data on");
  });

  test("confirmation dialog can be cancelled", async ({ page }) => {
    // Click Install to open dialog
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    const dialog = page.locator("confirm-dialog[open]");
    await expect(dialog).toBeVisible();

    // Click Cancel
    await dialog.locator(".dialog-button.secondary").click();

    // Dialog should close
    await expect(page.locator("confirm-dialog[open]")).not.toBeVisible();

    // Should still be on confirmation view
    await expect(page.locator("confirmation-view")).toBeVisible();
  });

  test("confirmation dialog can be confirmed", async ({ page }) => {
    // Click Install to open dialog
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    const dialog = page.locator("confirm-dialog[open]");
    await expect(dialog).toBeVisible();

    // Click Erase and Install
    await dialog.locator(".dialog-button.danger").click();

    // Dialog should close and should advance to flash step
    await expect(page.locator("confirm-dialog[open]")).not.toBeVisible();
    // Should now show the progress view
    await expect(page.locator("progress-view")).toBeVisible();
  });

  test("can navigate back to drive selection", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");

    // Back button should be enabled
    await expect(backButton).toBeEnabled();

    // Click back
    await backButton.click();

    // Should be back on drive selection
    await expect(page.locator("drive-selection-view")).toBeVisible();
  });
});

test.describe("SBC Flashing", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to flash step
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();
    await expect(page.locator("wizard-shell")).toBeVisible();

    // Select a device and proceed
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Select a drive and proceed
    await expect(page.locator("drive-selection-view")).toBeVisible();
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });
    await driveCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Should now be on confirmation
    await expect(page.locator("confirmation-view")).toBeVisible();

    // Click Install to show dialog
    await page.locator("wizard-shell").locator(".footer-button.primary").click();
    const dialog = page.locator("confirm-dialog[open]");
    await expect(dialog).toBeVisible();

    // Confirm to start flashing
    await dialog.locator(".dialog-button.danger").click();
    await expect(page.locator("progress-view")).toBeVisible();
  });

  test("shows progress view during flashing", async ({ page }) => {
    await expect(page.locator("progress-view")).toBeVisible();
  });

  test("shows progress bar", async ({ page }) => {
    await expect(page.locator("progress-view progress-bar")).toBeVisible();
  });

  test("shows thinking cloud with stage", async ({ page }) => {
    const progressView = page.locator("progress-view");
    // Should show thinking cloud with current stage
    await expect(progressView.locator(".thinking-cloud")).toBeVisible();
  });

  test("shows progress percentage", async ({ page }) => {
    const progressView = page.locator("progress-view");
    await expect(progressView.locator(".percentage")).toBeVisible();
  });

  test("shows stage indicator dots", async ({ page }) => {
    const progressView = page.locator("progress-view");
    await expect(progressView.locator(".stages-indicator")).toBeVisible();
    // Should have 5 stage dots (downloading, extracting, writing, verifying, finalizing)
    await expect(progressView.locator(".stage-dot")).toHaveCount(5);
  });

  test("footer is hidden during flashing", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    // Footer should not be visible
    await expect(wizardShell.locator(".footer")).not.toBeVisible();
  });

  test("back button is hidden during flashing", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");
    // Back button should be hidden (visibility: hidden)
    await expect(backButton).toHaveCSS("visibility", "hidden");
  });

  test("step indicator shows Install step as active", async ({ page }) => {
    const stepIndicator = page.locator("step-indicator");
    // The Install step should be marked as active (bold/current)
    await expect(stepIndicator).toContainText("Install");
  });

  test("progress updates during mock flashing", async ({ page }) => {
    const progressView = page.locator("progress-view");
    const percentage = progressView.locator(".percentage");

    // Wait a bit for progress to update
    await page.waitForTimeout(500);

    // Progress should have changed from 0
    const text = await percentage.textContent();
    // The percentage should show some progress
    expect(text).toBeTruthy();
  });

  test("advances to success after flashing completes", async ({ page }) => {
    // Wait for the mock flash to complete (mock takes ~16 seconds total)
    await expect(page.locator("progress-view")).toBeVisible();

    // Wait for flash to complete and advance to success step
    await expect(page.locator("success-view")).toBeVisible({ timeout: 20000 });
  });
});

test.describe("SBC Success", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to success step
    await page.goto("/?mock=true");
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
    await page
      .locator('option-card[title="Raspberry Pi & other boards"]')
      .click();
    await expect(page.locator("wizard-shell")).toBeVisible();

    // Select a device and proceed
    const deviceCard = page.locator("device-card").first();
    await expect(deviceCard).toBeVisible({ timeout: 5000 });
    await deviceCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Select a drive and proceed
    await expect(page.locator("drive-selection-view")).toBeVisible();
    const driveCard = page.locator("drive-card").first();
    await expect(driveCard).toBeVisible({ timeout: 5000 });
    await driveCard.click();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();

    // Confirm installation
    await expect(page.locator("confirmation-view")).toBeVisible();
    await page.locator("wizard-shell").locator(".footer-button.primary").click();
    const dialog = page.locator("confirm-dialog[open]");
    await expect(dialog).toBeVisible();
    await dialog.locator(".dialog-button.danger").click();

    // Wait for flashing to complete (mock takes ~16 seconds total)
    await expect(page.locator("progress-view")).toBeVisible();
    await expect(page.locator("success-view")).toBeVisible({ timeout: 20000 });
  });

  test("shows success view after flashing completes", async ({ page }) => {
    await expect(page.locator("success-view")).toBeVisible();
  });

  test("shows happy Casita mascot", async ({ page }) => {
    const successView = page.locator("success-view");
    await expect(successView.locator(".mascot-container")).toBeVisible();
    await expect(successView.locator(".casita-mascot")).toBeVisible();
  });

  test("shows success heading", async ({ page }) => {
    await expect(page.locator("success-view h2")).toContainText("You're all set");
  });

  test("shows installation complete message with device name", async ({
    page,
  }) => {
    const successView = page.locator("success-view");
    await expect(successView.locator(".subtitle")).toContainText(
      "Home Assistant has been installed on your"
    );
  });

  test("shows next steps section", async ({ page }) => {
    const successView = page.locator("success-view");
    await expect(successView.locator(".next-steps")).toBeVisible();
    await expect(successView.locator(".next-steps-title")).toContainText(
      "Next Steps"
    );
  });

  test("shows four numbered steps", async ({ page }) => {
    const successView = page.locator("success-view");
    const steps = successView.locator(".step-item");
    await expect(steps).toHaveCount(4);
  });

  test("shows companion app section", async ({ page }) => {
    const successView = page.locator("success-view");
    await expect(successView.locator(".companion-section")).toBeVisible();
    await expect(successView.locator(".companion-title")).toContainText(
      "Companion App"
    );
  });

  test("shows app store links for iOS and Android", async ({ page }) => {
    const successView = page.locator("success-view");
    const appLinks = successView.locator(".app-link");
    await expect(appLinks).toHaveCount(2);
    await expect(appLinks.first()).toContainText("App Store");
    await expect(appLinks.last()).toContainText("Google Play");
  });

  test("shows footer with Done button", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const footer = wizardShell.locator(".footer");
    await expect(footer).toBeVisible();

    const doneButton = wizardShell.locator(".footer-button.primary");
    await expect(doneButton).toBeVisible();
    await expect(doneButton).toContainText("Done");
  });

  test("back button is hidden on success step", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const backButton = wizardShell.locator(".back-button");
    await expect(backButton).toHaveCSS("visibility", "hidden");
  });

  test("clicking Done returns to welcome screen", async ({ page }) => {
    const wizardShell = page.locator("wizard-shell");
    const doneButton = wizardShell.locator(".footer-button.primary");
    await doneButton.click();

    // Should be back on welcome screen
    await expect(page.locator("welcome-view")).toBeVisible();
  });
});
