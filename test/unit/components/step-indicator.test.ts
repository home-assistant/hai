import { expect, fixture, html } from "@open-wc/testing";
import "../../../src/components/step-indicator.js";
import type { StepIndicator } from "../../../src/components/step-indicator.js";
import type { WizardStep } from "../../../src/state/wizard-state.js";

describe("step-indicator", () => {
  const mockSteps: WizardStep[] = [
    { id: "step1", title: "First Step" },
    { id: "step2", title: "Second Step" },
    { id: "step3", title: "Third Step" },
  ];

  it("renders all steps", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${0}></step-indicator>
    `);

    const steps = el.shadowRoot!.querySelectorAll(".step");
    expect(steps.length).to.equal(3);
  });

  it("shows step titles", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${0}></step-indicator>
    `);

    const labels = el.shadowRoot!.querySelectorAll(".step-label");
    expect(labels[0].textContent?.trim()).to.equal("First Step");
    expect(labels[1].textContent?.trim()).to.equal("Second Step");
    expect(labels[2].textContent?.trim()).to.equal("Third Step");
  });

  it("marks current step as active", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${1}></step-indicator>
    `);

    const dots = el.shadowRoot!.querySelectorAll(".step-dot");
    expect(dots[0].classList.contains("completed")).to.be.true;
    expect(dots[1].classList.contains("active")).to.be.true;
    expect(dots[2].classList.contains("active")).to.be.false;
  });

  it("marks previous steps as completed", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${2}></step-indicator>
    `);

    const dots = el.shadowRoot!.querySelectorAll(".step-dot");
    expect(dots[0].classList.contains("completed")).to.be.true;
    expect(dots[1].classList.contains("completed")).to.be.true;
    expect(dots[2].classList.contains("active")).to.be.true;
  });

  it("renders connectors between steps", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${0}></step-indicator>
    `);

    const connectors = el.shadowRoot!.querySelectorAll(".step-connector");
    expect(connectors.length).to.equal(2); // 3 steps = 2 connectors
  });

  it("marks connectors as completed for passed steps", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${mockSteps} .currentIndex=${2}></step-indicator>
    `);

    const connectors = el.shadowRoot!.querySelectorAll(".step-connector");
    expect(connectors[0].classList.contains("completed")).to.be.true;
    expect(connectors[1].classList.contains("completed")).to.be.true;
  });

  it("handles empty steps array", async () => {
    const el = await fixture<StepIndicator>(html`
      <step-indicator .steps=${[]} .currentIndex=${0}></step-indicator>
    `);

    const steps = el.shadowRoot!.querySelectorAll(".step");
    expect(steps.length).to.equal(0);
  });
});
