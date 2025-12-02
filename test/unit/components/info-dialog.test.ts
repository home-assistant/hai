import { expect, fixture, html, oneEvent } from "@open-wc/testing";
import "../../../src/components/info-dialog.js";
import type { InfoDialog } from "../../../src/components/info-dialog.js";

describe("info-dialog", () => {
  it("is hidden when open is false", async () => {
    const el = await fixture<InfoDialog>(html`<info-dialog></info-dialog>`);

    expect(el.hasAttribute("open")).to.be.false;
    expect(window.getComputedStyle(el).display).to.equal("none");
  });

  it("is visible when open is true", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    expect(el.hasAttribute("open")).to.be.true;
    expect(window.getComputedStyle(el).display).to.not.equal("none");
  });

  it("renders with custom title", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open title="Test Title"></info-dialog>
    `);

    const title = el.shadowRoot!.querySelector(".dialog-title");
    expect(title).to.exist;
    expect(title!.textContent).to.equal("Test Title");
  });

  it("renders with custom message", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open message="Test message content"></info-dialog>
    `);

    const message = el.shadowRoot!.querySelector(".dialog-message");
    expect(message).to.exist;
    expect(message!.textContent).to.equal("Test message content");
  });

  it("renders info icon", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const icon = el.shadowRoot!.querySelector(".info-icon");
    expect(icon).to.exist;
    expect(icon!.querySelector("svg")).to.exist;
  });

  it("renders primary button with default label", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const primaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.primary"
    );
    expect(primaryButton).to.exist;
    expect(primaryButton!.textContent!.trim()).to.equal("OK");
  });

  it("renders primary button with custom label", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open primaryLabel="Got it"></info-dialog>
    `);

    const primaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.primary"
    );
    expect(primaryButton!.textContent!.trim()).to.equal("Got it");
  });

  it("does not render secondary button by default", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const secondaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    );
    expect(secondaryButton).to.be.null;
  });

  it("renders secondary button when label is provided", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open secondaryLabel="Cancel"></info-dialog>
    `);

    const secondaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    );
    expect(secondaryButton).to.exist;
    expect(secondaryButton!.textContent!.trim()).to.equal("Cancel");
  });

  it("dispatches dialog-primary event when primary button is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const primaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.primary"
    ) as HTMLButtonElement;

    setTimeout(() => primaryButton.click());
    const event = await oneEvent(el, "dialog-primary");
    expect(event).to.exist;
  });

  it("dispatches dialog-secondary event when secondary button is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open secondaryLabel="Cancel"></info-dialog>
    `);

    const secondaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    ) as HTMLButtonElement;

    setTimeout(() => secondaryButton.click());
    const event = await oneEvent(el, "dialog-secondary");
    expect(event).to.exist;
  });

  it("closes dialog when primary button is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const primaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.primary"
    ) as HTMLButtonElement;

    expect(el.open).to.be.true;
    primaryButton.click();
    await el.updateComplete;
    expect(el.open).to.be.false;
  });

  it("closes dialog when secondary button is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open secondaryLabel="Cancel"></info-dialog>
    `);

    const secondaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    ) as HTMLButtonElement;

    expect(el.open).to.be.true;
    secondaryButton.click();
    await el.updateComplete;
    expect(el.open).to.be.false;
  });

  it("dispatches dialog-secondary when overlay is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const overlay = el.shadowRoot!.querySelector(".overlay") as HTMLElement;

    setTimeout(() => overlay.click());
    const event = await oneEvent(el, "dialog-secondary");
    expect(event).to.exist;
  });

  it("does not close when dialog content is clicked", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const dialog = el.shadowRoot!.querySelector(".dialog") as HTMLElement;

    expect(el.open).to.be.true;
    dialog.click();
    await el.updateComplete;
    expect(el.open).to.be.true;
  });

  it("renders slotted content", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open>
        <div id="test-content">Extra content</div>
      </info-dialog>
    `);

    const slot = el.shadowRoot!.querySelector("slot");
    expect(slot).to.exist;
  });

  it("has correct dialog structure", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    expect(el.shadowRoot!.querySelector(".overlay")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-header")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-content")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-actions")).to.exist;
  });

  it("stores title property", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog title="Custom Title"></info-dialog>
    `);

    expect(el.title).to.equal("Custom Title");
  });

  it("stores message property", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog message="Custom message"></info-dialog>
    `);

    expect(el.message).to.equal("Custom message");
  });

  it("stores primaryLabel property", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog primaryLabel="Confirm"></info-dialog>
    `);

    expect(el.primaryLabel).to.equal("Confirm");
  });

  it("stores secondaryLabel property", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog secondaryLabel="Dismiss"></info-dialog>
    `);

    expect(el.secondaryLabel).to.equal("Dismiss");
  });

  it("stores open property", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    expect(el.open).to.be.true;
  });

  it("can toggle open state", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog></info-dialog>
    `);

    expect(el.open).to.be.false;
    expect(el.hasAttribute("open")).to.be.false;

    el.open = true;
    await el.updateComplete;

    expect(el.open).to.be.true;
    expect(el.hasAttribute("open")).to.be.true;
  });

  it("events bubble and are composed", async () => {
    const el = await fixture<InfoDialog>(html`
      <info-dialog open></info-dialog>
    `);

    const primaryButton = el.shadowRoot!.querySelector(
      ".dialog-button.primary"
    ) as HTMLButtonElement;

    setTimeout(() => primaryButton.click());
    const event = await oneEvent(el, "dialog-primary");
    expect(event.bubbles).to.be.true;
    expect(event.composed).to.be.true;
  });
});
