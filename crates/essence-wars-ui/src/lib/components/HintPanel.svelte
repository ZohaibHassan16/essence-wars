<script lang="ts">
  import type { AiHintResponse, ActionInfo } from "$lib/api/types";

  let {
    hint = null,
    isLoading = false,
    onRequestHint,
    onApplyHint,
  }: {
    hint: AiHintResponse | null;
    isLoading: boolean;
    onRequestHint: () => void;
    onApplyHint: (action: ActionInfo) => void;
  } = $props();

  function formatActionDescription(action: ActionInfo): string {
    switch (action.actionType) {
      case "play_card":
        return `Play card ${action.handIndex !== undefined ? `#${action.handIndex + 1}` : ""} to slot ${action.targetSlot}`;
      case "attack":
        return `Attack with slot ${action.sourceSlot} -> slot ${action.targetSlot}`;
      case "use_ability":
        return `Use ability from slot ${action.sourceSlot}${action.targetSlot !== undefined ? ` on slot ${action.targetSlot}` : ""}`;
      case "end_turn":
        return "End turn";
      default:
        return action.description;
    }
  }

  function getActionIcon(actionType: string): string {
    switch (actionType) {
      case "play_card":
        return "🃏";
      case "attack":
        return "⚔️";
      case "use_ability":
        return "✨";
      case "end_turn":
        return "⏭️";
      default:
        return "❓";
    }
  }

  function formatScore(score: number): string {
    if (score > 0) return `+${score.toFixed(1)}`;
    return score.toFixed(1);
  }
</script>

<div class="hint-panel" data-tutorial-id="ai-hint-panel">
  <div class="hint-header">
    <span class="hint-title">AI Hint</span>
    <button class="hint-button" onclick={onRequestHint} disabled={isLoading}>
      {#if isLoading}
        <span class="spinner"></span>
        Thinking...
      {:else}
        Get Hint
      {/if}
    </button>
  </div>

  {#if hint}
    <div class="hint-content">
      <div class="recommended-action">
        <div class="action-label">Recommended:</div>
        <button
          class="action-item recommended"
          onclick={() => onApplyHint(hint.recommendedAction)}
        >
          <span class="action-icon"
            >{getActionIcon(hint.recommendedAction.actionType)}</span
          >
          <span class="action-desc"
            >{formatActionDescription(hint.recommendedAction)}</span
          >
          <span class="action-score positive"
            >{formatScore(hint.score)}</span
          >
        </button>
      </div>

      {#if hint.alternatives.length > 0}
        <div class="alternatives">
          <div class="action-label">Alternatives:</div>
          {#each hint.alternatives.slice(0, 3) as alt}
            <button
              class="action-item alternative"
              onclick={() => onApplyHint(alt.action)}
            >
              <span class="action-icon">{getActionIcon(alt.action.actionType)}</span
              >
              <span class="action-desc">{formatActionDescription(alt.action)}</span>
              <span class="action-score" class:negative={alt.scoreDelta < 0}>
                {formatScore(alt.score)}
                <span class="delta">({formatScore(alt.scoreDelta)})</span>
              </span>
            </button>
          {/each}
        </div>
      {/if}

      <div class="hint-meta">
        Computed in {hint.thinkingTimeMs}ms
      </div>
    </div>
  {:else if !isLoading}
    <div class="hint-placeholder">
      Click "Get Hint" for AI suggestions
    </div>
  {/if}
</div>

<style>
  .hint-panel {
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 8px;
    font-size: 0.85rem;
  }

  .hint-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .hint-title {
    font-weight: 600;
    color: #a78bfa;
    font-size: 0.9rem;
  }

  .hint-button {
    background: linear-gradient(135deg, #7c3aed, #6d28d9);
    border: none;
    border-radius: 4px;
    padding: 4px 10px;
    color: white;
    font-size: 0.8rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s;
  }

  .hint-button:hover:not(:disabled) {
    background: linear-gradient(135deg, #8b5cf6, #7c3aed);
    transform: translateY(-1px);
  }

  .hint-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .hint-content {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .action-label {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 2px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .action-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 6px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    width: 100%;
    text-align: left;
    color: white;
  }

  .action-item:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .action-item.recommended {
    background: rgba(167, 139, 250, 0.15);
    border-color: rgba(167, 139, 250, 0.3);
  }

  .action-item.recommended:hover {
    background: rgba(167, 139, 250, 0.25);
    border-color: rgba(167, 139, 250, 0.5);
  }

  .action-icon {
    font-size: 0.9rem;
  }

  .action-desc {
    flex: 1;
    font-size: 0.75rem;
  }

  .action-score {
    font-family: monospace;
    font-size: 0.7rem;
    padding: 2px 4px;
    background: rgba(34, 197, 94, 0.2);
    color: #4ade80;
    border-radius: 3px;
  }

  .action-score.positive {
    background: rgba(34, 197, 94, 0.2);
    color: #4ade80;
  }

  .action-score.negative {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }

  .delta {
    font-size: 0.65rem;
    opacity: 0.7;
    margin-left: 2px;
  }

  .alternatives {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .hint-meta {
    font-size: 0.65rem;
    color: rgba(255, 255, 255, 0.4);
    text-align: right;
  }

  .hint-placeholder {
    color: rgba(255, 255, 255, 0.4);
    font-size: 0.8rem;
    text-align: center;
    padding: 8px 0;
  }
</style>
