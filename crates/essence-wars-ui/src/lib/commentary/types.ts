// Commentary types for AI vs AI Spectator Mode

export interface CommentaryEntry {
  id: string;
  turn: number;
  player: 1 | 2;
  text: string;
  analysis?: string; // Detailed analysis (optional)
  isKeyMoment: boolean; // Triggers overlay
  momentType?: KeyMomentType;
  boardAdvantage?: number; // Negative = P2 advantage, Positive = P1 advantage
  winProbability?: number; // 0-1 for active player (from MCTS)
}

export type KeyMomentType =
  | 'game_start'
  | 'first_blood' // First creature death
  | 'board_swing' // Major advantage shift
  | 'commander_played' // Commander enters play
  | 'lethal_threat' // Player at very low life
  | 'game_end';
