import { expect, fixture, html } from "@open-wc/testing";
import "../../../src/components/drive-card.js";
import type { DriveCard } from "../../../src/components/drive-card.js";

describe("drive-card", () => {
  it("renders with drive name", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="My USB Drive"></drive-card>
    `);

    const name = el.shadowRoot!.querySelector(".name");
    expect(name).to.exist;
    expect(name!.textContent).to.equal("My USB Drive");
  });

  it("renders with formatted size", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" size="32000000000"></drive-card>
    `);

    const size = el.shadowRoot!.querySelector(".size");
    expect(size).to.exist;
    expect(size!.textContent).to.equal("30 GB");
  });

  it("formats large sizes in TB", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" size="2000000000000"></drive-card>
    `);

    const size = el.shadowRoot!.querySelector(".size");
    expect(size!.textContent).to.equal("1.8 TB");
  });

  it("displays 0 GB for zero size", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" size="0"></drive-card>
    `);

    const size = el.shadowRoot!.querySelector(".size");
    expect(size!.textContent).to.equal("0 GB");
  });

  it("shows selected state when selected", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" selected></drive-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("selected")).to.be.true;

    const indicator = el.shadowRoot!.querySelector(".selected-indicator");
    expect(indicator).to.exist;
  });

  it("does not show selected indicator when not selected", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test"></drive-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("selected")).to.be.false;

    const indicator = el.shadowRoot!.querySelector(".selected-indicator");
    expect(indicator).to.be.null;
  });

  it("shows disabled state when disabled", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" disabled></drive-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("disabled")).to.be.true;
  });

  it("displays disabled reason when provided", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card
        name="Test"
        disabled
        disabledReason="Drive is too small"
      ></drive-card>
    `);

    const details = el.shadowRoot!.querySelector(".details");
    expect(details!.textContent).to.equal("Drive is too small");
  });

  it("renders SD card icon for sd_card type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="sd_card"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("renders USB icon for usb_drive type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="usb_drive"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("renders SSD icon for ssd type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="ssd"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("renders HDD icon for hdd type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="hdd"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("renders SSD icon for nvme type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="nvme"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("renders generic icon for unknown type", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="unknown"></drive-card>
    `);

    const icon = el.shadowRoot!.querySelector(".icon-container svg");
    expect(icon).to.exist;
  });

  it("displays vendor and model in details", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" vendor="SanDisk" model="Ultra"></drive-card>
    `);

    const details = el.shadowRoot!.querySelector(".details");
    expect(details!.textContent).to.equal("SanDisk Ultra");
  });

  it("displays type label when vendor/model not provided", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="usb_drive"></drive-card>
    `);

    const details = el.shadowRoot!.querySelector(".details");
    expect(details!.textContent).to.equal("USB Drive");
  });

  it("shows description for SD card", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="sd_card"></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description!.textContent).to.equal(
      "Great for Raspberry Pi and similar single-board computers"
    );
  });

  it("shows description for USB drive", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="usb_drive"></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description!.textContent).to.equal("Portable and easy to set up");
  });

  it("shows description for SSD", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="ssd"></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description!.textContent).to.equal("Fast and reliable for daily use");
  });

  it("shows description for HDD", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="hdd"></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description!.textContent).to.equal("High capacity storage option");
  });

  it("shows description for NVMe", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="nvme"></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description!.textContent).to.equal("Maximum performance storage");
  });

  it("does not show description when disabled", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="ssd" disabled></drive-card>
    `);

    const description = el.shadowRoot!.querySelector(".description");
    expect(description).to.be.null;
  });

  it("has the correct structure", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test"></drive-card>
    `);

    expect(el.shadowRoot!.querySelector(".card")).to.exist;
    expect(el.shadowRoot!.querySelector(".icon-container")).to.exist;
    expect(el.shadowRoot!.querySelector(".info")).to.exist;
    expect(el.shadowRoot!.querySelector(".name")).to.exist;
    expect(el.shadowRoot!.querySelector(".details")).to.exist;
    expect(el.shadowRoot!.querySelector(".size")).to.exist;
  });

  it("stores driveId property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card driveId="/dev/sda" name="Test"></drive-card>
    `);

    expect(el.driveId).to.equal("/dev/sda");
  });

  it("stores name property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="My Drive"></drive-card>
    `);

    expect(el.name).to.equal("My Drive");
  });

  it("stores size property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" size="64000000000"></drive-card>
    `);

    expect(el.size).to.equal(64000000000);
  });

  it("stores deviceType property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" deviceType="usb_drive"></drive-card>
    `);

    expect(el.deviceType).to.equal("usb_drive");
  });

  it("stores model property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" model="XYZ123"></drive-card>
    `);

    expect(el.model).to.equal("XYZ123");
  });

  it("stores vendor property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" vendor="Samsung"></drive-card>
    `);

    expect(el.vendor).to.equal("Samsung");
  });

  it("stores selected property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" selected></drive-card>
    `);

    expect(el.selected).to.be.true;
  });

  it("stores disabled property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" disabled></drive-card>
    `);

    expect(el.disabled).to.be.true;
  });

  it("stores disabledReason property", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" disabledReason="Too small"></drive-card>
    `);

    expect(el.disabledReason).to.equal("Too small");
  });

  it("applies both selected and disabled classes when both true", async () => {
    const el = await fixture<DriveCard>(html`
      <drive-card name="Test" selected disabled></drive-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("selected")).to.be.true;
    expect(card!.classList.contains("disabled")).to.be.true;
  });
});
