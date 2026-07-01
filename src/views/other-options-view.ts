import { LitElement, html, css } from "lit";
import { customElement } from "lit/decorators.js";
import { openUrl } from "@tauri-apps/plugin-opener";

interface OtherOption {
  title: string;
  description: string;
  url: string;
  icon: string;
}

const OTHER_OPTIONS: OtherOption[] = [
  {
    title: "Docker container",
    description: "Run Home Assistant container",
    url: "https://www.home-assistant.io/installation/linux#docker-compose",
    icon: "docker",
  },
  {
    title: "Synology NAS",
    description:
      "Run Home Assistant on your Synology NAS using Virtual Machine Manager",
    url: "https://www.home-assistant.io/installation/synology",
    icon: "synology",
  },
  {
    title: "QNAP NAS",
    description:
      "Run Home Assistant on your QNAP NAS using Virtualization Station",
    url: "https://www.home-assistant.io/installation/qnap",
    icon: "qnap",
  },
  {
    title: "Linux virtual machine",
    description: "Run Home Assistant OS in KVM, VirtualBox, or VMware on Linux",
    url: "https://www.home-assistant.io/installation/linux",
    icon: "linux",
  },
  {
    title: "Windows virtual machine",
    description:
      "Run Home Assistant OS in Hyper-V, VirtualBox, or VMware on Windows",
    url: "https://www.home-assistant.io/installation/windows",
    icon: "windows",
  },
];

@customElement("other-options-view")
export class OtherOptionsView extends LitElement {
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
      overflow-y: auto;
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
      max-width: 500px;
    }

    .options-list {
      display: flex;
      flex-direction: column;
      gap: 1rem;
      max-width: 600px;
      width: 100%;
    }

    .option-item {
      display: flex;
      align-items: center;
      gap: 1rem;
      padding: 1rem 1.5rem;
      background-color: var(--ha-card-background, #ffffff);
      border: 2px solid var(--ha-border-color, #e0e0e0);
      border-radius: 12px;
      cursor: pointer;
      transition:
        border-color 0.2s ease,
        box-shadow 0.2s ease;
    }

    .option-item:hover {
      border-color: var(--ha-primary-color, #03a9f4);
      box-shadow: 0 2px 8px rgba(3, 169, 244, 0.15);
    }

    @media (prefers-color-scheme: dark) {
      .option-item {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }

      .option-item:hover {
        box-shadow: 0 2px 8px rgba(3, 169, 244, 0.25);
      }
    }

    .option-icon {
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }

    .option-icon img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }

    .icon-placeholder {
      width: 40px;
      height: 40px;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 8px;
      opacity: 0.2;
    }

    .option-text {
      flex: 1;
      min-width: 0;
    }

    .option-title {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.25rem 0;
    }

    .option-description {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0;
      line-height: 1.4;
    }

    .external-icon {
      color: var(--ha-secondary-text-color, #9e9e9e);
      font-size: 1.25rem;
      flex-shrink: 0;
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
        <h1>Other installation methods</h1>
        <p class="subtitle">
          These options are not directly supported by this installer, but you
          can follow our documentation to set them up.
        </p>

        <div class="options-list">
          ${OTHER_OPTIONS.map(
            (option) => html`
              <div
                class="option-item"
                @click=${() => this._openLink(option.url)}
              >
                <div class="option-icon">${this._renderIcon(option.icon)}</div>
                <div class="option-text">
                  <p class="option-title">${option.title}</p>
                  <p class="option-description">${option.description}</p>
                </div>
                <span class="external-icon">↗</span>
              </div>
            `
          )}
        </div>
      </div>
    `;
  }

  private _renderIcon(iconName: string) {
    const iconMap: Record<string, string> = {
      docker: "/assets/icons/docker.svg",
      synology: "/assets/icons/synology.svg",
      qnap: "/assets/icons/qnap.svg",
      linux: "/assets/icons/linux.svg",
      windows: "/assets/icons/windows.svg",
    };

    const iconSrc = iconMap[iconName];
    if (iconSrc) {
      return html`<img src=${iconSrc} alt=${iconName} />`;
    }

    return html`<div class="icon-placeholder"></div>`;
  }

  private _onBack() {
    this.dispatchEvent(
      new CustomEvent("navigate", {
        detail: { view: "path-selection" },
        bubbles: true,
        composed: true,
      })
    );
  }

  private async _openLink(url: string) {
    try {
      await openUrl(url);
    } catch {
      // Fallback for development/web
      window.open(url, "_blank");
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "other-options-view": OtherOptionsView;
  }
}
