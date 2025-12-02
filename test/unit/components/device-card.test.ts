import { expect, fixture, html } from "@open-wc/testing";
import "../../../src/components/device-card.js";
import type { DeviceCard } from "../../../src/components/device-card.js";

describe("device-card", () => {
  it("renders with name", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card name="Raspberry Pi 5"></device-card>
    `);

    const name = el.shadowRoot!.querySelector(".name");
    expect(name).to.exist;
    expect(name!.textContent).to.equal("Raspberry Pi 5");
  });

  it("renders with image when provided", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card
        name="Test Device"
        image="/assets/devices/test.svg"
      ></device-card>
    `);

    const img = el.shadowRoot!.querySelector(".image-container img");
    expect(img).to.exist;
    expect(img!.getAttribute("src")).to.equal("/assets/devices/test.svg");
  });

  it("renders placeholder when no image provided", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card name="Test Device"></device-card>
    `);

    const placeholder = el.shadowRoot!.querySelector(".image-placeholder");
    expect(placeholder).to.exist;
  });

  it("shows selected state when selected", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card name="Test Device" selected></device-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("selected")).to.be.true;

    const indicator = el.shadowRoot!.querySelector(".selected-indicator");
    expect(indicator).to.exist;
  });

  it("does not show selected indicator when not selected", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card name="Test Device"></device-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card!.classList.contains("selected")).to.be.false;

    const indicator = el.shadowRoot!.querySelector(".selected-indicator");
    expect(indicator).to.be.null;
  });

  it("has the correct structure", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card name="Test Device"></device-card>
    `);

    expect(el.shadowRoot!.querySelector(".card-wrapper")).to.exist;
    expect(el.shadowRoot!.querySelector(".card")).to.exist;
    expect(el.shadowRoot!.querySelector(".image-container")).to.exist;
    expect(el.shadowRoot!.querySelector(".name")).to.exist;
  });

  it("stores deviceId property", async () => {
    const el = await fixture<DeviceCard>(html`
      <device-card deviceId="rpi5" name="Raspberry Pi 5"></device-card>
    `);

    expect(el.deviceId).to.equal("rpi5");
  });
});
