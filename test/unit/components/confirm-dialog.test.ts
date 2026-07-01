import { expect, fixture, html, oneEvent } from "@open-wc/testing";
import "../../../src/components/confirm-dialog.js";
import type { ConfirmDialog } from "../../../src/components/confirm-dialog.js";

describe("confirm-dialog", () => {
  it("is hidden when open is false", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog></confirm-dialog>
    `);

    expect(el.hasAttribute("open")).to.be.false;
    expect(window.getComputedStyle(el).display).to.equal("none");
  });

  it("is visible when open is true", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    expect(el.hasAttribute("open")).to.be.true;
    expect(window.getComputedStyle(el).display).to.not.equal("none");
  });

  it("renders dialog with title", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const title = el.shadowRoot!.querySelector(".dialog-title");
    expect(title).to.exist;
    expect(title!.textContent).to.equal("Erase drive and install?");
  });

  it("renders dialog with warning icon", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const icon = el.shadowRoot!.querySelector(".warning-icon");
    expect(icon).to.exist;
  });

  it("renders dialog with drive name in message", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open driveName="My USB Drive"></confirm-dialog>
    `);

    const driveName = el.shadowRoot!.querySelector(".drive-name");
    expect(driveName).to.exist;
    expect(driveName!.textContent).to.equal("My USB Drive");
  });

  it("renders cancel button", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const cancelButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    );
    expect(cancelButton).to.exist;
    expect(cancelButton!.textContent!.trim()).to.equal("Cancel");
  });

  it("renders confirm button", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const confirmButton = el.shadowRoot!.querySelector(
      ".dialog-button.danger"
    );
    expect(confirmButton).to.exist;
    expect(confirmButton!.textContent!.trim()).to.equal("Erase and install");
  });

  it("dispatches dialog-cancel event when cancel button is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const cancelButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    ) as HTMLButtonElement;

    setTimeout(() => cancelButton.click());
    const event = await oneEvent(el, "dialog-cancel");
    expect(event).to.exist;
  });

  it("dispatches dialog-confirm event when confirm button is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const confirmButton = el.shadowRoot!.querySelector(
      ".dialog-button.danger"
    ) as HTMLButtonElement;

    setTimeout(() => confirmButton.click());
    const event = await oneEvent(el, "dialog-confirm");
    expect(event).to.exist;
  });

  it("closes dialog when cancel button is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const cancelButton = el.shadowRoot!.querySelector(
      ".dialog-button.secondary"
    ) as HTMLButtonElement;

    expect(el.open).to.be.true;
    cancelButton.click();
    await el.updateComplete;
    expect(el.open).to.be.false;
  });

  it("closes dialog when confirm button is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const confirmButton = el.shadowRoot!.querySelector(
      ".dialog-button.danger"
    ) as HTMLButtonElement;

    expect(el.open).to.be.true;
    confirmButton.click();
    await el.updateComplete;
    expect(el.open).to.be.false;
  });

  it("dispatches dialog-cancel when overlay is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const overlay = el.shadowRoot!.querySelector(".overlay") as HTMLElement;

    setTimeout(() => overlay.click());
    const event = await oneEvent(el, "dialog-cancel");
    expect(event).to.exist;
  });

  it("does not close when dialog content is clicked", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const dialog = el.shadowRoot!.querySelector(".dialog") as HTMLElement;

    expect(el.open).to.be.true;
    dialog.click();
    await el.updateComplete;
    expect(el.open).to.be.true;
  });

  it("has correct dialog structure", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    expect(el.shadowRoot!.querySelector(".overlay")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-header")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-content")).to.exist;
    expect(el.shadowRoot!.querySelector(".dialog-actions")).to.exist;
  });

  it("stores driveName property", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog driveName="Test Drive"></confirm-dialog>
    `);

    expect(el.driveName).to.equal("Test Drive");
  });

  it("stores open property", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    expect(el.open).to.be.true;
  });

  it("can toggle open state", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog></confirm-dialog>
    `);

    expect(el.open).to.be.false;
    expect(el.hasAttribute("open")).to.be.false;

    el.open = true;
    await el.updateComplete;

    expect(el.open).to.be.true;
    expect(el.hasAttribute("open")).to.be.true;
  });

  it("event bubbles and is composed", async () => {
    const el = await fixture<ConfirmDialog>(html`
      <confirm-dialog open></confirm-dialog>
    `);

    const confirmButton = el.shadowRoot!.querySelector(
      ".dialog-button.danger"
    ) as HTMLButtonElement;

    setTimeout(() => confirmButton.click());
    const event = await oneEvent(el, "dialog-confirm");
    expect(event.bubbles).to.be.true;
    expect(event.composed).to.be.true;
  });
});
