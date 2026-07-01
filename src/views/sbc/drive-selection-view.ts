import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { listBlockDevices, type BlockDevice } from "../../api/index.js";
import { wizardState } from "../../state/wizard-state.js";
import "../../components/drive-card.js";

@customElement("drive-selection-view")
export class DriveSelectionView extends LitElement {
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

    .warning {
      display: flex;
      align-items: flex-start;
      gap: 0.75rem;
      padding: 1rem;
      background-color: rgba(255, 152, 0, 0.1);
      border: 1px solid rgba(255, 152, 0, 0.3);
      border-radius: 8px;
      margin-bottom: 1.5rem;
      max-width: 500px;
      width: 100%;
    }

    .warning-icon {
      font-size: 1.25rem;
      flex-shrink: 0;
    }

    .warning-text {
      font-size: 0.875rem;
      color: var(--ha-text-color, #212121);
      margin: 0;
      line-height: 1.5;
    }

    @media (prefers-color-scheme: dark) {
      .warning {
        background-color: rgba(255, 152, 0, 0.15);
        border-color: rgba(255, 152, 0, 0.4);
      }
    }

    .drives-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
      max-width: 500px;
      margin-bottom: 1rem;
    }

    .drives-title {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--ha-secondary-text-color, #727272);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin: 0;
    }

    .refresh-button {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 0.75rem;
      font-size: 0.875rem;
      color: var(--ha-primary-color, #03a9f4);
      background: none;
      border: 1px solid var(--ha-primary-color, #03a9f4);
      border-radius: 6px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .refresh-button:hover {
      background-color: rgba(3, 169, 244, 0.1);
    }

    .refresh-button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .refresh-icon {
      font-size: 1rem;
    }

    .drives-list {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      width: 100%;
      max-width: 500px;
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

    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 2rem;
      text-align: center;
      color: var(--ha-secondary-text-color, #727272);
    }

    .empty-icon {
      width: 64px;
      height: 64px;
      margin-bottom: 1rem;
      opacity: 0.5;
    }

    .empty-icon svg {
      width: 100%;
      height: 100%;
      fill: var(--ha-secondary-text-color, #727272);
    }

    .empty-title {
      font-size: 1.125rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.5rem 0;
    }

    .empty-text {
      font-size: 0.875rem;
      margin: 0;
    }
  `;

  @state()
  private _drives: BlockDevice[] = [];

  @state()
  private _loading = true;

  @state()
  private _error: string | null = null;

  @state()
  private _selectedDriveId: string | null = null;

  connectedCallback() {
    super.connectedCallback();
    this._loadDrives();

    // Check if there's already a selection in wizard state
    const state = wizardState.getState();
    if (state.selections.drive) {
      this._selectedDriveId = state.selections.drive as string;
    }
  }

  private async _loadDrives() {
    this._loading = true;
    this._error = null;

    try {
      const drives = await listBlockDevices();
      // Filter to only show removable drives
      this._drives = drives.filter((drive) => drive.removable);
    } catch (err) {
      this._error =
        err instanceof Error ? err.message : "Failed to load drives";
    } finally {
      this._loading = false;
    }
  }

  private _isMiniPCFlow(): boolean {
    return wizardState.getState().currentFlow === "minipc";
  }

  private _getMinimumDriveSize(): number {
    // Mini PC flow requires 16GB minimum (NVMe/SSD)
    // SBC flow requires 2GB minimum (SD card)
    return this._isMiniPCFlow()
      ? 16 * 1000 * 1000 * 1000 // 16 GB
      : 2 * 1000 * 1000 * 1000; // 2 GB
  }

  render() {
    const isMiniPC = this._isMiniPCFlow();
    const subtitle = isMiniPC
      ? "Choose the NVMe/SSD drive to install Home Assistant on"
      : "Choose the SD card or USB drive to install Home Assistant on";

    return html`
      <h2>Select your drive</h2>
      <p class="subtitle">${subtitle}</p>

      <div class="warning">
        <span class="warning-icon">⚠️</span>
        <p class="warning-text">
          <strong>Warning:</strong> All data on the selected drive will be
          permanently erased. Make sure you have backed up any important files
          before proceeding.
        </p>
      </div>

      ${this._renderContent()}
    `;
  }

  private _renderContent() {
    if (this._loading) {
      return html`
        <div class="loading">
          <div class="loading-spinner"></div>
          <span>Scanning for drives...</span>
        </div>
      `;
    }

    if (this._error) {
      return html`
        <div class="error">
          <span class="error-icon">⚠️</span>
          <p class="error-message">${this._error}</p>
          <button class="retry-button" @click=${this._loadDrives}>
            Try again
          </button>
        </div>
      `;
    }

    if (this._drives.length === 0) {
      const emptyText = this._isMiniPCFlow()
        ? "Connect your drive via USB adapter and click refresh."
        : "Insert an SD card or USB drive and click refresh.";

      return html`
        <div class="empty-state">
          <span class="empty-icon">
            <svg viewBox="0 0 24 24">
              <path
                d="M18,8H16V4H18M15,8H13V4H15M12,8H10V4H12M18,2H10L4,8V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V4A2,2 0 0,0 18,2Z"
              />
            </svg>
          </span>
          <p class="empty-title">No drives found</p>
          <p class="empty-text">${emptyText}</p>
          <button
            class="refresh-button"
            @click=${this._loadDrives}
            style="margin-top: 1rem;"
          >
            <span class="refresh-icon">↻</span>
            Refresh
          </button>
        </div>
      `;
    }

    return html`
      <div class="drives-header">
        <p class="drives-title">Available drives</p>
        <button
          class="refresh-button"
          @click=${this._loadDrives}
          ?disabled=${this._loading}
        >
          <span class="refresh-icon">↻</span>
          Refresh
        </button>
      </div>

      <div class="drives-list">
        ${[...this._drives]
          .sort((a, b) => {
            const minSize = this._getMinimumDriveSize();
            const tolerance = 100 * 1000 * 1000;
            const aTooSmall = a.size < minSize - tolerance;
            const bTooSmall = b.size < minSize - tolerance;
            // Sort selectable drives first, then by size descending
            if (aTooSmall !== bTooSmall) return aTooSmall ? 1 : -1;
            return b.size - a.size;
          })
          .map((drive) => {
            const minSize = this._getMinimumDriveSize();
            const tooSmall = drive.size < minSize - 100 * 1000 * 1000; // Allow some tolerance for reported sizes
            const minSizeGB = minSize / (1000 * 1000 * 1000);

            return html`
              <drive-card
                .driveId=${drive.id}
                .name=${drive.name}
                .size=${drive.size}
                .deviceType=${drive.device_type}
                .model=${drive.model || ""}
                .vendor=${drive.vendor || ""}
                .selected=${this._selectedDriveId === drive.id}
                .disabled=${tooSmall}
                .disabledReason=${tooSmall
                  ? `⚠ Minimum ${minSizeGB} GB required`
                  : ""}
                @click=${() => !tooSmall && this._onSelectDrive(drive)}
              ></drive-card>
            `;
          })}
      </div>
    `;
  }

  private _onSelectDrive(drive: BlockDevice) {
    this._selectedDriveId = drive.id;
    wizardState.setSelection("drive", drive.id);
    wizardState.setSelection("driveName", drive.name);
    wizardState.setSelection("driveSize", drive.size);

    this.dispatchEvent(
      new CustomEvent("drive-selected", {
        detail: { drive },
        bubbles: true,
        composed: true,
      })
    );
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "drive-selection-view": DriveSelectionView;
  }
}
