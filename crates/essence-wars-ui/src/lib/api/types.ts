// TypeScript types matching the Rust DTOs

export interface DeckInfo {
  id: string;
  name: string;
  description: string;
  faction: string;
  cardCount: number;
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
}

export interface SupportDto {
  cardId: number;
  name: string;
  slot: number;
  faction: string;
  durability: number;
  artPath?: string;
}

export interface PlayerStateDto {
  life: number;
  maxLife: number;
  essence: number;
  maxEssence: number;
  actionPoints: number;
  deckCount: number;
  hand: CardDto[];
  creatures: (CreatureDto | null)[];
  supports: (SupportDto | null)[];
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
}

/** A single action in the spectator match with full context */
export interface SpectatorAction {
  turn: number;
  player: 1 | 2;
  action: ActionInfo;
  stateAfter: GameStateDto;
  events: GameEventDto[];
  thinking: MctsThinkingDto | null;
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
}

/** Result of a completed spectator match */
export interface SpectatorResult {
  winner: 1 | 2 | null;
  reason: string;
  player1FinalLife: number;
  player2FinalLife: number;
}
