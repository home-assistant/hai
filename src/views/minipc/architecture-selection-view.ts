import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { getManifest, type Device } from "../../api/index.js";
import { wizardState } from "../../state/wizard-state.js";

@customElement("minipc-architecture-selection-view")
export class MiniPCArchitectureSelectionView extends LitElement {
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
      max-width: 500px;
    }

    .options {
      display: flex;
      flex-direction: row;
      gap: 1rem;
      width: 100%;
      max-width: 700px;
    }

    .option-card {
      position: relative;
      display: flex;
      flex-direction: column;
      align-items: center;
      text-align: center;
      flex: 1;
      gap: 0.75rem;
      padding: 1.5rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 2px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      cursor: pointer;
      transition:
        border-color 0.2s ease,
        box-shadow 0.2s ease;
    }

    .option-card:hover {
      border-color: var(--ha-primary-color, #03a9f4);
      box-shadow: 0 2px 8px rgba(3, 169, 244, 0.15);
    }

    .option-card.selected {
      border-color: var(--ha-primary-color, #03a9f4);
      background-color: rgba(3, 169, 244, 0.05);
    }

    @media (prefers-color-scheme: dark) {
      .option-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }

      .option-card:hover {
        box-shadow: 0 2px 8px rgba(3, 169, 244, 0.25);
      }

      .option-card.selected {
        background-color: rgba(3, 169, 244, 0.1);
      }
    }

    .option-icon {
      width: 48px;
      height: 48px;
    }

    .option-icon svg {
      width: 100%;
      height: 100%;
      fill: var(--ha-primary-color, #03a9f4);
    }

    .option-content {
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .option-title {
      font-size: 1.125rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.25rem 0;
    }

    .option-description {
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0 0 0.75rem 0;
      line-height: 1.4;
    }

    .option-examples {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      margin: 0;
      line-height: 1.4;
    }

    .option-check {
      position: absolute;
      top: 8px;
      right: 8px;
      width: 24px;
      height: 24px;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
      font-size: 14px;
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
      width: 48px;
      height: 48px;
      margin-bottom: 1rem;
    }

    .error-icon svg {
      width: 100%;
      height: 100%;
      fill: var(--ha-error-color, #db4437);
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
  `;

  @state()
  private _x86Device: Device | null = null;

  @state()
  private _arm64Device: Device | null = null;

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
      // Find the generic x86-64 and ARM64 devices
      this._x86Device =
        manifest.devices.find((d) => d.category === "generic_x86") || null;
      this._arm64Device =
        manifest.devices.find((d) => d.category === "generic_arm64") || null;
    } catch (err) {
      this._error =
        err instanceof Error ? err.message : "Failed to load architectures";
    } finally {
      this._loading = false;
    }
  }

  render() {
    if (this._loading) {
      return html`
        <div class="loading">
          <div class="loading-spinner"></div>
          <span>Loading...</span>
        </div>
      `;
    }

    if (this._error) {
      return html`
        <div class="error">
          <span class="error-icon">
            <svg viewBox="0 0 24 24">
              <path d="M13,14H11V10H13M13,18H11V16H13M1,21H23L12,2L1,21Z" />
            </svg>
          </span>
          <p class="error-message">${this._error}</p>
          <button class="retry-button" @click=${this._loadDevices}>
            Try Again
          </button>
        </div>
      `;
    }

    return html`
      <h2>Select your architecture</h2>
      <p class="subtitle">
        Choose the CPU architecture of your mini PC.
      </p>

      <div class="options">
        ${this._x86Device
          ? html`
              <div
                class="option-card ${this._selectedDeviceId ===
                this._x86Device.id
                  ? "selected"
                  : ""}"
                @click=${() => this._onSelectDevice(this._x86Device!)}
              >
                <div class="option-icon">
                  <svg viewBox="0 0 24 24">
                    <path
                      d="M20.42 7.345v9.18h1.651v-9.18zM0 7.475v1.737h1.737V7.474zm9.78.352v6.053c0 .513.044.945.13 1.292.087.34.235.618.44.828.203.21.475.359.803.451.334.093.754.136 1.255.136h.216v-1.533c-.24 0-.445-.012-.593-.037a.672.672 0 0 1-.39-.173.693.693 0 0 1-.173-.377 4.002 4.002 0 0 1-.037-.606v-2.182h1.193v-1.416h-1.193V7.827zm-3.505 2.312c-.396 0-.76.08-1.082.241-.327.161-.6.384-.822.668l-.087.117v-.902H2.658v6.256h1.639v-3.214c.018-.588.16-1.02.433-1.299.29-.297.642-.445 1.044-.445.476 0 .841.149 1.082.433.235.284.359.686.359 1.2v3.324h1.663V12.97c.006-.89-.229-1.595-.686-2.09-.458-.495-1.1-.742-1.917-.742zm10.065.006a3.252 3.252 0 0 0-2.306.946c-.29.29-.525.637-.692 1.033a3.145 3.145 0 0 0-.254 1.273c0 .452.08.878.241 1.274.161.395.39.742.674 1.032.284.29.637.526 1.045.693.408.173.86.26 1.342.26 1.397 0 2.262-.637 2.782-1.23l-1.187-.904c-.248.297-.841.699-1.583.699-.464 0-.847-.105-1.138-.321a1.588 1.588 0 0 1-.593-.872l-.019-.056h4.915v-.587c0-.451-.08-.872-.235-1.267a3.393 3.393 0 0 0-.661-1.033 3.013 3.013 0 0 0-1.02-.692 3.345 3.345 0 0 0-1.311-.248zm-16.297.118v6.256h1.651v-6.256zm16.278 1.286c1.132 0 1.664.797 1.664 1.255l-3.32.006c0-.458.525-1.255 1.656-1.261zm7.073 3.814a.606.606 0 0 0-.606.606.606.606 0 0 0 .606.606.606.606 0 0 0 .606-.606.606.606 0 0 0-.606-.606zm-.008.105a.5.5 0 0 1 .002 0 .5.5 0 0 1 .5.501.5.5 0 0 1-.5.5.5.5 0 0 1-.5-.5.5.5 0 0 1 .498-.5zm-.233.155v.699h.13v-.285h.093l.173.285h.136l-.18-.297a.191.191 0 0 0 .118-.056c.03-.03.05-.074.05-.136 0-.068-.02-.117-.063-.154-.037-.038-.105-.056-.185-.056zm.13.099h.154c.019 0 .037.006.056.012a.064.064 0 0 1 .037.031c.013.013.012.031.012.056a.124.124 0 0 1-.012.055.164.164 0 0 1-.037.031c-.019.006-.037.013-.056.013h-.154Z"
                    />
                  </svg>
                </div>
                <div class="option-content">
                  <p class="option-title">Intel/AMD (x86-64)</p>
                  <p class="option-description">
                    Standard PC architecture used by most mini PCs, NUCs, and
                    desktops
                  </p>
                  <p class="option-examples">
                    Examples: Intel NUC, ASUS NUC, Beelink, GMKtec, Minisforum,
                    Dell, HP
                  </p>
                </div>
                ${this._selectedDeviceId === this._x86Device.id
                  ? html`<span class="option-check">✓</span>`
                  : ""}
              </div>
            `
          : ""}
        ${this._arm64Device
          ? html`
              <div
                class="option-card ${this._selectedDeviceId ===
                this._arm64Device.id
                  ? "selected"
                  : ""}"
                @click=${() => this._onSelectDevice(this._arm64Device!)}
              >
                <div class="option-icon">
                  <svg viewBox="0 0 24 24">
                    <path
                      d="M5.419 8.534h1.614v6.911H5.419v-.72c-.71.822-1.573.933-2.07.933C1.218 15.658 0 13.882 0 11.985c0-2.253 1.542-3.633 3.37-3.633.507 0 1.4.132 2.049.984zm-3.765 3.491c0 1.198.751 2.202 1.918 2.202 1.015 0 1.959-.74 1.959-2.181 0-1.512-.934-2.233-1.959-2.233-1.167-.01-1.918.974-1.918 2.212zm7.297-3.49h1.613v.618a3 3 0 0 1 .67-.578c.314-.183.619-.233.984-.233.396 0 .822.06 1.269.324l-.66 1.462a1.432 1.432 0 0 0-.822-.244c-.345 0-.69.05-1.005.376-.446.477-.446 1.136-.446 1.593v3.582H8.94zm5.56 0h1.614v.639c.538-.66 1.177-.822 1.705-.822.72 0 1.4.345 1.786 1.015.579-.822 1.441-1.015 2.05-1.015.842 0 1.573.396 1.969 1.086.132.233.365.74.365 1.745v4.272h-1.614V11.65c0-.771-.08-1.086-.152-1.228-.101-.264-.345-.609-.923-.609-.396 0-.741.213-.954.508-.284.395-.315.984-.315 1.572v3.562H18.43V11.65c0-.771-.081-1.086-.152-1.228-.102-.264-.345-.609-.924-.609-.396 0-.74.213-.954.508-.284.395-.314.984-.314 1.572v3.562h-1.573"
                    />
                  </svg>
                </div>
                <div class="option-content">
                  <p class="option-title">ARM (aarch64)</p>
                  <p class="option-description">
                    ARM-based architecture used by some newer mini PCs and
                    single-board computers
                  </p>
                  <p class="option-examples">
                    Examples: Apple Silicon Mac mini, Ampere-based systems
                  </p>
                </div>
                ${this._selectedDeviceId === this._arm64Device.id
                  ? html`<span class="option-check">✓</span>`
                  : ""}
              </div>
            `
          : ""}
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
    "minipc-architecture-selection-view": MiniPCArchitectureSelectionView;
  }
}
