import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";
import type { WizardStep } from "../state/wizard-state.js";

@customElement("step-indicator")
export class StepIndicator extends LitElement {
  static styles = css`
    :host {
      display: block;
    }

    .steps {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
    }

    .step {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }

    .step-dot {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background-color: var(--ha-border-color, #e0e0e0);
      transition: all 0.2s ease;
    }

    .step-dot.active {
      background-color: var(--ha-primary-color, #03a9f4);
      transform: scale(1.2);
    }

    .step-dot.completed {
      background-color: var(--ha-primary-color, #03a9f4);
    }

    .step-label {
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
      transition: color 0.2s ease;
    }

    .step-label.active {
      color: var(--ha-text-color, #212121);
      font-weight: 500;
    }

    .step-connector {
      width: 24px;
      height: 2px;
      background-color: var(--ha-border-color, #e0e0e0);
      transition: background-color 0.2s ease;
    }

    .step-connector.completed {
      background-color: var(--ha-primary-color, #03a9f4);
    }

    @media (prefers-color-scheme: dark) {
      .step-dot {
        background-color: var(--ha-border-color, #444444);
      }

      .step-connector {
        background-color: var(--ha-border-color, #444444);
      }
    }

    /* Compact mode - only show dots */
    :host([compact]) .step-label {
      display: none;
    }

    :host([compact]) .step-connector {
      width: 16px;
    }
  `;

  @property({ type: Array })
  steps: WizardStep[] = [];

  @property({ type: Number })
  currentIndex = 0;

  render() {
    return html`
      <div class="steps">
        ${this.steps.map((step, index) => this._renderStep(step, index))}
      </div>
    `;
  }

  private _renderStep(step: WizardStep, index: number) {
    const isActive = index === this.currentIndex;
    const isCompleted = index < this.currentIndex;
    const isLast = index === this.steps.length - 1;

    return html`
      <div class="step">
        <span
          class="step-dot ${isActive ? "active" : ""} ${isCompleted ? "completed" : ""}"
        ></span>
        <span
          class="step-label ${isActive ? "active" : ""}"
        >
          ${step.title}
        </span>
      </div>
      ${!isLast
        ? html`<span
            class="step-connector ${isCompleted ? "completed" : ""}"
          ></span>`
        : ""}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "step-indicator": StepIndicator;
  }
}
