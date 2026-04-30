import { LitElement, html, css, svg } from "lit";
import { customElement, state } from "lit/decorators.js";
import { wizardState, type WizardState } from "../../state/wizard-state.js";
import { flashImage, formatBytes, type FlashProgress } from "../../api/index.js";
import "../../components/progress-bar.js";

@customElement("progress-view")
export class ProgressView extends LitElement {
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

    .cancel-link {
      display: block;
      margin-top: 1.5rem;
      font-size: 0.875rem;
      color: var(--ha-secondary-text-color, #727272);
      text-decoration: none;
      cursor: pointer;
    }

    .cancel-link:hover {
      text-decoration: underline;
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
      0%, 100% {
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

    /* Build cloud shape from overlapping circles - like a real thought bubble */
    /* Left bump */
    .thinking-cloud .bump-1 {
      width: 70px;
      height: 70px;
      top: 30px;
      left: 0;
    }

    /* Top left bump */
    .thinking-cloud .bump-2 {
      width: 80px;
      height: 80px;
      top: 0;
      left: 25px;
    }

    /* Top center bump */
    .thinking-cloud .bump-3 {
      width: 90px;
      height: 85px;
      top: -5px;
      left: 70px;
    }

    /* Top right bump */
    .thinking-cloud .bump-4 {
      width: 75px;
      height: 75px;
      top: 5px;
      right: 10px;
    }

    /* Right bump */
    .thinking-cloud .bump-5 {
      width: 65px;
      height: 65px;
      top: 40px;
      right: 0;
    }

    /* Bottom right bump */
    .thinking-cloud .bump-6 {
      width: 70px;
      height: 70px;
      bottom: 0;
      right: 20px;
    }

    /* Bottom center bump */
    .thinking-cloud .bump-7 {
      width: 80px;
      height: 75px;
      bottom: -5px;
      left: 60px;
    }

    /* Bottom left bump */
    .thinking-cloud .bump-8 {
      width: 65px;
      height: 65px;
      bottom: 5px;
      left: 10px;
    }

    /* Center fill */
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
  private _progress: FlashProgress | null = null;

  @state()
  private _error: string | null = null;

  @state()
  private _isFlashing = false;

  private _stageStartTime: number | null = null;
  private _stageStartBytes: number = 0;

  /** Whether the flash operation has failed */
  get hasError(): boolean {
    return this._error !== null;
  }

  /** Retry the flash operation */
  retry(): void {
    this._error = null;
    this._progress = null;
    this._stageStartTime = null;
    this._stageStartBytes = 0;
    this._startFlashing();
  }

  private _unsubscribe?: () => void;

  connectedCallback() {
    super.connectedCallback();
    this._unsubscribe = wizardState.subscribe((state) => {
      this._wizardState = state;
    });

    // Start flashing when view is connected
    this._startFlashing();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unsubscribe?.();
  }

  private async _startFlashing() {
    if (this._isFlashing) return;

    this._isFlashing = true;
    this._error = null;

    const selections = this._wizardState.selections;
    const driveId = selections.drive as string;
    const deviceConfig = selections.deviceConfig as { board: string } | undefined;

    if (!driveId || !deviceConfig) {
      this._error = "Missing drive or device configuration";
      this._isFlashing = false;
      return;
    }

    try {
      const result = await flashImage(
        {
          device_id: driveId,
          board: deviceConfig.board,
          verify: true,
        },
        (progress) => {
          // Track stage changes for ETA calculation
          const prevStage = this._progress?.stage;
          if (prevStage !== progress.stage) {
            this._stageStartTime = Date.now();
            this._stageStartBytes = progress.bytes_processed;
          }

          this._progress = progress;

          // If we've completed, advance to next step
          if (progress.stage === "complete") {
            this._onComplete();
          }
        }
      );

      if (!result.success) {
        this._setError(result.error || "Flash failed");
      }
    } catch (err) {
      // Tauri invoke errors come as strings, not Error objects
      const errorMessage = typeof err === "string"
        ? err
        : err instanceof Error
          ? err.message
          : "An unexpected error occurred";
      this._setError(errorMessage);
    } finally {
      this._isFlashing = false;
    }
  }

  private _setError(message: string) {
    // Provide user-friendly messages for specific error types
    if (message.toLowerCase().includes("disconnected")) {
      this._error = "The storage device was disconnected during the installation. Please reconnect it and try again.";
    } else {
      this._error = message;
    }
    this.dispatchEvent(
      new CustomEvent("flash-error", {
        bubbles: true,
        composed: true,
      })
    );
  }

  private _onComplete() {
    // Dispatch event to notify parent
    this.dispatchEvent(
      new CustomEvent("flash-complete", {
        bubbles: true,
        composed: true,
      })
    );
  }

  render() {
    if (this._error) {
      return this._renderError();
    }

    return this._renderProgress();
  }

  private _renderProgress() {
    const progress = this._progress;
    const stage = progress?.stage || "downloading";
    const percentage = progress?.progress || 0;
    const bytesProcessed = progress?.bytes_processed || 0;
    const totalBytes = progress?.total_bytes || 0;

    const title = this._getStageTitle(stage);
    const description = this._getStageDescription(stage);
    const eta = this._calculateEta();

    // Add with-bubble class when showing thinking bubble (not complete or error)
    const hasBubble = stage !== "complete" && stage !== "error";

    // Use indeterminate progress when total_bytes is 0 (e.g., during extraction)
    const isIndeterminate = totalBytes === 0 && stage !== "complete";

    return html`
      <div class="mascot-container ${hasBubble ? "with-bubble" : ""}">
        ${this._renderCasitaMascot(stage, title)}
      </div>

      <h2>${description}</h2>
      <p class="stage-description">Please keep this window open during installation</p>

      ${this._renderStagesIndicator(stage)}

      <div class="progress-section">
        <progress-bar .progress=${percentage} ?indeterminate=${isIndeterminate}></progress-bar>
        <div class="progress-details">
          <div class="progress-left">
            <span class="bytes-info">${isIndeterminate ? formatBytes(bytesProcessed) : (totalBytes > 0 ? `${formatBytes(bytesProcessed)} / ${formatBytes(totalBytes)}` : "")}</span>
            <span class="speed">${totalBytes > 0 ? this._calculateSpeed() : ""}</span>
          </div>
          <div class="progress-right">
            <span class="percentage">${isIndeterminate ? "" : `${percentage}%`}</span>
            <span class="eta">${totalBytes > 0 ? (eta ? `${eta} remaining` : "Calculating...") : ""}</span>
          </div>
        </div>
      </div>
    `;
  }

  private _renderError() {
    return html`
      <div class="mascot-container">
        ${this._renderCasitaSad()}
      </div>

      <h2>Installation failed</h2>

      <p class="error-message">${this._error}</p>
    `;
  }

  private _renderStagesIndicator(currentStage: string) {
    const stages = ["downloading", "extracting", "writing", "verifying", "finalizing"];
    const currentIndex = stages.indexOf(currentStage);
    const isComplete = currentStage === "complete";
    const isError = currentStage === "error";

    return html`
      <div class="stages-indicator">
        ${stages.map((_stage, index) => {
          let stateClass = "";
          if (isError) {
            stateClass = index <= currentIndex ? "error" : "";
          } else if (isComplete || index < currentIndex) {
            stateClass = "complete";
          } else if (index === currentIndex) {
            stateClass = "active";
          }
          return html`<div class="stage-dot ${stateClass}"></div>`;
        })}
      </div>
    `;
  }

  private _calculateEta(): string | null {
    if (!this._progress || !this._stageStartTime) return null;

    const { bytes_processed, total_bytes } = this._progress;

    // No ETA for stages without byte tracking
    if (total_bytes === 0) return null;

    const elapsed = (Date.now() - this._stageStartTime) / 1000; // seconds
    const bytesInStage = bytes_processed - this._stageStartBytes;

    // Need at least 1 second of data and some progress
    if (elapsed < 1 || bytesInStage <= 0) return null;

    const bytesPerSecond = bytesInStage / elapsed;
    if (bytesPerSecond <= 0) return null;

    const remainingBytes = total_bytes - bytes_processed;
    const remainingSeconds = remainingBytes / bytesPerSecond;

    if (remainingSeconds < 0 || !isFinite(remainingSeconds)) return null;

    // Format the time
    if (remainingSeconds < 60) {
      return "Less than a minute";
    } else if (remainingSeconds < 3600) {
      const minutes = Math.ceil(remainingSeconds / 60);
      return `About ${minutes} minute${minutes !== 1 ? "s" : ""}`;
    } else {
      const hours = Math.floor(remainingSeconds / 3600);
      const minutes = Math.ceil((remainingSeconds % 3600) / 60);
      return `About ${hours}h ${minutes}m`;
    }
  }

  private _calculateSpeed(): string {
    if (!this._progress || !this._stageStartTime) return "";

    const { bytes_processed, total_bytes } = this._progress;

    // No speed for stages without byte tracking
    if (total_bytes === 0) return "";

    const elapsed = (Date.now() - this._stageStartTime) / 1000; // seconds
    const bytesInStage = bytes_processed - this._stageStartBytes;

    // Need at least 0.5 seconds of data and some progress
    if (elapsed < 0.5 || bytesInStage <= 0) return "";

    const bytesPerSecond = bytesInStage / elapsed;
    if (bytesPerSecond <= 0 || !isFinite(bytesPerSecond)) return "";

    return `${formatBytes(bytesPerSecond)}/s`;
  }

  private _getStageTitle(stage: string): string {
    switch (stage) {
      case "downloading":
        return "Downloading";
      case "extracting":
        return "Extracting";
      case "writing":
        return "Writing";
      case "verifying":
        return "Verifying";
      case "finalizing":
        return "Finalizing";
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
        return "Fetching the Home Assistant image";
      case "extracting":
        return "Extracting the image";
      case "writing":
        return "Writing Home Assistant to your drive";
      case "verifying":
        return "Verifying the written data";
      case "finalizing":
        return "Finishing up the installation";
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
        // Use Focusing/Loading for active stages with thinking bubble
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
    // Pleading Casita with animated tears and blinking eyes
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
    "progress-view": ProgressView;
  }
}
