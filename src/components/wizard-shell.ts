import { LitElement, html, css } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import {
  wizardState,
  type WizardState,
  type WizardStep,
} from "../state/wizard-state.js";
import "./step-indicator.js";

@customElement("wizard-shell")
export class WizardShell extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
    }

    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 1rem 2rem;
      border-bottom: 1px solid var(--ha-border-color, #e0e0e0);
    }

    @media (prefers-color-scheme: dark) {
      .header {
        border-bottom-color: var(--ha-border-color, #333333);
      }
    }

    .back-button {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
      font-size: 1rem;
      color: var(--ha-secondary-text-color, #727272);
      background: none;
      border: none;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
      min-width: 80px;
    }

    .back-button:hover:not(:disabled) {
      background-color: rgba(0, 0, 0, 0.05);
    }

    .back-button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    @media (prefers-color-scheme: dark) {
      .back-button:hover:not(:disabled) {
        background-color: rgba(255, 255, 255, 0.1);
      }
    }

    .back-arrow {
      font-size: 1.25rem;
    }

    .header-center {
      flex: 1;
      display: flex;
      justify-content: center;
    }

    .header-right {
      min-width: 80px;
    }

    .content {
      flex: 1;
      overflow-y: auto;
      padding: 2rem;
    }

    .footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1rem 2rem;
      border-top: 1px solid var(--ha-border-color, #e0e0e0);
    }

    .footer-left {
      display: flex;
      gap: 1rem;
    }

    .footer-right {
      display: flex;
      gap: 1rem;
    }

    .cancel-button {
      padding: 0.75rem 1.5rem;
      font-size: 1rem;
      color: var(--ha-secondary-text-color, #727272);
      background: none;
      border: none;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .cancel-button:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }

    @media (prefers-color-scheme: dark) {
      .cancel-button:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }
    }

    @media (prefers-color-scheme: dark) {
      .footer {
        border-top-color: var(--ha-border-color, #333333);
      }
    }

    .footer-button {
      padding: 0.75rem 1.5rem;
      font-size: 1rem;
      font-weight: 500;
      border-radius: 8px;
      cursor: pointer;
      transition:
        background-color 0.2s ease,
        transform 0.1s ease;
    }

    .footer-button:active {
      transform: scale(0.98);
    }

    .footer-button.secondary {
      color: var(--ha-secondary-text-color, #727272);
      background: none;
      border: 1px solid var(--ha-border-color, #e0e0e0);
    }

    .footer-button.secondary:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }

    @media (prefers-color-scheme: dark) {
      .footer-button.secondary {
        border-color: var(--ha-border-color, #444444);
      }

      .footer-button.secondary:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }
    }

    .footer-button.primary {
      color: white;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
    }

    .footer-button.primary:hover {
      background-color: var(--ha-primary-color-dark, #0288d1);
    }

    .footer-button.primary:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  `;

  @state()
  private _wizardState: WizardState = wizardState.getState();

  @property({ type: String })
  nextLabel = "Next";

  @property({ type: Boolean })
  nextDisabled = false;

  @property({ type: Boolean })
  hideBack = false;

  @property({ type: Boolean })
  hideNext = false;

  @property({ type: Boolean })
  hideFooter = false;

  private _unsubscribe?: () => void;

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  get steps(): WizardStep[] {
    return this._wizardState.steps;
  }

  get currentIndex(): number {
    return this._wizardState.currentStepIndex;
  }

  get isFirstStep(): boolean {
    return wizardState.isFirstStep;
  }

  get isLastStep(): boolean {
    return wizardState.isLastStep;
  }

  render() {
    return html`
      <div class="header">
        <button
          class="back-button"
          @click=${this._onBack}
          ?disabled=${this.isFirstStep || this.hideBack}
          style=${this.hideBack ? "visibility: hidden" : ""}
        >
          <span class="back-arrow">←</span> Back
        </button>

        <div class="header-center">
          <step-indicator
            .steps=${this.steps}
            .currentIndex=${this.currentIndex}
          ></step-indicator>
        </div>

        <div class="header-right"></div>
      </div>

      <div class="content">
        <slot></slot>
      </div>

      ${!this.hideFooter
        ? html`
            <div class="footer">
              <div class="footer-left">
                <button class="cancel-button" @click=${this._onCancel}>
                  Cancel
                </button>
              </div>
              <div class="footer-right">
                ${!this.hideNext
                  ? html`
                      <button
                        class="footer-button primary"
                        @click=${this._onNext}
                        ?disabled=${this.nextDisabled}
                      >
                        ${this.nextLabel}
                      </button>
                    `
                  : ""}
              </div>
            </div>
          `
        : ""}
    `;
  }

  private _onBack() {
    if (!this.isFirstStep) {
      wizardState.previousStep();
      this.dispatchEvent(
        new CustomEvent("wizard-back", {
          bubbles: true,
          composed: true,
        })
      );
    }
  }

  private _onNext() {
    this.dispatchEvent(
      new CustomEvent("wizard-next", {
        bubbles: true,
        composed: true,
      })
    );
  }

  private _onCancel() {
    this.dispatchEvent(
      new CustomEvent("wizard-cancel", {
        bubbles: true,
        composed: true,
      })
    );
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wizard-shell": WizardShell;
  }
}
