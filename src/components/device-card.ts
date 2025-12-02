import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("device-card")
export class DeviceCard extends LitElement {
  static styles = css`
    :host {
      display: block;
    }

    .card {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 1.25rem;
      height: 180px;
      box-sizing: border-box;
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

    .card.selected {
      border-color: var(--ha-primary-color, #03a9f4);
      box-shadow: 0 0 0 3px rgba(3, 169, 244, 0.2);
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

    .image-container {
      width: 100px;
      height: 100px;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 0.75rem;
      flex-shrink: 0;
    }

    .image-container img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }

    .image-placeholder {
      width: 80px;
      height: 80px;
      background-color: var(--ha-primary-color, #03a9f4);
      border-radius: 12px;
      opacity: 0.2;
    }

    .name {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      text-align: center;
      margin: 0;
      line-height: 1.3;
      min-height: 2.6em;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .selected-indicator {
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

    .card-wrapper {
      position: relative;
    }
  `;

  @property({ type: String })
  deviceId = "";

  @property({ type: String })
  name = "";

  @property({ type: String })
  image = "";

  @property({ type: Boolean })
  selected = false;

  render() {
    return html`
      <div class="card-wrapper">
        <div class="card ${this.selected ? "selected" : ""}">
          <div class="image-container">${this._renderImage()}</div>
          <p class="name">${this.name}</p>
        </div>
        ${this.selected
          ? html`<span class="selected-indicator">✓</span>`
          : ""}
      </div>
    `;
  }

  private _renderImage() {
    if (this.image) {
      return html`<img src=${this.image} alt=${this.name} />`;
    }
    return html`<div class="image-placeholder"></div>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "device-card": DeviceCard;
  }
}
