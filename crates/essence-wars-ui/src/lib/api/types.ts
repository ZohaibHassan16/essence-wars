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
