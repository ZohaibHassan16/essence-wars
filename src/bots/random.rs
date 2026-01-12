//! Random bot implementation - selects actions uniformly at random.
//!
//! This serves as a baseline for comparing other bots.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::actions::Action;
use crate::bots::Bot;
use crate::tensor::STATE_TENSOR_SIZE;

/// A bot that selects actions uniformly at random.
///
/// Useful as a baseline for measuring other bots' performance.
#[derive(Clone)]
pub struct RandomBot {
    name: String,
    rng: SmallRng,
    seed: u64,
}

impl RandomBot {
    /// Create a new RandomBot with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            name: "RandomBot".to_string(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Create a RandomBot with a custom name.
    pub fn with_name(name: impl Into<String>, seed: u64) -> Self {
        Self {
            name: name.into(),
            rng: SmallRng::seed_from_u64(seed),
            seed,
        }
    }
}

impl Bot for RandomBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn select_action(
        &mut self,
        _state_tensor: &[f32; STATE_TENSOR_SIZE],
        _legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action {
        // Select uniformly at random from legal actions
        let idx = self.rng.gen_range(0..legal_actions.len());
        legal_actions[idx]
    }

    fn reset(&mut self) {
        // Re-seed the RNG for reproducibility across games
        self.rng = SmallRng::seed_from_u64(self.seed);
    }

    fn clone_box(&self) -> Box<dyn Bot> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Slot;

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
}
