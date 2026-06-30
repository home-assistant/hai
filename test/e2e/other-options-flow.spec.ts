import { test, expect } from "@playwright/test";

test.describe("Other Options View", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Navigate to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("navigates to Other Options view when clicking Others option", async ({
    page,
  }) => {
    // Click Others option
    await page.locator('option-card[title="Others"]').click();

    // Should show other options view
    const otherOptionsView = page.locator("other-options-view");
    await expect(otherOptionsView).toBeVisible();
  });

  test("shows correct title", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const title = page.locator("other-options-view").locator("h1");
    await expect(title).toContainText("Other installation methods");
  });

  test("shows subtitle explaining these options are not directly supported", async ({
    page,
  }) => {
    await page.locator('option-card[title="Others"]').click();

    const subtitle = page.locator("other-options-view").locator(".subtitle");
    await expect(subtitle).toContainText("not directly supported");
    await expect(subtitle).toContainText("documentation");
  });

  test("has back button that returns to path selection", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const otherOptionsView = page.locator("other-options-view");
    await expect(otherOptionsView).toBeVisible();

    // Click back button
    const backButton = otherOptionsView.locator(".back-button");
    await expect(backButton).toBeVisible();
    await backButton.click();

    // Should be back on path selection
    await expect(page.locator("path-selection-view")).toBeVisible();
  });

  test("displays Docker Container option", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const dockerOption = page
      .locator("other-options-view")
      .locator(".option-item")
      .filter({ hasText: "Docker container" });
    await expect(dockerOption).toBeVisible();
    await expect(dockerOption).toContainText("Home Assistant container");
  });

  test("displays Synology NAS option", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const synologyOption = page
      .locator("other-options-view")
      .locator(".option-item")
      .filter({ hasText: "Synology NAS" });
    await expect(synologyOption).toBeVisible();
    await expect(synologyOption).toContainText("Virtual Machine Manager");
  });

  test("displays QNAP NAS option", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const qnapOption = page
      .locator("other-options-view")
      .locator(".option-item")
      .filter({ hasText: "QNAP NAS" });
    await expect(qnapOption).toBeVisible();
    await expect(qnapOption).toContainText("Virtualization Station");
  });

  test("displays Linux Virtual Machine option", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const linuxVMOption = page
      .locator("other-options-view")
      .locator(".option-item")
      .filter({ hasText: "Linux virtual machine" });
    await expect(linuxVMOption).toBeVisible();
    await expect(linuxVMOption).toContainText("KVM");
    await expect(linuxVMOption).toContainText("VirtualBox");
  });

  test("displays Windows Virtual Machine option", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const windowsVMOption = page
      .locator("other-options-view")
      .locator(".option-item")
      .filter({ hasText: "Windows virtual machine" });
    await expect(windowsVMOption).toBeVisible();
    await expect(windowsVMOption).toContainText("Hyper-V");
    await expect(windowsVMOption).toContainText("VMware");
  });

  test("all options have external link indicator", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const options = page
      .locator("other-options-view")
      .locator(".option-item");
    const count = await options.count();

    // Should have 5 options
    expect(count).toBe(5);

    // Each option should have an external icon
    for (let i = 0; i < count; i++) {
      const externalIcon = options.nth(i).locator(".external-icon");
      await expect(externalIcon).toBeVisible();
      await expect(externalIcon).toContainText("↗");
    }
  });

  test("all options have icons", async ({ page }) => {
    await page.locator('option-card[title="Others"]').click();

    const options = page
      .locator("other-options-view")
      .locator(".option-item");
    const count = await options.count();

    // Each option should have an icon
    for (let i = 0; i < count; i++) {
      const icon = options.nth(i).locator(".option-icon");
      await expect(icon).toBeVisible();
    }
  });
});

test.describe("Other Options - Navigation", () => {
  test("can navigate from welcome to other options and back to welcome", async ({
    page,
  }) => {
    await page.goto("/");

    // Step 1: Welcome to path selection
    await page.locator("welcome-view").locator(".lets-go-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();

    // Step 2: Path selection to other options
    await page.locator('option-card[title="Others"]').click();
    await expect(page.locator("other-options-view")).toBeVisible();

    // Step 3: Other options back to path selection
    await page.locator("other-options-view").locator(".back-button").click();
    await expect(page.locator("path-selection-view")).toBeVisible();

    // Step 4: Path selection back to welcome
    await page.locator("path-selection-view").locator(".back-button").click();
    await expect(page.locator("welcome-view")).toBeVisible();
  });
});
