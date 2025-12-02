import { expect, fixture, html, oneEvent } from "@open-wc/testing";
import "../../../src/views/path-selection-view.js";
import type { PathSelectionView } from "../../../src/views/path-selection-view.js";

describe("path-selection-view", () => {
  it("renders the back button", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const backButton = el.shadowRoot!.querySelector(".back-button");
    expect(backButton).to.exist;
    expect(backButton!.textContent).to.include("Back");
  });

  it("renders the title and subtitle", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const title = el.shadowRoot!.querySelector("h1");
    const subtitle = el.shadowRoot!.querySelector(".subtitle");

    expect(title).to.exist;
    expect(title!.textContent).to.include("install on");
    expect(subtitle).to.exist;
    expect(subtitle!.textContent).to.include("Select how");
  });

  it("renders all installation options", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const optionCards = el.shadowRoot!.querySelectorAll("option-card");

    // Should have at least 5 options (HA Hardware, Raspberry Pi, Mini PC, Proxmox, Others)
    // VM option is conditional based on OS
    expect(optionCards.length).to.be.at.least(5);

    const titles = Array.from(optionCards).map((card) =>
      card.getAttribute("title")
    );

    expect(titles).to.include("Home Assistant Hardware");
    expect(titles).to.include("Raspberry Pi & other boards");
    expect(titles).to.include("Generic (mini) PC");
    expect(titles).to.include("Proxmox Server");
    expect(titles).to.include("Others");
  });

  it("dispatches navigate event when back is clicked", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const backButton = el.shadowRoot!.querySelector(
      ".back-button"
    ) as HTMLButtonElement;

    setTimeout(() => backButton.click());
    const event = await oneEvent(el, "navigate");

    expect(event).to.exist;
    expect((event as CustomEvent).detail.view).to.equal("welcome");
  });

  it("dispatches select-path event when an option is clicked", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const haHardwareCard = el.shadowRoot!.querySelector(
      'option-card[icon="ha-hardware"]'
    ) as HTMLElement;

    setTimeout(() => haHardwareCard.click());
    const event = await oneEvent(el, "select-path");

    expect(event).to.exist;
    expect((event as CustomEvent).detail.path).to.equal("ha-hardware");
  });

  it("renders SBC option with correct details", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const sbcCard = el.shadowRoot!.querySelector('option-card[icon="sbc"]');
    expect(sbcCard).to.exist;
    expect(sbcCard!.getAttribute("description")).to.include("Raspberry Pi");
  });

  it("renders Mini PC option with correct details", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const minipcCard = el.shadowRoot!.querySelector(
      'option-card[icon="minipc"]'
    );
    expect(minipcCard).to.exist;
    expect(minipcCard!.getAttribute("title")).to.include("Generic");
    expect(minipcCard!.getAttribute("description")).to.include("x86-64");
  });

  it("renders Proxmox option with correct details", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const proxmoxCard = el.shadowRoot!.querySelector(
      'option-card[icon="proxmox"]'
    );
    expect(proxmoxCard).to.exist;
    expect(proxmoxCard!.getAttribute("description")).to.include("VM");
  });

  it("has options grid layout", async () => {
    const el = await fixture<PathSelectionView>(
      html`<path-selection-view></path-selection-view>`
    );

    const grid = el.shadowRoot!.querySelector(".options-grid");
    expect(grid).to.exist;
  });
});
