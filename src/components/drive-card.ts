import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";
import type { DeviceType } from "../api/types.js";

@customElement("drive-card")
export class DriveCard extends LitElement {
  static styles = css`
    :host {
      display: block;
    }

    .card {
      display: flex;
      align-items: center;
      gap: 1rem;
      padding: 1rem 1.25rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 2px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      cursor: pointer;
      transition:
        border-color 0.2s ease,
        box-shadow 0.2s ease,
        transform 0.1s ease;
    }

    .card:hover {
      border-color: var(--ha-primary-color, #03a9f4);
      box-shadow: 0 4px 12px rgba(3, 169, 244, 0.15);
    }

    .card:active {
      transform: scale(0.99);
    }

    .card.selected {
      border-color: var(--ha-primary-color, #03a9f4);
      box-shadow: 0 0 0 3px rgba(3, 169, 244, 0.2);
    }

    .card.disabled {
      opacity: 0.5;
      cursor: not-allowed;
      pointer-events: none;
    }

    .card.disabled:hover {
      border-color: var(--ha-border-color, #e0e0e0);
      box-shadow: none;
    }

    @media (prefers-color-scheme: dark) {
      .card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }

      .card:hover {
        box-shadow: 0 4px 12px rgba(3, 169, 244, 0.25);
      }

      .card.selected {
        box-shadow: 0 0 0 3px rgba(3, 169, 244, 0.3);
      }
    }

    .icon-container {
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }

    .icon-container svg {
      width: 40px;
      height: 40px;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .card.selected .icon-container svg {
      fill: var(--ha-primary-color, #03a9f4);
    }

    .info {
      flex: 1;
      min-width: 0;
    }

    .name {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.25rem 0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .details {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0;
    }

    .description {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      margin: 0.25rem 0 0 0;
    }

    .size {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      flex-shrink: 0;
    }

    .selected-indicator {
      width: 24px;
      height: 24px;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
      font-size: 14px;
      flex-shrink: 0;
    }
  `;

  @property({ type: String })
  driveId = "";

  @property({ type: String })
  name = "";

  @property({ type: Number })
  size = 0;

  @property({ type: String })
  deviceType: DeviceType = "unknown";

  @property({ type: String })
  model = "";

  @property({ type: String })
  vendor = "";

  @property({ type: Boolean })
  selected = false;

  @property({ type: Boolean })
  disabled = false;

  @property({ type: String })
  disabledReason = "";

  render() {
    const classes = [
      "card",
      this.selected ? "selected" : "",
      this.disabled ? "disabled" : "",
    ]
      .filter(Boolean)
      .join(" ");

    return html`
      <div class="${classes}">
        <div class="icon-container">${this._renderIcon()}</div>
        <div class="info">
          <p class="name">${this.name}</p>
          <p class="details">${this.disabled && this.disabledReason ? this.disabledReason : this._getDetails()}</p>
          ${!this.disabled ? html`<p class="description">${this._getDescription()}</p>` : ""}
        </div>
        <span class="size">${this._formatSize(this.size)}</span>
        ${this.selected
          ? html`<span class="selected-indicator">✓</span>`
          : ""}
      </div>
    `;
  }

  private _getDetails(): string {
    const parts: string[] = [];
    if (this.vendor) parts.push(this.vendor);
    if (this.model) parts.push(this.model);
    if (parts.length === 0) {
      parts.push(this._getTypeLabel());
    }
    return parts.join(" ");
  }

  private _getDescription(): string {
    switch (this.deviceType) {
      case "sd_card":
        return "Great for Raspberry Pi and similar single-board computers";
      case "usb_drive":
        return "Portable and easy to set up";
      case "ssd":
        return "Fast and reliable for daily use";
      case "hdd":
        return "High capacity storage option";
      case "nvme":
        return "Maximum performance storage";
      default:
        return "External storage device";
    }
  }

  private _getTypeLabel(): string {
    switch (this.deviceType) {
      case "sd_card":
        return "SD Card";
      case "usb_drive":
        return "USB Drive";
      case "ssd":
        return "SSD";
      case "hdd":
        return "Hard Drive";
      case "nvme":
        return "NVMe";
      default:
        return "Storage Device";
    }
  }

  private _formatSize(bytes: number): string {
    if (bytes === 0) return "0 GB";
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1000) {
      return `${(gb / 1024).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  }

  private _renderIcon() {
    switch (this.deviceType) {
      case "sd_card":
        return this._renderSdCardIcon();
      case "usb_drive":
        return this._renderUsbIcon();
      case "ssd":
      case "nvme":
        return this._renderSsdIcon();
      case "hdd":
        return this._renderHddIcon();
      default:
        return this._renderGenericIcon();
    }
  }

  private _renderSdCardIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M18 2H10L4 8V20C4 21.1 4.9 22 6 22H18C19.1 22 20 21.1 20 20V4C20 2.9 19.1 2 18 2ZM12 8H10V4H12V8ZM15 8H13V4H15V8ZM18 8H16V4H18V8Z"
        />
      </svg>
    `;
  }

  private _renderUsbIcon() {
    // USB trident symbol
    return html`
      <svg viewBox="0 0 192.756 192.756" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M81.114 37.464l16.415-28.96 16.834 28.751-12.164.077-.174 70.181c.988-.552 2.027-1.09 3.096-1.643 6.932-3.586 15.674-8.11 15.998-28.05h-8.533V53.251h24.568V77.82h-7.611c-.334 25.049-11.627 30.892-20.572 35.519-3.232 1.672-6.012 3.111-6.975 5.68l-.09 36.683a14.503 14.503 0 0 1 10.68 14.02 14.5 14.5 0 0 1-14.533 14.532 14.5 14.5 0 0 1-14.533-14.532 14.504 14.504 0 0 1 9.454-13.628l.057-22.801c-2.873-1.613-5.62-2.704-8.139-3.705-11.142-4.43-18.705-7.441-18.857-33.4a14.381 14.381 0 0 1-10.43-13.869c0-7.946 6.482-14.428 14.428-14.428 7.946 0 14.428 6.482 14.428 14.428 0 6.488-4.21 11.889-10.004 13.74.116 20.396 5.54 22.557 13.528 25.732 1.61.641 3.303 1.312 5.069 2.114l.214-86.517-12.154.076z"
        />
      </svg>
    `;
  }

  private _renderSsdIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M2 6C2 4.9 2.9 4 4 4H20C21.1 4 22 4.9 22 6V18C22 19.1 21.1 20 20 20H4C2.9 20 2 19.1 2 18V6ZM4 6V18H20V6H4ZM6 8H8V10H6V8ZM6 12H8V14H6V12ZM10 8H12V10H10V8ZM10 12H12V14H10V12Z"
        />
      </svg>
    `;
  }

  private _renderHddIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M2 6C2 4.9 2.9 4 4 4H20C21.1 4 22 4.9 22 6V18C22 19.1 21.1 20 20 20H4C2.9 20 2 19.1 2 18V6ZM4 6V18H20V6H4ZM18 15C18 15.55 17.55 16 17 16C16.45 16 16 15.55 16 15C16 14.45 16.45 14 17 14C17.55 14 18 14.45 18 15Z"
        />
      </svg>
    `;
  }

  private _renderGenericIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M2 6C2 4.9 2.9 4 4 4H20C21.1 4 22 4.9 22 6V18C22 19.1 21.1 20 20 20H4C2.9 20 2 19.1 2 18V6ZM4 6V18H20V6H4Z"
        />
      </svg>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "drive-card": DriveCard;
  }
}
