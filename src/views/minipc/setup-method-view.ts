import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { openUrl } from "@tauri-apps/plugin-opener";
import { wizardState } from "../../state/wizard-state.js";
import "../../components/info-dialog.js";

@customElement("minipc-setup-method-view")
export class MiniPCSetupMethodView extends LitElement {
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
      flex-direction: column;
      gap: 1rem;
      width: 100%;
      max-width: 500px;
    }

    .option-card {
      display: flex;
      align-items: center;
      gap: 1rem;
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

    @media (prefers-color-scheme: dark) {
      .option-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }

      .option-card:hover {
        box-shadow: 0 2px 8px rgba(3, 169, 244, 0.25);
      }
    }

    .option-icon {
      width: 48px;
      height: 48px;
      flex-shrink: 0;
    }

    .option-icon img {
      width: 100%;
      height: 100%;
      object-fit: contain;
    }

    .option-content {
      flex: 1;
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
      margin: 0;
      line-height: 1.4;
    }

    .option-arrow {
      font-size: 1.25rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
    }
  `;

  @state()
  private _showUsbDialog = false;

  render() {
    return html`
      <h2>How will you install?</h2>
      <p class="subtitle">
        Choose how you want to install Home Assistant on your mini PC
      </p>

      <div class="options">
        <div class="option-card" @click=${this._onConnectDrive}>
          <div class="option-icon">
            <img src="/assets/icons/drive-connect.svg" alt="Connect drive" />
          </div>
          <div class="option-content">
            <p class="option-title">I can connect the drive</p>
            <p class="option-description">
              Connect the SSD or NVMe drive from your mini PC to this computer
              via USB adapter
            </p>
          </div>
          <span class="option-arrow">→</span>
        </div>

        <div class="option-card" @click=${this._onUsbBoot}>
          <div class="option-icon">
            <img src="/assets/icons/usb-boot.svg" alt="USB boot" />
          </div>
          <div class="option-content">
            <p class="option-title">I need to boot from USB</p>
            <p class="option-description">
              Create a bootable USB drive to install Home Assistant directly on
              the mini PC
            </p>
          </div>
          <span class="option-arrow">→</span>
        </div>
      </div>

      <info-dialog
        ?open=${this._showUsbDialog}
        title="USB boot installation"
        message="Creating bootable USB drives is not supported by this installer. However, we have detailed instructions in our documentation that will guide you through the process."
        primaryLabel="View Instructions"
        secondaryLabel="Go Back"
        @dialog-primary=${this._onOpenDocs}
        @dialog-secondary=${this._onCloseDialog}
      ></info-dialog>
    `;
  }

  private _onConnectDrive() {
    wizardState.setSelection("installMethod", "direct");
    wizardState.nextStep();
  }

  private _onUsbBoot() {
    this._showUsbDialog = true;
  }

  private _onCloseDialog() {
    this._showUsbDialog = false;
  }

  private async _onOpenDocs() {
    this._showUsbDialog = false;
    try {
      await openUrl(
        "https://www.home-assistant.io/installation/generic-x86-64"
      );
    } catch {
      window.open(
        "https://www.home-assistant.io/installation/generic-x86-64",
        "_blank"
      );
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "minipc-setup-method-view": MiniPCSetupMethodView;
  }
}
