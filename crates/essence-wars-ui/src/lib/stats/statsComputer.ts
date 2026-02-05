/**
 * Match Statistics Computer
 *
 * Computes comprehensive statistics from match data including actions,
 * events, and state snapshots.
 */

import type {
  SpectatorMatch,
  SpectatorAction,
  GameStateDto,
  SpectatorResult as _SpectatorResult,
  GameEventDto as _GameEventDto,
} from "$lib/api/types";

import type {
  MatchStatistics,
  OverviewStats,
  ActionStats,
  CombatStats,
  ResourceStats,
  KeywordStats,
  CardPerformanceStats,
  AiAnalysisStats,
  TimelineData,
  CriticalMoment,
  CardPlayStats,
  CardImpact,
} from "./types";

// =============================================================================
// Main Computation Function
// =============================================================================

/**
 * Compute comprehensive statistics from a completed match.
 */
export function computeMatchStatistics(match: SpectatorMatch): MatchStatistics {
  const { initialState, actions, result: _result } = match;

  // Compute all statistics
  const actionEconomy = {
    player1: computeActionStats(actions, 1),
    player2: computeActionStats(actions, 2),
  };

  const combat = {
    player1: computeCombatStats(actions, 1),
    player2: computeCombatStats(actions, 2),
  };

  const resources = {
    player1: computeResourceStats(actions, initialState, 1),
    player2: computeResourceStats(actions, initialState, 2),
  };

  const keywords = {
    player1: computeKeywordStats(actions, 1),
    player2: computeKeywordStats(actions, 2),
  };

  const cardPerformance = {
    player1: computeCardPerformanceStats(actions, 1),
    player2: computeCardPerformanceStats(actions, 2),
  };

  const timeline = computeTimelineData(initialState, actions);
  const aiAnalysis = computeAiAnalysis(actions);
  const overview = computeOverviewStats(match, cardPerformance, timeline);

  return {
    overview,
    actionEconomy,
    combat,
    resources,
    keywords,
    cardPerformance,
    aiAnalysis,
    timeline,
  };
}

// =============================================================================
// Overview Statistics
// =============================================================================

function computeOverviewStats(
  match: SpectatorMatch,
  cardPerformance: { player1: CardPerformanceStats; player2: CardPerformanceStats },
  _timeline: TimelineData
): OverviewStats {
  const { actions, result } = match;

  // Find MVP card (highest impact across both players)
  const mvp1 = cardPerformance.player1.mvpCard;
  const mvp2 = cardPerformance.player2.mvpCard;
  let mvpCard: CardImpact | null = null;
  if (mvp1 && mvp2) {
    mvpCard = mvp1.impact >= mvp2.impact ? mvp1 : mvp2;
  } else {
    mvpCard = mvp1 || mvp2;
  }

  // Calculate total game duration from thinking times
  const gameDurationMs = actions.reduce((sum, a) => sum + a.thinkingTimeMs, 0);

  return {
    winner: result.winner,
    reason: result.reason,
    totalTurns: match.totalTurns,
    totalActions: actions.length,
    gameDurationMs,
    lifeDifferential: result.player1FinalLife - result.player2FinalLife,
    player1FinalLife: result.player1FinalLife,
    player2FinalLife: result.player2FinalLife,
    mvpCard,
  };
}

// =============================================================================
// Action Economy Statistics
// =============================================================================

function computeActionStats(actions: SpectatorAction[], player: 1 | 2): ActionStats {
  const playerActions = actions.filter(a => a.player === player);

  let creaturesPlayed = 0;
  let spellsCast = 0;
  let supportsPlaced = 0;
  let attacksMade = 0;
  let faceAttacks = 0;
  let creatureAttacks = 0;
  let abilitiesUsed = 0;
  let apSpent = 0;
  let apAvailable = 0;

  // Track turns played (count end_turn actions or unique turn numbers)
  const turnsPlayed = new Set(playerActions.map(a => a.turn)).size;

  for (const action of playerActions) {
    const actionType = action.action.actionType;

    switch (actionType) {
      case "play_card": {
        // Check events to determine card type
        const hasCreatureSpawn = action.events.some(e => e.eventType === "creature_spawned");
        const hasSupportPlaced = action.events.some(e => e.eventType === "support_placed");

        if (hasCreatureSpawn) {
          creaturesPlayed++;
        } else if (hasSupportPlaced) {
          supportsPlaced++;
        } else {
          spellsCast++;
        }
        apSpent++; // Playing a card costs 1 AP
        break;
      }

      case "attack": {
        attacksMade++;
        // Check if it was a face attack or creature attack
        const targetSlot = action.action.targetSlot;
        if (targetSlot === 5) {
          // Face attack (slot 5 typically represents face)
          faceAttacks++;
        } else {
          creatureAttacks++;
        }
        apSpent++; // Attacks cost 1 AP
        break;
      }

      case "ability": {
        abilitiesUsed++;
        apSpent++;
        break;
      }

      case "end_turn":
        // End turn - calculate AP that was available this turn
        // We can estimate from state if available
        break;
    }
  }

  // Estimate AP available (3 AP per turn in standard rules)
  apAvailable = turnsPlayed * 3;

  const apEfficiency = apAvailable > 0 ? (apSpent / apAvailable) * 100 : 0;

  return {
    cardsPlayed: creaturesPlayed + spellsCast + supportsPlaced,
    creaturesPlayed,
    spellsCast,
    supportsPlaced,
    attacksMade,
    faceAttacks,
    creatureAttacks,
    abilitiesUsed,
    turnsPlayed,
    apSpent,
    apAvailable,
    apEfficiency: Math.round(apEfficiency * 10) / 10,
  };
}

// =============================================================================
// Combat Statistics
// =============================================================================

function computeCombatStats(actions: SpectatorAction[], player: 1 | 2): CombatStats {
  let totalDamageDealt = 0;
  let damageToCreatures = 0;
  let damageToFace = 0;
  let creaturesKilled = 0;
  let creaturesLost = 0;
  let favorableTrades = 0;
  let evenTrades = 0;
  let unfavorableTrades = 0;

  const opponent = player === 1 ? 2 : 1;

  for (const action of actions) {
    for (const event of action.events) {
      const data = event.data as Record<string, unknown>;

      switch (event.eventType) {
        case "life_changed": {
          const eventPlayer = data.player as number;
          const oldLife = data.old as number;
          const newLife = data.new as number;

          // If opponent's life decreased, we dealt face damage
          if (eventPlayer === opponent && newLife < oldLife) {
            const damage = oldLife - newLife;
            // Only count as "our" damage if it was from our action
            if (action.player === player) {
              damageToFace += damage;
              totalDamageDealt += damage;
            }
          }
          break;
        }

        case "creature_died": {
          const eventPlayer = data.player as number;

          if (eventPlayer === opponent && action.player === player) {
            // We killed an enemy creature
            creaturesKilled++;
          } else if (eventPlayer === player) {
            // Our creature died
            creaturesLost++;
          }
          break;
        }

        case "creature_damaged": {
          const eventPlayer = data.player as number;
          const damage = (data.damage as number) || 0;

          // If we damaged an enemy creature
          if (eventPlayer === opponent && action.player === player) {
            damageToCreatures += damage;
            totalDamageDealt += damage;
          }
          break;
        }
      }
    }

    // Analyze combat trades for attack actions
    if (action.player === player && action.action.actionType === "attack") {
      const attackerDied = action.events.some(
        e => e.eventType === "creature_died" &&
             (e.data as Record<string, unknown>).player === player
      );
      const defenderDied = action.events.some(
        e => e.eventType === "creature_died" &&
             (e.data as Record<string, unknown>).player === opponent
      );

      if (defenderDied && !attackerDied) {
        favorableTrades++;
      } else if (defenderDied && attackerDied) {
        evenTrades++;
      } else if (!defenderDied && attackerDied) {
        unfavorableTrades++;
      }
    }
  }

  const kdRatio = creaturesLost > 0 ? creaturesKilled / creaturesLost : creaturesKilled;

  return {
    totalDamageDealt,
    damageToCreatures,
    damageToFace,
    creaturesKilled,
    creaturesLost,
    kdRatio: Math.round(kdRatio * 100) / 100,
    favorableTrades,
    evenTrades,
    unfavorableTrades,
  };
}

// =============================================================================
// Resource Economy Statistics
// =============================================================================

function computeResourceStats(
  actions: SpectatorAction[],
  initialState: GameStateDto,
  player: 1 | 2
): ResourceStats {
  let totalEssenceSpent = 0;
  let cardsDrawn = 0;
  let cardsDiscarded = 0;
  let totalHandSize = 0;
  let handSizeCount = 0;
  let maxHandSize = 0;
  let emptyHandTurns = 0;
  let turnsPlayed = 0;

  // Get initial hand size
  const initialHandSize = player === 1
    ? initialState.player.hand.length
    : initialState.opponent.hand.length;
  maxHandSize = initialHandSize;

  // Track hand size through the game
  for (const action of actions) {
    const state = action.stateAfter;
    const playerState = player === 1 ? state.player : state.opponent;

    // Track hand size at each action
    const handSize = playerState.hand.length;
    totalHandSize += handSize;
    handSizeCount++;
    maxHandSize = Math.max(maxHandSize, handSize);

    // Check for card played events to track essence spent
    for (const event of action.events) {
      const data = event.data as Record<string, unknown>;

      switch (event.eventType) {
        case "card_drawn":
          if ((data.player as number) === player) {
            cardsDrawn++;
          }
          break;

        case "card_discarded":
          if ((data.player as number) === player) {
            cardsDiscarded++;
          }
          break;

        case "essence_changed":
          if ((data.player as number) === player) {
            const oldEssence = data.old as number;
            const newEssence = data.new as number;
            if (newEssence < oldEssence) {
              totalEssenceSpent += oldEssence - newEssence;
            }
          }
          break;
      }
    }

    // Track empty hand turns
    if (action.player === player && action.action.actionType === "end_turn") {
      turnsPlayed++;
      if (handSize === 0) {
        emptyHandTurns++;
      }
    }
  }

  const avgEssencePerTurn = turnsPlayed > 0 ? totalEssenceSpent / turnsPlayed : 0;
  const averageHandSize = handSizeCount > 0 ? totalHandSize / handSizeCount : 0;

  return {
    totalEssenceSpent,
    avgEssencePerTurn: Math.round(avgEssencePerTurn * 10) / 10,
    cardsDrawn,
    cardsDiscarded,
    averageHandSize: Math.round(averageHandSize * 10) / 10,
    maxHandSize,
    emptyHandTurns,
  };
}

// =============================================================================
// Keyword Statistics
// =============================================================================

function computeKeywordStats(actions: SpectatorAction[], player: 1 | 2): KeywordStats {
  const stats: KeywordStats = {
    rushAttacks: 0,
    guardBlocks: 0,
    lethalKills: 0,
    lifestealHealing: 0,
    piercingDamage: 0,
    shieldAbsorbed: 0,
    quickStrikes: 0,
    fortifyReduced: 0,
    wardBlocks: 0,
    rangedAttacks: 0,
    stealthEvades: 0,
    regenerateHealing: 0,
    volatileDamage: 0,
    frenzyStacks: 0,
    chargeBonus: 0,
  };

  for (const action of actions) {
    for (const event of action.events) {
      const data = event.data as Record<string, unknown>;

      // Check for keyword_activated events
      if (event.eventType === "keyword_activated") {
        const eventPlayer = data.player as number;
        if (eventPlayer !== player) continue;

        const keyword = data.keyword as string;
        const value = (data.value as number) || 0;

        switch (keyword) {
          case "Rush":
            stats.rushAttacks++;
            break;
          case "Guard":
            stats.guardBlocks++;
            break;
          case "Lethal":
            stats.lethalKills++;
            break;
          case "Lifesteal":
            stats.lifestealHealing += value;
            break;
          case "Piercing":
            stats.piercingDamage += value;
            break;
          case "Shield":
            stats.shieldAbsorbed += value;
            break;
          case "Quick":
            stats.quickStrikes++;
            break;
          case "Fortify":
            stats.fortifyReduced += value;
            break;
          case "Ward":
            stats.wardBlocks++;
            break;
          case "Ranged":
            stats.rangedAttacks++;
            break;
          case "Stealth":
            stats.stealthEvades++;
            break;
          case "Regenerate":
            stats.regenerateHealing += value;
            break;
          case "Volatile":
            stats.volatileDamage += value;
            break;
          case "Frenzy":
            stats.frenzyStacks += value;
            break;
          case "Charge":
            stats.chargeBonus += value;
            break;
        }
      }

      // Infer Lifesteal from life_changed events with source
      if (event.eventType === "life_changed") {
        const eventPlayer = data.player as number;
        const source = data.source as string;
        const oldLife = data.old as number;
        const newLife = data.new as number;

        if (eventPlayer === player && source === "Lifesteal" && newLife > oldLife) {
          stats.lifestealHealing += newLife - oldLife;
        }

        if (eventPlayer !== player && source === "Piercing" && newLife < oldLife) {
          // This player dealt piercing damage
          if (action.player === player) {
            stats.piercingDamage += oldLife - newLife;
          }
        }
      }
    }
  }

  return stats;
}

// =============================================================================
// Card Performance Statistics
// =============================================================================

function computeCardPerformanceStats(
  actions: SpectatorAction[],
  player: 1 | 2
): CardPerformanceStats {
  const cardStats = new Map<number, CardPlayStats>();
  const opponent = player === 1 ? 2 : 1;

  // Track card plays and performance
  for (const action of actions) {
    if (action.player !== player) continue;

    const cardId = action.action.cardId;
    if (!cardId) continue;

    // Get or create card stats
    let stats = cardStats.get(cardId);
    if (!stats) {
      stats = {
        cardId,
        name: `Card ${cardId}`, // Will be resolved from card database later
        timesPlayed: 0,
        totalDamageDealt: 0,
        creaturesKilled: 0,
        survivalTurns: 0,
      };
      cardStats.set(cardId, stats);
    }

    if (action.action.actionType === "play_card") {
      stats.timesPlayed++;
    }

    // Track damage and kills from this action
    for (const event of action.events) {
      const data = event.data as Record<string, unknown>;

      if (event.eventType === "creature_died" && (data.player as number) === opponent) {
        stats.creaturesKilled++;
      }

      if (event.eventType === "creature_damaged" && (data.player as number) === opponent) {
        stats.totalDamageDealt += (data.damage as number) || 0;
      }

      if (event.eventType === "life_changed" && (data.player as number) === opponent) {
        const oldLife = data.old as number;
        const newLife = data.new as number;
        if (newLife < oldLife) {
          stats.totalDamageDealt += oldLife - newLife;
        }
      }
    }
  }

  // Find MVP, most played, highest damage, most kills
  let mvpCard: CardImpact | null = null;
  let mostPlayed: { cardId: number; name: string; count: number } | null = null;
  let highestDamage: { cardId: number; name: string; damage: number } | null = null;
  let mostKills: { cardId: number; name: string; kills: number } | null = null;

  for (const [_, stats] of cardStats) {
    // Calculate impact score (weighted combination)
    const impact = stats.totalDamageDealt * 1 + stats.creaturesKilled * 10 + stats.survivalTurns * 2;

    if (!mvpCard || impact > mvpCard.impact) {
      mvpCard = { cardId: stats.cardId, name: stats.name, impact };
    }

    if (!mostPlayed || stats.timesPlayed > mostPlayed.count) {
      mostPlayed = { cardId: stats.cardId, name: stats.name, count: stats.timesPlayed };
    }

    if (!highestDamage || stats.totalDamageDealt > highestDamage.damage) {
      highestDamage = { cardId: stats.cardId, name: stats.name, damage: stats.totalDamageDealt };
    }

    if (!mostKills || stats.creaturesKilled > mostKills.kills) {
      mostKills = { cardId: stats.cardId, name: stats.name, kills: stats.creaturesKilled };
    }
  }

  return {
    mvpCard,
    mostPlayed,
    highestDamage,
    mostKills,
    cardsPlayed: cardStats,
  };
}

// =============================================================================
// AI Analysis Statistics
// =============================================================================

function computeAiAnalysis(actions: SpectatorAction[]): AiAnalysisStats | undefined {
  // Check if we have any decision insights data (works for ALL bot types)
  const actionsWithInsights = actions.filter(a => a.insights !== null);

  // Fallback: also check legacy MCTS thinking data
  const actionsWithThinking = actions.filter(a => a.thinking !== null);

  if (actionsWithInsights.length === 0 && actionsWithThinking.length === 0) {
    return undefined;
  }

  let totalSimulations = 0;
  let totalThinkingTime = 0;
  let maxThinkingTime = 0;
  let totalConfidence = 0;
  let confidenceCount = 0;
  const winProbs: number[] = [];
  const criticalMoments: CriticalMoment[] = [];

  let prevWinProb = 0.5; // Start at 50/50
  let prevEvalScore = 0;

  for (const action of actions) {
    totalThinkingTime += action.thinkingTimeMs;
    maxThinkingTime = Math.max(maxThinkingTime, action.thinkingTimeMs);

    // Primary source: DecisionInsightsDto (works for all bots)
    if (action.insights) {
      // Track simulations from search stats
      if (action.insights.searchStats.simulations) {
        totalSimulations += action.insights.searchStats.simulations;
      }

      // Track confidence from entropy-based calculation
      totalConfidence += action.insights.confidence;
      confidenceCount++;

      // Calculate win probability from evaluation score
      // evalBreakdown.totalScore ranges roughly -50 to +50
      // Use sigmoid normalization to convert to 0-1 probability
      const evalScore = action.insights.evalBreakdown?.totalScore ?? 0;
      const winProb = 1 / (1 + Math.exp(-evalScore / 20));

      // Adjust for player perspective (score is from current player's view)
      const p1WinProb = action.player === 1 ? winProb : 1 - winProb;
      winProbs.push(p1WinProb);

      // Check for critical moments using evaluation score swings
      // A swing of 10 points in eval score is significant (~15% win prob change)
      const evalSwing = evalScore - prevEvalScore;
      const winProbSwing = p1WinProb - prevWinProb;

      if (Math.abs(evalSwing) >= 10 || Math.abs(winProbSwing) >= 0.15) {
        criticalMoments.push({
          turn: action.turn,
          player: action.player,
          actionDescription: action.action.description,
          winProbBefore: prevWinProb,
          winProbAfter: p1WinProb,
          swing: winProbSwing,
        });
      }

      prevWinProb = p1WinProb;
      prevEvalScore = evalScore;
    }
    // Fallback: Legacy MCTS thinking data
    else if (action.thinking) {
      totalSimulations += action.thinking.totalSimulations;

      const winProb = action.player === 1
        ? action.thinking.selectedWinRate
        : 1 - action.thinking.selectedWinRate;
      winProbs.push(winProb);

      const swing = winProb - prevWinProb;
      if (Math.abs(swing) >= 0.15) {
        criticalMoments.push({
          turn: action.turn,
          player: action.player,
          actionDescription: action.action.description,
          winProbBefore: prevWinProb,
          winProbAfter: winProb,
          swing,
        });
      }

      prevWinProb = winProb;
    }
  }

  const actionsWithData = actionsWithInsights.length > 0 ? actionsWithInsights : actionsWithThinking;
  const avgSimulationsPerMove = actionsWithData.length > 0
    ? totalSimulations / actionsWithData.length
    : 0;

  const avgThinkingTimeMs = actions.length > 0
    ? totalThinkingTime / actions.length
    : 0;

  // Calculate average confidence from insights
  const avgMoveConfidence = confidenceCount > 0
    ? totalConfidence / confidenceCount
    : 0.8; // Fallback for legacy data

  // Find max win prob swing
  let maxWinProbSwing = 0;
  for (let i = 1; i < winProbs.length; i++) {
    const swing = Math.abs(winProbs[i] - winProbs[i - 1]);
    maxWinProbSwing = Math.max(maxWinProbSwing, swing);
  }

  return {
    totalSimulations,
    avgSimulationsPerMove: Math.round(avgSimulationsPerMove),
    avgThinkingTimeMs: Math.round(avgThinkingTimeMs),
    maxThinkingTimeMs: maxThinkingTime,
    criticalMoments,
    startingWinProb: winProbs.length > 0 ? winProbs[0] : 0.5,
    finalWinProb: winProbs.length > 0 ? winProbs[winProbs.length - 1] : 0.5,
    maxWinProbSwing: Math.round(maxWinProbSwing * 100) / 100,
    avgMoveConfidence: Math.round(avgMoveConfidence * 100) / 100,
  };
}

// =============================================================================
// Timeline Data
// =============================================================================

function computeTimelineData(
  initialState: GameStateDto,
  actions: SpectatorAction[]
): TimelineData {
  const turns: number[] = [0];
  const player1Life: number[] = [initialState.player.life];
  const player2Life: number[] = [initialState.opponent.life];
  const player1WinProb: number[] = [0.5];
  const player2WinProb: number[] = [0.5];
  const player1BoardPresence: number[] = [computeBoardPresence(initialState, 1)];
  const player2BoardPresence: number[] = [computeBoardPresence(initialState, 2)];
  const player1EssenceSpent: number[] = [0];
  const player2EssenceSpent: number[] = [0];
  const player1HandSize: number[] = [initialState.player.hand.length];
  const player2HandSize: number[] = [initialState.opponent.hand.length];

  let p1TotalEssence = 0;
  let p2TotalEssence = 0;
  let lastTurn = 0;

  for (const action of actions) {
    const state = action.stateAfter;

    // Only record at turn boundaries (when turn changes)
    if (action.turn !== lastTurn) {
      turns.push(action.turn);
      player1Life.push(state.player.life);
      player2Life.push(state.opponent.life);
      player1BoardPresence.push(computeBoardPresence(state, 1));
      player2BoardPresence.push(computeBoardPresence(state, 2));
      player1HandSize.push(state.player.hand.length);
      player2HandSize.push(state.opponent.hand.length);

      // Track essence spent from events
      for (const event of action.events) {
        if (event.eventType === "essence_changed") {
          const data = event.data as Record<string, unknown>;
          const eventPlayer = data.player as number;
          const oldEssence = data.old as number;
          const newEssence = data.new as number;
          if (newEssence < oldEssence) {
            if (eventPlayer === 1) {
              p1TotalEssence += oldEssence - newEssence;
            } else {
              p2TotalEssence += oldEssence - newEssence;
            }
          }
        }
      }

      player1EssenceSpent.push(p1TotalEssence);
      player2EssenceSpent.push(p2TotalEssence);

      // Win probability from decision insights (works for all bots)
      // or fallback to legacy MCTS thinking data
      if (action.insights?.evalBreakdown) {
        // Calculate win probability from evaluation score using sigmoid
        const evalScore = action.insights.evalBreakdown.totalScore;
        const winProb = 1 / (1 + Math.exp(-evalScore / 20));
        // Adjust for player perspective
        const p1Prob = action.player === 1 ? winProb : 1 - winProb;
        player1WinProb.push(p1Prob);
        player2WinProb.push(1 - p1Prob);
      } else if (action.thinking) {
        // Legacy MCTS data
        const p1Prob = action.player === 1
          ? action.thinking.selectedWinRate
          : 1 - action.thinking.selectedWinRate;
        player1WinProb.push(p1Prob);
        player2WinProb.push(1 - p1Prob);
      } else {
        // Carry forward last value
        player1WinProb.push(player1WinProb[player1WinProb.length - 1]);
        player2WinProb.push(player2WinProb[player2WinProb.length - 1]);
      }

      lastTurn = action.turn;
    }
  }

  return {
    turns,
    player1Life,
    player2Life,
    player1WinProb,
    player2WinProb,
    player1BoardPresence,
    player2BoardPresence,
    player1EssenceSpent,
    player2EssenceSpent,
    player1HandSize,
    player2HandSize,
  };
}

/**
 * Compute board presence score for a player.
 * Based on creature count, total stats, and keywords.
 */
function computeBoardPresence(state: GameStateDto, player: 1 | 2): number {
  const playerState = player === 1 ? state.player : state.opponent;
  let presence = 0;

  for (const creature of playerState.creatures) {
    if (creature) {
      // Base presence from stats
      presence += creature.attack + creature.health;

      // Bonus for keywords (simplified)
      presence += creature.keywords.length * 2;
    }
  }

  return presence;
}
