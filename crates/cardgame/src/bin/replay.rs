//! Replay CLI - Step through recorded games.
//!
//! Usage:
//!   # From replay file
//!   cargo run --release --bin replay -- --file game.replay.json
//!   cargo run --release --bin replay -- --file game.replay.json.gz --interactive
//!
//!   # From seed (recreate game with bots)
//!   cargo run --release --bin replay -- --seed 12345 --deck1 broodmother_pack --deck2 artificer_control
//!
//!   # From JSONL dataset
//!   cargo run --release --bin replay -- --dataset data.jsonl.gz --game-index 42
//!
//!   # Export formats
//!   cargo run --release --bin replay -- --file game.replay.json --export transcript -o game.txt

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::{Parser, ValueEnum};
use flate2::read::GzDecoder;
use serde::Deserialize;

use cardgame::bots::{create_bot, AlphaBetaConfig, MctsConfig};
use cardgame::core::actions::Action;
use cardgame::core::state::GameMode;
use cardgame::core::types::CardId;
use cardgame::execution::{parse_bot_type_or_exit, GameData, MAX_ACTIONS_PER_GAME};
use cardgame::replay::{
    load, save, validate_replay, GameReplay, PlayerConfig, ReplayBuilder, ReplayError,
    ReplayIterator,
};
use cardgame::GameEngine;

/// Replay CLI - Step through recorded games
#[derive(Parser, Debug)]
#[command(name = "replay")]
#[command(about = "Replay Essence Wars games from files, seeds, or datasets")]
struct Args {
    // === Source (exactly one required) ===
    /// Load replay from file (.replay.json or .replay.json.gz)
    #[arg(long, short = 'f', group = "source")]
    file: Option<PathBuf>,

    /// Recreate game from seed (requires --deck1 and --deck2)
    #[arg(long, short = 's', group = "source")]
    seed: Option<u64>,

    /// Load game from JSONL dataset file
    #[arg(long, group = "source")]
    dataset: Option<PathBuf>,

    // === Source Options ===
    /// Deck 1 ID (required with --seed)
    #[arg(long)]
    deck1: Option<String>,

    /// Deck 2 ID (required with --seed)
    #[arg(long)]
    deck2: Option<String>,

    /// Bot type for replay generation (with --seed)
    #[arg(long, default_value = "greedy")]
    bot: String,

    /// Game index to extract from dataset (0-based, with --dataset)
    #[arg(long)]
    game_index: Option<usize>,

    // === Output Mode ===
    /// Interactive step-through mode
    #[arg(long, short = 'i')]
    interactive: bool,

    /// Export to specified format
    #[arg(long, short = 'e')]
    export: Option<ExportFormat>,

    /// Output path for export
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    // === Display Options ===
    /// Step to specific action index before displaying
    #[arg(long)]
    seek: Option<usize>,

    // === Data Paths ===
    /// Path to card database
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commander definitions
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to deck definitions
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    /// Plain text transcript of actions
    Transcript,
    /// Convert to compressed replay format
    Replay,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Load game data
    let game_data = match GameData::load_with_overrides(
        Some(&args.cards),
        Some(&args.commanders),
        Some(&args.decks),
        None,
        true, // quiet
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading game data: {}", e);
            process::exit(1);
        }
    };

    // Load or construct the replay
    let replay = match load_replay(&args, &game_data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Dispatch to output mode
    let result = if args.interactive {
        run_interactive(&replay, &game_data, &args)
    } else if let Some(format) = args.export {
        run_export(&replay, &args, format)
    } else {
        run_summary(&replay, &game_data)
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Load a GameReplay from the specified source.
fn load_replay(args: &Args, game_data: &GameData) -> Result<GameReplay, String> {
    if let Some(ref path) = args.file {
        load_from_file(path, &game_data.card_db)
    } else if let Some(seed) = args.seed {
        let deck1 = args
            .deck1
            .as_ref()
            .ok_or("--deck1 required with --seed")?;
        let deck2 = args
            .deck2
            .as_ref()
            .ok_or("--deck2 required with --seed")?;
        generate_from_seed(seed, deck1, deck2, &args.bot, game_data)
    } else if let Some(ref path) = args.dataset {
        load_from_dataset(path, args.game_index, game_data)
    } else {
        Err("Must specify --file, --seed, or --dataset. See --help.".into())
    }
}

/// Load replay from a file with automatic validation.
fn load_from_file(
    path: &std::path::Path,
    card_db: &cardgame::cards::CardDatabase,
) -> Result<GameReplay, String> {
    let replay =
        load(path).map_err(|e| format!("Failed to load replay from {:?}: {}", path, e))?;

    // Auto-validate file-based replays to catch corruption early
    validate_replay(&replay, card_db).map_err(|e| format!("Replay validation failed: {}", e))?;

    Ok(replay)
}

/// Generate a replay by running a game with bots.
fn generate_from_seed(
    seed: u64,
    deck1_id: &str,
    deck2_id: &str,
    bot_str: &str,
    game_data: &GameData,
) -> Result<GameReplay, String> {
    // Get deck definitions
    let deck1 = game_data
        .deck_registry
        .get(deck1_id)
        .ok_or_else(|| format!("Deck not found: {}", deck1_id))?;
    let deck2 = game_data
        .deck_registry
        .get(deck2_id)
        .ok_or_else(|| format!("Deck not found: {}", deck2_id))?;

    // Parse bot type
    let bot_type = parse_bot_type_or_exit(bot_str);

    // Create replay builder
    let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

    let mut builder = ReplayBuilder::new(
        seed,
        GameMode::Attrition,
        PlayerConfig {
            name: "Player 1".into(),
            player_type: bot_type.name().into(),
            deck: deck1_cards.clone(),
            deck_name: Some(deck1_id.into()),
            commander: Some(CardId(deck1.commander)),
        },
        PlayerConfig {
            name: "Player 2".into(),
            player_type: bot_type.name().into(),
            deck: deck2_cards.clone(),
            deck_name: Some(deck2_id.into()),
            commander: Some(CardId(deck2.commander)),
        },
    );

    // Initialize game
    let mut engine = GameEngine::new(&game_data.card_db);
    engine
        .start_game_raw(
            deck1_cards,
            deck2_cards,
            CardId(deck1.commander),
            CardId(deck2.commander),
            seed,
            GameMode::Attrition,
        )
        .map_err(|e| format!("Failed to start game: {}", e))?;

    // Create bots with default configs
    let mcts_config = MctsConfig::default();
    let alphabeta_config = AlphaBetaConfig::default();
    let mut bot1 = create_bot(
        &game_data.card_db,
        &bot_type,
        None,
        &mcts_config,
        &alphabeta_config,
        seed,
    );
    let mut bot2 = create_bot(
        &game_data.card_db,
        &bot_type,
        None,
        &mcts_config,
        &alphabeta_config,
        seed.wrapping_add(1),
    );

    // Run the game
    let start = Instant::now();
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < MAX_ACTIONS_PER_GAME {
        let turn = engine.state.current_turn;
        let current_player = engine.current_player();

        // Select action
        let tensor = engine.get_state_tensor();
        let mask = engine.get_legal_action_mask();
        let actions = engine.get_legal_actions();

        let action = if current_player.0 == 0 {
            bot1.select_action(&tensor, &mask, &actions)
        } else {
            bot2.select_action(&tensor, &mask, &actions)
        };

        // Record and apply
        builder.record_action(turn, action, None);
        engine
            .apply_action(action)
            .map_err(|e| format!("Action failed: {}", e))?;
        action_count += 1;
    }

    let elapsed = start.elapsed();
    let final_life = [engine.state.players[0].life, engine.state.players[1].life];

    println!(
        "Generated game: {} actions in {:.1}ms",
        action_count,
        elapsed.as_secs_f64() * 1000.0
    );

    Ok(builder.finalize(engine.state.result, engine.state.current_turn, final_life))
}

/// Dataset record structure (matches generate_dataset output).
#[derive(Deserialize)]
struct DatasetRecord {
    #[allow(dead_code)] // Present in dataset, may be used for lookup by ID in future
    game_id: String,
    deck1: String,
    deck2: String,
    winner: i8,
    moves: Vec<DatasetMove>,
    metadata: DatasetMetadata,
}

#[derive(Deserialize)]
struct DatasetMove {
    turn: u8,
    action: u8,
}

#[derive(Deserialize)]
struct DatasetMetadata {
    total_turns: u8,
    seed: u64,
}

/// Load a game from a JSONL dataset file.
fn load_from_dataset(
    path: &PathBuf,
    game_index: Option<usize>,
    game_data: &GameData,
) -> Result<GameReplay, String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open dataset {:?}: {}", path, e))?;

    let reader: Box<dyn BufRead> = if path.extension().is_some_and(|e| e == "gz") {
        Box::new(BufReader::new(GzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let target_idx = game_index.unwrap_or(0);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Read error at line {}: {}", idx, e))?;

        if idx == target_idx {
            let record: DatasetRecord = serde_json::from_str(&line)
                .map_err(|e| format!("Parse error at line {}: {}", idx, e))?;

            return convert_dataset_to_replay(record, game_data);
        }
    }

    Err(format!(
        "Game index {} not found in dataset (file may have fewer games)",
        target_idx
    ))
}

/// Convert a dataset record to a GameReplay.
fn convert_dataset_to_replay(
    record: DatasetRecord,
    game_data: &GameData,
) -> Result<GameReplay, String> {
    // Look up decks to get full card lists and commanders
    let deck1 = game_data
        .deck_registry
        .get(&record.deck1)
        .ok_or_else(|| format!("Deck not found: {}", record.deck1))?;
    let deck2 = game_data
        .deck_registry
        .get(&record.deck2)
        .ok_or_else(|| format!("Deck not found: {}", record.deck2))?;

    let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();

    let mut replay = GameReplay::new(
        record.metadata.seed,
        GameMode::Attrition,
        PlayerConfig {
            name: "Player 1".into(),
            player_type: "mcts".into(),
            deck: deck1_cards,
            deck_name: Some(record.deck1),
            commander: Some(CardId(deck1.commander)),
        },
        PlayerConfig {
            name: "Player 2".into(),
            player_type: "mcts".into(),
            deck: deck2_cards,
            deck_name: Some(record.deck2),
            commander: Some(CardId(deck2.commander)),
        },
    );

    for m in record.moves {
        if let Some(action) = Action::from_index(m.action) {
            replay.add_action(m.turn as u16, action, None);
        }
    }

    replay.set_result(
        if record.winner >= 0 {
            Some(cardgame::core::state::GameResult::Win {
                winner: cardgame::core::types::PlayerId(record.winner as u8),
                reason: cardgame::core::state::WinReason::LifeReachedZero,
            })
        } else {
            Some(cardgame::core::state::GameResult::Draw)
        },
        record.metadata.total_turns as u16,
        [0, 0], // Not available in dataset
    );

    Ok(replay)
}

/// Display a summary of the replay.
fn run_summary(replay: &GameReplay, game_data: &GameData) -> Result<(), String> {
    println!("=== Replay Summary ===");
    println!();

    // Header info
    println!("Engine Version: {}", replay.header.engine_version);
    println!("Seed: {}", replay.header.seed);
    println!("Mode: {:?}", replay.header.mode);
    println!();

    // Player info
    println!(
        "Player 1: {} ({})",
        replay.player1.deck_name.as_deref().unwrap_or("unknown"),
        replay.player1.player_type
    );
    if let Some(cmd_id) = replay.player1.commander {
        if let Some(def) = game_data.card_db.get_commander(cmd_id) {
            println!("  Commander: {}", def.name);
        }
    }

    println!(
        "Player 2: {} ({})",
        replay.player2.deck_name.as_deref().unwrap_or("unknown"),
        replay.player2.player_type
    );
    if let Some(cmd_id) = replay.player2.commander {
        if let Some(def) = game_data.card_db.get_commander(cmd_id) {
            println!("  Commander: {}", def.name);
        }
    }
    println!();

    // Result
    println!("Total Turns: {}", replay.result.total_turns);
    println!("Total Actions: {}", replay.actions.len());
    println!(
        "Final Life: P1={}, P2={}",
        replay.result.final_life[0], replay.result.final_life[1]
    );

    match replay.result.winner {
        Some(0) => println!("Result: Player 1 WINS ({})", replay.result.reason),
        Some(1) => println!("Result: Player 2 WINS ({})", replay.result.reason),
        _ => println!("Result: DRAW"),
    }

    // Action breakdown
    println!();
    println!("Action Breakdown:");
    let mut plays = 0;
    let mut attacks = 0;
    let mut abilities = 0;
    let mut end_turns = 0;

    for ra in &replay.actions {
        match ra.action {
            Action::PlayCard { .. } => plays += 1,
            Action::Attack { .. } => attacks += 1,
            Action::UseAbility { .. } | Action::CommanderInsight => abilities += 1,
            Action::EndTurn => end_turns += 1,
        }
    }

    println!("  PlayCard: {}", plays);
    println!("  Attack: {}", attacks);
    println!("  Abilities: {}", abilities);
    println!("  EndTurn: {}", end_turns);

    Ok(())
}

/// Interactive step-through mode.
fn run_interactive(replay: &GameReplay, game_data: &GameData, args: &Args) -> Result<(), String> {
    let mut iter = ReplayIterator::new(replay, &game_data.card_db);

    // Print header
    println!("=== Essence Wars Replay ===");
    println!(
        "Seed: {} | Mode: {:?}",
        replay.header.seed, replay.header.mode
    );
    println!(
        "P1: {} vs P2: {}",
        replay.player1.deck_name.as_deref().unwrap_or("?"),
        replay.player2.deck_name.as_deref().unwrap_or("?")
    );
    println!("Total: {} actions", replay.actions.len());
    println!();

    // Seek to initial position if specified
    if let Some(pos) = args.seek {
        iter.seek(pos).map_err(|e| e.to_string())?;
        println!("Seeked to action {}", pos);
    }

    // Show initial state
    print_state(&iter, &game_data.card_db);

    // Interactive loop
    loop {
        print!(
            "\n[{}/{}{}] (n)ext, (p)rev, (s)eek N, (q)uit > ",
            iter.current_index(),
            iter.total_actions(),
            if iter.is_at_end() { " END" } else { "" }
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        match input {
            "n" | "" => {
                match iter.step() {
                    Ok(Some(step)) => {
                        println!(
                            "\n--- Action {}: {:?} (Turn {}) ---",
                            step.action_index, step.action, step.turn
                        );
                        print_state(&iter, &game_data.card_db);
                    }
                    Ok(None) => println!("\nEnd of replay."),
                    Err(ReplayError::EndOfReplay) => println!("\nEnd of replay."),
                    Err(e) => println!("\nError: {}", e),
                }
            }
            "p" => {
                let idx = iter.current_index();
                if idx > 1 {
                    iter.seek(idx - 2).ok();
                    iter.step().ok();
                    print_state(&iter, &game_data.card_db);
                } else if idx == 1 {
                    iter.reset();
                    print_state(&iter, &game_data.card_db);
                } else {
                    println!("Already at beginning.");
                }
            }
            "q" => break,
            s if s.starts_with('s') => {
                let num_str = s.trim_start_matches('s').trim();
                if let Ok(pos) = num_str.parse::<usize>() {
                    match iter.seek(pos) {
                        Ok(()) => {
                            println!("Seeked to action {}", pos);
                            print_state(&iter, &game_data.card_db);
                        }
                        Err(e) => println!("Seek error: {}", e),
                    }
                } else {
                    println!("Usage: s <action_index>");
                }
            }
            _ => println!("Unknown command. Use: n, p, s N, q"),
        }
    }

    Ok(())
}

/// Print current game state.
fn print_state(iter: &ReplayIterator, card_db: &cardgame::cards::CardDatabase) {
    if let Some(state) = iter.current_state() {
        println!(
            "\nTurn {} | Active: P{}",
            state.current_turn,
            state.active_player.0 + 1
        );
        println!(
            "P1: {} HP, {} essence | P2: {} HP, {} essence",
            state.players[0].life,
            state.players[0].current_essence,
            state.players[1].life,
            state.players[1].current_essence
        );

        // Show creatures
        for (player_idx, player) in state.players.iter().enumerate() {
            let creatures: Vec<String> = player
                .creatures
                .iter()
                .map(|c| {
                    let name = card_db
                        .get(c.card_id)
                        .map(|def| def.name.as_str())
                        .unwrap_or("?");
                    format!("{}({}/{})", name, c.attack, c.current_health)
                })
                .collect();

            if !creatures.is_empty() {
                println!("P{} Creatures: [{}]", player_idx + 1, creatures.join(", "));
            }
        }
    } else {
        println!("\n(Initial state - step to begin)");
    }
}

/// Export replay to different formats.
fn run_export(replay: &GameReplay, args: &Args, format: ExportFormat) -> Result<(), String> {
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("output"));

    match format {
        ExportFormat::Transcript => export_transcript(replay, &output),
        ExportFormat::Replay => export_replay(replay, &output),
    }
}

/// Export to plain text transcript.
fn export_transcript(replay: &GameReplay, path: &std::path::Path) -> Result<(), String> {
    let path = if path.extension().is_none() {
        path.with_extension("txt")
    } else {
        path.to_path_buf()
    };

    let mut file =
        File::create(&path).map_err(|e| format!("Failed to create file {:?}: {}", path, e))?;

    writeln!(file, "=== Essence Wars Game Transcript ===").ok();
    writeln!(file, "Seed: {}", replay.header.seed).ok();
    writeln!(
        file,
        "P1: {} | P2: {}",
        replay.player1.deck_name.as_deref().unwrap_or("?"),
        replay.player2.deck_name.as_deref().unwrap_or("?")
    )
    .ok();
    writeln!(file).ok();

    let mut current_turn = 0u16;
    for (i, ra) in replay.actions.iter().enumerate() {
        if ra.turn != current_turn {
            current_turn = ra.turn;
            writeln!(file, "\n--- Turn {} ---", current_turn).ok();
        }

        writeln!(file, "[{:3}] {:?}", i, ra.action).ok();
    }

    writeln!(file, "\n=== Result ===").ok();
    writeln!(
        file,
        "Winner: {:?} ({})",
        replay.result.winner, replay.result.reason
    )
    .ok();
    writeln!(
        file,
        "Final Life: P1={}, P2={}",
        replay.result.final_life[0], replay.result.final_life[1]
    )
    .ok();

    println!("Exported transcript to {:?}", path);
    Ok(())
}

/// Export to compressed replay format.
fn export_replay(replay: &GameReplay, path: &std::path::Path) -> Result<(), String> {
    let path = if path.extension().is_none() {
        path.with_extension("replay.json.gz")
    } else {
        path.to_path_buf()
    };

    save(replay, &path).map_err(|e| format!("Failed to save replay to {:?}: {}", path, e))?;

    println!("Exported replay to {:?}", path);
    Ok(())
}
