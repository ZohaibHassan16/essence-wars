// Commentary generator for AI vs AI Spectator Mode
// Analyzes game state and MCTS data to generate analytical commentary

import type { SpectatorAction, GameStateDto } from '$lib/api/types';
import type { CommentaryEntry, KeyMomentType } from './types';

// Track first blood state across calls
let hasFirstBloodOccurred = false;

export function resetCommentaryState(): void {
  hasFirstBloodOccurred = false;
}

export function generateCommentaryForAction(
  action: SpectatorAction,
  prevState: GameStateDto | null
): CommentaryEntry {
  const { turn, player, action: act, stateAfter, thinking, events: _events } = action;

  // Calculate board advantage
  const advantage = calculateBoardAdvantage(stateAfter);

  // Check for key moments
  const keyMoment = detectKeyMoment(action, prevState, stateAfter);

  // Generate text based on action type and context
  const text = generateCommentaryText(action, advantage, keyMoment);

  return {
    id: `${turn}-${player}-${act.index}`,
    turn,
    player: player as 1 | 2,
    text,
    isKeyMoment: keyMoment !== null,
    momentType: keyMoment ?? undefined,
    boardAdvantage: advantage,
    winProbability: thinking?.selectedWinRate,
  };
}

function calculateBoardAdvantage(state: GameStateDto): number {
  // Calculate advantage based on multiple factors
  // Positive = P1 advantage, Negative = P2 advantage

  // Creature count (weighted)
  const p1Creatures = state.player.creatures.filter((c) => c !== null).length;
  const p2Creatures = state.opponent.creatures.filter((c) => c !== null).length;
  const creatureDiff = (p1Creatures - p2Creatures) * 8;

  // Life difference
  const lifeDiff = state.player.life - state.opponent.life;

  // Essence advantage (minor factor)
  const essenceDiff = (state.player.essence - state.opponent.essence) * 2;

  // Total creature stats (attack + health)
  const p1Stats = state.player.creatures
    .filter((c) => c !== null)
    .reduce((sum, c) => sum + (c?.attack ?? 0) + (c?.health ?? 0), 0);
  const p2Stats = state.opponent.creatures
    .filter((c) => c !== null)
    .reduce((sum, c) => sum + (c?.attack ?? 0) + (c?.health ?? 0), 0);
  const statsDiff = (p1Stats - p2Stats) * 0.5;

  return Math.round(creatureDiff + lifeDiff + essenceDiff + statsDiff);
}

function detectKeyMoment(
  action: SpectatorAction,
  prevState: GameStateDto | null,
  newState: GameStateDto
): KeyMomentType | null {
  const { turn, player, events } = action;

  // Game start (first action of the game)
  if (turn === 1 && player === 1 && !prevState) {
    return 'game_start';
  }

  // Game end
  if (newState.isGameOver) {
    return 'game_end';
  }

  // First blood - first creature death in the game
  const deathEvent = events.find((e) => e.eventType === 'creature_died');
  if (deathEvent && !hasFirstBloodOccurred) {
    hasFirstBloodOccurred = true;
    return 'first_blood';
  }

  // Board swing - advantage changed by 15+ points
  if (prevState) {
    const prevAdv = calculateBoardAdvantage(prevState);
    const newAdv = calculateBoardAdvantage(newState);
    if (Math.abs(newAdv - prevAdv) >= 15) {
      return 'board_swing';
    }
  }

  // Lethal threat - either player at very low life (5 or less)
  if (newState.player.life <= 5 || newState.opponent.life <= 5) {
    // Only trigger if this is a new lethal threat situation
    if (prevState) {
      const wasLethal = prevState.player.life <= 5 || prevState.opponent.life <= 5;
      if (!wasLethal) {
        return 'lethal_threat';
      }
    }
  }

  return null;
}

function generateCommentaryText(
  action: SpectatorAction,
  advantage: number,
  keyMoment: KeyMomentType | null
): string {
  const { player, action: act, thinking, stateAfter } = action;
  const playerName = `P${player}`;
  const _opponentName = `P${player === 1 ? 2 : 1}`;

  // Key moment commentary (more detailed)
  if (keyMoment === 'game_start') {
    return `Match begins! Both players start with 30 life. The battle for board control starts now.`;
  }

  if (keyMoment === 'game_end') {
    const winner = stateAfter.winner;
    if (winner !== undefined) {
      return `Game Over! Player ${winner} claims victory!`;
    }
    return `Game Over! The match has ended.`;
  }

  if (keyMoment === 'first_blood') {
    return `First blood! A creature falls in combat. The tempo battle intensifies.`;
  }

  if (keyMoment === 'board_swing') {
    if (advantage > 0) {
      return `Major board swing! P1 seizes a significant advantage (+${advantage}).`;
    } else {
      return `Major board swing! P2 seizes a significant advantage (${advantage}).`;
    }
  }

  if (keyMoment === 'lethal_threat') {
    if (stateAfter.player.life <= 5) {
      return `Danger! P1 is at ${stateAfter.player.life} life - one wrong move could be fatal.`;
    } else {
      return `Danger! P2 is at ${stateAfter.opponent.life} life - the end may be near.`;
    }
  }

  // Regular action commentary based on action type
  const actionType = act.actionType;

  // With MCTS data - show win rate analysis
  if (thinking) {
    const winRate = Math.round(thinking.selectedWinRate * 100);
    const confidence = winRate >= 70 ? 'confidently' : winRate >= 50 ? '' : 'desperately';

    if (actionType === 'play_card') {
      return `${playerName} ${confidence} plays a card. Win rate: ${winRate}%`;
    }
    if (actionType === 'attack') {
      return `${playerName} ${confidence} attacks. Win rate: ${winRate}%`;
    }
    if (actionType === 'end_turn') {
      return `${playerName} ends turn. Current evaluation: ${winRate}% win rate.`;
    }
    return `${playerName}: ${act.description}. Win rate: ${winRate}%`;
  }

  // Without MCTS data - simpler commentary
  if (actionType === 'play_card') {
    return `${playerName} plays a card to develop their board.`;
  }
  if (actionType === 'attack') {
    return `${playerName} goes on the offensive!`;
  }
  if (actionType === 'end_turn') {
    const advantageText =
      advantage > 5
        ? 'P1 leads'
        : advantage < -5
          ? 'P2 leads'
          : 'Even board';
    return `${playerName} passes. ${advantageText}.`;
  }

  return `${playerName}: ${act.description}`;
}
