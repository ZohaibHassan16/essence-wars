// Tutorial state store using Svelte 5 runes with localStorage persistence

const STORAGE_KEY = 'essence-wars-tutorial';

// Types
export type TutorialPhase = 'inactive' | 'active' | 'completed';

export type AdvanceCondition =
  | { type: 'click_next' }
  | { type: 'action_performed'; actionType: string }
  | { type: 'card_played' }
  | { type: 'creature_attacked' }
  | { type: 'turn_ended' }
  | { type: 'auto'; delayMs: number };

export interface TutorialStep {
  id: string;
  title: string;
  message: string;
  targetElementId?: string;
  arrowDirection?: 'top' | 'bottom' | 'left' | 'right';
  advanceCondition: AdvanceCondition;
  allowedActions?: number[];
  blockOtherActions?: boolean;
}

interface TutorialProgress {
  currentStepIndex: number;
  completedSteps: string[];
  tutorialCompleted: boolean;
  startedAt: number;
  lastPlayedAt: number;
}

function loadProgress(): TutorialProgress | null {
  if (typeof localStorage === 'undefined') {
    return null;
  }
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        currentStepIndex: typeof parsed.currentStepIndex === 'number' ? parsed.currentStepIndex : 0,
        completedSteps: Array.isArray(parsed.completedSteps) ? parsed.completedSteps : [],
        tutorialCompleted: typeof parsed.tutorialCompleted === 'boolean' ? parsed.tutorialCompleted : false,
        startedAt: typeof parsed.startedAt === 'number' ? parsed.startedAt : Date.now(),
        lastPlayedAt: typeof parsed.lastPlayedAt === 'number' ? parsed.lastPlayedAt : Date.now(),
      };
    }
  } catch (e) {
    console.warn('Failed to load tutorial progress:', e);
  }
  return null;
}

function saveProgress(progress: TutorialProgress) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(progress));
  } catch (e) {
    console.warn('Failed to save tutorial progress:', e);
  }
}

class TutorialStore {
  // Core state
  private _phase = $state<TutorialPhase>('inactive');
  private _currentStepIndex = $state(0);
  private _steps = $state<TutorialStep[]>([]);
  private _progress = $state<TutorialProgress | null>(null);

  // UI state
  private _isOverlayVisible = $state(true);
  private _autoAdvanceTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this._progress = loadProgress();
    }
  }

  // Getters
  get phase() { return this._phase; }
  get currentStepIndex() { return this._currentStepIndex; }
  get steps() { return this._steps; }
  get progress() { return this._progress; }
  get isOverlayVisible() { return this._isOverlayVisible; }

  get currentStep(): TutorialStep | null {
    if (this._phase !== 'active' || this._currentStepIndex >= this._steps.length) {
      return null;
    }
    return this._steps[this._currentStepIndex];
  }

  get isActive(): boolean {
    return this._phase === 'active';
  }

  get progressPercent(): number {
    if (this._steps.length === 0) return 0;
    return Math.round((this._currentStepIndex / this._steps.length) * 100);
  }

  get hasCompletedTutorial(): boolean {
    return this._progress?.tutorialCompleted ?? false;
  }

  // Actions
  startTutorial(steps: TutorialStep[]) {
    this._steps = steps;
    this._currentStepIndex = 0;
    this._phase = 'active';
    this._isOverlayVisible = true;

    // Initialize or update progress
    const now = Date.now();
    this._progress = {
      currentStepIndex: 0,
      completedSteps: [],
      tutorialCompleted: false,
      startedAt: this._progress?.startedAt ?? now,
      lastPlayedAt: now,
    };
    this.persist();

    // Handle auto-advance for first step if needed
    this.setupAutoAdvance();
  }

  advanceStep() {
    if (this._phase !== 'active') return;

    this.clearAutoAdvance();

    const currentStep = this.currentStep;
    if (currentStep) {
      // Mark step as completed
      if (this._progress && !this._progress.completedSteps.includes(currentStep.id)) {
        this._progress.completedSteps.push(currentStep.id);
      }
    }

    this._currentStepIndex++;

    if (this._currentStepIndex >= this._steps.length) {
      this.completeTutorial();
    } else {
      if (this._progress) {
        this._progress.currentStepIndex = this._currentStepIndex;
        this._progress.lastPlayedAt = Date.now();
      }
      this.persist();
      this.setupAutoAdvance();
    }
  }

  goToStep(index: number) {
    if (index < 0 || index >= this._steps.length) return;
    this.clearAutoAdvance();
    this._currentStepIndex = index;
    if (this._progress) {
      this._progress.currentStepIndex = index;
    }
    this.persist();
    this.setupAutoAdvance();
  }

  checkAdvanceCondition(actionType?: string) {
    const step = this.currentStep;
    if (!step) return;

    const condition = step.advanceCondition;

    switch (condition.type) {
      case 'action_performed':
        if (actionType === condition.actionType) {
          this.advanceStep();
        }
        break;
      case 'card_played':
        if (actionType === 'play_card') {
          this.advanceStep();
        }
        break;
      case 'creature_attacked':
        if (actionType === 'attack') {
          this.advanceStep();
        }
        break;
      case 'turn_ended':
        if (actionType === 'end_turn') {
          this.advanceStep();
        }
        break;
      // 'click_next' and 'auto' are handled elsewhere
    }
  }

  isActionAllowed(actionIndex: number): boolean {
    const step = this.currentStep;
    if (!step || !step.blockOtherActions) return true;
    if (!step.allowedActions) return true;
    return step.allowedActions.includes(actionIndex);
  }

  toggleOverlay() {
    this._isOverlayVisible = !this._isOverlayVisible;
  }

  hideOverlay() {
    this._isOverlayVisible = false;
  }

  showOverlay() {
    this._isOverlayVisible = true;
  }

  skipTutorial() {
    this.clearAutoAdvance();
    this._phase = 'inactive';
    this._steps = [];
    this._currentStepIndex = 0;
    // Don't mark as completed when skipped
    this.persist();
  }

  completeTutorial() {
    this.clearAutoAdvance();
    this._phase = 'completed';

    if (this._progress) {
      this._progress.tutorialCompleted = true;
      this._progress.lastPlayedAt = Date.now();
    } else {
      this._progress = {
        currentStepIndex: this._steps.length,
        completedSteps: this._steps.map(s => s.id),
        tutorialCompleted: true,
        startedAt: Date.now(),
        lastPlayedAt: Date.now(),
      };
    }
    this.persist();
  }

  resetProgress() {
    this._phase = 'inactive';
    this._steps = [];
    this._currentStepIndex = 0;
    this._progress = null;
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(STORAGE_KEY);
    }
  }

  endTutorial() {
    this.clearAutoAdvance();
    this._phase = 'inactive';
    this._steps = [];
    this._currentStepIndex = 0;
  }

  // Private methods
  private setupAutoAdvance() {
    const step = this.currentStep;
    if (!step) return;

    if (step.advanceCondition.type === 'auto') {
      this._autoAdvanceTimer = setTimeout(() => {
        this.advanceStep();
      }, step.advanceCondition.delayMs);
    }
  }

  private clearAutoAdvance() {
    if (this._autoAdvanceTimer) {
      clearTimeout(this._autoAdvanceTimer);
      this._autoAdvanceTimer = null;
    }
  }

  private persist() {
    if (this._progress) {
      saveProgress(this._progress);
    }
  }
}

export const tutorialStore = new TutorialStore();
