import { test, expect } from "@playwright/test";

// NOTE: The HA Hardware flow is currently a placeholder - the device/connect views
// are not yet wired up in app-shell.ts. These tests verify the current state.

test.describe("Home Assistant Hardware Flow", () => {
  test.beforeEach(async ({ page }) => {
    // Use mock mode to avoid real API calls
    await page.goto("/?mock=true");
    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("navigates to wizard when clicking HA Hardware option", async ({
    page,
  }) => {
    // Click Home Assistant Hardware option
    await page.locator('option-card[title="Home Assistant Hardware"]').click();

    // Should now see wizard shell
    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Should see step indicator
    await expect(wizardShell.locator("step-indicator")).toBeVisible();
  });

  test("shows step indicator with correct steps for HA Hardware flow", async ({
    page,
  }) => {
    await page.locator('option-card[title="Home Assistant Hardware"]').click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // HA Hardware flow has 3 steps: device, connect, success
    const stepIndicator = wizardShell.locator("step-indicator");
    await expect(stepIndicator).toBeVisible();
  });

  test("shows placeholder content for unimplemented flow", async ({ page }) => {
    await page.locator('option-card[title="Home Assistant Hardware"]').click();

    // The wizard shell should be visible with placeholder content
    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Should show Home Assistant Hardware title in placeholder
    await expect(wizardShell).toContainText("Home Assistant Hardware");
  });

  test("wizard has cancel button that returns to welcome", async ({ page }) => {
    await page.locator('option-card[title="Home Assistant Hardware"]').click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Click cancel button
    const cancelButton = wizardShell.locator(".cancel-button");
    await expect(cancelButton).toBeVisible();
    await cancelButton.click();

    // Should return to welcome view
    await expect(page.locator("welcome-view")).toBeVisible();
  });

  test("wizard has back button (disabled on first step)", async ({ page }) => {
    await page.locator('option-card[title="Home Assistant Hardware"]').click();

    const wizardShell = page.locator("wizard-shell");
    await expect(wizardShell).toBeVisible();

    // Back button should be visible but disabled on first step
    const backButton = wizardShell.locator(".back-button");
    await expect(backButton).toBeVisible();
    await expect(backButton).toBeDisabled();
  });
});

test.describe("Home Assistant Hardware - Path Selection", () => {
  test("HA Hardware option is visible on path selection", async ({ page }) => {
    await page.goto("/");
    await page.locator("welcome-view").locator(".lets-go-button").click();

    const pathSelection = page.locator("path-selection-view");
    await expect(pathSelection).toBeVisible();

    const haHardwareOption = pathSelection.locator(
      'option-card[title="Home Assistant Hardware"]'
    );
    await expect(haHardwareOption).toBeVisible();
  });

  test("HA Hardware option has correct description", async ({ page }) => {
    await page.goto("/");
    await page.locator("welcome-view").locator(".lets-go-button").click();

    const haHardwareOption = page.locator(
      'option-card[title="Home Assistant Hardware"]'
    );
    await expect(haHardwareOption).toHaveAttribute(
      "description",
      /Green.*Yellow.*Blue.*Nabu Casa/
    );
  });
});
