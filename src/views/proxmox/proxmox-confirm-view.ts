import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { wizardState, type WizardState } from "../../state/wizard-state.js";
import { getHaosRelease } from "../../api/commands.js";

@customElement("proxmox-confirm-view")
export class ProxmoxConfirmView extends LitElement {
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

    .icon-container svg {
      width: 36px;
      height: 36px;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .ha-icon {
      width: 48px;
      height: 48px;
    }

    .proxmox-icon {
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
    this._loadInfo();
  }

  private async _loadInfo() {
    try {
      const release = await getHaosRelease();
      this._haosVersion = release.version;
    } catch (error) {
      console.error("Failed to load info:", error);
      this._haosVersion = "Unknown";
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  render() {
    const selections = this._wizardState.selections;
    const vmName = (selections.vmName as string) || "home-assistant";
    const vmId = (selections.proxmoxVmId as number) || 100;
    const node = (selections.proxmoxNode as string) || "pve";
    const storage = (selections.proxmoxStorage as string) || "local";
    const cpuCores = (selections.cpuCores as number) || 4;
    const memoryMb = (selections.memoryMb as number) || 4096;
    const diskSizeGb = (selections.diskSizeGb as number) || 32;

    return html`
      <h2>Ready to install</h2>
      <p class="subtitle">
        Review your virtual machine configuration before installing
      </p>

      <div class="summary-card">
        <!-- Proxmox Server -->
        <div class="summary-row">
          <div class="icon-container">${this._renderProxmoxIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Proxmox server</p>
            <p class="summary-value">Node: ${node}</p>
            <p class="summary-detail">Storage: ${storage}</p>
          </div>
        </div>

        <div class="divider"></div>

        <!-- VM Configuration -->
        <div class="summary-row">
          <div class="icon-container">${this._renderVmIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Virtual machine</p>
            <p class="summary-value">${vmName} (ID: ${vmId})</p>
            <p class="summary-detail">
              ${cpuCores} CPU cores, ${this._formatMemory(memoryMb)}
            </p>
          </div>
        </div>

        <div class="divider"></div>

        <!-- Disk Size -->
        <div class="summary-row">
          <div class="icon-container">${this._renderDiskIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Storage</p>
            <p class="summary-value">${this._formatDiskSize(diskSizeGb)}</p>
            <p class="summary-detail">Virtual disk for Home Assistant data</p>
          </div>
        </div>

        <div class="divider"></div>

        <!-- HAOS Version -->
        <div class="summary-row">
          <div class="icon-container">${this._renderHaIcon()}</div>
          <div class="summary-info">
            <p class="summary-label">Home Assistant Operating System</p>
            <p class="summary-value">
              ${this._haosVersion
                ? `Version ${this._haosVersion}`
                : "Loading..."}
            </p>
            <p class="summary-detail">Latest stable release</p>
          </div>
        </div>
      </div>
    `;
  }

  private _formatMemory(mb: number): string {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(0)} GB RAM`;
    }
    return `${mb} MB RAM`;
  }

  private _formatDiskSize(gb: number): string {
    if (gb >= 1024) {
      return `${gb / 1024} TB`;
    }
    return `${gb} GB`;
  }

  private _renderProxmoxIcon() {
    return html`
      <img
        class="proxmox-icon"
        src="/assets/icons/proxmox-placeholder.svg"
        alt="Proxmox"
      />
    `;
  }

  private _renderVmIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M21,16H3V4H21M21,2H3C1.89,2 1,2.89 1,4V16A2,2 0 0,0 3,18H10V20H8V22H16V20H14V18H21A2,2 0 0,0 23,16V4C23,2.89 22.1,2 21,2Z"
        />
      </svg>
    `;
  }

  private _renderDiskIcon() {
    return html`
      <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M6,2H18A2,2 0 0,1 20,4V20A2,2 0 0,1 18,22H6A2,2 0 0,1 4,20V4A2,2 0 0,1 6,2M12,4A6,6 0 0,0 6,10C6,13.31 8.69,16 12.1,16L11.22,13.77C10.95,13.29 11.11,12.68 11.59,12.4L12.45,11.9C12.93,11.63 13.54,11.79 13.82,12.27L15.74,14.69C17.12,13.59 18,11.9 18,10A6,6 0 0,0 12,4M12,9A1,1 0 0,1 13,10A1,1 0 0,1 12,11A1,1 0 0,1 11,10A1,1 0 0,1 12,9M7,18A1,1 0 0,0 6,19A1,1 0 0,0 7,20A1,1 0 0,0 8,19A1,1 0 0,0 7,18M12.09,13.27L14.58,19.58L17.17,18.08L12.95,12.77L12.09,13.27Z"
        />
      </svg>
    `;
  }

  private _renderHaIcon() {
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
    "proxmox-confirm-view": ProxmoxConfirmView;
  }
}
