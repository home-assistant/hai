import { LitElement, html, css, svg } from "lit";
import { customElement, state } from "lit/decorators.js";
import { wizardState, type WizardState } from "../../state/wizard-state.js";
import { proxmoxCreateVm, formatBytes } from "../../api/commands.js";
import type {
  FlashProgress,
  ProxmoxSession,
  ProxmoxVmConfig,
} from "../../api/types.js";
import "../../components/progress-bar.js";

type InstallStage =
  | "downloading"
  | "extracting"
  | "writing"
  | "verifying"
  | "finalizing"
  | "ready"
  | "updating"
  | "complete"
  | "error";

// Stages that have measurable progress (0-100%)
const MEASURABLE_STAGES: InstallStage[] = ["downloading"];

// Stages that use indeterminate progress (waiting for something, or unknown total size)
const INDETERMINATE_STAGES: InstallStage[] = [
  "extracting",
  "writing",
  "verifying",
  "finalizing",
  "ready",
  "updating",
];

@customElement("proxmox-progress-view")
export class ProxmoxProgressView extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      box-sizing: border-box;
    }

    .mascot-container {
      width: 120px;
      height: 120px;
      margin-bottom: 2rem;
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
      overflow: visible;
    }

    .mascot-container.with-bubble {
      margin-left: 40px;
    }

    .casita-mascot {
      width: 100%;
      height: 100%;
    }

    .casita-tear {
      animation: tear-fall 1.5s ease-in infinite;
    }

    .casita-tear.delay {
      animation-delay: 0.75s;
    }

    @keyframes tear-fall {
      0% {
        opacity: 0;
        transform: translateY(-4px);
      }
      20% {
        opacity: 1;
      }
      100% {
        opacity: 0;
        transform: translateY(12px);
      }
    }

    h2 {
      font-size: 1.25rem;
      font-weight: 500;
      color: var(--ha-text-color, #212121);
      margin: 0 0 0.25rem 0;
      text-align: center;
    }

    .progress-section {
      width: 100%;
      max-width: 400px;
      margin-bottom: 1rem;
    }

    .progress-details {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-top: 0.75rem;
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
    }

    .percentage {
      font-weight: 500;
      color: var(--ha-text-color, #212121);
    }

    .progress-right {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      gap: 0.125rem;
    }

    .eta {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #727272);
      min-height: 1.125rem;
    }

    .bytes-info {
      min-height: 1.25rem;
    }

    .progress-left {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      gap: 0.125rem;
    }

    .speed {
      font-size: 0.75rem;
      color: var(--ha-secondary-text-color, #727272);
      min-height: 1.125rem;
    }

    .error-section {
      text-align: center;
      margin-top: 1.5rem;
    }

    .error-message {
      color: var(--ha-error-color, #db4437);
      font-size: 0.9375rem;
      margin: 0 0 1.5rem 0;
      max-width: 400px;
    }

    .retry-button {
      padding: 0.75rem 1.5rem;
      font-size: 1rem;
      font-weight: 500;
      color: white;
      background-color: var(--ha-primary-color, #03a9f4);
      border: none;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .retry-button:hover {
      background-color: var(--ha-primary-color-dark, #0288d1);
    }

    .stages-indicator {
      display: flex;
      justify-content: center;
      gap: 0.5rem;
      margin-bottom: 1.5rem;
    }

    .stage-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background-color: var(--ha-border-color, #e0e0e0);
      transition: background-color 0.3s ease;
    }

    @media (prefers-color-scheme: dark) {
      .stage-dot {
        background-color: var(--ha-border-color, #444444);
      }
    }

    .stage-dot.active {
      animation: pulse-dot 1s ease-in-out infinite;
    }

    @keyframes pulse-dot {
      0%,
      100% {
        background-color: var(--ha-primary-color, #03a9f4);
      }
      50% {
        background-color: var(--ha-border-color, #e0e0e0);
      }
    }

    .stage-dot.complete {
      background-color: var(--ha-primary-color, #03a9f4);
    }

    .stage-dot.error {
      background-color: var(--ha-error-color, #db4437);
    }

    .thinking-cloud {
      position: absolute;
      top: -110px;
      right: -175px;
      width: 200px;
      height: 120px;
      z-index: 1;
    }

    .thinking-cloud .cloud-text {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      color: white;
      font-size: 1.25rem;
      font-weight: 500;
      white-space: nowrap;
      z-index: 2;
    }

    .thinking-cloud .cloud-bump {
      position: absolute;
      background-color: #18bcf2;
      border-radius: 50%;
    }

    .thinking-cloud .bump-1 {
      width: 70px;
      height: 70px;
      top: 30px;
      left: 0;
    }

    .thinking-cloud .bump-2 {
      width: 80px;
      height: 80px;
      top: 0;
      left: 25px;
    }

    .thinking-cloud .bump-3 {
      width: 90px;
      height: 85px;
      top: -5px;
      left: 70px;
    }

    .thinking-cloud .bump-4 {
      width: 75px;
      height: 75px;
      top: 5px;
      right: 10px;
    }

    .thinking-cloud .bump-5 {
      width: 65px;
      height: 65px;
      top: 40px;
      right: 0;
    }

    .thinking-cloud .bump-6 {
      width: 70px;
      height: 70px;
      bottom: 0;
      right: 20px;
    }

    .thinking-cloud .bump-7 {
      width: 80px;
      height: 75px;
      bottom: -5px;
      left: 60px;
    }

    .thinking-cloud .bump-8 {
      width: 65px;
      height: 65px;
      bottom: 5px;
      left: 10px;
    }

    .thinking-cloud .bump-center {
      width: 140px;
      height: 80px;
      border-radius: 50%;
      top: 20px;
      left: 30px;
    }

    .stage-description {
      font-size: 0.9375rem;
      font-weight: 400;
      color: var(--ha-secondary-text-color, #727272);
      margin: 0 0 1.5rem 0;
      text-align: center;
    }
  `;

  @state()
  private _wizardState: WizardState = wizardState.getState();

  @state()
  private _stage: InstallStage = "downloading";

  @state()
  private _progress = 0;

  @state()
  private _bytesProcessed = 0;

  @state()
  private _totalBytes = 0;

  @state()
  private _error: string | null = null;

  @state()
  private _isInstalling = false;

  private _stageStartTime: number | null = null;
  private _stageStartBytes: number = 0;
  private _unsubscribe?: () => void;

  /** Whether the install operation has failed */
  get hasError(): boolean {
    return this._error !== null;
  }

  /** Retry the install operation */
  retry(): void {
    this._error = null;
    this._stage = "downloading";
    this._progress = 0;
    this._stageStartTime = null;
    this._stageStartBytes = 0;
    this._startInstall();
  }

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });

    this._startInstall();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  private async _startInstall() {
    if (this._isInstalling) return;

    this._isInstalling = true;
    this._error = null;

    const selections = this._wizardState.selections;
    const session = selections.proxmoxSession as ProxmoxSession | undefined;

    if (!session) {
      this._error = "No Proxmox session available";
      this._stage = "error";
      this._isInstalling = false;
      return;
    }

    const config: ProxmoxVmConfig = {
      node: (selections.proxmoxNode as string) || "pve",
      storage: (selections.proxmoxStorage as string) || "local",
      vm_id: (selections.proxmoxVmId as number) || 100,
      name: (selections.vmName as string) || "home-assistant",
      cpu_cores: (selections.cpuCores as number) || 4,
      memory_mb: (selections.memoryMb as number) || 4096,
      disk_size_gb: (selections.diskSizeGb as number) || 32,
      auto_start: true,
    };

    try {
      this._stage = "downloading";
      this._stageStartTime = Date.now();
      this._stageStartBytes = 0;

      const result = await proxmoxCreateVm(
        session,
        config,
        (progress: FlashProgress) => {
          // Use raw per-stage progress
          const newStage = progress.stage as InstallStage;
          if (newStage !== this._stage) {
            this._stage = newStage;
            this._stageStartTime = Date.now();
            this._stageStartBytes = progress.bytes_processed;
          }

          this._progress = progress.progress;
          this._bytesProcessed = progress.bytes_processed;
          this._totalBytes = progress.total_bytes;
        }
      );

      // Store result in wizard state
      wizardState.setSelection("proxmoxVmResult", result);
      if (result.ip_address) {
        wizardState.setSelection("ipAddress", result.ip_address);
      }

      // Complete
      this._stage = "complete";
      this._progress = 100;

      // Dispatch event to advance wizard
      this.dispatchEvent(
        new CustomEvent("install-complete", {
          bubbles: true,
          composed: true,
          detail: { result },
        })
      );
    } catch (error) {
      this._stage = "error";
      this._error =
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "Failed to create virtual machine";
      this.dispatchEvent(
        new CustomEvent("install-error", {
          bubbles: true,
          composed: true,
        })
      );
    } finally {
      this._isInstalling = false;
    }
  }

  render() {
    if (this._error) {
      return this._renderError();
    }

    return this._renderProgress();
  }

  private _renderProgress() {
    const stage = this._stage;
    const percentage = this._progress;

    const title = this._getStageTitle(stage);
    const description = this._getStageDescription(stage);

    // Add with-bubble class when showing thinking bubble (not complete or error)
    const hasBubble = stage !== "complete" && stage !== "error";

    // Use indeterminate for stages without measurable progress
    const isIndeterminate = this._isIndeterminate(stage);
    const hasMeasurable = this._hasMeasurableProgress(stage);

    return html`
      <div class="mascot-container ${hasBubble ? "with-bubble" : ""}">
        ${this._renderCasitaMascot(stage, title)}
      </div>

      <h2>${description}</h2>
      <p class="stage-description">
        Please keep this window open during installation
      </p>

      ${this._renderStagesIndicator(stage)}

      <div class="progress-section">
        <progress-bar
          .progress=${percentage}
          ?indeterminate=${isIndeterminate}
        ></progress-bar>
        <div class="progress-details">
          <div class="progress-left">
            <span class="bytes-info"
              >${hasMeasurable && this._totalBytes > 0
                ? `${formatBytes(this._bytesProcessed)} / ${formatBytes(this._totalBytes)}`
                : ""}</span
            >
            <span class="speed"
              >${hasMeasurable && this._totalBytes > 0
                ? this._calculateSpeed()
                : ""}</span
            >
          </div>
          <div class="progress-right">
            <span class="percentage"
              >${hasMeasurable ? `${percentage}%` : ""}</span
            >
            <span class="eta"
              >${hasMeasurable && this._totalBytes > 0
                ? this._calculateEta() || "Calculating..."
                : ""}</span
            >
          </div>
        </div>
      </div>
    `;
  }

  private _renderError() {
    return html`
      <div class="mascot-container">${this._renderCasitaSad()}</div>

      <h2>Installation failed</h2>

      <p class="error-message">${this._error}</p>
    `;
  }

  private _renderStagesIndicator(currentStage: string) {
    const stages = [
      { id: "downloading", label: "Downloading Home Assistant OS" },
      { id: "extracting", label: "Uploading image to Proxmox" },
      { id: "writing", label: "Creating virtual machine" },
      { id: "verifying", label: "Starting Home Assistant OS" },
      { id: "finalizing", label: "Waiting for network connection" },
      { id: "ready", label: "Waiting for Home Assistant" },
      { id: "updating", label: "Installing latest Home Assistant" },
    ];
    const currentIndex = stages.findIndex((s) => s.id === currentStage);
    const isComplete = currentStage === "complete";
    const isError = currentStage === "error";

    return html`
      <div class="stages-indicator">
        ${stages.map((stage, index) => {
          let stateClass = "";
          if (isError) {
            stateClass = index <= currentIndex ? "error" : "";
          } else if (isComplete || index < currentIndex) {
            stateClass = "complete";
          } else if (index === currentIndex) {
            stateClass = "active";
          }
          return html`<div
            class="stage-dot ${stateClass}"
            title=${stage.label}
          ></div>`;
        })}
      </div>
    `;
  }

  private _calculateEta(): string | null {
    if (!this._stageStartTime) return null;

    // No ETA for stages without byte tracking
    if (this._totalBytes === 0) return null;

    const elapsed = (Date.now() - this._stageStartTime) / 1000;
    const bytesInStage = this._bytesProcessed - this._stageStartBytes;

    if (elapsed < 1 || bytesInStage <= 0) return null;

    const bytesPerSecond = bytesInStage / elapsed;
    if (bytesPerSecond <= 0) return null;

    const remainingBytes = this._totalBytes - this._bytesProcessed;
    const remainingSeconds = remainingBytes / bytesPerSecond;

    if (remainingSeconds < 0 || !isFinite(remainingSeconds)) return null;

    if (remainingSeconds < 60) {
      return "Less than a minute remaining";
    } else if (remainingSeconds < 3600) {
      const minutes = Math.ceil(remainingSeconds / 60);
      return `About ${minutes} minute${minutes !== 1 ? "s" : ""} remaining`;
    } else {
      const hours = Math.floor(remainingSeconds / 3600);
      const minutes = Math.ceil((remainingSeconds % 3600) / 60);
      return `About ${hours}h ${minutes}m remaining`;
    }
  }

  private _calculateSpeed(): string {
    if (!this._stageStartTime) return "";

    // No speed for stages without byte tracking
    if (this._totalBytes === 0) return "";

    const elapsed = (Date.now() - this._stageStartTime) / 1000;
    const bytesInStage = this._bytesProcessed - this._stageStartBytes;

    if (elapsed < 0.5 || bytesInStage <= 0) return "";

    const bytesPerSecond = bytesInStage / elapsed;
    if (bytesPerSecond <= 0 || !isFinite(bytesPerSecond)) return "";

    return `${formatBytes(bytesPerSecond)}/s`;
  }

  /** Check if the current stage uses indeterminate progress */
  private _isIndeterminate(stage: InstallStage): boolean {
    return INDETERMINATE_STAGES.includes(stage);
  }

  /** Check if the current stage has measurable progress */
  private _hasMeasurableProgress(stage: InstallStage): boolean {
    return MEASURABLE_STAGES.includes(stage);
  }

  private _getStageTitle(stage: string): string {
    switch (stage) {
      case "downloading":
        return "Downloading";
      case "extracting":
        return "Uploading";
      case "writing":
        return "Creating";
      case "verifying":
        return "Starting";
      case "finalizing":
        return "Connecting";
      case "ready":
        return "Waiting";
      case "updating":
        return "Updating";
      case "complete":
        return "Complete!";
      case "error":
        return "Error";
      default:
        return "Installing";
    }
  }

  private _getStageDescription(stage: string): string {
    switch (stage) {
      case "downloading":
        return "Downloading Home Assistant OS";
      case "extracting":
        return "Uploading image to Proxmox";
      case "writing":
        return "Creating virtual machine";
      case "verifying":
        return "Starting Home Assistant OS";
      case "finalizing":
        return "Waiting for network connection";
      case "ready":
        return "Waiting for Home Assistant";
      case "updating":
        return "Installing latest Home Assistant (this can take up to 20 minutes)";
      case "complete":
        return "Installation complete!";
      default:
        return "Installing Home Assistant";
    }
  }

  private _renderCasitaMascot(stage: string, thinkingText: string) {
    switch (stage) {
      case "complete":
        return this._renderCasitaHappy();
      case "error":
        return this._renderCasitaSad();
      default:
        return this._renderCasitaThinking(thinkingText);
    }
  }

  private _renderCasitaThinking(text: string) {
    return html`
      <div style="position: relative; width: 100%; height: 100%;">
        ${svg`
          <svg class="casita-mascot" viewBox="0 -20 160 136.88" xmlns="http://www.w3.org/2000/svg">
            <!-- House body -->
            <path fill="#18bcf2" d="M120,109.38c0,4.12-3.38,7.5-7.5,7.5H7.5c-4.12,0-7.5-3.38-7.5-7.5v-45c0-4.12,2.39-9.89,5.3-12.8L54.7,2.19c2.92-2.92,7.69-2.92,10.61,0l49.39,49.39c2.92,2.92,5.3,8.68,5.3,12.8v45Z"/>
            <!-- Mouth with tongue (animated to the left and back) -->
            <path fill="#f2f4f9" d="M80,88.88c0-6.63-5.37-12-12-12s-12,5.37-12,12h24Z">
              <animateTransform attributeName="transform" type="translate" values="0,0;-6,0;0,0" dur="0.8s" begin="3.5s;tongueLick.end+4s" id="tongueLick"/>
            </path>
            <!-- Eyes (with blink animation) -->
            <ellipse fill="#f2f4f9" cx="33" cy="65.88" rx="8" ry="8">
              <animate attributeName="ry" values="8;1;8" dur="0.15s" begin="2.5s;blink1.end+3s" id="blink1"/>
            </ellipse>
            <ellipse fill="#f2f4f9" cx="87" cy="65.88" rx="8" ry="8">
              <animate attributeName="ry" values="8;1;8" dur="0.15s" begin="2.55s;blink2.end+3s" id="blink2"/>
            </ellipse>
            <!-- Nose/line -->
            <line fill="none" stroke="#f2f4f9" stroke-miterlimit="10" stroke-width="6" x1="40" y1="91.88" x2="80" y2="91.88"/>
            <!-- Thinking dots -->
            <circle fill="#18bcf2" cx="100" cy="25" r="5"/>
            <circle fill="#18bcf2" cx="120" cy="5" r="7"/>
            <circle fill="#18bcf2" cx="142" cy="-12" r="9"/>
          </svg>
        `}
        <div class="thinking-cloud">
          <div class="cloud-bump bump-center"></div>
          <div class="cloud-bump bump-1"></div>
          <div class="cloud-bump bump-2"></div>
          <div class="cloud-bump bump-3"></div>
          <div class="cloud-bump bump-4"></div>
          <div class="cloud-bump bump-5"></div>
          <div class="cloud-bump bump-6"></div>
          <div class="cloud-bump bump-7"></div>
          <div class="cloud-bump bump-8"></div>
          <div class="cloud-text">${text}...</div>
        </div>
      </div>
    `;
  }

  private _renderCasitaHappy() {
    return svg`
      <svg class="casita-mascot" viewBox="0 0 120 116.88" xmlns="http://www.w3.org/2000/svg">
        <path fill="#18bcf2" d="M120,109.38c0,4.12-3.38,7.5-7.5,7.5H7.5c-4.12,0-7.5-3.38-7.5-7.5v-45c0-4.12,2.39-9.89,5.3-12.8L54.7,2.19c2.92-2.92,7.69-2.92,10.61,0l49.39,49.39c2.92,2.92,5.3,8.68,5.3,12.8v45Z"/>
        <path fill="#f2f4f9" d="M80,80.88c0,11.05-8.95,20-20,20s-20-8.95-20-20h40Z"/>
        <circle fill="#f2f4f9" cx="33" cy="65.88" r="8"/>
        <circle fill="#f2f4f9" cx="87" cy="65.88" r="8"/>
      </svg>
    `;
  }

  private _renderCasitaSad() {
    return svg`
      <svg class="casita-mascot" viewBox="0 0 120 116.88" xmlns="http://www.w3.org/2000/svg">
        <path fill="#f7931e" d="M120,109.38c0,4.12-3.38,7.5-7.5,7.5H7.5c-4.12,0-7.5-3.38-7.5-7.5v-45c0-4.12,2.39-9.89,5.3-12.8L54.7,2.19c2.92-2.92,7.69-2.92,10.61,0l49.39,49.39c2.92,2.92,5.3,8.68,5.3,12.8v45Z"/>
        <!-- Big pleading eyes with blink -->
        <ellipse fill="#f2f4f9" cx="33" cy="65.88" rx="12" ry="12">
          <animate attributeName="ry" values="12;2;12" dur="0.15s" begin="2s;sadBlink1.end+3s" id="sadBlink1"/>
        </ellipse>
        <ellipse fill="#f2f4f9" cx="87" cy="65.88" rx="12" ry="12">
          <animate attributeName="ry" values="12;2;12" dur="0.15s" begin="2.05s;sadBlink2.end+3s" id="sadBlink2"/>
        </ellipse>
        <!-- Sad mouth -->
        <path fill="none" stroke="#f2f4f9" stroke-miterlimit="10" stroke-width="6" d="M44,96.88c0-8.84,7.16-16,16-16s16,7.16,16,16"/>
        <!-- Animated tears -->
        <path class="casita-tear" fill="#f2f4f9" d="M96.24,86.64c2.34,2.34,2.34,6.14,0,8.49s-6.14,2.34-8.49,0-2.34-6.14,0-8.49l4.24-4.24,4.24,4.24Z"/>
        <path class="casita-tear delay" fill="#f2f4f9" d="M32.24,86.64c2.34,2.34,2.34,6.14,0,8.49s-6.14,2.34-8.49,0-2.34-6.14,0-8.49l4.24-4.24,4.24,4.24Z"/>
      </svg>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "proxmox-progress-view": ProxmoxProgressView;
  }
}
