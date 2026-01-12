#  Proposed Architecture

  src/
  ├── lib.rs                    # Existing core engine (unchanged)
  ├── engine.rs                 # GameEngine + GameEnvironment trait (unchanged)
  └── ...                       # Other existing modules (unchanged)

  src/bots/                     # NEW: Bot implementations
  ├── mod.rs                    # Bot trait + re-exports
  ├── random.rs                 # RandomBot (baseline)
  ├── greedy.rs                 # GreedyBot (heuristic-based)
  └── mcts.rs                   # MctsBot (tree search)

  src/arena/                    # NEW: Game runner infrastructure
  ├── mod.rs                    # Re-exports
  ├── runner.rs                 # GameRunner - orchestrates matches
  ├── logger.rs                 # ActionLogger - debug tracing
  └── stats.rs                  # MatchStats - win rates, timing

  src/tuning/                   # NEW: Parameter optimization
  ├── mod.rs
  ├── evaluator.rs              # Runs N games, returns win rate
  ├── greedy_tuner.rs           # Tunes GreedyBot heuristic weights
  └── mcts_tuner.rs             # Tunes MCTS parameters

  src/bin/                      # NEW: CLI binaries
  ├── arena.rs                  # Run bot vs bot matches
  └── tune.rs                   # Run tuning pipelines

##  Key Design Decisions

###  1. Bot Trait (engine-agnostic)

  // src/bots/mod.rs
  pub trait Bot: Send {
      fn name(&self) -> &str;

      /// Select an action given the current state
      /// Receives only what GameEnvironment provides
      fn select_action(
          &mut self,
          state_tensor: &[f32; 326],
          legal_mask: &[f32; 256],
          legal_actions: &[Action],
      ) -> Action;

      /// Reset internal state between games
      fn reset(&mut self);
  }

  Why this signature?
  - Bots receive tensor + mask (same as neural network would)
  - Also get legal_actions list for convenience
  - Engine never imports Bot - complete decoupling
  - Send bound enables parallel game execution

###  2. GameRunner

  // src/arena/runner.rs
  pub struct GameRunner<'a> {
      card_db: &'a CardDatabase,
      logger: Option<ActionLogger>,
  }

  pub struct GameResult {
      pub winner: Option<PlayerId>,
      pub turns: u32,
      pub actions: Vec<ActionRecord>,  // Only if logging enabled
      pub seed: u64,
  }

  impl GameRunner {
      pub fn run_game(
          &mut self,
          bot1: &mut dyn Bot,
          bot2: &mut dyn Bot,
          deck1: Vec<CardId>,
          deck2: Vec<CardId>,
          seed: u64,
      ) -> GameResult;

      pub fn run_match(
          &mut self,
          bot1: &mut dyn Bot,
          bot2: &mut dyn Bot,
          games: usize,
          base_seed: u64,
      ) -> MatchStats;
  }

###  3. Debug Logging

  // src/arena/logger.rs
  pub struct ActionLogger {
      verbose: bool,
      output: LogOutput,  // File or Stdout
  }

  pub struct ActionRecord {
      pub turn: u32,
      pub player: PlayerId,
      pub action: Action,
      pub thinking_time_us: u64,
      pub state_before: Option<GameState>,  // Only in verbose mode
  }

  CLI usage:
  # Normal run - just results
  cargo run --bin arena -- --bot1 greedy --bot2 mcts --games 1000

  # Debug single game
  cargo run --bin arena -- --bot1 greedy --bot2 mcts --games 1 --seed 12345 --debug

  # Debug to file
  cargo run --bin arena -- --bot1 greedy --bot2 mcts --seed 12345 --debug --log-file game.log

###  4. GreedyBot Design

  // src/bots/greedy.rs
  pub struct GreedyBot {
      weights: GreedyWeights,
      rng: SmallRng,  // For tie-breaking
  }

  pub struct GreedyWeights {
      pub face_damage: f32,
      pub creature_damage: f32,
      pub creature_value: f32,      // attack + health
      pub keyword_value: f32,
      pub board_control: f32,
      pub life_advantage: f32,
      // ... more weights
  }

  impl GreedyBot {
      /// Evaluate a single action by simulating it
      fn evaluate_action(&self, engine: &GameEngine, action: Action) -> f32;
  }

###  5. MCTS Bot Design

  // src/bots/mcts.rs
  pub struct MctsBot {
      params: MctsParams,
      rng: SmallRng,
  }

  pub struct MctsParams {
      pub exploration_c: f32,        // UCB1 constant (default ~1.4)
      pub simulations: usize,        // Simulations per move
      pub max_depth: usize,          // Max simulation depth
      pub time_limit_ms: Option<u64>, // Optional time limit
  }

  struct MctsNode {
      action: Option<Action>,
      visits: u32,
      total_value: f32,
      children: Vec<MctsNode>,
  }

###  6. Tuning Pipeline

  // src/tuning/evaluator.rs
  pub struct Evaluator<'a> {
      card_db: &'a CardDatabase,
      baseline_bot: Box<dyn Bot>,
      games_per_eval: usize,
      parallelism: usize,
  }

  impl Evaluator {
      /// Returns win rate against baseline
      pub fn evaluate(&self, candidate: &dyn Bot) -> f32;
  }

  // src/tuning/greedy_tuner.rs
  pub struct GreedyTuner {
      evaluator: Evaluator,
      method: TuningMethod,  // GridSearch, RandomSearch, Evolutionary
  }

  CLI usage:
  # Tune greedy weights
  cargo run --release --bin tune -- --bot greedy --method random --iterations 1000

  # Tune MCTS parameters
  cargo run --release --bin tune -- --bot mcts --method grid --games-per-eval 100

  Performance Considerations

  1. Parallel game execution - Use rayon for running multiple games simultaneously
  2. Engine cloning - Already optimized with ArrayVec for fast fork()
  3. MCTS node pool - Pre-allocate nodes to avoid allocation during search
  4. Release mode - Always run with --release for benchmarks

  Questions for You

  1. Greedy evaluation: Should GreedyBot simulate each action (slower but accurate) or use a static heuristic on current state?
  2. MCTS rollout policy: Random actions, or use GreedyBot for smarter rollouts?
  3. Tuning method preference: Simple grid/random search, or something more sophisticated like CMA-ES?
  4. Parallelism: Should individual MCTS searches be parallelized (root parallelization), or just run multiple games in parallel?
  5. Deck handling: Should bots always use the same deck pair, or should we test across multiple deck matchups?

✻ Crunched for 43s

❯ Fantastic Architecture, lets please go with that!

  I full agree with all your Key Design Decisions

  Questions

  1. Greedy Eval: GreedyBot should simulate each action
  2. MCTS Rollout Policy: Lets use greedybot for smarter rollouts
  3. Tuning Method: Let us go with a sophisticated setup
  4. Parallelism: Lets go with Root parallelization
  5. Decks: As there are currently only two decks, but more sets, cards and decks are planned (future work), how do you best propose we make this future proof now? Also, it
  sounds interesting to have Greedy/MCTSBots as generalists, and as Specialists for each Deck, maybe with each bot having different weights for generalist, and for each
  specific deck, this would need to be considered for our tuning pipeline.

● Excellent choices! Let me expand the design to handle the generalist/specialist bot concept and create a detailed implementation plan.

##  Expanded Design: Generalist vs Specialist Bots

###  Weight Configuration System

  // src/bots/weights.rs
  use std::collections::HashMap;

  /// Weights can be loaded from TOML files
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct BotWeights {
      pub name: String,
      pub version: u32,

      /// Default weights (generalist)
      pub default: WeightSet,

      /// Deck-specific overrides (specialist)
      /// Key: deck_id (hash or name)
      #[serde(default)]
      pub deck_specific: HashMap<String, WeightSet>,
  }

  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct WeightSet {
      // Greedy evaluation weights
      pub greedy: GreedyWeights,

      // MCTS parameters (for specialist MCTS configs)
      pub mcts: Option<MctsParams>,
  }

  impl BotWeights {
      pub fn load(path: &Path) -> Result<Self, WeightError>;
      pub fn save(&self, path: &Path) -> Result<(), WeightError>;

      /// Get weights for a specific deck, falling back to default
      pub fn for_deck(&self, deck_id: &str) -> &WeightSet {
          self.deck_specific.get(deck_id).unwrap_or(&self.default)
      }
  }

###  Deck Registry (Future-Proof)

  // src/decks/mod.rs
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct DeckDefinition {
      pub id: String,           // Unique identifier
      pub name: String,         // Human-readable name
      pub cards: Vec<CardId>,
      pub tags: Vec<String>,    // "aggro", "control", etc.
  }

  pub struct DeckRegistry {
      decks: HashMap<String, DeckDefinition>,
  }

  impl DeckRegistry {
      pub fn load_from_directory(path: &Path) -> Result<Self, DeckError>;
      pub fn get(&self, id: &str) -> Option<&DeckDefinition>;
      pub fn all_pairs(&self) -> Vec<(&DeckDefinition, &DeckDefinition)>;
  }

  Deck file format (data/decks/aggressive_assault.toml):
  id = "aggressive_assault"
  name = "Aggressive Assault"
  tags = ["aggro", "rush", "starter"]
  cards = [1, 1, 2, 2, 3, 3, 6, 6, 8, 8, 11, 11, 16, 16, 20, 20, 29, 34, 34, 35]

### Updated Tuning Pipeline

  src/tuning/
  ├── mod.rs
  ├── evaluator.rs          # Multi-deck evaluation
  ├── optimizer.rs          # CMA-ES implementation
  ├── greedy_tuner.rs       # Greedy weight optimization
  ├── mcts_tuner.rs         # MCTS parameter optimization
  └── specialist.rs         # Deck-specific tuning

  // src/tuning/evaluator.rs
  pub struct EvaluationConfig {
      pub games_per_matchup: usize,
      pub deck_matchups: DeckMatchupMode,
      pub parallelism: usize,
      pub base_seed: u64,
  }

  pub enum DeckMatchupMode {
      /// Single deck pair (for specialist tuning)
      Single { deck1: String, deck2: String },

      /// All deck pairs (for generalist tuning)
      AllPairs,

      /// Mirror matches only (deck vs same deck)
      MirrorOnly,

      /// Custom subset
      Custom(Vec<(String, String)>),
  }

  pub struct EvaluationResult {
      pub overall_win_rate: f32,
      pub per_matchup: HashMap<(String, String), MatchupStats>,
      pub total_games: usize,
      pub total_time_ms: u64,
  }

  // src/tuning/optimizer.rs
  pub struct CmaEsOptimizer {
      pub population_size: usize,
      pub initial_sigma: f64,
      pub max_generations: usize,
      pub target_fitness: Option<f64>,
  }

  impl CmaEsOptimizer {
      pub fn optimize<F>(&mut self,
          initial: &[f64],
          bounds: &[(f64, f64)],
          fitness_fn: F,
      ) -> OptimizationResult
      where
          F: Fn(&[f64]) -> f64 + Sync;
  }

###  CLI Design

  # === ARENA: Run matches ===

  # Quick match with default weights
  cargo run --release --bin arena -- \
    --bot1 greedy --bot2 mcts --games 1000

  # Specialist bots with specific weights
  cargo run --release --bin arena -- \
    --bot1 greedy:weights/greedy_aggro.toml \
    --bot2 mcts:weights/mcts_control.toml \
    --deck1 aggressive_assault --deck2 iron_fortress \
    --games 1000

  # Debug mode with full logging
  cargo run --release --bin arena -- \
    --bot1 greedy --bot2 mcts \
    --seed 12345 --debug --log-file debug.log

  # Round-robin tournament across all decks
  cargo run --release --bin arena -- \
    --bot1 greedy --bot2 mcts \
    --matchups all-pairs --games-per-matchup 100


  # === TUNE: Optimize parameters ===

  # Tune generalist greedy weights
  cargo run --release --bin tune -- \
    --bot greedy \
    --mode generalist \
    --matchups all-pairs \
    --games-per-eval 500 \
    --generations 100 \
    --output weights/greedy_generalist.toml

  # Tune specialist for specific deck
  cargo run --release --bin tune -- \
    --bot greedy \
    --mode specialist \
    --deck aggressive_assault \
    --games-per-eval 200 \
    --generations 50 \
    --output weights/greedy_aggro_specialist.toml

  # Tune MCTS parameters
  cargo run --release --bin tune -- \
    --bot mcts \
    --mode generalist \
    --games-per-eval 100 \
    --output weights/mcts_tuned.toml

  ---
##  Implementation Plan

  I'll break this into phases with clear deliverables:

###  Phase 1: Foundation (Bot Trait + Infrastructure)

  1. Create src/bots/mod.rs - Bot trait definition
  2. Create src/bots/random.rs - RandomBot (baseline)
  3. Create src/arena/mod.rs - Module structure
  4. Create src/arena/logger.rs - ActionLogger with debug support
  5. Create src/arena/runner.rs - GameRunner for single games
  6. Create src/arena/stats.rs - MatchStats collection
  7. Add basic src/bin/arena.rs CLI

  Deliverable: Can run RandomBot vs RandomBot with seeded, logged games

###  Phase 2: GreedyBot

  1. Create src/bots/weights.rs - Weight structures and serialization
  2. Create src/bots/greedy.rs - GreedyBot with action simulation
  3. Add default weight values
  4. Unit tests for evaluation logic

  Deliverable: GreedyBot that beats RandomBot consistently

###  Phase 3: MCTS Bot

  1. Create src/bots/mcts.rs - MCTS with root parallelization
  2. Implement UCB1 selection
  3. Use GreedyBot for rollout policy
  4. Add simulation limits (count and time)
  5. Unit tests for tree operations

  Deliverable: MctsBot that beats GreedyBot

###  Phase 4: Deck System

  1. Create src/decks/mod.rs - DeckDefinition, DeckRegistry
  2. Create data/decks/ with starter decks as TOML files
  3. Update arena CLI to support deck selection
  4. Add matchup modes (all-pairs, mirror, custom)

  Deliverable: Can run tournaments across multiple decks

###  Phase 5: Tuning Pipeline

  1. Create src/tuning/evaluator.rs - Multi-game evaluation
  2. Create src/tuning/optimizer.rs - CMA-ES optimizer
  3. Create src/tuning/greedy_tuner.rs - Greedy weight tuning
  4. Create src/tuning/mcts_tuner.rs - MCTS parameter tuning
  5. Create src/bin/tune.rs - Tuning CLI
  6. Add generalist vs specialist modes

  Deliverable: Full tuning pipeline producing optimized weights

###  Phase 6: Polish & Performance

  1. Benchmark and optimize hot paths
  2. Add progress bars for long-running operations
  3. Add weight file versioning
  4. Documentation and examples
  5. Integration tests for full pipeline

  ---
###  File Structure Summary

  src/
  ├── lib.rs                      # Add: pub mod bots, arena, tuning, decks
  ├── bots/
  │   ├── mod.rs                  # Bot trait, re-exports
  │   ├── weights.rs              # BotWeights, WeightSet, serialization
  │   ├── random.rs               # RandomBot
  │   ├── greedy.rs               # GreedyBot
  │   └── mcts.rs                 # MctsBot with root parallelization
  ├── arena/
  │   ├── mod.rs
  │   ├── runner.rs               # GameRunner
  │   ├── logger.rs               # ActionLogger
  │   └── stats.rs                # MatchStats, MatchupStats
  ├── tuning/
  │   ├── mod.rs
  │   ├── evaluator.rs            # EvaluationConfig, run evaluations
  │   ├── optimizer.rs            # CMA-ES implementation
  │   ├── greedy_tuner.rs
  │   └── mcts_tuner.rs
  ├── decks/
  │   └── mod.rs                  # DeckDefinition, DeckRegistry
  └── bin/
      ├── arena.rs                # Match runner CLI
      └── tune.rs                 # Tuning CLI

  data/
  ├── cards/sets/                 # Existing card definitions
  │   └── starter.yaml
  ├── decks/                      # NEW: Deck definitions
  │   ├── aggressive_assault.toml
  │   └── iron_fortress.toml
  └── weights/                    # NEW: Bot weight files
      ├── greedy_default.toml
      ├── mcts_default.toml
      └── ... (tuned weights)

  ---