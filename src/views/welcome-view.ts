import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";

@customElement("welcome-view")
export class WelcomeView extends LitElement {
  @state()
  private _logoClickCount = 0;

  private _clickResetTimer?: number;

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      padding: 2rem;
      text-align: center;
      position: relative;
    }

    @keyframes soft-pulse {
      0%,
      100% {
        transform: scale(1);
        filter: drop-shadow(0 0 0 rgba(24, 188, 242, 0));
      }
      50% {
        transform: scale(1.02);
        filter: drop-shadow(0 0 15px rgba(24, 188, 242, 0.3));
      }
    }

    .logo-container {
      margin-bottom: 2rem;
    }

    .logo {
      width: 500px;
      max-width: 100%;
      height: auto;
    }

    .logo-container:hover .logo {
      animation: soft-pulse 3s ease-in-out infinite;
    }

    .logo-dark {
      display: none;
    }

    @media (prefers-color-scheme: dark) {
      .logo-light {
        display: none;
      }
      .logo-dark {
        display: block;
      }
    }

    .welcome-text {
      max-width: 520px;
      color: var(--ha-secondary-text-color, #727272);
      line-height: 1.6;
      margin-bottom: 2rem;
    }

    .welcome-text p {
      margin: 0 0 1rem 0;
    }

    .welcome-text p:last-child {
      margin-bottom: 0;
    }

    .lets-go-button {
      display: inline-flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1.25rem 3rem;
      font-size: 1.375rem;
      font-weight: 500;
      color: white;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
      border-radius: 12px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .lets-go-button:hover {
      background-color: var(--ha-primary-color-dark, #0288d1);
    }

    .lets-go-button:active {
      transform: scale(0.98);
    }

    .arrow {
      font-size: 1.5rem;
    }

    .learn-more {
      margin-top: 1rem;
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      text-decoration: underline;
    }

    .ohf-link {
      position: absolute;
      bottom: 2rem;
      text-decoration: none;
    }

    .ohf-logo {
      width: 180px;
      opacity: 0.7;
      transition: opacity 0.2s ease;
    }

    .ohf-link:hover .ohf-logo {
      opacity: 1;
    }

    .ohf-logo-dark {
      display: none;
    }

    @media (prefers-color-scheme: dark) {
      .ohf-logo-light {
        display: none;
      }
      .ohf-logo-dark {
        display: block;
      }
    }
  `;

  render() {
    return html`
      <div class="logo-container" @click=${this._onLogoClick}>
        <img
          class="logo logo-light"
          src="/assets/home-assistant-logo-light.svg"
          alt="Home Assistant"
        />
        <img
          class="logo logo-dark"
          src="/assets/home-assistant-logo-dark.svg"
          alt="Home Assistant"
        />
      </div>

      <div class="welcome-text">
        <p>
          Welcome to the exciting start of your local and
          <span style="white-space: nowrap">privacy-first</span><br />home
          automation journey.
        </p>
        <p>
          This application will help you get Home Assistant installed<br />on
          the hardware of your choice in just a few steps, ensuring your smart
          home adventure has a smooth and
          <span style="white-space: nowrap">worry-free</span> start 🚀
        </p>
      </div>

      <button class="lets-go-button" @click=${this._onLetsGo}>
        Let's go <span class="arrow">→</span>
      </button>

      <a
        class="learn-more"
        href="https://www.home-assistant.io/installation/"
        target="_blank"
        rel="noopener noreferrer"
      >
        I want to learn more first...
      </a>

      <a
        class="ohf-link"
        href="https://www.openhomefoundation.org/"
        target="_blank"
        rel="noopener noreferrer"
      >
        <img
          class="ohf-logo ohf-logo-light"
          src="/assets/ohf-logo-light.svg"
          alt="Open Home Foundation"
        />
        <img
          class="ohf-logo ohf-logo-dark"
          src="/assets/ohf-logo-dark.svg"
          alt="Open Home Foundation"
        />
      </a>
    `;
  }

  private _onLogoClick() {
    this._logoClickCount++;

    // Reset the counter after 2 seconds of no clicks
    if (this._clickResetTimer) {
      clearTimeout(this._clickResetTimer);
    }
    this._clickResetTimer = window.setTimeout(() => {
      this._logoClickCount = 0;
    }, 2000);

    // Play sound after 5 clicks
    if (this._logoClickCount === 5) {
      this._playEasterEgg();
      this._logoClickCount = 0;
    }
  }

  private _playEasterEgg() {
    const audio = new Audio("/assets/audio/home-assistant.wav");
    audio.play().catch((error) => {
      console.error("Failed to play easter egg audio:", error);
    });
  }

  private _onLetsGo() {
    this.dispatchEvent(
      new CustomEvent("navigate", {
        detail: { view: "path-selection" },
        bubbles: true,
        composed: true,
      })
    );
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._clickResetTimer) {
      clearTimeout(this._clickResetTimer);
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "welcome-view": WelcomeView;
  }
}
