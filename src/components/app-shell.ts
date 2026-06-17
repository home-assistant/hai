import { LitElement, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";

// Import views
import "../views/welcome-view.js";
import "../views/path-selection-view.js";
import "../views/other-options-view.js";
import "../views/sbc/device-selection-view.js";
import "../views/sbc/drive-selection-view.js";
import "../views/sbc/confirmation-view.js";
import "../views/sbc/progress-view.js";
import "../views/sbc/success-view.js";
import "../views/minipc/setup-method-view.js";
import "../views/minipc/architecture-selection-view.js";
import "../views/utm/utm-check-view.js";
import "../views/utm/utm-configure-view.js";
import "../views/utm/utm-confirm-view.js";
import "../views/utm/utm-progress-view.js";
import "../views/utm/utm-success-view.js";
import "../views/proxmox/proxmox-connect-view.js";
import "../views/proxmox/proxmox-configure-view.js";
import "../views/proxmox/proxmox-confirm-view.js";
import "../views/proxmox/proxmox-progress-view.js";
import "../views/proxmox/proxmox-success-view.js";

// Import components
import "./wizard-shell.js";
import "./confirm-dialog.js";

// Import state
import {
  wizardState,
  type WizardFlow,
  type WizardState,
} from "../state/wizard-state.js";

import { openUrl } from "@tauri-apps/plugin-opener";

export type ViewName =
  | "welcome"
  | "path-selection"
  | "other-options"
  | "wizard";

@customElement("app-shell")
export class AppShell extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100vh;
      background-color: var(--ha-background-color, #ffffff);
      position: relative;
    }

    :host > * {
      flex: 1;
    }

    .toolbox-button {
      position: fixed;
      bottom: 1.5rem;
      right: 1.5rem;
      width: 56px;
      height: 56px;
      border-radius: 50%;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      transition:
        transform 0.2s ease,
        box-shadow 0.2s ease,
        background-color 0.2s ease;
      z-index: 100;
    }

    .toolbox-button:hover {
      transform: scale(1.05);
      box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
      background-color: var(--ha-primary-color-dark, #0288d1);
    }

    .toolbox-button:active {
      transform: scale(0.98);
    }

    .toolbox-button svg {
      width: 28px;
      height: 28px;
      fill: white;
    }

    .toolbox-button-tooltip {
      position: absolute;
      right: 68px;
      background-color: var(--ha-card-background, #333333);
      color: white;
      padding: 0.5rem 0.75rem;
      border-radius: 6px;
      font-size: 0.8125rem;
      white-space: nowrap;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.2s ease;
    }

    @media (prefers-color-scheme: dark) {
      .toolbox-button-tooltip {
        background-color: var(--ha-card-background, #424242);
      }
    }

    .toolbox-button:hover .toolbox-button-tooltip {
      opacity: 1;
    }
  `;

  @state()
  private _currentView: ViewName = "welcome";

  @state()
  private _wizardState: WizardState = wizardState.getState();

  @state()
  private _showConfirmDialog = false;

  @state()
  private _flashError = false;

  @state()
  private _utmInstallError = false;

  @state()
  private _proxmoxInstallError = false;

  @state()
  private _proxmoxConnecting = false;

  private _unsubscribe?: () => void;

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  render() {
    const driveName =
      (this._wizardState.selections.driveName as string) ||
      "the selected drive";

    return html`
      ${this._renderView()}
      ${this._currentView === "welcome" ? this._renderToolboxButton() : ""}
      <confirm-dialog
        ?open=${this._showConfirmDialog}
        .driveName=${driveName}
        @dialog-cancel=${this._onDialogCancel}
        @dialog-confirm=${this._onDialogConfirm}
      ></confirm-dialog>
    `;
  }

  private _renderView() {
    switch (this._currentView) {
      case "welcome":
        return html`<welcome-view
          @navigate=${this._onNavigate}
        ></welcome-view>`;
      case "path-selection":
        return html`<path-selection-view
          @navigate=${this._onNavigate}
          @select-path=${this._onSelectPath}
        ></path-selection-view>`;
      case "other-options":
        return html`<other-options-view
          @navigate=${this._onNavigate}
        ></other-options-view>`;
      case "wizard":
        return this._renderWizard();
      default:
        return html`<welcome-view
          @navigate=${this._onNavigate}
        ></welcome-view>`;
    }
  }

  private _renderWizard() {
    const currentStep = wizardState.currentStep;
    const flow = this._wizardState.currentFlow;
    const nextDisabled = this._isNextDisabled(flow, currentStep?.id);
    const nextLabel = this._getNextLabel(currentStep?.id);

    // Determine when to hide footer (during active processes)
    const hideFooter =
      (currentStep?.id === "flash" && !this._flashError) ||
      (flow === "vm" &&
        currentStep?.id === "install" &&
        !this._utmInstallError) ||
      (flow === "proxmox" &&
        currentStep?.id === "install" &&
        !this._proxmoxInstallError);

    // Determine when to hide back button
    const hideBack =
      currentStep?.id === "flash" ||
      currentStep?.id === "success" ||
      (flow === "vm" && currentStep?.id === "install") ||
      (flow === "proxmox" && currentStep?.id === "install");

    // Determine when to hide next button
    const hideNext = currentStep?.id === "method";

    return html`
      <wizard-shell
        .nextDisabled=${nextDisabled || this._proxmoxConnecting}
        .nextLabel=${this._proxmoxConnecting ? "Connecting..." : nextLabel}
        .hideFooter=${hideFooter}
        .hideBack=${hideBack}
        .hideNext=${hideNext}
        @wizard-cancel=${this._onWizardCancel}
        @wizard-next=${this._onWizardNext}
        @flash-complete=${this._onFlashComplete}
        @flash-error=${this._onFlashError}
      >
        ${this._renderWizardStep(flow, currentStep?.id)}
      </wizard-shell>
    `;
  }

  private _getNextLabel(stepId: string | undefined): string {
    if (stepId === "flash" && this._flashError) {
      return "Try Again";
    }
    if (
      stepId === "install" &&
      (this._utmInstallError || this._proxmoxInstallError)
    ) {
      return "Try Again";
    }
    if (stepId === "confirm") {
      return "Install";
    }
    if (stepId === "success") {
      return "Done";
    }
    return "Next";
  }

  private _isNextDisabled(
    flow: WizardFlow | null,
    stepId: string | undefined
  ): boolean {
    const selections = this._wizardState.selections;

    // Check if required selections are made for current step
    if (flow === "sbc") {
      if (stepId === "device") {
        return !selections.device;
      }
      if (stepId === "drive") {
        return !selections.drive;
      }
    }

    if (flow === "minipc") {
      if (stepId === "architecture") {
        return !selections.device;
      }
      if (stepId === "drive") {
        return !selections.drive;
      }
    }

    // VM (UTM) flow - require UTM to be installed on check step
    if (flow === "vm") {
      if (stepId === "check") {
        return !selections.utmInstalled;
      }
    }

    // Proxmox flow
    if (flow === "proxmox") {
      if (stepId === "configure") {
        return !selections.proxmoxNode || !selections.proxmoxStorage;
      }
    }

    return false;
  }

  private _renderWizardStep(
    flow: WizardFlow | null,
    stepId: string | undefined
  ) {
    // SBC Flow steps
    if (flow === "sbc") {
      switch (stepId) {
        case "device":
          return html`<device-selection-view></device-selection-view>`;
        case "drive":
          return html`<drive-selection-view></drive-selection-view>`;
        case "confirm":
          return html`<confirmation-view></confirmation-view>`;
        case "flash":
          return html`<progress-view></progress-view>`;
        case "success":
          return html`<success-view></success-view>`;
      }
    }

    // Mini PC Flow steps
    if (flow === "minipc") {
      switch (stepId) {
        case "method":
          return html`<minipc-setup-method-view></minipc-setup-method-view>`;
        case "architecture":
          return html`<minipc-architecture-selection-view></minipc-architecture-selection-view>`;
        case "drive":
          return html`<drive-selection-view></drive-selection-view>`;
        case "confirm":
          return html`<confirmation-view></confirmation-view>`;
        case "flash":
          return html`<progress-view></progress-view>`;
        case "success":
          return html`<success-view></success-view>`;
      }
    }

    // VM (UTM) Flow steps
    if (flow === "vm") {
      switch (stepId) {
        case "check":
          return html`<utm-check-view></utm-check-view>`;
        case "configure":
          return html`<utm-configure-view></utm-configure-view>`;
        case "confirm":
          return html`<utm-confirm-view></utm-confirm-view>`;
        case "install":
          return html`<utm-progress-view
            @install-complete=${this._onUtmInstallComplete}
            @install-error=${this._onUtmInstallError}
          ></utm-progress-view>`;
        case "success":
          return html`<utm-success-view></utm-success-view>`;
      }
    }

    // Proxmox Flow steps
    if (flow === "proxmox") {
      switch (stepId) {
        case "connection":
          return html`<proxmox-connect-view></proxmox-connect-view>`;
        case "configure":
          return html`<proxmox-configure-view></proxmox-configure-view>`;
        case "confirm":
          return html`<proxmox-confirm-view></proxmox-confirm-view>`;
        case "install":
          return html`<proxmox-progress-view
            @install-complete=${this._onProxmoxInstallComplete}
            @install-error=${this._onProxmoxInstallError}
          ></proxmox-progress-view>`;
        case "success":
          return html`<proxmox-success-view></proxmox-success-view>`;
      }
    }

    // Placeholder content for unimplemented steps
    return html`
      <div
        style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; text-align: center;"
      >
        <h2 style="color: var(--ha-text-color, #212121); margin: 0 0 1rem 0;">
          ${this._getFlowTitle(flow)}
        </h2>
        <p style="color: var(--ha-secondary-text-color, #727272); margin: 0;">
          Step: ${stepId || "unknown"}
        </p>
        <p
          style="color: var(--ha-secondary-text-color, #9e9e9e); font-size: 0.875rem; margin-top: 2rem;"
        >
          (Step content coming soon)
        </p>
      </div>
    `;
  }

  private _getFlowTitle(flow: WizardFlow | null): string {
    switch (flow) {
      case "sbc":
        return "Raspberry Pi & Other Boards";
      case "minipc":
        return "Generic (mini) PC";
      case "ha-hardware":
        return "Home Assistant Hardware";
      case "proxmox":
        return "Proxmox Server";
      case "vm":
        return "Virtual Machine";
      default:
        return "Installation";
    }
  }

  private _onNavigate(e: CustomEvent<{ view: ViewName }>) {
    this._currentView = e.detail.view;
  }

  private _onSelectPath(e: CustomEvent<{ path: WizardFlow }>) {
    wizardState.startFlow(e.detail.path);
    this._currentView = "wizard";
  }

  private _onWizardCancel() {
    wizardState.reset();
    this._currentView = "welcome";
  }

  private async _onWizardNext() {
    const currentStep = wizardState.currentStep;
    const flow = this._wizardState.currentFlow;

    // Handle retry on flash error
    if (currentStep?.id === "flash" && this._flashError) {
      this._flashError = false;
      const wizardShell = this.shadowRoot?.querySelector("wizard-shell");
      const progressView = wizardShell?.querySelector("progress-view") as
        | (HTMLElement & { retry: () => void })
        | null;
      progressView?.retry();
      return;
    }

    // Handle retry on UTM install error
    if (
      flow === "vm" &&
      currentStep?.id === "install" &&
      this._utmInstallError
    ) {
      this._utmInstallError = false;
      const wizardShell = this.shadowRoot?.querySelector("wizard-shell");
      const utmProgressView = wizardShell?.querySelector(
        "utm-progress-view"
      ) as (HTMLElement & { retry: () => void }) | null;
      utmProgressView?.retry();
      return;
    }

    // Handle retry on Proxmox install error
    if (
      flow === "proxmox" &&
      currentStep?.id === "install" &&
      this._proxmoxInstallError
    ) {
      this._proxmoxInstallError = false;
      const wizardShell = this.shadowRoot?.querySelector("wizard-shell");
      const proxmoxProgressView = wizardShell?.querySelector(
        "proxmox-progress-view"
      ) as (HTMLElement & { retry: () => void }) | null;
      proxmoxProgressView?.retry();
      return;
    }

    // Handle Proxmox connection step - connect then proceed if successful
    if (flow === "proxmox" && currentStep?.id === "connection") {
      const wizardShell = this.shadowRoot?.querySelector("wizard-shell");
      const connectView = wizardShell?.querySelector("proxmox-connect-view") as
        | (HTMLElement & { connect: () => Promise<boolean> })
        | null;
      if (connectView) {
        this._proxmoxConnecting = true;
        try {
          const success = await connectView.connect();
          if (!success) {
            return; // Stay on current step, error shown in view
          }
        } finally {
          this._proxmoxConnecting = false;
        }
      }
    }

    // Show confirmation dialog before proceeding from confirm step (only for SBC/minipc flows)
    if (
      currentStep?.id === "confirm" &&
      (flow === "sbc" || flow === "minipc")
    ) {
      this._showConfirmDialog = true;
      return;
    }

    if (wizardState.isLastStep) {
      // Flow complete - go back to welcome
      wizardState.reset();
      this._currentView = "welcome";
    } else {
      wizardState.nextStep();
    }
  }

  private _onDialogCancel() {
    this._showConfirmDialog = false;
  }

  private _onDialogConfirm() {
    this._showConfirmDialog = false;
    // Proceed to flash step
    wizardState.nextStep();
  }

  private _onFlashComplete() {
    // Advance to success/done step after flash completes
    wizardState.nextStep();
  }

  private _onFlashError() {
    this._flashError = true;
  }

  private _onUtmInstallComplete() {
    // Advance to success step after UTM install completes
    wizardState.nextStep();
  }

  private _onUtmInstallError() {
    this._utmInstallError = true;
  }

  private _onProxmoxInstallComplete() {
    // Advance to success step after Proxmox install completes
    wizardState.nextStep();
  }

  private _onProxmoxInstallError() {
    this._proxmoxInstallError = true;
  }

  private _renderToolboxButton() {
    // mdi:toolbox-outline
    return html`
      <button class="toolbox-button" @click=${this._onToolboxOpen}>
        <span class="toolbox-button-tooltip">Open Home Toolbox</span>
        <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
          <path
            d="M18 16H16V15H8V16H6V15H2V20H22V15H18V16M20 8H17V6C17 4.9 16.1 4 15 4H9C7.9 4 7 4.9 7 6V8H4C2.9 8 2 8.9 2 10V14H6V12H8V14H16V12H18V14H22V10C22 8.9 21.1 8 20 8M15 8H9V6H15V8Z"
          />
        </svg>
      </button>
    `;
  }

  private async _onToolboxOpen() {
    try {
      await openUrl("https://toolbox.openhomefoundation.org/");
    } catch {
      // Fallback for browser-only mode
      window.open("https://toolbox.openhomefoundation.org/", "_blank");
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "app-shell": AppShell;
  }
}
