<script lang="ts">
  import { gameStore } from "$lib/stores/gameState.svelte";
  import { gameSettings } from "$lib/stores/gameSettings.svelte";
  import { tutorialStore } from "$lib/stores/tutorialState.svelte";
  import BattlefieldRow from "./board/BattlefieldRow.svelte";
  import FanningHand from "./board/FanningHand.svelte";
  import CommanderCardLarge from "./board/CommanderCardLarge.svelte";
  import EssenceBar from "./board/EssenceBar.svelte";
  import CollapsibleSidebar from "./board/CollapsibleSidebar.svelte";
  import ActionLog from "./ActionLog.svelte";
  import HintPanel from "./HintPanel.svelte";
  import TurnTransition from "./TurnTransition.svelte";
  import TutorialOverlay from "./TutorialOverlay.svelte";
  import AudioControls from "./AudioControls.svelte";
  import CreatureActionMenu from "./CreatureActionMenu.svelte";
  import { playSound, playMusic } from "$lib/audio";

  // Play battle music when component mounts
  $effect(() => {
    playMusic('battle');
  });

  const gameState = $derived(gameStore.gameState);
  const isPlayerTurn = $derived(gameStore.isPlayerTurn);

  // Track turn changes for transition animation
  let lastActivePlayer: number | null = $state(null);
  let showTurnTransition = $state(false);

  $effect(() => {
    const currentPlayer = gameState?.activePlayer;
    if (currentPlayer !== undefined && lastActivePlayer !== null && currentPlayer !== lastActivePlayer) {
      // Turn changed, show transition
      showTurnTransition = true;
      setTimeout(() => {
        showTurnTransition = false;
      }, 1500);
    }
    lastActivePlayer = currentPlayer ?? null;
  });

  function handlePlayerCreatureClick(slot: number, event?: MouseEvent) {
    if (!isPlayerTurn) return;

    const action = gameStore.getActionForTarget(slot);
    if (action) {
      // Check tutorial restrictions
      if (tutorialStore.isActive && !tutorialStore.isActionAllowed(action.index)) {
        return;
      }
      // Play ability sound if this is an ability action
      if (action.actionType === "use_ability") {
        playSound('abilityActivate');
      }
      gameStore.applyAction(action.index);
      tutorialStore.checkAdvanceCondition(action.actionType);
    } else if (gameStore.selectedCardIndex === null) {
      // Select creature for attack or ability
      const creature = gameState?.player.creatures[slot];
      if (creature) {
        const hasActions = gameStore.hasAttackActions(slot) || gameStore.hasAbilityActions(slot);
        if (hasActions) {
          gameStore.selectCreature(slot, event);
        }
      }
    }
  }

  function handlePlayerSupportClick(slot: number) {
    if (!isPlayerTurn) return;
    const action = gameStore.getActionForTarget(slot);
    if (action) {
      if (tutorialStore.isActive && !tutorialStore.isActionAllowed(action.index)) {
        return;
      }
      gameStore.applyAction(action.index);
      tutorialStore.checkAdvanceCondition(action.actionType);
    }
  }

  function handleOpponentCreatureClick(slot: number) {
    if (!isPlayerTurn) return;

    const action = gameStore.getActionForTarget(slot);
    if (action) {
      if (tutorialStore.isActive && !tutorialStore.isActionAllowed(action.index)) {
        return;
      }
      gameStore.applyAction(action.index);
      tutorialStore.checkAdvanceCondition(action.actionType);
    }
  }

  function handleCardClick(index: number) {
    if (!isPlayerTurn) return;
    gameStore.selectCard(index);
  }

  function isCardPlayable(index: number): boolean {
    return gameStore.legalActions.some(
      a => a.actionType === "play_card" && a.handIndex === index
    );
  }

  // Check if an opponent slot is a valid attack target
  function getValidAttackTargets(): number[] {
    if (gameStore.selectedCreatureSlot === null) return [];
    return gameStore.highlightedSlots;
  }

  // Get highlighted slots for card placement
  function getHighlightedSlots(): number[] {
    if (gameStore.selectedCardIndex === null) return [];
    return gameStore.highlightedSlots;
  }

  // Convert LoggedAction to ActionInfo for the log
  const actionsForLog = $derived(
    gameStore.actionHistory.map(a => ({
      index: a.index,
      actionType: a.actionType,
      description: `[P${a.player}] ${a.description}`,
      sourceSlot: a.sourceSlot,
      targetSlot: a.targetSlot,
      handIndex: a.handIndex,
      cardId: a.cardId,
    }))
  );

  // Derive player faction from first available card (hand or board)
  const playerFaction = $derived(() => {
    const state = gameState;
    if (!state) return "default";

    // Check hand first
    for (const card of state.player.hand) {
      if (card.faction && card.faction !== "unknown") {
        return card.faction;
      }
    }
    // Check creatures
    for (const creature of state.player.creatures) {
      if (creature?.faction && creature.faction !== "neutral") {
        return creature.faction;
      }
    }
    return "default";
  });

  const boardBgClass = $derived(`board-bg-${playerFaction()}`);

  // Show insight indicator when approaching eligibility (Turn >= 8, low hand, has essence)
  // This hints that Commander's Insight could become available soon
  const showInsightIndicator = $derived(() => {
    if (!gameState) return false;
    const turn = gameState.turn ?? 0;
    const handSize = gameState.player.hand?.length ?? 10;
    const essence = gameState.player.essence ?? 0;
    return turn >= 8 && handSize <= 3 && essence >= 4;
  });
</script>

<!-- Turn transition overlay -->
<TurnTransition
  isYourTurn={isPlayerTurn}
  turnNumber={gameState?.turn ?? 1}
  visible={showTurnTransition}
/>

<!-- Error banner -->
{#if gameStore.error}
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 max-w-lg">
    <div class="p-4 bg-damage/90 border border-damage rounded-lg text-white shadow-xl">
      <div class="flex items-start gap-3">
        <div class="flex-1">
          <p class="font-semibold">Error</p>
          <p class="text-sm mt-1">{gameStore.error}</p>
        </div>
        <div class="flex gap-2">
          {#if gameStore.aiTurnFailed}
            <button
              class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors"
              onclick={() => gameStore.retryAiTurn()}
            >
              Retry
            </button>
          {/if}
          <button
            class="px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-sm font-medium transition-colors"
            onclick={() => gameStore.clearError()}
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<div class="w-full h-full flex {boardBgClass}">
  <!-- LEFT COLUMN: Commander Cards -->
  <div class="flex flex-col justify-between p-2 bg-ui-panel/30 border-r border-gray-700/50 overflow-visible"
       style="width: var(--commander-card-width, 250px);">
    <!-- Opponent Commander (top) -->
    <div class="flex flex-col items-center overflow-visible">
      <div class="flex items-stretch gap-2">
        <!-- Opponent Essence Bar -->
        <EssenceBar
          current={gameState?.opponent.essence ?? 0}
          max={gameState?.opponent.maxEssence ?? 0}
          isPlayer={false}
        />
        <CommanderCardLarge
          commander={gameState?.opponent.commander ?? null}
          life={gameState?.opponent.life ?? 0}
          maxLife={gameState?.opponent.maxLife ?? 30}
          essence={gameState?.opponent.essence ?? 0}
          maxEssence={gameState?.opponent.maxEssence ?? 0}
          isActive={!isPlayerTurn}
          isPlayer={false}
          tutorialId="opponent-commander"
          isValidFaceTarget={gameStore.canTargetFace}
          onFaceTargetClick={() => gameStore.executeAbilityOnFace()}
        />
      </div>
      <!-- Opponent compact stats below commander -->
      <div class="mt-2 flex items-center gap-3 text-xs text-ui-text-dim">
        <span title="Cards in hand">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          {gameState?.opponent.hand.length ?? 0}
        </span>
        <span title="Cards in deck">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          {gameState?.opponent.deckCount ?? 0}
        </span>
        <span title="Action Points" class="text-gold">
          AP: {gameState?.opponent.actionPoints ?? 0}
        </span>
      </div>
    </div>

    <!-- Player Commander (bottom) -->
    <div class="flex flex-col items-center overflow-visible">
      <!-- Player compact stats above commander -->
      <div class="mb-2 flex items-center gap-3 text-xs text-ui-text-dim">
        <span title="Cards in deck">
          <svg class="w-3.5 h-3.5 inline mr-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          {gameState?.player.deckCount ?? 0}
        </span>
        <span title="Action Points" class="text-gold">
          AP: {gameState?.player.actionPoints ?? 0}
        </span>
      </div>
      <div class="flex items-stretch gap-2">
        <!-- Player Essence Bar -->
        <EssenceBar
          current={gameState?.player.essence ?? 0}
          max={gameState?.player.maxEssence ?? 0}
          isPlayer={true}
        />
        <CommanderCardLarge
          commander={gameState?.player.commander ?? null}
          life={gameState?.player.life ?? 0}
          maxLife={gameState?.player.maxLife ?? 30}
          essence={gameState?.player.essence ?? 0}
          maxEssence={gameState?.player.maxEssence ?? 0}
          isActive={isPlayerTurn}
          isPlayer={true}
          insightAvailable={gameStore.insightAvailable && isPlayerTurn}
          insightIndicator={showInsightIndicator() && !gameStore.insightAvailable}
          onInsightClick={() => gameStore.applyCommanderInsight()}
          tutorialId="player-commander"
        />
      </div>
    </div>
  </div>

  <!-- CENTER COLUMN: Main Game Area -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Opponent hand (hidden cards, compact) -->
    <div class="bg-gray-900/30 py-1 border-b border-gray-700/30">
      <FanningHand
        cards={gameState?.opponent.hand ?? []}
        compact={true}
        previewPosition="bottom"
        tutorialId="opponent-hand"
      />
    </div>

    <!-- Main board area -->
    <div class="flex-1 flex flex-col justify-center px-4 py-2" style="gap: var(--board-gap);">
      <!-- Opponent's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.opponent.creatures ?? [null, null, null, null, null]}
        supports={gameState?.opponent.supports ?? [null, null]}
        isPlayerSide={false}
        validAttackTargets={getValidAttackTargets()}
        onCreatureClick={handleOpponentCreatureClick}
      />

      <!-- Center turn indicator -->
      <div class="flex items-center justify-center gap-2">
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
        <div class="px-4 py-1.5 rounded-full border border-gray-600 bg-ui-panel/80 flex items-center gap-2">
          <span class="text-ui-text-dim text-sm">Turn</span>
          <span class="text-ui-text font-bold text-lg">{gameState?.turn ?? 0}</span>
          <div class="w-px h-4 bg-gray-600"></div>
          <span class="text-sm font-semibold {isPlayerTurn ? 'text-health' : 'text-ui-action'}">
            {isPlayerTurn ? "Your Turn" : "AI Turn"}
          </span>
        </div>
        <div class="h-px flex-1 bg-gradient-to-r from-transparent via-gray-600 to-transparent"></div>
      </div>

      <!-- Player's battlefield row -->
      <BattlefieldRow
        creatures={gameState?.player.creatures ?? [null, null, null, null, null]}
        supports={gameState?.player.supports ?? [null, null]}
        isPlayerSide={true}
        selectedCreatureSlot={gameStore.selectedCreatureSlot}
        highlightedSlots={getHighlightedSlots()}
        onCreatureClick={handlePlayerCreatureClick}
        onSupportClick={handlePlayerSupportClick}
        showKeyHints={isPlayerTurn}
      />
    </div>

    <!-- Player hand (interactive, with fanning) -->
    <div class="bg-gray-900/30 py-2 border-t border-gray-700/30">
      <FanningHand
        cards={gameState?.player.hand ?? []}
        selectedCardIndex={gameStore.selectedCardIndex}
        isInteractive={true}
        isPlayableCallback={isCardPlayable}
        onCardClick={handleCardClick}
        showKeyHints={isPlayerTurn}
        tutorialId="player-hand"
      />
    </div>

    <!-- Action bar -->
    <div class="flex items-center justify-center gap-3 px-4 py-2 bg-ui-panel/50 border-t border-gray-700">
      {#if isPlayerTurn}
        <button
          class="px-4 py-2 bg-amber-700 text-white rounded-lg font-semibold
                 hover:bg-amber-600 active:scale-95 transition-all
                 disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={() => {
            playSound('buttonClick');
            gameStore.undoAction();
          }}
          onmouseenter={() => playSound('buttonHover')}
          disabled={gameStore.isLoading || gameStore.actionHistory.length === 0}
          title="Undo last action (dev mode)"
        >
          Undo
        </button>
        <button
          data-tutorial-id="end-turn-btn"
          class="px-6 py-2 bg-ui-action text-white rounded-lg font-semibold
                 hover:bg-ui-action/80 active:scale-95 transition-all
                 disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={() => {
            playSound('buttonClick');
            gameStore.endTurn();
            tutorialStore.checkAdvanceCondition('end_turn');
          }}
          onmouseenter={() => playSound('buttonHover')}
          disabled={gameStore.isLoading}
        >
          End Turn{#if gameSettings.showKeyboardHints}<span class="ml-2 text-xs opacity-70">(Space)</span>{/if}
        </button>
      {:else}
        <div class="px-5 py-2 bg-gray-700 text-ui-text-dim rounded-lg font-semibold flex items-center gap-2">
          <div class="w-4 h-4 border-2 border-ui-text-dim border-t-transparent rounded-full animate-spin"></div>
          AI Thinking...
        </div>
      {/if}
      <button
        class="px-4 py-2 bg-gray-700 text-ui-text rounded-lg hover:bg-gray-600 transition-colors"
        onclick={() => {
          playSound('buttonClick');
          gameStore.quitGame();
        }}
        onmouseenter={() => playSound('buttonHover')}
      >
        Quit
      </button>
      <AudioControls />
    </div>
  </div>

  <!-- RIGHT COLUMN: Sidebar -->
  <CollapsibleSidebar>
    <!-- AI Hint Panel (only when player's turn) -->
    {#if isPlayerTurn}
      <div class="p-2 border-b border-gray-700">
        <HintPanel
          hint={gameStore.currentHint}
          isLoading={gameStore.isHintLoading}
          onRequestHint={() => gameStore.requestHint()}
          onApplyHint={(action) => gameStore.applyHint(action)}
        />
      </div>
    {/if}

    <!-- Action Log -->
    <div class="flex-1 p-2 overflow-hidden">
      <ActionLog actions={actionsForLog} />
    </div>
  </CollapsibleSidebar>
</div>

<!-- Creature Action Menu (Attack/Ability selection) -->
{#if gameStore.showActionMenu && gameStore.actionMenuPosition && gameStore.selectedCreatureSlot !== null}
  {@const selectedCreature = gameState?.player.creatures[gameStore.selectedCreatureSlot]}
  {#if selectedCreature}
    <CreatureActionMenu
      creature={selectedCreature}
      position={gameStore.actionMenuPosition}
    />
  {/if}
{/if}

<!-- Tutorial overlay -->
<TutorialOverlay />
