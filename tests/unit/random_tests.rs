//! Unit tests for random bot.

use cardgame::actions::Action;
use cardgame::bots::{Bot, RandomBot};
use cardgame::tensor::STATE_TENSOR_SIZE;
use cardgame::types::Slot;

#[test]
fn test_random_bot_selects_from_legal_actions() {
    let mut bot = RandomBot::new(42);
    let state = [0.0f32; STATE_TENSOR_SIZE];
    let mask = [0.0f32; 256];
    let legal = vec![Action::EndTurn, Action::PlayCard { hand_index: 0, slot: Slot(0) }];

    for _ in 0..10 {
        let action = bot.select_action(&state, &mask, &legal);
        assert!(legal.contains(&action));
    }
}

#[test]
fn test_random_bot_deterministic_with_same_seed() {
    let state = [0.0f32; STATE_TENSOR_SIZE];
    let mask = [0.0f32; 256];
    let legal = vec![
        Action::EndTurn,
        Action::PlayCard { hand_index: 0, slot: Slot(0) },
        Action::PlayCard { hand_index: 1, slot: Slot(1) },
    ];

    let mut bot1 = RandomBot::new(12345);
    let mut bot2 = RandomBot::new(12345);

    let actions1: Vec<_> = (0..10)
        .map(|_| bot1.select_action(&state, &mask, &legal))
        .collect();
    let actions2: Vec<_> = (0..10)
        .map(|_| bot2.select_action(&state, &mask, &legal))
        .collect();

    assert_eq!(actions1, actions2);
}

#[test]
fn test_random_bot_reset_reseeds() {
    let mut bot = RandomBot::new(42);
    let state = [0.0f32; STATE_TENSOR_SIZE];
    let mask = [0.0f32; 256];
    let legal = vec![Action::EndTurn, Action::PlayCard { hand_index: 0, slot: Slot(0) }];

    let first_run: Vec<_> = (0..5)
        .map(|_| bot.select_action(&state, &mask, &legal))
        .collect();

    bot.reset();

    let second_run: Vec<_> = (0..5)
        .map(|_| bot.select_action(&state, &mask, &legal))
        .collect();

    assert_eq!(first_run, second_run);
}
