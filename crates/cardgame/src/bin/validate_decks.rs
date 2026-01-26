//! Validates all decks to ensure they only reference cards that exist in the database.
//!
//! This tool helps prevent runtime errors by checking deck integrity before deployment.

use cardgame::{CardDatabase, DeckRegistry};
use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "validate_decks")]
#[command(about = "Validate that all decks only reference existing cards")]
struct Args {
    /// Path to cards directory
    #[arg(long, default_value = "data/cards/core_set")]
    cards: PathBuf,

    /// Path to commanders directory
    #[arg(long, default_value = "data/commanders")]
    commanders: PathBuf,

    /// Path to decks directory
    #[arg(long, default_value = "data/decks")]
    decks: PathBuf,

    /// Exit with error code if validation fails
    #[arg(long, default_value = "true")]
    strict: bool,
}

fn main() {
    let args = Args::parse();

    println!("Loading card database from: {:?}", args.cards);
    println!("Loading commanders from: {:?}", args.commanders);
    let card_db = match CardDatabase::load_with_commanders(&args.cards, &args.commanders) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading card database and commanders: {}", e);
            process::exit(1);
        }
    };

    println!("Loading deck registry from: {:?}", args.decks);
    let deck_registry = match DeckRegistry::load_from_directory(&args.decks) {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("Error loading deck registry: {}", e);
            process::exit(1);
        }
    };

    println!("\nCard Database:");
    println!("  Total cards: {}", card_db.len());

    println!("\nValidating decks...\n");

    let mut total_decks = 0;
    let mut total_errors = 0;
    let mut decks_with_errors = Vec::new();

    for deck in deck_registry.decks() {
        total_decks += 1;
        let card_ids = deck.to_card_ids();
        let mut deck_errors = Vec::new();

        // Check each card ID
        for card_id in &card_ids {
            if card_db.get(*card_id).is_none() {
                deck_errors.push(*card_id);
            }
        }

        if deck_errors.is_empty() {
            println!("✓ {} ({} cards)", deck.name, card_ids.len());
        } else {
            let error_count = deck_errors.len();
            println!("✗ {} ({} cards) - {} INVALID CARD(S)", 
                     deck.name, card_ids.len(), error_count);
            for invalid_id in &deck_errors {
                println!("    - Card ID {} not found in database", invalid_id.0);
            }
            decks_with_errors.push((deck.name.clone(), deck_errors));
            total_errors += error_count;
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("  Total decks checked: {}", total_decks);
    println!("  Decks with errors: {}", decks_with_errors.len());
    println!("  Total invalid card references: {}", total_errors);

    if !decks_with_errors.is_empty() {
        println!("\n{}", "=".repeat(60));
        println!("VALIDATION FAILED!");
        println!("\nDecks with invalid card references:");
        for (deck_name, _errors) in &decks_with_errors {
            println!("  - {}", deck_name);
        }

        if args.strict {
            process::exit(1);
        }
    } else {
        println!("\n✓ All decks validated successfully!");
    }
}
