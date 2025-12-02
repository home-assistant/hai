import { expect, fixture, html } from "@open-wc/testing";
import "../../../src/components/progress-bar.js";
import type { ProgressBar } from "../../../src/components/progress-bar.js";

describe("progress-bar", () => {
  it("renders with default progress (0)", async () => {
    const el = await fixture<ProgressBar>(html`<progress-bar></progress-bar>`);

    const container = el.shadowRoot!.querySelector(".progress-container");
    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;

    expect(container).to.exist;
    expect(fill).to.exist;
    expect(fill.style.width).to.equal("0%");
  });

  it("renders with 0% progress", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="0"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("0%");
  });

  it("renders with 50% progress", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="50"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("50%");
  });

  it("renders with 100% progress", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="100"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("100%");
  });

  it("clamps progress above 100 to 100%", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="150"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("100%");
  });

  it("clamps negative progress to 0%", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="-10"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("0%");
  });

  it("renders in indeterminate mode", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar indeterminate></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill");
    expect(fill!.classList.contains("indeterminate")).to.be.true;
  });

  it("indeterminate mode overrides progress value", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="75" indeterminate></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.classList.contains("indeterminate")).to.be.true;
    expect(fill.style.width).to.equal("30%");
  });

  it("updates progress when property changes", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="25"></progress-bar>
    `);

    let fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("25%");

    el.progress = 75;
    await el.updateComplete;

    fill = el.shadowRoot!.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).to.equal("75%");
  });

  it("renders with error state", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="50" error></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill");
    expect(fill!.classList.contains("error")).to.be.true;
  });

  it("does not show error class when error is false", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="50"></progress-bar>
    `);

    const fill = el.shadowRoot!.querySelector(".progress-fill");
    expect(fill!.classList.contains("error")).to.be.false;
  });

  it("can toggle error state", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="50"></progress-bar>
    `);

    let fill = el.shadowRoot!.querySelector(".progress-fill");
    expect(fill!.classList.contains("error")).to.be.false;

    el.error = true;
    await el.updateComplete;

    fill = el.shadowRoot!.querySelector(".progress-fill");
    expect(fill!.classList.contains("error")).to.be.true;
  });

  it("has correct structure", async () => {
    const el = await fixture<ProgressBar>(html`<progress-bar></progress-bar>`);

    expect(el.shadowRoot!.querySelector(".progress-container")).to.exist;
    expect(el.shadowRoot!.querySelector(".progress-fill")).to.exist;
  });

  it("stores progress property", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar progress="42"></progress-bar>
    `);

    expect(el.progress).to.equal(42);
  });

  it("stores indeterminate property", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar indeterminate></progress-bar>
    `);

    expect(el.indeterminate).to.be.true;
  });

  it("stores error property", async () => {
    const el = await fixture<ProgressBar>(html`
      <progress-bar error></progress-bar>
    `);

    expect(el.error).to.be.true;
  });
});
