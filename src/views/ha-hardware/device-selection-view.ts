import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { getManifest, type Device } from "../../api/index.js";
import { wizardState } from "../../state/wizard-state.js";
import "../../components/device-card.js";

@customElement("ha-hardware-device-selection-view")
export class HaHardwareDeviceSelectionView extends LitElement {
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
      margin: 0 0 2rem 0;
      text-align: center;
    }

    .devices-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 1.5rem;
      width: 100%;
      max-width: 700px;
    }

    .loading {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 3rem;
      color: var(--ha-secondary-text-color, #727272);
    }

    .loading-spinner {
      width: 40px;
      height: 40px;
      border: 3px solid var(--ha-border-color, #e0e0e0);
      border-top-color: var(--ha-primary-color, #03a9f4);
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin-bottom: 1rem;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }

    .error {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 2rem;
      text-align: center;
    }

    .error-icon {
      font-size: 3rem;
      margin-bottom: 1rem;
    }

    .error-message {
      color: var(--ha-error-color, #db4437);
      margin-bottom: 1rem;
    }

    .retry-button {
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
      color: var(--ha-primary-color, #03a9f4);
      background: none;
      border: 1px solid var(--ha-primary-color, #03a9f4);
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .retry-button:hover {
      background-color: rgba(3, 169, 244, 0.1);
    }

    .info-box {
      max-width: 600px;
      padding: 1rem;
      margin-top: 2rem;
      background-color: var(--ha-card-background, #f5f5f5);
      border-radius: 8px;
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
      text-align: center;
    }

    @media (prefers-color-scheme: dark) {
      .info-box {
        background-color: rgba(255, 255, 255, 0.05);
      }
    }
  `;

  @state()
  private _devices: Device[] = [];

  @state()
  private _loading = true;

  @state()
  private _error: string | null = null;

  @state()
  private _selectedDeviceId: string | null = null;

  connectedCallback() {
    super.connectedCallback();
    this._loadDevices();

    // Check if there's already a selection in wizard state
    const state = wizardState.getState();
    if (state.selections.device) {
      this._selectedDeviceId = state.selections.device as string;
    }
  }

  private async _loadDevices() {
    this._loading = true;
    this._error = null;

    try {
      const manifest = await getManifest();
      // Filter to only show Home Assistant Hardware devices
      this._devices = manifest.devices.filter(
        (device) => device.category === "home_assistant_hardware"
      );
    } catch (err) {
      this._error =
        err instanceof Error ? err.message : "Failed to load devices";
    } finally {
      this._loading = false;
    }
  }

  render() {
    if (this._loading) {
      return html`
        <div class="loading">
          <div class="loading-spinner"></div>
          <span>Loading devices...</span>
        </div>
      `;
    }

    if (this._error) {
      return html`
        <div class="error">
          <span class="error-icon">⚠️</span>
          <p class="error-message">${this._error}</p>
          <button class="retry-button" @click=${this._loadDevices}>
            Try Again
          </button>
        </div>
      `;
    }

    return html`
      <h2>Select your Home Assistant device</h2>
      <p class="subtitle">
        Choose your official Home Assistant hardware by Nabu Casa
      </p>

      <div class="devices-grid">
        ${this._devices.map(
          (device) => html`
            <device-card
              .deviceId=${device.id}
              .name=${device.name}
              .image=${device.image_url || ""}
              .selected=${this._selectedDeviceId === device.id}
              @click=${() => this._onSelectDevice(device)}
            ></device-card>
          `
        )}
      </div>

      <div class="info-box">
        💡 Connect your device to this computer using a USB cable or adapter.
        You'll flash the storage directly.
      </div>
    `;
  }

  private _onSelectDevice(device: Device) {
    this._selectedDeviceId = device.id;
    wizardState.setSelection("device", device.id);
    wizardState.setSelection("deviceName", device.name);
    wizardState.setSelection("deviceImage", device.image_url || "");
    wizardState.setSelection("deviceConfig", device.haos);

    this.dispatchEvent(
      new CustomEvent("device-selected", {
        detail: { device },
        bubbles: true,
        composed: true,
      })
    );
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ha-hardware-device-selection-view": HaHardwareDeviceSelectionView;
  }
}
