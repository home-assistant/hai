import type { InstallationPath } from "../views/path-selection-view.js";

export type WizardFlow = InstallationPath;

export interface WizardStep {
  id: string;
  title: string;
}

export interface WizardSelections {
  device?: string;
  drive?: string;
  [key: string]: unknown;
}

export interface WizardState {
  currentFlow: WizardFlow | null;
  currentStepIndex: number;
  steps: WizardStep[];
  selections: WizardSelections;
}

type WizardStateListener = (state: WizardState) => void;

const FLOW_STEPS: Record<WizardFlow, WizardStep[]> = {
  sbc: [
    { id: "device", title: "Select Device" },
    { id: "drive", title: "Select Drive" },
    { id: "confirm", title: "Confirm" },
    { id: "flash", title: "Install" },
    { id: "success", title: "Done" },
  ],
  minipc: [
    { id: "method", title: "Installation Method" },
    { id: "architecture", title: "Select Architecture" },
    { id: "drive", title: "Select Drive" },
    { id: "confirm", title: "Confirm" },
    { id: "flash", title: "Install" },
    { id: "success", title: "Done" },
  ],
  "ha-hardware": [
    { id: "device", title: "Select Device" },
    { id: "connect", title: "Connect" },
    { id: "success", title: "Done" },
  ],
  proxmox: [
    { id: "connection", title: "Connect to Proxmox" },
    { id: "configure", title: "Configure VM" },
    { id: "confirm", title: "Confirm" },
    { id: "install", title: "Install" },
    { id: "success", title: "Done" },
  ],
  vm: [
    { id: "check", title: "Check Requirements" },
    { id: "configure", title: "Configure VM" },
    { id: "confirm", title: "Confirm" },
    { id: "install", title: "Install" },
    { id: "success", title: "Done" },
  ],
};

function createInitialState(): WizardState {
  return {
    currentFlow: null,
    currentStepIndex: 0,
    steps: [],
    selections: {},
  };
}

class WizardStateStore {
  private state: WizardState = createInitialState();
  private listeners: Set<WizardStateListener> = new Set();

  getState(): WizardState {
    return this.state;
  }

  subscribe(listener: WizardStateListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify() {
    this.listeners.forEach((listener) => listener(this.state));
  }

  startFlow(flow: WizardFlow) {
    this.state = {
      currentFlow: flow,
      currentStepIndex: 0,
      steps: FLOW_STEPS[flow] || [],
      selections: {},
    };
    this.notify();
  }

  nextStep() {
    if (this.state.currentStepIndex < this.state.steps.length - 1) {
      this.state = {
        ...this.state,
        currentStepIndex: this.state.currentStepIndex + 1,
      };
      this.notify();
    }
  }

  previousStep() {
    if (this.state.currentStepIndex > 0) {
      this.state = {
        ...this.state,
        currentStepIndex: this.state.currentStepIndex - 1,
      };
      this.notify();
    }
  }

  goToStep(index: number) {
    if (index >= 0 && index < this.state.steps.length) {
      this.state = {
        ...this.state,
        currentStepIndex: index,
      };
      this.notify();
    }
  }

  setSelection<K extends keyof WizardSelections>(
    key: K,
    value: WizardSelections[K]
  ) {
    this.state = {
      ...this.state,
      selections: {
        ...this.state.selections,
        [key]: value,
      },
    };
    this.notify();
  }

  reset() {
    this.state = createInitialState();
    this.notify();
  }

  get currentStep(): WizardStep | null {
    return this.state.steps[this.state.currentStepIndex] || null;
  }

  get isFirstStep(): boolean {
    return this.state.currentStepIndex === 0;
  }

  get isLastStep(): boolean {
    return this.state.currentStepIndex === this.state.steps.length - 1;
  }

  get progress(): number {
    if (this.state.steps.length === 0) return 0;
    return (this.state.currentStepIndex + 1) / this.state.steps.length;
  }
}

// Singleton instance
export const wizardState = new WizardStateStore();
