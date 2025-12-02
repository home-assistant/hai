import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { wizardState, type WizardState } from "../../state/wizard-state.js";
import {
  proxmoxListNodes,
  proxmoxListStorage,
  proxmoxGetNextVmId,
  formatBytes,
} from "../../api/commands.js";
import type {
  ProxmoxSession,
  ProxmoxNode,
  ProxmoxStorage,
} from "../../api/types.js";

@customElement("proxmox-configure-view")
export class ProxmoxConfigureView extends LitElement {
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
      margin: 0 0 0.75rem 0;
      text-align: center;
    }

    .config-card {
      display: flex;
      flex-direction: column;
      gap: 1.75rem;
      padding: 1rem 1.25rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      width: 100%;
      max-width: 500px;
    }

    @media (prefers-color-scheme: dark) {
      .config-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }
    }

    .setting-row {
      display: flex;
      align-items: flex-start;
      gap: 0.75rem;
    }

    .setting-icon {
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      margin-top: 2px;
    }

    .setting-icon svg {
      width: 22px;
      height: 22px;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .setting-content {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.3125rem;
      min-width: 0;
    }

    .setting-header {
      display: flex;
      justify-content: space-between;
      align-items: baseline;
    }

    .setting-label {
      font-size: 0.9375rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
    }

    .setting-value {
      font-size: 0.9375rem;
      font-weight: 600;
      color: var(--ha-primary-color, #03a9f4);
      min-width: 70px;
      text-align: right;
    }

    .setting-description {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      margin: 0;
      line-height: 1.3;
    }

    .name-input {
      padding: 0.5rem 0.75rem;
      font-size: 0.875rem;
      color: var(--ha-text-color, #212121);
      background-color: var(--ha-background-color, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 6px;
      outline: none;
      transition: border-color 0.2s ease;
      width: 100%;
      box-sizing: border-box;
    }

    .name-input:focus {
      border-color: var(--ha-primary-color, #03a9f4);
    }

    @media (prefers-color-scheme: dark) {
      .name-input {
        background-color: var(--ha-background-color, #121212);
        border-color: var(--ha-border-color, #333333);
        color: var(--ha-text-color, #e0e0e0);
      }
    }

    /* Select dropdown styles */
    .select-dropdown {
      padding: 0.5rem 0.75rem;
      font-size: 0.875rem;
      color: var(--ha-text-color, #212121);
      background-color: var(--ha-background-color, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 6px;
      outline: none;
      transition: border-color 0.2s ease;
      width: 100%;
      box-sizing: border-box;
      cursor: pointer;
      -webkit-appearance: none;
      -moz-appearance: none;
      appearance: none;
      background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24'%3E%3Cpath fill='%23727272' d='M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6 1.41-1.41z'/%3E%3C/svg%3E");
      background-repeat: no-repeat;
      background-position: right 0.75rem center;
      padding-right: 2rem;
    }

    .select-dropdown:focus {
      border-color: var(--ha-primary-color, #03a9f4);
    }

    .select-dropdown:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    @media (prefers-color-scheme: dark) {
      .select-dropdown {
        background-color: var(--ha-background-color, #121212);
        border-color: var(--ha-border-color, #333333);
        color: var(--ha-text-color, #e0e0e0);
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24'%3E%3Cpath fill='%23e0e0e0' d='M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6 1.41-1.41z'/%3E%3C/svg%3E");
      }
    }

    /* Slider container */
    .slider-container {
      position: relative;
      width: 100%;
      padding-bottom: 4px;
    }

    /* Slider styles */
    input[type="range"] {
      -webkit-appearance: none;
      appearance: none;
      width: 100%;
      height: 6px;
      background: var(--ha-border-color, #e0e0e0);
      border-radius: 3px;
      outline: none;
      cursor: pointer;
    }

    input[type="range"]::-webkit-slider-thumb {
      -webkit-appearance: none;
      appearance: none;
      width: 18px;
      height: 18px;
      background: var(--ha-primary-color, #03a9f4);
      border-radius: 50%;
      cursor: pointer;
      transition: transform 0.1s ease;
    }

    input[type="range"]::-webkit-slider-thumb:hover {
      transform: scale(1.1);
    }

    input[type="range"]::-moz-range-thumb {
      width: 18px;
      height: 18px;
      background: var(--ha-primary-color, #03a9f4);
      border-radius: 50%;
      border: none;
      cursor: pointer;
    }

    @media (prefers-color-scheme: dark) {
      input[type="range"] {
        background: var(--ha-border-color, #333333);
      }
    }

    /* Tick marks */
    .slider-ticks {
      display: flex;
      justify-content: space-between;
      padding: 0 9px;
      margin-top: 4px;
    }

    .slider-tick {
      width: 2px;
      height: 6px;
      background: #c0c0c0;
      border-radius: 1px;
    }

    @media (prefers-color-scheme: dark) {
      .slider-tick {
        background: #555555;
      }
    }

    .loading-text {
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      font-style: italic;
    }

    .error-text {
      font-size: 0.875rem;
      color: var(--ha-error-color, #db4437);
    }
  `;

  @state()
  private _wizardState: WizardState = wizardState.getState();

  @state()
  private _nodes: ProxmoxNode[] = [];

  @state()
  private _storages: ProxmoxStorage[] = [];

  @state()
  private _loadingNodes = true;

  @state()
  private _loadingStorage = false;

  @state()
  private _error: string | null = null;

  @state()
  private _selectedNode = "";

  @state()
  private _selectedStorage = "";

  @state()
  private _vmId = 100;

  @state()
  private _vmName = "home-assistant";

  @state()
  private _cpuCores = 4;

  @state()
  private _memoryMb = 4096;

  @state()
  private _diskSizeGb = 32;

  private _unsubscribe?: () => void;

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });
    this._loadNodes();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  private async _loadNodes() {
    const session = this._wizardState.selections.proxmoxSession as
      | ProxmoxSession
      | undefined;

    if (!session) {
      this._error = "No Proxmox session available";
      this._loadingNodes = false;
      return;
    }

    try {
      const [nodes, nextVmId] = await Promise.all([
        proxmoxListNodes(session),
        proxmoxGetNextVmId(session),
      ]);

      this._nodes = nodes.filter((n) => n.status === "online");
      this._vmId = nextVmId;

      if (this._nodes.length > 0) {
        this._selectedNode = this._nodes[0].name;
        await this._loadStorage();
      }

      this._saveSelections();
    } catch (error) {
      // Tauri invoke errors are strings, not Error objects
      this._error =
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "Failed to load Proxmox nodes";
    } finally {
      this._loadingNodes = false;
    }
  }

  private async _loadStorage() {
    if (!this._selectedNode) return;

    const session = this._wizardState.selections.proxmoxSession as
      | ProxmoxSession
      | undefined;

    if (!session) return;

    this._loadingStorage = true;
    try {
      const storages = await proxmoxListStorage(session, this._selectedNode);
      // Filter to only show storage that supports VM images
      this._storages = storages.filter(
        (s) => s.active && s.content.includes("images")
      );

      if (this._storages.length > 0 && !this._selectedStorage) {
        this._selectedStorage = this._storages[0].name;
      }

      this._saveSelections();
    } catch (error) {
      // Show storage error to user
      this._error =
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "Failed to load storage";
    } finally {
      this._loadingStorage = false;
    }
  }

  private _saveSelections() {
    wizardState.setSelection("proxmoxNode", this._selectedNode);
    wizardState.setSelection("proxmoxStorage", this._selectedStorage);
    wizardState.setSelection("proxmoxVmId", this._vmId);
    wizardState.setSelection("vmName", this._vmName);
    wizardState.setSelection("cpuCores", this._cpuCores);
    wizardState.setSelection("memoryMb", this._memoryMb);
    wizardState.setSelection("diskSizeGb", this._diskSizeGb);
  }

  private async _onNodeChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    this._selectedNode = select.value;
    this._selectedStorage = "";
    await this._loadStorage();
  }

  private _onStorageChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    this._selectedStorage = select.value;
    this._saveSelections();
  }

  private _onVmIdChange(e: Event) {
    const input = e.target as HTMLInputElement;
    this._vmId = parseInt(input.value, 10) || 100;
    this._saveSelections();
  }

  private _onNameChange(e: Event) {
    const input = e.target as HTMLInputElement;
    // Proxmox VM names: alphanumeric, dash, underscore, period only
    // Replace spaces with dashes and remove invalid characters
    let name = input.value.replace(/\s+/g, "-").replace(/[^a-zA-Z0-9._-]/g, "");
    // Max 63 characters
    name = name.slice(0, 63);
    this._vmName = name || "home-assistant";
    // Update input to show sanitized value
    input.value = this._vmName;
    this._saveSelections();
  }

  private _onCoresChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const index = parseInt(input.value, 10);
    const coreOptions = this._getCoreOptions();
    this._cpuCores = coreOptions[index] || 4;
    this._saveSelections();
  }

  private _onMemoryChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const index = parseInt(input.value, 10);
    const memoryOptions = this._getMemoryOptions();
    this._memoryMb = memoryOptions[index] || 4096;
    this._saveSelections();
  }

  private _onDiskSizeChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const index = parseInt(input.value, 10);
    const diskOptions = this._getDiskSizeOptions();
    this._diskSizeGb = diskOptions[index] || 32;
    this._saveSelections();
  }

  private _getCoreOptions(): number[] {
    return [2, 4, 6, 8, 10, 12, 16];
  }

  private _getMemoryOptions(): number[] {
    return [2048, 4096, 6144, 8192, 12288, 16384, 24576, 32768];
  }

  private _formatMemory(mb: number): string {
    const gb = mb / 1024;
    return `${gb} GB`;
  }

  private _getDiskSizeOptions(): number[] {
    return [32, 64, 128, 256, 512];
  }

  private _formatDiskSize(gb: number): string {
    return gb >= 1024 ? `${gb / 1024} TB` : `${gb} GB`;
  }

  private _getCpuDescription(): string {
    if (this._cpuCores <= 2) {
      return "Minimum for basic operation";
    } else if (this._cpuCores <= 4) {
      return "Recommended for most users";
    } else if (this._cpuCores <= 8) {
      return "Better performance with many integrations";
    } else {
      return "Maximum performance for power users";
    }
  }

  private _getMemoryDescription(): string {
    const gb = this._memoryMb / 1024;
    if (gb <= 2) {
      return "Minimum for basic operation";
    } else if (gb <= 4) {
      return "Recommended for most users";
    } else if (gb <= 8) {
      return "Better for add-ons and many integrations";
    } else {
      return "Maximum performance for power users";
    }
  }

  private _getDiskDescription(): string {
    if (this._diskSizeGb <= 32) {
      return "Good for getting started";
    } else if (this._diskSizeGb <= 64) {
      return "Room for add-ons and history";
    } else if (this._diskSizeGb <= 128) {
      return "Plenty of space for long-term use";
    } else {
      return "Extended storage for recordings and backups";
    }
  }

  // Icons
  private _renderServerIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M4,1H20A1,1 0 0,1 21,2V6A1,1 0 0,1 20,7H4A1,1 0 0,1 3,6V2A1,1 0 0,1 4,1M4,9H20A1,1 0 0,1 21,10V14A1,1 0 0,1 20,15H4A1,1 0 0,1 3,14V10A1,1 0 0,1 4,9M4,17H20A1,1 0 0,1 21,18V22A1,1 0 0,1 20,23H4A1,1 0 0,1 3,22V18A1,1 0 0,1 4,17M9,5H10V3H9V5M9,13H10V11H9V13M9,21H10V19H9V21M5,3V5H7V3H5M5,11V13H7V11H5M5,19V21H7V19H5Z"/></svg>`;
  }

  private _renderDatabaseIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M12,3C7.58,3 4,4.79 4,7C4,9.21 7.58,11 12,11C16.42,11 20,9.21 20,7C20,4.79 16.42,3 12,3M4,9V12C4,14.21 7.58,16 12,16C16.42,16 20,14.21 20,12V9C20,11.21 16.42,13 12,13C7.58,13 4,11.21 4,9M4,14V17C4,19.21 7.58,21 12,21C16.42,21 20,19.21 20,17V14C20,16.21 16.42,18 12,18C7.58,18 4,16.21 4,14Z"/></svg>`;
  }

  private _renderIdIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M9,7H11V15H9V7M13,7H15V15H13V7M5,3H19A2,2 0 0,1 21,5V19A2,2 0 0,1 19,21H5A2,2 0 0,1 3,19V5A2,2 0 0,1 5,3M5,5V19H19V5H5Z"/></svg>`;
  }

  private _renderLabelIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M16,17H5V7H16L19.55,12M17.63,5.84C17.27,5.33 16.67,5 16,5H5A2,2 0 0,0 3,7V17A2,2 0 0,0 5,19H16C16.67,19 17.27,18.66 17.63,18.15L22,12L17.63,5.84Z"/></svg>`;
  }

  private _renderCpuIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M6,4H18V5H21V7H18V9H21V11H18V13H21V15H18V17H21V19H18V20H6V19H3V17H6V15H3V13H6V11H3V9H6V7H3V5H6V4M11,15V18H12V15H11M13,15V18H14V15H13M15,15V18H16V15H15Z"/></svg>`;
  }

  private _renderMemoryIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M17,17H7V7H17M21,11V9H19V7C19,5.89 18.1,5 17,5H15V3H13V5H11V3H9V5H7C5.89,5 5,5.89 5,7V9H3V11H5V13H3V15H5V17A2,2 0 0,0 7,19H9V21H11V19H13V21H15V19H17A2,2 0 0,0 19,17V15H21V13H19V11M13,13H11V11H13M15,9H9V15H15V9Z"/></svg>`;
  }

  private _renderDiskIcon() {
    return html`<svg viewBox="0 0 24 24"><path d="M12,3C7.58,3 4,4.79 4,7C4,9.21 7.58,11 12,11C16.42,11 20,9.21 20,7C20,4.79 16.42,3 12,3M4,9V12C4,14.21 7.58,16 12,16C16.42,16 20,14.21 20,12V9C20,11.21 16.42,13 12,13C7.58,13 4,11.21 4,9M4,14V17C4,19.21 7.58,21 12,21C16.42,21 20,19.21 20,17V14C20,16.21 16.42,18 12,18C7.58,18 4,16.21 4,14Z"/></svg>`;
  }

  private _renderTicks(count: number) {
    return html`
      <div class="slider-ticks">
        ${Array(count)
          .fill(0)
          .map(() => html`<div class="slider-tick"></div>`)}
      </div>
    `;
  }

  render() {
    if (this._error) {
      return html`
        <h2>Configure Virtual Machine</h2>
        <p class="subtitle">Configure your Home Assistant VM on Proxmox</p>
        <div class="config-card">
          <p class="error-text">${this._error}</p>
        </div>
      `;
    }

    const coreOptions = this._getCoreOptions();
    const memoryOptions = this._getMemoryOptions();
    const diskSizeOptions = this._getDiskSizeOptions();

    const coreIndex = coreOptions.indexOf(this._cpuCores);
    const memoryIndex = memoryOptions.indexOf(this._memoryMb);
    const diskIndex = diskSizeOptions.indexOf(this._diskSizeGb);

    return html`
      <h2>Configure Virtual Machine</h2>
      <p class="subtitle">Configure your Home Assistant VM on Proxmox</p>

      <div class="config-card">
        <!-- VM Name (Display Name) - First -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderLabelIcon()}</div>
          <div class="setting-content">
            <span class="setting-label">Display Name</span>
            <input
              type="text"
              class="name-input"
              .value=${this._vmName}
              @input=${this._onNameChange}
              placeholder="home-assistant"
              maxlength="63"
              pattern="[a-zA-Z0-9._-]+"
              title="Only letters, numbers, dash, underscore, and period allowed"
            />
            <p class="setting-description">
              Name shown in Proxmox (letters, numbers, dash, underscore only)
            </p>
          </div>
        </div>

        <!-- Node Selection -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderServerIcon()}</div>
          <div class="setting-content">
            <span class="setting-label">Node</span>
            ${this._loadingNodes
              ? html`<span class="loading-text">Loading nodes...</span>`
              : html`
                  <select
                    class="select-dropdown"
                    .value=${this._selectedNode}
                    @change=${this._onNodeChange}
                  >
                    ${this._nodes.map(
                      (node) => html`
                        <option value=${node.name}>
                          ${node.name}
                          ${node.cpu_usage !== undefined
                            ? `(CPU: ${node.cpu_usage.toFixed(1)}%)`
                            : ""}
                        </option>
                      `
                    )}
                  </select>
                `}
            <p class="setting-description">
              Proxmox node where the VM will be created
            </p>
          </div>
        </div>

        <!-- Storage Selection -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderDatabaseIcon()}</div>
          <div class="setting-content">
            <span class="setting-label">Storage</span>
            ${this._loadingStorage
              ? html`<span class="loading-text">Loading storage...</span>`
              : html`
                  <select
                    class="select-dropdown"
                    .value=${this._selectedStorage}
                    @change=${this._onStorageChange}
                    ?disabled=${this._storages.length === 0}
                  >
                    ${this._storages.map(
                      (storage) => html`
                        <option value=${storage.name}>
                          ${storage.name} (${formatBytes(storage.available)}
                          free)
                        </option>
                      `
                    )}
                  </select>
                `}
            <p class="setting-description">
              Storage location for the VM disk image
            </p>
          </div>
        </div>

        <!-- VM ID -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderIdIcon()}</div>
          <div class="setting-content">
            <span class="setting-label">VM ID</span>
            <input
              type="number"
              class="name-input"
              .value=${String(this._vmId)}
              @input=${this._onVmIdChange}
              min="100"
              max="999999999"
            />
            <p class="setting-description">
              Unique identifier for the virtual machine
            </p>
          </div>
        </div>

        <!-- CPU Cores -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderCpuIcon()}</div>
          <div class="setting-content">
            <div class="setting-header">
              <span class="setting-label">CPU Cores</span>
              <span class="setting-value">${this._cpuCores} Cores</span>
            </div>
            <div class="slider-container">
              <input
                type="range"
                min="0"
                max="${coreOptions.length - 1}"
                step="1"
                .value=${String(coreIndex >= 0 ? coreIndex : 1)}
                @input=${this._onCoresChange}
              />
              ${this._renderTicks(coreOptions.length)}
            </div>
            <p class="setting-description">${this._getCpuDescription()}</p>
          </div>
        </div>

        <!-- Memory -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderMemoryIcon()}</div>
          <div class="setting-content">
            <div class="setting-header">
              <span class="setting-label">Memory</span>
              <span class="setting-value"
                >${this._formatMemory(this._memoryMb)}</span
              >
            </div>
            <div class="slider-container">
              <input
                type="range"
                min="0"
                max="${memoryOptions.length - 1}"
                step="1"
                .value=${String(memoryIndex >= 0 ? memoryIndex : 1)}
                @input=${this._onMemoryChange}
              />
              ${this._renderTicks(memoryOptions.length)}
            </div>
            <p class="setting-description">${this._getMemoryDescription()}</p>
          </div>
        </div>

        <!-- Disk Size -->
        <div class="setting-row">
          <div class="setting-icon">${this._renderDiskIcon()}</div>
          <div class="setting-content">
            <div class="setting-header">
              <span class="setting-label">Disk Size</span>
              <span class="setting-value"
                >${this._formatDiskSize(this._diskSizeGb)}</span
              >
            </div>
            <div class="slider-container">
              <input
                type="range"
                min="0"
                max="${diskSizeOptions.length - 1}"
                step="1"
                .value=${String(diskIndex >= 0 ? diskIndex : 0)}
                @input=${this._onDiskSizeChange}
              />
              ${this._renderTicks(diskSizeOptions.length)}
            </div>
            <p class="setting-description">${this._getDiskDescription()}</p>
          </div>
        </div>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "proxmox-configure-view": ProxmoxConfigureView;
  }
}
