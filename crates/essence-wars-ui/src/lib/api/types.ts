// TypeScript types matching the Rust DTOs

export interface DeckInfo {
  id: string;
  name: string;
  description: string;
  /** Short playstyle tag (e.g., "Token Pack", "Aggressive Piercing") */
  playstyle: string;
  faction: string;
  cardCount: number;
  /** Commander information for this deck */
  commander: CommanderDto | null;
}

export interface BotInfo {
  id: string;
  name: string;
  description: string;
}

export interface GameConfig {
  playerDeckId: string;
  opponentDeckId: string;
  opponentBotType: string;
  playerGoesFirst?: boolean;
  seed?: number;
}

export interface CardDto {
  cardId: number;
  name: string;
  cost: number;
  cardType: string;
  faction: string;
  attack?: number;
  health?: number;
  keywords: string[];
  durability?: number;
  artPath?: string;
  /** Effect description for spell and support cards */
  effectDescription?: string;
}

/** Activated ability on a creature (typically tokens) */
export interface AbilityDto {
  /** Ability index (0-5) for action matching */
  index: number;
  /** Display name (e.g., "Fungal Rot") */
  name: string;
  /** Essence cost to activate */
  essenceCost: number;
  /** Targeting type: "no_target", "enemy_creature", "enemy_player", "any", etc. */
  targetingType: string;
  /** Human-readable effect description */
  description: string;
}

export interface CreatureDto {
  instanceId: number;
  cardId: number;
  name: string;
  slot: number;
  faction: string;
  attack: number;
  baseAttack: number;
  health: number;
  maxHealth: number;
  keywords: string[];
  canAttack: boolean;
  isExhausted: boolean;
  artPath?: string;
  /** Activated abilities (empty for most creatures, populated for tokens with abilities) */
  abilities: AbilityDto[];
}

export interface SupportDto {
  cardId: number;
  name: string;
  slot: number;
  faction: string;
  durability: number;
  artPath?: string;
  /** Human-readable description of what this support does */
  effectDescription: string;
}

export interface CommanderDto {
  id: number;
  name: string;
  faction: string;
  abilityDescription: string;
  /** Portrait path relative to static folder (e.g., "portrait/the_grand_architect.webp") */
  portraitPath: string;
}

export interface PlayerStateDto {
  life: number;
  maxLife: number;
  essence: number;
  maxEssence: number;
  actionPoints: number;
  deckCount: number;
  /** Essence extracted from opponent (total face damage dealt). At 50, wins the game in EssenceWar mode. */
  essenceExtracted: number;
  hand: CardDto[];
  creatures: (CreatureDto | null)[];
  supports: (SupportDto | null)[];
  /** Commander information (always present in a game) */
  commander: CommanderDto | null;
}

export interface GameStateDto {
  id: string;
  turn: number;
  phase: string;
  activePlayer: number;
  player: PlayerStateDto;
  opponent: PlayerStateDto;
  isGameOver: boolean;
  winner?: number;
  gameOverReason?: string;
}

export interface ActionInfo {
  index: number;
  actionType: string;
  description: string;
  sourceSlot?: number;
  targetSlot?: number;
  handIndex?: number;
  cardId?: number;
  /** For use_ability actions: which ability (0-5) */
  abilityIndex?: number;
}

export interface GameEventDto {
  eventType: string;
  data: Record<string, unknown>;
}

export interface GameStateUpdate {
  state: GameStateDto;
  lastAction?: ActionInfo;
  events: GameEventDto[];
}

export interface GameResultDto {
  winner?: number;
  reason: string;
  finalTurn: number;
  playerFinalLife: number;
  opponentFinalLife: number;
}

export interface AiHintResponse {
  recommendedAction: ActionInfo;
  score: number;
  alternatives: AlternativeAction[];
  thinkingTimeMs: number;
}

export interface AlternativeAction {
  action: ActionInfo;
  score: number;
  scoreDelta: number;
}

// ============================================================================
// Spectator Mode Types
// ============================================================================

/** Configuration for starting a spectator match */
export interface SpectatorConfig {
  player1DeckId: string;
  player1BotType: string;
  player2DeckId: string;
  player2BotType: string;
  seed?: number;
  /** Number of MCTS simulations per move (default: 100 for fast playback) */
  mctsSimulations?: number;
  /** Alpha-Beta search depth (default: 4 for fast playback) */
  alphabetaDepth?: number;
}

/** A single action in the spectator match with full context */
export interface SpectatorAction {
  turn: number;
  player: 1 | 2;
  action: ActionInfo;
  stateAfter: GameStateDto;
  events: GameEventDto[];
  thinking: MctsThinkingDto | null;
  /** AI decision insights with move scores and evaluation breakdown */
  insights: DecisionInsightsDto | null;
  thinkingTimeMs: number;
}

/** MCTS thinking data for visualization */
export interface MctsThinkingDto {
  totalSimulations: number;
  topMoves: MctsMoveDto[];
  selectedWinRate: number;
}

/** A candidate move considered by MCTS */
export interface MctsMoveDto {
  action: ActionInfo;
  visits: number;
  winRate: number;
}

// ============================================================================
// AI Decision Insights Types
// ============================================================================

/** Move score for UI display */
export interface MoveScoreDto {
  /** The action info */
  action: ActionInfo;
  /** Raw evaluation score */
  score: number;
  /** Normalized probability (0-1) */
  probability: number;
  /** Whether this was the chosen move */
  isChosen: boolean;
  /** MCTS visits (if available) */
  visits: number | null;
  /** MCTS win rate (if available) */
  winRate: number | null;
}

/** Evaluation factor for UI display */
export interface EvalFactorDto {
  /** Factor name */
  name: string;
  /** Player 1 value */
  p1Value: number;
  /** Player 2 value */
  p2Value: number;
  /** Weight */
  weight: number;
  /** Contribution to total score */
  contribution: number;
}

/** Position evaluation breakdown for UI display */
export interface EvalBreakdownDto {
  /** Total evaluation score (positive = P1 advantage) */
  totalScore: number;
  /** Individual factors */
  factors: EvalFactorDto[];
}

/** Search statistics for UI display */
export interface SearchStatsDto {
  /** Algorithm name */
  algorithm: string;
  /** Think time in ms */
  timeMs: number;
  /** Number of legal actions */
  numActions: number;
  /** MCTS simulations (if applicable) */
  simulations: number | null;
  /** AlphaBeta depth (if applicable) */
  depth: number | null;
  /** Nodes evaluated (if applicable) */
  nodes: number | null;
}

/** A node in the search tree (for visualization) */
export interface TreeNodeDto {
  /** Action that led to this node (null for root) */
  action: ActionInfo | null;
  /** Human-readable action description */
  actionStr: string;
  /** Visit count (MCTS) or node count (AlphaBeta) */
  visits: number;
  /** Score: win rate (0-1) for MCTS, eval score for AlphaBeta */
  score: number;
  /** Win rate as percentage string (e.g., "54%") */
  scoreDisplay: string;
  /** Whether this is the best/chosen path */
  isBestPath: boolean;
  /** Whether this node is fully expanded */
  isExpanded: boolean;
  /** Child nodes (limited by depth) */
  children: TreeNodeDto[];
  /** Whether children were truncated due to depth limit */
  isTruncated: boolean;
  /** Number of children that were truncated */
  truncatedChildCount: number;
  /** Depth of this node in the tree (0 = root) */
  depth: number;
}

// ============================================================================
// Neural Agent Types (for future transformer/policy network agents)
// ============================================================================

/** Attention weights for transformer-based agents */
export interface AttentionWeightsDto {
  /** Layer index (0-based) */
  layer: number;
  /** Attention head index (0-based) */
  head: number;
  /** Attention entries (source -> weight) */
  weights: AttentionEntryDto[];
}

/** A single attention weight entry */
export interface AttentionEntryDto {
  /** Source type (card, creature, commander, global) */
  sourceType: string;
  /** Source identifier (slot number, card index, etc.) */
  sourceId: number;
  /** Attention weight (0-1) */
  weight: number;
}

/** Neural network output for visualization */
export interface NeuralOutputDto {
  /** Policy head output (action probabilities) */
  policy: number[];
  /** Value head output (position evaluation, -1 to 1) */
  value: number;
  /** Attention weights (if transformer-based) */
  attention: AttentionWeightsDto[] | null;
  /** Model name/version */
  modelName: string;
  /** Inference time in ms */
  inferenceTimeMs: number;
}

/** Confidence level for a decision */
export type ConfidenceLevel = "high" | "medium" | "low";

/** Complete decision insights for UI display */
export interface DecisionInsightsDto {
  /** Player who made this decision (1 or 2) */
  player: number;
  /** Turn number */
  turn: number;
  /** The chosen action */
  chosenAction: ActionInfo;
  /** All moves ranked by score */
  moveScores: MoveScoreDto[];
  /** Evaluation breakdown (if available) */
  evalBreakdown: EvalBreakdownDto | null;
  /** Search statistics */
  searchStats: SearchStatsDto;
  /** Search tree visualization (first level from move scores) */
  treeRoot: TreeNodeDto | null;
  /** Confidence in the decision (0-1, based on probability concentration) */
  confidence: number;
  /** Confidence level category */
  confidenceLevel: ConfidenceLevel;
  /** Neural network output (for future neural agents) */
  neuralOutput: NeuralOutputDto | null;
}

/** Complete pre-computed spectator match */
export interface SpectatorMatch {
  id: string;
  config: SpectatorConfig;
  initialState: GameStateDto;
  actions: SpectatorAction[];
  result: SpectatorResult;
  totalTurns: number;
  player1DeckName: string;
  player2DeckName: string;
  player1BotName: string;
  player2BotName: string;
  /** Evaluation history for timeline visualization */
  evalHistory: EvalPoint[];
  /** Key moments where evaluation swung significantly */
  keyMoments: KeyMoment[];
}

/** Result of a completed spectator match */
export interface SpectatorResult {
  winner: 1 | 2 | null;
  reason: string;
  player1FinalLife: number;
  player2FinalLife: number;
}

/** A point on the evaluation timeline */
export interface EvalPoint {
  /** Action index (0-based) */
  actionIndex: number;
  /** Turn number */
  turn: number;
  /** Which player acted (1 or 2) */
  player: 1 | 2;
  /** Evaluation score (positive = P1 advantage) */
  evalScore: number;
}

/** A key moment in the game (significant eval swing) */
export interface KeyMoment {
  /** Action index where this moment occurred */
  actionIndex: number;
  /** Turn number */
  turn: number;
  /** Which player acted */
  player: 1 | 2;
  /** Description of the action */
  actionDescription: string;
  /** Evaluation change (delta from previous) */
  evalDelta: number;
  /** New evaluation score after this action */
  evalAfter: number;
  /** Type of moment */
  momentType: KeyMomentType;
}

/** Types of key moments */
export type KeyMomentType = "p1Surge" | "p2Surge" | "leadChange" | "decisive";

// ============================================================================
// Replay Mode Types
// ============================================================================

// ============================================================================
// MCP Sync Mode Types
// ============================================================================

/** Response from get_mcp_synced_state command */
export interface McpSyncedState {
  state: GameStateDto;
  ageMs: number;
  timestamp: number;
}

/** Information about a saved replay for the browser UI */
export interface ReplayInfo {
  /** Full path to the replay file */
  path: string;
  /** Just the filename */
  filename: string;
  /** Unix timestamp when replay was saved */
  timestamp: number;
  /** Human-readable date string */
  dateString: string;
  /** Player 1 deck display name */
  player1DeckName: string;
  /** Player 2 deck display name */
  player2DeckName: string;
  /** Player 1 type (e.g., "Human", "MCTS Bot") */
  player1Type: string;
  /** Player 2 type (e.g., "Human", "MCTS Bot") */
  player2Type: string;
  /** Winner (1 or 2, or null for draw) */
  winner: 1 | 2 | null;
  /** Total turns in the game */
  totalTurns: number;
  /** Total number of actions */
  totalActions: number;
}

// ============================================================================
// Deck Builder Types
// ============================================================================

/** Playstyle classification */
export type Playstyle = "aggro" | "control" | "tempo" | "midrange";

/** Playstyle scores for all archetypes */
export interface PlaystyleBreakdown {
  aggro: number;
  control: number;
  tempo: number;
  midrange: number;
}

/** Result of playstyle calculation */
export interface PlaystyleScore {
  /** The dominant playstyle */
  primary: Playstyle;
  /** Scores for all playstyles */
  scores: PlaystyleBreakdown;
}

/** Deck validation result */
export interface DeckValidation {
  /** Whether the deck is valid and playable */
  isValid: boolean;
  /** Critical errors that prevent playing */
  errors: string[];
  /** Non-critical warnings */
  warnings: string[];
}

/** Card information for the deck builder browser */
export interface BrowsableCard {
  cardId: number;
  name: string;
  cost: number;
  cardType: string;
  faction: string;
  rarity: string;
  /** For creatures */
  attack?: number;
  health?: number;
  keywords: string[];
  /** For supports */
  durability?: number;
  /** Art path (relative to static folder) */
  artPath: string;
  /** Copy limit based on rarity */
  copyLimit: number;
  /** Effect description for spells/supports */
  effectDescription?: string;
}

/** A custom deck definition */
export interface CustomDeck {
  /** Unique identifier (e.g., "user_my_rush_deck") */
  id: string;
  /** Display name */
  name: string;
  /** Commander card ID */
  commander: number;
  /** Card IDs in the deck (29-100 cards, can have duplicates) */
  cards: number[];
  /** User-provided description */
  description: string;
  /** Tags for categorization */
  tags: string[];
}

/** Summary info for a custom deck (used in deck lists) */
export interface CustomDeckInfo {
  id: string;
  name: string;
  description: string;
  commanderId: number;
  commanderName: string;
  faction: string;
  cardCount: number;
  playstyle?: PlaystyleScore;
  createdAt?: string;
  modifiedAt?: string;
}
