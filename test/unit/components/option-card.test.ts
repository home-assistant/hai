import { expect, fixture, html } from "@open-wc/testing";
import "../../../src/components/option-card.js";
import type { OptionCard } from "../../../src/components/option-card.js";

describe("option-card", () => {
  it("renders with title and description", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card
        title="Test Title"
        description="Test Description"
      ></option-card>
    `);

    const title = el.shadowRoot!.querySelector(".title");
    const description = el.shadowRoot!.querySelector(".description");

    expect(title).to.exist;
    expect(title!.textContent).to.equal("Test Title");
    expect(description).to.exist;
    expect(description!.textContent).to.equal("Test Description");
  });

  it("renders with an icon from the icon map", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card title="SBC" icon="sbc"></option-card>
    `);

    const img = el.shadowRoot!.querySelector(".icon-container img");
    expect(img).to.exist;
    expect(img!.getAttribute("src")).to.include("sbc-placeholder.svg");
  });

  it("renders with a custom image", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card
        title="Custom"
        image="/custom/path.svg"
      ></option-card>
    `);

    const img = el.shadowRoot!.querySelector(".icon-container img");
    expect(img).to.exist;
    expect(img!.getAttribute("src")).to.equal("/custom/path.svg");
  });

  it("renders placeholder when no icon or image provided", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card title="No Icon"></option-card>
    `);

    const placeholder = el.shadowRoot!.querySelector(".icon-placeholder");
    expect(placeholder).to.exist;
  });

  it("has the correct card structure", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card title="Test"></option-card>
    `);

    const card = el.shadowRoot!.querySelector(".card");
    expect(card).to.exist;
    expect(card!.querySelector(".icon-container")).to.exist;
    expect(card!.querySelector(".title")).to.exist;
    expect(card!.querySelector(".description")).to.exist;
  });

  it("uses Home Assistant logo for ha-hardware icon", async () => {
    const el = await fixture<OptionCard>(html`
      <option-card title="HA Hardware" icon="ha-hardware"></option-card>
    `);

    const img = el.shadowRoot!.querySelector(".icon-container img");
    expect(img).to.exist;
    expect(img!.getAttribute("src")).to.include("home-assistant-hardware.svg");
  });
});
