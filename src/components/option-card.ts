import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("option-card")
export class OptionCard extends LitElement {
  static styles = css`
    :host {
      display: block;
    }

    .card {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 1.5rem;
      min-height: 200px;
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
      transform: scale(0.98);
    }

    @media (prefers-color-scheme: dark) {
      .card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }

      .card:hover {
        box-shadow: 0 4px 12px rgba(3, 169, 244, 0.25);
      }
    }

    .icon-container {
      width: 80px;
      height: 80px;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 1rem;
    }

    .icon-container img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }

    .icon-placeholder {
      width: 64px;
      height: 64px;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 12px;
      opacity: 0.2;
    }

    .title {
      font-size: 1rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      text-align: center;
      margin: 0 0 0.5rem 0;
    }

    .description {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      text-align: center;
      line-height: 1.4;
      margin: 0;
    }
  `;

  @property({ type: String })
  title = "";

  @property({ type: String })
  description = "";

  @property({ type: String })
  icon = "";

  @property({ type: String })
  image = "";

  render() {
    return html`
      <div class="card">
        <div class="icon-container">${this._renderIcon()}</div>
        <p class="title">${this.title}</p>
        <p class="description">${this.description}</p>
      </div>
    `;
  }

  private _renderIcon() {
    if (this.image) {
      return html`<img src=${this.image} alt=${this.title} />`;
    }

    // Use placeholder icons based on icon type
    const iconMap: Record<string, string> = {
      sbc: "/assets/icons/sbc-placeholder.svg",
      minipc: "/assets/icons/minipc-placeholder.svg",
      "ha-hardware": "/assets/icons/home-assistant-hardware.svg",
      proxmox: "/assets/icons/proxmox-placeholder.svg",
      vm: "/assets/icons/vm-placeholder.svg",
      others: "/assets/icons/others-placeholder.svg",
    };

    const iconSrc = iconMap[this.icon];
    if (iconSrc) {
      return html`<img src=${iconSrc} alt=${this.title} />`;
    }

    return html`<div class="icon-placeholder"></div>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "option-card": OptionCard;
  }
}
