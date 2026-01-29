/**
 * Match Statistics Types
 *
 * Comprehensive statistics computed from match data for display in the
 * Match Summary screen and for research/designer export.
 */

// =============================================================================
// Main Statistics Container
// =============================================================================

/**
 * Complete statistics for a match, computed from actions and events.
 */
export interface MatchStatistics {
  /** High-level match overview */
  overview: OverviewStats;

  /** Action economy statistics per player */
  actionEconomy: PlayerPairStats<ActionStats>;

  /** Combat statistics per player */
  combat: PlayerPairStats<CombatStats>;

  /** Resource economy statistics per player */
  resources: PlayerPairStats<ResourceStats>;

  /** Keyword activation statistics per player */
  keywords: PlayerPairStats<KeywordStats>;

  /** Card performance statistics per player */
  cardPerformance: PlayerPairStats<CardPerformanceStats>;

  /** AI analysis (only available if MCTS data present) */
  aiAnalysis?: AiAnalysisStats;

  /** Timeline data for graphs */
  timeline: TimelineData;
}

/** Helper type for stats that are tracked per player */
export interface PlayerPairStats<T> {
  player1: T;
  player2: T;
}

// =============================================================================
// Overview Statistics
// =============================================================================

export interface OverviewStats {
  /** Winner (1 or 2, or null for draw) */
  winner: 1 | 2 | null;

  /** Reason for game end (e.g., "CommanderDefeated", "TurnLimit") */
  reason: string;

  /** Total number of turns played */
  totalTurns: number;

  /** Total number of actions taken by both players */
  totalActions: number;

  /** Total game duration in milliseconds (sum of thinking times) */
  gameDurationMs: number;

  /** Final life differential (P1 life - P2 life) */
  lifeDifferential: number;

  /** Player 1 final life */
  player1FinalLife: number;

  /** Player 2 final life */
  player2FinalLife: number;

  /** Most valuable card (highest impact) */
  mvpCard: CardImpact | null;
}

// =============================================================================
// Action Economy Statistics
// =============================================================================

export interface ActionStats {
  /** Total cards played (creatures + spells + supports) */
  cardsPlayed: number;

  /** Creatures played */
  creaturesPlayed: number;

  /** Spells cast */
  spellsCast: number;

  /** Supports placed */
  supportsPlaced: number;

  /** Total attacks made */
  attacksMade: number;

  /** Face attacks (direct commander damage) */
  faceAttacks: number;

  /** Creature-to-creature attacks */
  creatureAttacks: number;

  /** Abilities used (triggered effects) */
  abilitiesUsed: number;

  /** Turns played */
  turnsPlayed: number;

  /** Total action points spent */
  apSpent: number;

  /** Total action points available */
  apAvailable: number;

  /** AP efficiency (apSpent / apAvailable as percentage) */
  apEfficiency: number;
}

// =============================================================================
// Combat Statistics
// =============================================================================

export interface CombatStats {
  /** Total damage dealt (to creatures + face) */
  totalDamageDealt: number;

  /** Damage dealt to enemy creatures */
  damageToCreatures: number;

  /** Damage dealt directly to enemy commander (face damage) */
  damageToFace: number;

  /** Enemy creatures killed */
  creaturesKilled: number;

  /** Own creatures lost (died) */
  creaturesLost: number;

  /** Kill/Death ratio (creaturesKilled / creaturesLost) */
  kdRatio: number;

  /** Number of favorable trades (killed enemy without dying) */
  favorableTrades: number;

  /** Number of even trades (both creatures died) */
  evenTrades: number;

  /** Number of unfavorable trades (died without killing) */
  unfavorableTrades: number;
}

// =============================================================================
// Resource Economy Statistics
// =============================================================================

export interface ResourceStats {
  /** Total essence spent over the game */
  totalEssenceSpent: number;

  /** Average essence spent per turn */
  avgEssencePerTurn: number;

  /** Total cards drawn */
  cardsDrawn: number;

  /** Cards discarded (overdraw, effects) */
  cardsDiscarded: number;

  /** Average hand size during the game */
  averageHandSize: number;

  /** Maximum hand size reached */
  maxHandSize: number;

  /** Turns with empty hand */
  emptyHandTurns: number;
}

// =============================================================================
// Keyword Statistics
// =============================================================================

export interface KeywordStats {
  /** Rush: Creatures that attacked the turn they were played */
  rushAttacks: number;

  /** Guard: Attacks redirected by Guard creatures */
  guardBlocks: number;

  /** Lethal: One-shot kills from Lethal keyword */
  lethalKills: number;

  /** Lifesteal: Total health healed via Lifesteal */
  lifestealHealing: number;

  /** Piercing: Total overflow damage dealt to face */
  piercingDamage: number;

  /** Shield: Total damage absorbed by Shield */
  shieldAbsorbed: number;

  /** Quick: Times Quick strike order was relevant */
  quickStrikes: number;

  /** Fortify: Total damage reduced by Fortify */
  fortifyReduced: number;

  /** Ward: Spells/abilities blocked by Ward */
  wardBlocks: number;

  /** Ranged: Attacks without counter-damage */
  rangedAttacks: number;

  /** Stealth: Attacks prevented by Stealth */
  stealthEvades: number;

  /** Regenerate: Total health regenerated */
  regenerateHealing: number;

  /** Volatile: Total splash damage dealt on death */
  volatileDamage: number;

  /** Frenzy: Total bonus attack gained from Frenzy */
  frenzyStacks: number;

  /** Charge: Times Charge bonus was applied */
  chargeBonus: number;
}

// =============================================================================
// Card Performance Statistics
// =============================================================================

export interface CardPerformanceStats {
  /** Most valuable card (highest impact score) */
  mvpCard: CardImpact | null;

  /** Most frequently played card */
  mostPlayed: CardCount | null;

  /** Card that dealt the most damage */
  highestDamage: CardDamage | null;

  /** Card that got the most kills */
  mostKills: CardKills | null;

  /** Cards played breakdown by card ID */
  cardsPlayed: Map<number, CardPlayStats>;
}

export interface CardImpact {
  cardId: number;
  name: string;
  /** Impact score based on damage dealt, kills, and survival */
  impact: number;
}

export interface CardCount {
  cardId: number;
  name: string;
  count: number;
}

export interface CardDamage {
  cardId: number;
  name: string;
  damage: number;
}

export interface CardKills {
  cardId: number;
  name: string;
  kills: number;
}

export interface CardPlayStats {
  cardId: number;
  name: string;
  timesPlayed: number;
  totalDamageDealt: number;
  creaturesKilled: number;
  survivalTurns: number;
}

// =============================================================================
// AI Analysis Statistics
// =============================================================================

export interface AiAnalysisStats {
  /** Total MCTS simulations run */
  totalSimulations: number;

  /** Average simulations per move */
  avgSimulationsPerMove: number;

  /** Average thinking time per move (ms) */
  avgThinkingTimeMs: number;

  /** Longest thinking time for a single move (ms) */
  maxThinkingTimeMs: number;

  /** Critical moments where win rate swung significantly */
  criticalMoments: CriticalMoment[];

  /** Starting win probability (P1 perspective, 0-1) */
  startingWinProb: number;

  /** Final win probability before game end */
  finalWinProb: number;

  /** Largest win probability swing in a single turn */
  maxWinProbSwing: number;

  /** Average confidence in moves (low variance = high confidence) */
  avgMoveConfidence: number;
}

export interface CriticalMoment {
  turn: number;
  player: 1 | 2;
  actionDescription: string;
  winProbBefore: number;
  winProbAfter: number;
  swing: number;
}

// =============================================================================
// Timeline Data (for Graphs)
// =============================================================================

export interface TimelineData {
  /** Turn numbers (x-axis) */
  turns: number[];

  /** Player 1 life at each turn boundary */
  player1Life: number[];

  /** Player 2 life at each turn boundary */
  player2Life: number[];

  /** P1 win probability at each action (from MCTS, 0-1 scale) */
  player1WinProb: number[];

  /** P2 win probability (1 - P1 win prob) */
  player2WinProb: number[];

  /** P1 board presence (creature count * avg stats) */
  player1BoardPresence: number[];

  /** P2 board presence */
  player2BoardPresence: number[];

  /** P1 cumulative essence spent */
  player1EssenceSpent: number[];

  /** P2 cumulative essence spent */
  player2EssenceSpent: number[];

  /** P1 hand size at each turn */
  player1HandSize: number[];

  /** P2 hand size at each turn */
  player2HandSize: number[];
}

// =============================================================================
// Export Types
// =============================================================================

export interface ExportData {
  /** Match metadata */
  matchId: string;
  player1Deck: string;
  player2Deck: string;
  player1Bot: string;
  player2Bot: string;
  timestamp: string;

  /** Full statistics */
  statistics: MatchStatistics;
}
