import { expect, fixture, html, oneEvent } from "@open-wc/testing";
import "../../../src/components/wizard-shell.js";
import type { WizardShell } from "../../../src/components/wizard-shell.js";
import { wizardState } from "../../../src/state/wizard-state.js";

describe("wizard-shell", () => {
  beforeEach(() => {
    wizardState.reset();
    wizardState.startFlow("sbc");
  });

  afterEach(() => {
    wizardState.reset();
  });

  it("renders the header with back button", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const backButton = el.shadowRoot!.querySelector(".back-button");
    expect(backButton).to.exist;
    expect(backButton!.textContent).to.include("Back");
  });

  it("renders the step indicator", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const stepIndicator = el.shadowRoot!.querySelector("step-indicator");
    expect(stepIndicator).to.exist;
  });

  it("renders the cancel button", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const cancelButton = el.shadowRoot!.querySelector(".cancel-button");
    expect(cancelButton).to.exist;
    expect(cancelButton!.textContent).to.include("Cancel");
  });

  it("renders slotted content", async () => {
    const el = await fixture<WizardShell>(html`
      <wizard-shell>
        <div id="test-content">Test Content</div>
      </wizard-shell>
    `);

    const slot = el.shadowRoot!.querySelector("slot");
    expect(slot).to.exist;
  });

  it("renders the footer with next button", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const nextButton = el.shadowRoot!.querySelector(".footer-button.primary");
    expect(nextButton).to.exist;
    expect(nextButton!.textContent).to.include("Next");
  });

  it("uses custom next label", async () => {
    const el = await fixture<WizardShell>(html`
      <wizard-shell nextLabel="Continue"></wizard-shell>
    `);

    const nextButton = el.shadowRoot!.querySelector(".footer-button.primary");
    expect(nextButton!.textContent).to.include("Continue");
  });

  it("disables back button on first step", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const backButton = el.shadowRoot!.querySelector(
      ".back-button"
    ) as HTMLButtonElement;
    expect(backButton.disabled).to.be.true;
  });

  it("enables back button after first step", async () => {
    wizardState.nextStep();

    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const backButton = el.shadowRoot!.querySelector(
      ".back-button"
    ) as HTMLButtonElement;
    expect(backButton.disabled).to.be.false;
  });

  it("dispatches wizard-cancel event", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const cancelButton = el.shadowRoot!.querySelector(
      ".cancel-button"
    ) as HTMLButtonElement;

    setTimeout(() => cancelButton.click());
    const event = await oneEvent(el, "wizard-cancel");
    expect(event).to.exist;
  });

  it("dispatches wizard-next event", async () => {
    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const nextButton = el.shadowRoot!.querySelector(
      ".footer-button.primary"
    ) as HTMLButtonElement;

    setTimeout(() => nextButton.click());
    const event = await oneEvent(el, "wizard-next");
    expect(event).to.exist;
  });

  it("dispatches wizard-back event and goes to previous step", async () => {
    wizardState.nextStep(); // Go to step 2

    const el = await fixture<WizardShell>(html`<wizard-shell></wizard-shell>`);

    const backButton = el.shadowRoot!.querySelector(
      ".back-button"
    ) as HTMLButtonElement;

    setTimeout(() => backButton.click());
    const event = await oneEvent(el, "wizard-back");
    expect(event).to.exist;
    expect(wizardState.getState().currentStepIndex).to.equal(0);
  });

  it("hides footer when hideFooter is true", async () => {
    const el = await fixture<WizardShell>(html`
      <wizard-shell hideFooter></wizard-shell>
    `);

    const footer = el.shadowRoot!.querySelector(".footer");
    expect(footer).to.be.null;
  });

  it("hides next button when hideNext is true", async () => {
    const el = await fixture<WizardShell>(html`
      <wizard-shell hideNext></wizard-shell>
    `);

    const nextButton = el.shadowRoot!.querySelector(".footer-button.primary");
    expect(nextButton).to.be.null;
  });

  it("disables next button when nextDisabled is true", async () => {
    const el = await fixture<WizardShell>(html`
      <wizard-shell nextDisabled></wizard-shell>
    `);

    const nextButton = el.shadowRoot!.querySelector(
      ".footer-button.primary"
    ) as HTMLButtonElement;
    expect(nextButton.disabled).to.be.true;
  });
});
