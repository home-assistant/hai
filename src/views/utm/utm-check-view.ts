import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { checkUtmStatus } from "../../api/commands.js";
import type { UtmStatus } from "../../api/types.js";
import { openUrl } from "@tauri-apps/plugin-opener";
import { wizardState } from "../../state/wizard-state.js";

@customElement("utm-check-view")
export class UtmCheckView extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      height: 100%;
    }

    h2 {
      font-size: 1.5rem;
      font-weight: 400;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.5rem 0;
      text-align: center;
    }

    .subtitle {
      font-size: 1rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0 0 1rem 0;
      text-align: center;
    }

    .status-card {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 1rem;
      padding: 1.25rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      width: 100%;
      max-width: 500px;
    }

    @media (prefers-color-scheme: dark) {
      .status-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }
    }

    .utm-logo {
      width: 56px;
      height: 56px;
    }

    .status-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }

    .status-icon {
      width: 28px;
      height: 28px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }

    .status-icon.loading {
      width: 24px;
      height: 24px;
      background: none;
    }

    .status-icon.success {
      background-color: #4caf50;
    }

    .status-icon.warning {
      background-color: #ff9800;
    }

    .status-icon svg {
      width: 16px;
      height: 16px;
      fill: white;
    }

    .spinner {
      width: 24px;
      height: 24px;
      border: 2px solid var(--ha-secondary-text-color, #727272);
      border-top-color: transparent;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }

    .status-text {
      display: flex;
      flex-direction: column;
    }

    .status-title {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0;
    }

    .status-description {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0.25rem 0 0 0;
    }

    .version-info {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      margin: 0;
    }

    .download-button {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.75rem 1.5rem;
      font-size: 1rem;
      font-weight: 500;
      color: white;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .download-button:hover {
      background-color: var(--ha-primary-color-dark, #0288d1);
    }

    .download-button svg {
      width: 20px;
      height: 20px;
      fill: currentColor;
    }

    .refresh-button {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
      color: var(--ha-primary-color, #03a9f4);
      background: none;
      border: 1px solid var(--ha-primary-color, #03a9f4);
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .refresh-button:hover {
      background-color: rgba(3, 169, 244, 0.1);
    }

    .refresh-button svg {
      width: 16px;
      height: 16px;
      fill: currentColor;
    }

    .warning-card {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      padding: 1rem 1.25rem;
      background-color: rgba(255, 152, 0, 0.1);
      border: 1px solid rgba(255, 152, 0, 0.3);
      border-radius: 12px;
      width: 100%;
      max-width: 500px;
      margin-bottom: 1rem;
    }

    @media (prefers-color-scheme: dark) {
      .warning-card {
        background-color: rgba(255, 152, 0, 0.15);
        border-color: rgba(255, 152, 0, 0.4);
      }
    }

    .warning-title {
      font-size: 0.875rem;
      font-weight: 500;
      color: #e65100;
      margin: 0;
    }

    @media (prefers-color-scheme: dark) {
      .warning-title {
        color: #ffb74d;
      }
    }

    .warning-description {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0;
    }

    .warning-list {
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
    }

    .warning-list li {
      padding-left: 1rem;
      position: relative;
      line-height: 1.4;
    }

    .warning-list li::before {
      content: "•";
      position: absolute;
      left: 0;
      color: #ff9800;
    }
  `;

  @state()
  private _loading = true;

  @state()
  private _utmStatus: UtmStatus | null = null;

  @state()
  private _error: string | null = null;

  connectedCallback() {
    super.connectedCallback();
    this._checkStatus();
  }

  private async _checkStatus() {
    this._loading = true;
    this._error = null;

    try {
      const status = await checkUtmStatus();
      this._utmStatus = status;

      // Store UTM installed status in wizard state
      wizardState.setSelection("utmInstalled", status.installed);
    } catch (error) {
      this._error =
        error instanceof Error ? error.message : "Failed to check UTM status";
      wizardState.setSelection("utmInstalled", false);
    } finally {
      this._loading = false;
    }
  }

  render() {
    return html`
      <h2>Virtual machine setup</h2>
      <p class="subtitle">Run Home Assistant in a virtual machine using UTM</p>

      <div class="warning-card">
        <p class="warning-title">Best for testing & evaluation</p>
        <p class="warning-description">
          A virtual machine in UTM is great for trying Home Assistant out, but
          maybe not the best solution to run your actual smart home on.
        </p>
        <ul class="warning-list">
          <li>Your Mac needs to be running and you need to be logged in</li>
          <li>The virtual machine won't start automatically on boot</li>
          <li>
            For always-on Home Assistant, dedicated hardware is recommended
          </li>
        </ul>
      </div>

      <div class="status-card">
        ${this._renderUtmLogo()}
        ${this._loading
          ? this._renderLoading()
          : this._error
            ? this._renderError()
            : this._utmStatus?.installed
              ? this._renderInstalled()
              : this._renderNotInstalled()}
      </div>
    `;
  }

  private _renderUtmLogo() {
    return html`<img class="utm-logo" src="/assets/icons/utm.svg" alt="UTM" />`;
  }

  private _renderLoading() {
    return html`
      <div class="status-row">
        <div class="status-icon loading">
          <div class="spinner"></div>
        </div>
        <div class="status-text">
          <p class="status-title">Checking for UTM...</p>
        </div>
      </div>
    `;
  }

  private _renderError() {
    return html`
      <div class="status-row">
        <div class="status-icon warning">
          <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
            />
          </svg>
        </div>
        <div class="status-text">
          <p class="status-title">Error checking UTM</p>
          <p class="status-description">${this._error}</p>
        </div>
      </div>
      <button class="refresh-button" @click=${this._checkStatus}>
        ${this._renderRefreshIcon()} Try again
      </button>
    `;
  }

  private _renderInstalled() {
    return html`
      <div class="status-row">
        <div class="status-icon success">
          <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
        <div class="status-text">
          <p class="status-title">
            UTM is
            installed${this._utmStatus?.version
              ? html` <span class="version-info"
                  >(v${this._utmStatus.version})</span
                >`
              : ""}
          </p>
          <p class="status-description">
            Ready to create a Home Assistant virtual machine
          </p>
        </div>
      </div>
    `;
  }

  private _renderNotInstalled() {
    return html`
      <div class="status-row">
        <div class="status-icon warning">
          <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
            />
          </svg>
        </div>
        <div class="status-text">
          <p class="status-title">UTM is not installed</p>
          <p class="status-description">
            Download and install UTM to continue. UTM is a free, open-source
            virtualization app for macOS.
          </p>
        </div>
      </div>
      <button class="download-button" @click=${this._openUtmDownload}>
        ${this._renderDownloadIcon()} Download UTM
      </button>
      <button class="refresh-button" @click=${this._checkStatus}>
        ${this._renderRefreshIcon()} I've installed UTM
      </button>
    `;
  }

  private _renderDownloadIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z" />
      </svg>
    `;
  }

  private _renderRefreshIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"
        />
      </svg>
    `;
  }

  private async _openUtmDownload() {
    try {
      await openUrl("https://mac.getutm.app/");
    } catch {
      // Fallback for browser-only mode
      window.open("https://mac.getutm.app/", "_blank");
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "utm-check-view": UtmCheckView;
  }
}
