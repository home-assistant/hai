import { LitElement, html, css } from "lit";
import { customElement } from "lit/decorators.js";

import "../components/option-card.js";

export type InstallationPath =
  | "sbc"
  | "minipc"
  | "ha-hardware"
  | "proxmox"
  | "vm";

@customElement("path-selection-view")
export class PathSelectionView extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      padding: 2rem;
    }

    .header {
      display: flex;
      align-items: center;
      margin-bottom: 2rem;
    }

    .back-button {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
      font-size: 1rem;
      color: var(--ha-secondary-text-color, #727272);
      background: none;
      border: none;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .back-button:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }

    @media (prefers-color-scheme: dark) {
      .back-button:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }
    }

    .back-arrow {
      font-size: 1.25rem;
    }

    .content {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }

    h1 {
      font-size: 1.75rem;
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

    .options-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1.5rem;
      max-width: 900px;
      width: 100%;
    }

    .other-options {
      margin-top: 2rem;
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      text-decoration: underline;
      cursor: pointer;
    }

    .other-options:hover {
      color: var(--ha-primary-color, #03a9f4);
    }
  `;

  render() {
    return html`
      <div class="header">
        <button class="back-button" @click=${this._onBack}>
          <span class="back-arrow">←</span> Back
        </button>
      </div>

      <div class="content">
        <h1>What would you like to install on?</h1>
        <p class="subtitle">Select how you want to run Home Assistant</p>

        <div class="options-grid">
          <option-card
            title="Home Assistant Hardware"
            description="Home Assistant Green, Yellow, or Blue by Nabu Casa"
            icon="ha-hardware"
            @click=${() => this._onSelectPath("ha-hardware")}
          ></option-card>

          <option-card
            title="Raspberry Pi & other boards"
            description="Single board computers like Raspberry Pi, ODROID, and more"
            icon="sbc"
            @click=${() => this._onSelectPath("sbc")}
          ></option-card>

          <option-card
            title="Generic (mini) PC"
            description="x86-64 or ARM64 computers like Beelink, Intel NUC, and more"
            icon="minipc"
            @click=${() => this._onSelectPath("minipc")}
          ></option-card>

          <option-card
            title="Proxmox Server"
            description="Create a VM on your Proxmox virtualization server"
            icon="proxmox"
            @click=${() => this._onSelectPath("proxmox")}
          ></option-card>

          ${this._renderVMOption()}

          <option-card
            title="Others"
            description="Other options like Docker can be found in our documentation"
            icon="others"
            @click=${this._onOtherOptions}
          ></option-card>
        </div>
      </div>
    `;
  }

  private _renderVMOption() {
    // TODO: Check if running on macOS
    const isMacOS = window.navigator.platform.toLowerCase().includes("mac");

    if (!isMacOS) {
      return null;
    }

    return html`
      <option-card
        title="Virtual Machine"
        description="Run Home Assistant in UTM on your Mac"
        icon="vm"
        @click=${() => this._onSelectPath("vm")}
      ></option-card>
    `;
  }

  private _onBack() {
    this.dispatchEvent(
      new CustomEvent("navigate", {
        detail: { view: "welcome" },
        bubbles: true,
        composed: true,
      })
    );
  }

  private _onSelectPath(path: InstallationPath) {
    this.dispatchEvent(
      new CustomEvent("select-path", {
        detail: { path },
        bubbles: true,
        composed: true,
      })
    );
  }

  private _onOtherOptions() {
    this.dispatchEvent(
      new CustomEvent("navigate", {
        detail: { view: "other-options" },
        bubbles: true,
        composed: true,
      })
    );
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "path-selection-view": PathSelectionView;
  }
}
