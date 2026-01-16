//! Keyword definitions and bitfield implementation.
//!
//! Keywords are packed into a u16 for efficiency while allowing up to 16 keywords.
//! Each bit represents one keyword, allowing O(1) checks and modifications.

use serde::{Deserialize, Serialize};

/// Keywords packed into a u16 for efficiency.
/// Each bit represents one keyword. Bits 0-7 are original keywords, bits 8-11 are new.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Keywords(pub u16);

impl Keywords {
    // Original keywords (bits 0-7)
    pub const RUSH: u16      = 0x0001;  // bit 0
    pub const RANGED: u16    = 0x0002;  // bit 1
    pub const PIERCING: u16  = 0x0004;  // bit 2
    pub const GUARD: u16     = 0x0008;  // bit 3
    pub const LIFESTEAL: u16 = 0x0010;  // bit 4
    pub const LETHAL: u16    = 0x0020;  // bit 5
    pub const SHIELD: u16    = 0x0040;  // bit 6
    pub const QUICK: u16     = 0x0080;  // bit 7

    // New keywords (bits 8-11)
    pub const EPHEMERAL: u16   = 0x0100;  // bit 8 - dies at end of turn
    pub const REGENERATE: u16  = 0x0200;  // bit 9 - heals 2 HP at start of turn
    pub const STEALTH: u16     = 0x0400;  // bit 10 - untargetable by enemy attacks/spells
    pub const CHARGE: u16      = 0x0800;  // bit 11 - +2 attack when attacking

    // Symbiote keywords (bits 12-13) - added in v0.5.0 for balance
    pub const FRENZY: u16      = 0x1000;  // bit 12 - +1 attack after each attack this turn
    pub const VOLATILE: u16    = 0x2000;  // bit 13 - deal 2 damage to all enemy creatures on death

    // Phase 5 keywords (bits 14-15) - final keyword slots
    pub const FORTIFY: u16     = 0x4000;  // bit 14 - takes 1 less damage from all sources (min 1)
    pub const WARD: u16        = 0x8000;  // bit 15 - first spell/ability targeting this has no effect

    /// Create empty keywords
    pub const fn none() -> Self { Self(0) }

    /// Create with all keywords (16 keywords currently defined)
    pub const fn all() -> Self { Self(0xFFFF) }

    // Fast keyword checks for original keywords - single bitwise AND
    #[inline(always)] pub const fn has_rush(self) -> bool { self.0 & Self::RUSH != 0 }
    #[inline(always)] pub const fn has_ranged(self) -> bool { self.0 & Self::RANGED != 0 }
    #[inline(always)] pub const fn has_piercing(self) -> bool { self.0 & Self::PIERCING != 0 }
    #[inline(always)] pub const fn has_guard(self) -> bool { self.0 & Self::GUARD != 0 }
    #[inline(always)] pub const fn has_lifesteal(self) -> bool { self.0 & Self::LIFESTEAL != 0 }
    #[inline(always)] pub const fn has_lethal(self) -> bool { self.0 & Self::LETHAL != 0 }
    #[inline(always)] pub const fn has_shield(self) -> bool { self.0 & Self::SHIELD != 0 }
    #[inline(always)] pub const fn has_quick(self) -> bool { self.0 & Self::QUICK != 0 }

    // Fast keyword checks for new keywords
    #[inline(always)] pub const fn has_ephemeral(self) -> bool { self.0 & Self::EPHEMERAL != 0 }
    #[inline(always)] pub const fn has_regenerate(self) -> bool { self.0 & Self::REGENERATE != 0 }
    #[inline(always)] pub const fn has_stealth(self) -> bool { self.0 & Self::STEALTH != 0 }
    #[inline(always)] pub const fn has_charge(self) -> bool { self.0 & Self::CHARGE != 0 }

    // Fast keyword checks for Symbiote keywords
    #[inline(always)] pub const fn has_frenzy(self) -> bool { self.0 & Self::FRENZY != 0 }
    #[inline(always)] pub const fn has_volatile(self) -> bool { self.0 & Self::VOLATILE != 0 }

    // Fast keyword checks for Phase 5 keywords
    #[inline(always)] pub const fn has_fortify(self) -> bool { self.0 & Self::FORTIFY != 0 }
    #[inline(always)] pub const fn has_ward(self) -> bool { self.0 & Self::WARD != 0 }

    // Generic check
    #[inline(always)]
    pub const fn has(self, keyword: u16) -> bool { self.0 & keyword != 0 }

    // Mutators
    #[inline(always)] pub fn add(&mut self, keyword: u16) { self.0 |= keyword; }
    #[inline(always)] pub fn remove(&mut self, keyword: u16) { self.0 &= !keyword; }
    #[inline(always)] pub fn clear(&mut self) { self.0 = 0; }

    // Builder pattern for readable card definitions - original keywords
    pub const fn with_rush(self) -> Self { Self(self.0 | Self::RUSH) }
    pub const fn with_ranged(self) -> Self { Self(self.0 | Self::RANGED) }
    pub const fn with_piercing(self) -> Self { Self(self.0 | Self::PIERCING) }
    pub const fn with_guard(self) -> Self { Self(self.0 | Self::GUARD) }
    pub const fn with_lifesteal(self) -> Self { Self(self.0 | Self::LIFESTEAL) }
    pub const fn with_lethal(self) -> Self { Self(self.0 | Self::LETHAL) }
    pub const fn with_shield(self) -> Self { Self(self.0 | Self::SHIELD) }
    pub const fn with_quick(self) -> Self { Self(self.0 | Self::QUICK) }

    // Builder pattern for new keywords
    pub const fn with_ephemeral(self) -> Self { Self(self.0 | Self::EPHEMERAL) }
    pub const fn with_regenerate(self) -> Self { Self(self.0 | Self::REGENERATE) }
    pub const fn with_stealth(self) -> Self { Self(self.0 | Self::STEALTH) }
    pub const fn with_charge(self) -> Self { Self(self.0 | Self::CHARGE) }

    // Builder pattern for Symbiote keywords
    pub const fn with_frenzy(self) -> Self { Self(self.0 | Self::FRENZY) }
    pub const fn with_volatile(self) -> Self { Self(self.0 | Self::VOLATILE) }

    // Builder pattern for Phase 5 keywords
    pub const fn with_fortify(self) -> Self { Self(self.0 | Self::FORTIFY) }
    pub const fn with_ward(self) -> Self { Self(self.0 | Self::WARD) }

    /// Combine keywords from two sources
    pub const fn union(self, other: Keywords) -> Keywords {
        Keywords(self.0 | other.0)
    }

    /// Parse keywords from a list of string names (for YAML loading)
    pub fn from_names(names: &[&str]) -> Self {
        let mut kw = Self::none();
        for name in names {
            match name.to_lowercase().as_str() {
                // Original keywords
                "rush" => kw.add(Self::RUSH),
                "ranged" => kw.add(Self::RANGED),
                "piercing" => kw.add(Self::PIERCING),
                "guard" => kw.add(Self::GUARD),
                "lifesteal" => kw.add(Self::LIFESTEAL),
                "lethal" => kw.add(Self::LETHAL),
                "shield" => kw.add(Self::SHIELD),
                "quick" => kw.add(Self::QUICK),
                // New keywords
                "ephemeral" => kw.add(Self::EPHEMERAL),
                "regenerate" => kw.add(Self::REGENERATE),
                "stealth" => kw.add(Self::STEALTH),
                "charge" => kw.add(Self::CHARGE),
                // Symbiote keywords
                "frenzy" => kw.add(Self::FRENZY),
                "volatile" => kw.add(Self::VOLATILE),
                // Phase 5 keywords
                "fortify" => kw.add(Self::FORTIFY),
                "ward" => kw.add(Self::WARD),
                _ => {} // Ignore unknown keywords
            }
        }
        kw
    }

    /// Convert to a list of keyword names (for debugging/display)
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        // Original keywords
        if self.has_rush() { names.push("Rush"); }
        if self.has_ranged() { names.push("Ranged"); }
        if self.has_piercing() { names.push("Piercing"); }
        if self.has_guard() { names.push("Guard"); }
        if self.has_lifesteal() { names.push("Lifesteal"); }
        if self.has_lethal() { names.push("Lethal"); }
        if self.has_shield() { names.push("Shield"); }
        if self.has_quick() { names.push("Quick"); }
        // New keywords
        if self.has_ephemeral() { names.push("Ephemeral"); }
        if self.has_regenerate() { names.push("Regenerate"); }
        if self.has_stealth() { names.push("Stealth"); }
        if self.has_charge() { names.push("Charge"); }
        // Symbiote keywords
        if self.has_frenzy() { names.push("Frenzy"); }
        if self.has_volatile() { names.push("Volatile"); }
        // Phase 5 keywords
        if self.has_fortify() { names.push("Fortify"); }
        if self.has_ward() { names.push("Ward"); }
        names
    }
}
