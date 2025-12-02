import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";
import { proxmoxConnect } from "../../api/commands.js";
import { wizardState } from "../../state/wizard-state.js";

@customElement("proxmox-connect-view")
export class ProxmoxConnectView extends LitElement {
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
      margin: 0 0 1rem 0;
      text-align: center;
    }

    .connect-card {
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
      .connect-card {
        background-color: var(--ha-card-background, #1e1e1e);
        border-color: var(--ha-border-color, #333333);
      }
    }

    .form-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .form-label {
      font-size: 0.875rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
    }

    .form-input {
      padding: 0.75rem;
      font-size: 1rem;
      color: var(--ha-text-color, #212121);
      background-color: var(--ha-background-color, #ffffff);
      border: 1px solid var(--ha-border-color, #e0e0e0);
      border-radius: 8px;
      outline: none;
      transition: border-color 0.2s ease;
    }

    .form-input:focus {
      border-color: var(--ha-primary-color, #03a9f4);
    }

    .form-input::placeholder {
      color: var(--ha-secondary-text-color, #9e9e9e);
    }

    @media (prefers-color-scheme: dark) {
      .form-input {
        background-color: var(--ha-background-color, #121212);
        border-color: var(--ha-border-color, #333333);
        color: var(--ha-text-color, #e0e0e0);
      }
    }

    .form-hint {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #9e9e9e);
      margin: 0;
    }

    .status-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1rem;
      border-radius: 8px;
      background-color: rgba(244, 67, 54, 0.1);
      border: 1px solid rgba(244, 67, 54, 0.3);
    }

    .status-icon {
      width: 24px;
      height: 24px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      background-color: #f44336;
    }

    .status-icon svg {
      width: 14px;
      height: 14px;
      fill: white;
    }

    .status-text {
      flex: 1;
    }

    .status-title {
      font-size: 0.9375rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0;
    }

    .status-description {
      font-size: 0.8125rem;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0.25rem 0 0 0;
    }
  `;

  @state()
  private _serverUrl = "";

  @state()
  private _username = "root@pam";

  @state()
  private _password = "";

  @state()
  private _connecting = false;

  @state()
  private _connected = false;

  @state()
  private _error: string | null = null;

  /** Connect to Proxmox server. Returns true if successful. */
  async connect(): Promise<boolean> {
    if (!this._serverUrl || !this._username || !this._password) {
      this._error = "Please fill in all fields";
      return false;
    }

    // Validate URL format - must be HTTPS for security
    const url = this._serverUrl.trim().replace(/\/$/, "");
    try {
      const parsed = new URL(url);
      if (parsed.protocol !== "https:") {
        this._error = "URL must use HTTPS (e.g., https://192.168.1.100:8006)";
        return false;
      }
    } catch {
      this._error = "Please enter a valid URL (e.g., https://192.168.1.100:8006)";
      return false;
    }

    this._connecting = true;
    this._error = null;

    try {
      const session = await proxmoxConnect({
        server_url: url,
        username: this._username,
        password: this._password,
      });

      this._connected = true;

      // Store session in wizard state
      wizardState.setSelection("proxmoxSession", session);
      wizardState.setSelection("proxmoxConnected", true);
      return true;
    } catch (error) {
      // Tauri invoke errors can be strings, Error objects, or other types
      if (typeof error === "string") {
        this._error = error;
      } else if (error instanceof Error) {
        this._error = error.message;
      } else if (error && typeof error === "object" && "message" in error) {
        this._error = String((error as { message: unknown }).message);
      } else {
        this._error = String(error) || "Failed to connect to Proxmox";
      }
      wizardState.setSelection("proxmoxConnected", false);
      return false;
    } finally {
      this._connecting = false;
    }
  }

  /** Check if form is valid (all required fields filled) */
  isFormValid(): boolean {
    if (!this._serverUrl || !this._username || !this._password) {
      return false;
    }
    try {
      const url = this._serverUrl.trim();
      const parsed = new URL(url);
      return parsed.protocol === "https:";
    } catch {
      return false;
    }
  }

  private _onServerUrlChange(e: Event) {
    const input = e.target as HTMLInputElement;
    this._serverUrl = input.value;
    this._resetConnection();
  }

  private _onUsernameChange(e: Event) {
    const input = e.target as HTMLInputElement;
    this._username = input.value;
    this._resetConnection();
  }

  private _onPasswordChange(e: Event) {
    const input = e.target as HTMLInputElement;
    this._password = input.value;
    this._resetConnection();
  }

  private _resetConnection() {
    if (this._connected) {
      this._connected = false;
      wizardState.setSelection("proxmoxConnected", false);
    }
    this._error = null;
  }

  private _onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !this._connecting && !this._connected) {
      this.connect();
    }
  }

  render() {
    return html`
      <h2>Connect to Proxmox VE</h2>
      <p class="subtitle">Enter your Proxmox server credentials</p>

      <div class="connect-card">
        ${this._error ? this._renderError() : ""}

        <div class="form-group">
          <label class="form-label" for="server-url">Server URL</label>
          <input
            type="url"
            id="server-url"
            class="form-input"
            .value=${this._serverUrl}
            @input=${this._onServerUrlChange}
            @keydown=${this._onKeyDown}
            placeholder="https://192.168.1.100:8006"
            ?disabled=${this._connecting}
          />
          <p class="form-hint">Full URL to your Proxmox server (e.g., https://192.168.1.100:8006)</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="username">Username</label>
          <input
            type="text"
            id="username"
            class="form-input"
            .value=${this._username}
            @input=${this._onUsernameChange}
            @keydown=${this._onKeyDown}
            placeholder="root@pam"
            ?disabled=${this._connecting}
          />
          <p class="form-hint">Usually root@pam for the default admin account</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="password">Password</label>
          <input
            type="password"
            id="password"
            class="form-input"
            .value=${this._password}
            @input=${this._onPasswordChange}
            @keydown=${this._onKeyDown}
            placeholder="Enter your password"
            ?disabled=${this._connecting}
          />
        </div>
      </div>
    `;
  }

  private _renderError() {
    return html`
      <div class="status-row">
        <div class="status-icon">
          <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
            />
          </svg>
        </div>
        <div class="status-text">
          <p class="status-title">Connection failed</p>
          <p class="status-description">${this._error}</p>
        </div>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "proxmox-connect-view": ProxmoxConnectView;
  }
}
