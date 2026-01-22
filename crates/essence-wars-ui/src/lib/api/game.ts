// Game API - Tauri command wrappers

import { invoke } from "@tauri-apps/api/core";
import type {
  DeckInfo,
  BotInfo,
  GameConfig,
  GameStateDto,
  ActionInfo,
  GameStateUpdate,
  GameResultDto,
} from "./types";

export async function listDecks(): Promise<DeckInfo[]> {
  return await invoke<DeckInfo[]>("list_decks");
}

export async function listBots(): Promise<BotInfo[]> {
  return await invoke<BotInfo[]>("list_bots");
}

export async function newGame(config: GameConfig): Promise<GameStateDto> {
  return await invoke<GameStateDto>("new_game", { config });
}

export async function getGameState(gameId: string): Promise<GameStateDto> {
  return await invoke<GameStateDto>("get_game_state", { gameId });
}

export async function getLegalActions(gameId: string): Promise<ActionInfo[]> {
  return await invoke<ActionInfo[]>("get_legal_actions", { gameId });
}

export async function applyAction(
  gameId: string,
  actionIndex: number
): Promise<GameStateUpdate> {
  return await invoke<GameStateUpdate>("apply_action", { gameId, actionIndex });
}

export async function getAiMove(gameId: string): Promise<ActionInfo> {
  return await invoke<ActionInfo>("get_ai_move", { gameId });
}

export async function endGame(gameId: string): Promise<GameResultDto> {
  return await invoke<GameResultDto>("end_game", { gameId });
}
