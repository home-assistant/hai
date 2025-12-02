import { expect, fixture, html, oneEvent } from "@open-wc/testing";
import "../../../src/views/welcome-view.js";
import type { WelcomeView } from "../../../src/views/welcome-view.js";

describe("welcome-view", () => {
  it("renders the Home Assistant logo", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const lightLogo = el.shadowRoot!.querySelector(".logo-light");
    const darkLogo = el.shadowRoot!.querySelector(".logo-dark");

    expect(lightLogo).to.exist;
    expect(darkLogo).to.exist;
    expect(lightLogo!.getAttribute("src")).to.include(
      "home-assistant-logo-light.svg"
    );
    expect(darkLogo!.getAttribute("src")).to.include(
      "home-assistant-logo-dark.svg"
    );
  });

  it("renders the welcome text", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const welcomeText = el.shadowRoot!.querySelector(".welcome-text");
    expect(welcomeText).to.exist;
    expect(welcomeText!.textContent).to.include("privacy-first");
    expect(welcomeText!.textContent).to.include("Home Assistant");
  });

  it("renders the Let's go button", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const button = el.shadowRoot!.querySelector(".lets-go-button");
    expect(button).to.exist;
    expect(button!.textContent).to.include("Let's go");
  });

  it("dispatches navigate event when Let's go is clicked", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const button = el.shadowRoot!.querySelector(
      ".lets-go-button"
    ) as HTMLButtonElement;

    setTimeout(() => button.click());
    const event = await oneEvent(el, "navigate");

    expect(event).to.exist;
    expect((event as CustomEvent).detail.view).to.equal("path-selection");
  });

  it("renders the learn more link", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const learnMore = el.shadowRoot!.querySelector(".learn-more");
    expect(learnMore).to.exist;
    expect(learnMore!.getAttribute("href")).to.include("home-assistant.io");
    expect(learnMore!.getAttribute("target")).to.equal("_blank");
  });

  it("renders the OHF logo with light and dark variants", async () => {
    const el = await fixture<WelcomeView>(html`<welcome-view></welcome-view>`);

    const ohfLink = el.shadowRoot!.querySelector(".ohf-link");
    const lightLogo = el.shadowRoot!.querySelector(".ohf-logo-light");
    const darkLogo = el.shadowRoot!.querySelector(".ohf-logo-dark");

    expect(ohfLink).to.exist;
    expect(ohfLink!.getAttribute("href")).to.include("openhomefoundation.org");
    expect(lightLogo).to.exist;
    expect(darkLogo).to.exist;
  });
});
