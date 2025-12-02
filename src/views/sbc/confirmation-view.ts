import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { wizardState, type WizardState } from "../../state/wizard-state.js";
import type { HaosConfig } from "../../api/types.js";
import { getHaosRelease } from "../../api/commands.js";

@customElement("confirmation-view")
export class ConfirmationView extends LitElement {
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
      margin: 0 0 1.5rem 0;
      text-align: center;
    }

    .summary-card {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
      padding: 1.5rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      width: 100%;
      max-width: 500px;
    }

    @media (prefers-color-scheme: dark) {
      .summary-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }
    }

    .summary-row {
      display: flex;
      align-items: center;
      gap: 1rem;
    }

    .icon-container {
      width: 64px;
      height: 64px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }

    .device-image {
      width: 64px;
      height: 64px;
      object-fit: contain;
    }

    .device-icon {
      width: 36px;
      height: 36px;
    }

    .device-image-placeholder {
      width: 64px;
      height: 64px;
      display: flex;
      align-items: center;
      justify-content: center;
      background-color: var(--ha-border-color, #e0e0e0);
      border-radius: 8px;
    }

    @media (prefers-color-scheme: dark) {
      .device-image-placeholder {
        background-color: var(--ha-border-color, #333333);
      }
    }

    .device-image-placeholder svg {
      width: 36px;
      height: 36px;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .icon-container svg {
      width: 36px;
      height: 36px;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .ha-icon {
      width: 48px;
      height: 48px;
    }

    .summary-info {
      flex: 1;
      min-width: 0;
    }

    .summary-label {
      font-size: 0.75rem;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0 0 0.25rem 0;
    }

    .summary-value {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .summary-detail {
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0.25rem 0 0 0;
    }

    .divider {
      height: 1px;
      background-color: var(--ha-border-color, #e0e0e0);
      margin: 0;
    }

    @media (prefers-color-scheme: dark) {
      .divider {
        background-color: var(--ha-border-color, #333333);
      }
    }
  `;

  @state()
  private _wizardState: WizardState = wizardState.getState();

  @state()
  private _haosVersion: string = "";

  private _unsubscribe?: () => void;

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });
    this._loadHaosVersion();
  }

  private async _loadHaosVersion() {
    try {
      const release = await getHaosRelease();
      this._haosVersion = release.version;
    } catch (error) {
      console.error("Failed to load HAOS version:", error);
      this._haosVersion = "Unknown";
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  render() {
    const selections = this._wizardState.selections;
    const deviceName = (selections.deviceName as string) || "Unknown Device";
    const deviceImage = selections.deviceImage as string | undefined;
    const driveName = (selections.driveName as string) || "Unknown Drive";
    const driveSize = (selections.driveSize as number) || 0;
    const deviceConfig = selections.deviceConfig as HaosConfig | undefined;

    return html`
      <h2>Ready to install</h2>
      <p class="subtitle">Review your selections before installing</p>

      <div class="summary-card">
        <!-- Device -->
        <div class="summary-row">
          <div class="icon-container">
            ${deviceImage
              ? html`<img
                  class="${deviceImage.endsWith('.svg') ? 'device-icon' : 'device-image'}"
                  src="${deviceImage}"
                  alt="${deviceName}"
                />`
              : html`<div class="device-image-placeholder">
                  ${this._renderBoardIcon()}
                </div>`}
          </div>
          <div class="summary-info">
            <p class="summary-label">Device</p>
            <p class="summary-value">${deviceName}</p>
            ${deviceConfig
              ? html`<p class="summary-detail">
                  Board: ${deviceConfig.board}
                </p>`
              : ""}
          </div>
        </div>

        <div class="divider"></div>

        <!-- Drive -->
        <div class="summary-row">
          <div class="icon-container">${this._renderDriveIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Target Drive</p>
            <p class="summary-value">${driveName}</p>
            <p class="summary-detail">${this._formatSize(driveSize)}</p>
          </div>
        </div>

        <div class="divider"></div>

        <!-- HAOS Version -->
        <div class="summary-row">
          <div class="icon-container">${this._renderHaIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Home Assistant Operating System</p>
            <p class="summary-value">
              ${this._haosVersion ? `Version ${this._haosVersion}` : "Loading..."}
            </p>
            <p class="summary-detail">Latest stable release</p>
          </div>
        </div>
      </div>
    `;
  }

  private _formatSize(bytes: number): string {
    if (bytes === 0) return "Unknown size";
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1000) {
      return `${(gb / 1024).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  }

  private _renderBoardIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M6 2C4.9 2 4 2.9 4 4V20C4 21.1 4.9 22 6 22H18C19.1 22 20 21.1 20 20V4C20 2.9 19.1 2 18 2H6ZM6 4H18V20H6V4ZM8 6V8H10V6H8ZM14 6V8H16V6H14ZM11 10V12H13V10H11ZM8 14V16H10V14H8ZM14 14V16H16V14H14Z"
        />
      </svg>
    `;
  }

  private _renderDriveIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M18 2H10L4 8V20C4 21.1 4.9 22 6 22H18C19.1 22 20 21.1 20 20V4C20 2.9 19.1 2 18 2ZM12 8H10V4H12V8ZM15 8H13V4H15V8ZM18 8H16V4H18V8Z"
        />
      </svg>
    `;
  }

  private _renderHaIcon() {
    // Home Assistant icon from the official logo
    return html`
      <svg
        class="ha-icon"
        viewBox="0 0 240 240"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M240 224.813C240 233.063 233.25 239.813 225 239.813H15C6.75 239.813 0 233.063 0 224.813V134.813C0 126.563 4.77 115.043 10.61 109.203L109.39 10.423C115.22 4.59304 124.77 4.59304 130.6 10.423L229.39 109.213C235.22 115.043 240 126.573 240 134.823V224.823V224.813Z"
          fill="#F2F4F9"
        />
        <path
          d="M229.39 109.203L130.61 10.423C124.78 4.59304 115.23 4.59304 109.4 10.423L10.61 109.203C4.78 115.033 0 126.563 0 134.813V224.813C0 233.063 6.75 239.813 15 239.813H107.27L66.64 199.183C64.55 199.903 62.32 200.313 60 200.313C48.7 200.313 39.5 191.113 39.5 179.813C39.5 168.513 48.7 159.313 60 159.313C71.3 159.313 80.5 168.513 80.5 179.813C80.5 182.143 80.09 184.373 79.37 186.463L111 218.093V102.213C104.2 98.873 99.5 91.893 99.5 83.823C99.5 72.523 108.7 63.323 120 63.323C131.3 63.323 140.5 72.523 140.5 83.823C140.5 91.893 135.8 98.873 129 102.213V183.483L160.46 152.023C159.84 150.063 159.5 147.983 159.5 145.823C159.5 134.523 168.7 125.323 180 125.323C191.3 125.323 200.5 134.523 200.5 145.823C200.5 157.123 191.3 166.323 180 166.323C177.5 166.323 175.12 165.853 172.91 165.033L129 208.943V239.823H225C233.25 239.823 240 233.073 240 224.823V134.823C240 126.573 235.23 115.053 229.39 109.213V109.203Z"
          fill="#18BCF2"
        />
      </svg>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "confirmation-view": ConfirmationView;
  }
}
