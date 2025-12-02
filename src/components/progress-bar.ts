import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("progress-bar")
export class ProgressBar extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
    }

    .progress-container {
      width: 100%;
      height: 8px;
      background-color: var(--ha-border-color, #e0e0e0);
      border-radius: 4px;
      overflow: hidden;
    }

    @media (prefers-color-scheme: dark) {
      .progress-container {
        background-color: var(--ha-border-color, #333333);
      }
    }

    .progress-fill {
      height: 100%;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 4px;
      transition: width 0.3s ease-out;
    }

    .progress-fill.error {
      background-color: var(--ha-error-color, #db4437);
    }

    .progress-fill.indeterminate {
      width: 30% !important;
      animation: indeterminate 1.5s ease-in-out infinite;
    }

    @keyframes indeterminate {
      0% {
        transform: translateX(-100%);
      }
      100% {
        transform: translateX(400%);
      }
    }
  `;

  @property({ type: Number })
  progress = 0;

  @property({ type: Boolean })
  indeterminate = false;

  @property({ type: Boolean })
  error = false;

  render() {
    const width = this.indeterminate ? 30 : Math.min(100, Math.max(0, this.progress));

    return html`
      <div class="progress-container">
        <div
          class="progress-fill ${this.error ? "error" : ""} ${this.indeterminate ? "indeterminate" : ""}"
          style="width: ${width}%"
        ></div>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "progress-bar": ProgressBar;
  }
}
