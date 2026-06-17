import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("info-dialog")
export class InfoDialog extends LitElement {
  static styles = css`
    :host {
      display: none;
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 1000;
    }

    :host([open]) {
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .overlay {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: -1;
      animation: fadeIn 0.15s ease-out;
    }

    @keyframes fadeIn {
      from {
        opacity: 0;
      }
      to {
        opacity: 1;
      }
    }

    .dialog {
      background-color: var(--ha-card-background, #ffffff);
      border-radius: 16px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
      max-width: 480px;
      width: 90%;
      animation: slideUp 0.2s ease-out;
    }

    @keyframes slideUp {
      from {
        transform: translateY(20px);
        opacity: 0;
      }
      to {
        transform: translateY(0);
        opacity: 1;
      }
    }

    @media (prefers-color-scheme: dark) {
      .dialog {
        background-color: var(--ha-card-background, #1e1e1e);
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
      }
    }

    .dialog-header {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1.5rem 1.5rem 1rem 1.5rem;
    }

    .info-icon {
      width: 28px;
      height: 28px;
      flex-shrink: 0;
    }

    .info-icon svg {
      width: 100%;
      height: 100%;
      fill: var(--ha-primary-color, #03a9f4);
    }

    .dialog-title {
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--ha-text-color, #212121);
      margin: 0;
    }

    .dialog-content {
      padding: 0 1.5rem 1.5rem 1.5rem;
    }

    .dialog-message {
      font-size: 0.9375rem;
      color: var(--ha-text-color, #212121);
      line-height: 1.6;
      margin: 0;
    }

    .dialog-actions {
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      padding: 1rem 1.5rem;
      border-top: 1px solid var(--ha-border-color, #e0e0e0);
    }

    @media (prefers-color-scheme: dark) {
      .dialog-actions {
        border-top-color: var(--ha-border-color, #333333);
      }
    }

    .dialog-button {
      padding: 0.625rem 1.25rem;
      font-size: 0.9375rem;
      font-weight: 500;
      border-radius: 8px;
      cursor: pointer;
      transition:
        background-color 0.2s ease,
        transform 0.1s ease;
    }

    .dialog-button:active {
      transform: scale(0.98);
    }

    .dialog-button.secondary {
      color: var(--ha-secondary-text-color, #727272);
      background: none;
      border: 1px solid var(--ha-border-color, #e0e0e0);
    }

    .dialog-button.secondary:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }

    @media (prefers-color-scheme: dark) {
      .dialog-button.secondary {
        border-color: var(--ha-border-color, #444444);
      }

      .dialog-button.secondary:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }
    }

    .dialog-button.primary {
      color: white;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
    }

    .dialog-button.primary:hover {
      background-color: var(--ha-primary-color-dark, #0288d1);
    }
  `;

  @property({ type: Boolean, reflect: true })
  open = false;

  @property({ type: String })
  title = "";

  @property({ type: String })
  message = "";

  @property({ type: String })
  primaryLabel = "OK";

  @property({ type: String })
  secondaryLabel = "";

  render() {
    return html`
      <div class="overlay" @click=${this._onOverlayClick}>
        <div class="dialog" @click=${this._onDialogClick}>
          <div class="dialog-header">
            <span class="info-icon">
              <svg viewBox="0 0 24 24">
                <path
                  d="M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z"
                />
              </svg>
            </span>
            <h2 class="dialog-title">${this.title}</h2>
          </div>
          <div class="dialog-content">
            <p class="dialog-message">${this.message}</p>
            <slot></slot>
          </div>
          <div class="dialog-actions">
            ${this.secondaryLabel
              ? html`
                  <button
                    class="dialog-button secondary"
                    @click=${this._onSecondary}
                  >
                    ${this.secondaryLabel}
                  </button>
                `
              : ""}
            <button class="dialog-button primary" @click=${this._onPrimary}>
              ${this.primaryLabel}
            </button>
          </div>
        </div>
      </div>
    `;
  }

  private _onOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      this._onSecondary();
    }
  }

  private _onDialogClick(e: MouseEvent) {
    e.stopPropagation();
  }

  private _onSecondary() {
    this.open = false;
    this.dispatchEvent(
      new CustomEvent("dialog-secondary", {
        bubbles: true,
        composed: true,
      })
    );
  }

  private _onPrimary() {
    this.open = false;
    this.dispatchEvent(
      new CustomEvent("dialog-primary", {
        bubbles: true,
        composed: true,
      })
    );
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "info-dialog": InfoDialog;
  }
}
