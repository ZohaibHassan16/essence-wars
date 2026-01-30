//! Effect handlers implementing the Strategy pattern.
//!
//! Each effect type has a dedicated handler that implements the `EffectHandler` trait.
//! This module provides the trait definition and the dispatch function.

mod damage;
mod stats;
mod keyword;
mod utility;
mod summon;

pub use damage::{DamageHandler, HealHandler};
pub use stats::{BuffStatsHandler, SetStatsHandler};
pub use keyword::{GrantKeywordHandler, RemoveKeywordHandler, SilenceHandler};
pub use utility::{DrawHandler, GainEssenceHandler, RefreshCreatureHandler, BounceHandler, DestroyHandler, DestroySelfHandler};
pub use summon::{SummonHandler, SummonTokenHandler, TransformHandler, CopyHandler};

use crate::core::effects::Effect;
use crate::core::types::PlayerId;

use super::effect_context::EffectContext;

/// Trait for handling specific effect types.
///
/// Each effect variant has a corresponding handler struct that implements
/// this trait. The handler encapsulates all the logic for applying that
/// specific effect type to the game state.
pub trait EffectHandler {
    /// Apply the effect to the game state.
    ///
    /// Returns `true` if the effect was successfully applied,
    /// `false` if it could not be applied (e.g., no valid targets).
    fn apply(&self, ctx: &mut EffectContext) -> bool;

    /// Returns `true` if this effect type is blocked by Ward on single targets.
    ///
    /// Most effects are blocked by Ward (default). Effects that don't target
    /// creatures (like Draw) or summon effects override this to return `false`.
    fn blocked_by_ward(&self) -> bool {
        true
    }
}

/// Create an appropriate handler for the given effect.
///
/// This is the central dispatch point that maps Effect enum variants
/// to their corresponding handler implementations.
pub fn create_handler(effect: &Effect, _source_player: PlayerId) -> Box<dyn EffectHandler + '_> {
    match effect {
        Effect::Damage { target, amount, filter } => {
            Box::new(DamageHandler {
                target: *target,
                amount: *amount,
                filter: filter.clone(),
            })
        }
        Effect::Heal { target, amount, filter } => {
            Box::new(HealHandler {
                target: *target,
                amount: *amount,
                filter: filter.clone(),
            })
        }
        Effect::Draw { player, count } => {
            Box::new(DrawHandler {
                player: *player,
                count: *count,
            })
        }
        Effect::BuffStats { target, attack, health, filter } => {
            Box::new(BuffStatsHandler {
                target: *target,
                attack: *attack,
                health: *health,
                filter: filter.clone(),
            })
        }
        Effect::SetStats { target, attack, health } => {
            Box::new(SetStatsHandler {
                target: *target,
                attack: *attack,
                health: *health,
            })
        }
        Effect::Destroy { target, filter } => {
            Box::new(DestroyHandler {
                target: *target,
                filter: filter.clone(),
            })
        }
        Effect::Summon { owner, card_id, slot } => {
            Box::new(SummonHandler {
                owner: *owner,
                card_id: *card_id,
                slot: *slot,
            })
        }
        Effect::SummonToken { owner, token, slot } => {
            Box::new(SummonTokenHandler {
                owner: *owner,
                token: token.clone(),
                slot: *slot,
            })
        }
        Effect::Transform { target, into } => {
            Box::new(TransformHandler {
                target: *target,
                into: into.clone(),
            })
        }
        Effect::Copy { target, owner } => {
            Box::new(CopyHandler {
                target: *target,
                owner: *owner,
            })
        }
        Effect::GrantKeyword { target, keyword, filter } => {
            Box::new(GrantKeywordHandler {
                target: *target,
                keyword: *keyword,
                filter: filter.clone(),
            })
        }
        Effect::RemoveKeyword { target, keyword, filter } => {
            Box::new(RemoveKeywordHandler {
                target: *target,
                keyword: *keyword,
                filter: filter.clone(),
            })
        }
        Effect::Silence { target, filter } => {
            Box::new(SilenceHandler {
                target: *target,
                filter: filter.clone(),
            })
        }
        Effect::GainEssence { player, amount } => {
            Box::new(GainEssenceHandler {
                player: *player,
                amount: *amount,
            })
        }
        Effect::RefreshCreature { target } => {
            Box::new(RefreshCreatureHandler {
                target: *target,
            })
        }
        Effect::Bounce { target, filter } => {
            Box::new(BounceHandler {
                target: *target,
                filter: filter.clone(),
            })
        }
        Effect::DestroySelf { owner, slot } => {
            Box::new(DestroySelfHandler {
                owner: *owner,
                slot: *slot,
            })
        }
    }
}
