//! Keyword definitions and bitfield implementation.
//!
//! Keywords are packed into a single byte for maximum efficiency.
//! Each bit represents one keyword, allowing O(1) checks and modifications.

use serde::{Deserialize, Serialize};

/// Keywords packed into a single byte for efficiency.
/// Each bit represents one keyword.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Keywords(pub u8);

impl Keywords {
    // Bit positions for each keyword
    pub const RUSH: u8      = 0b0000_0001;
    pub const RANGED: u8    = 0b0000_0010;
    pub const PIERCING: u8  = 0b0000_0100;
    pub const GUARD: u8     = 0b0000_1000;
    pub const LIFESTEAL: u8 = 0b0001_0000;
    pub const LETHAL: u8    = 0b0010_0000;
    pub const SHIELD: u8    = 0b0100_0000;
    pub const QUICK: u8     = 0b1000_0000;

    /// Create empty keywords
    pub const fn none() -> Self { Self(0) }

    /// Create with all keywords
    pub const fn all() -> Self { Self(0xFF) }

    // Fast keyword checks - single bitwise AND
    #[inline(always)] pub const fn has_rush(self) -> bool { self.0 & Self::RUSH != 0 }
    #[inline(always)] pub const fn has_ranged(self) -> bool { self.0 & Self::RANGED != 0 }
    #[inline(always)] pub const fn has_piercing(self) -> bool { self.0 & Self::PIERCING != 0 }
    #[inline(always)] pub const fn has_guard(self) -> bool { self.0 & Self::GUARD != 0 }
    #[inline(always)] pub const fn has_lifesteal(self) -> bool { self.0 & Self::LIFESTEAL != 0 }
    #[inline(always)] pub const fn has_lethal(self) -> bool { self.0 & Self::LETHAL != 0 }
    #[inline(always)] pub const fn has_shield(self) -> bool { self.0 & Self::SHIELD != 0 }
    #[inline(always)] pub const fn has_quick(self) -> bool { self.0 & Self::QUICK != 0 }

    // Generic check
    #[inline(always)]
    pub const fn has(self, keyword: u8) -> bool { self.0 & keyword != 0 }

    // Mutators
    #[inline(always)] pub fn add(&mut self, keyword: u8) { self.0 |= keyword; }
    #[inline(always)] pub fn remove(&mut self, keyword: u8) { self.0 &= !keyword; }
    #[inline(always)] pub fn clear(&mut self) { self.0 = 0; }

    // Builder pattern for readable card definitions
    pub const fn with_rush(self) -> Self { Self(self.0 | Self::RUSH) }
    pub const fn with_ranged(self) -> Self { Self(self.0 | Self::RANGED) }
    pub const fn with_piercing(self) -> Self { Self(self.0 | Self::PIERCING) }
    pub const fn with_guard(self) -> Self { Self(self.0 | Self::GUARD) }
    pub const fn with_lifesteal(self) -> Self { Self(self.0 | Self::LIFESTEAL) }
    pub const fn with_lethal(self) -> Self { Self(self.0 | Self::LETHAL) }
    pub const fn with_shield(self) -> Self { Self(self.0 | Self::SHIELD) }
    pub const fn with_quick(self) -> Self { Self(self.0 | Self::QUICK) }

    /// Combine keywords from two sources
    pub const fn union(self, other: Keywords) -> Keywords {
        Keywords(self.0 | other.0)
    }

    /// Parse keywords from a list of string names (for YAML loading)
    pub fn from_names(names: &[&str]) -> Self {
        let mut kw = Self::none();
        for name in names {
            match name.to_lowercase().as_str() {
                "rush" => kw.add(Self::RUSH),
                "ranged" => kw.add(Self::RANGED),
                "piercing" => kw.add(Self::PIERCING),
                "guard" => kw.add(Self::GUARD),
                "lifesteal" => kw.add(Self::LIFESTEAL),
                "lethal" => kw.add(Self::LETHAL),
                "shield" => kw.add(Self::SHIELD),
                "quick" => kw.add(Self::QUICK),
                _ => {} // Ignore unknown keywords
            }
        }
        kw
    }

    /// Convert to a list of keyword names (for debugging/display)
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.has_rush() { names.push("Rush"); }
        if self.has_ranged() { names.push("Ranged"); }
        if self.has_piercing() { names.push("Piercing"); }
        if self.has_guard() { names.push("Guard"); }
        if self.has_lifesteal() { names.push("Lifesteal"); }
        if self.has_lethal() { names.push("Lethal"); }
        if self.has_shield() { names.push("Shield"); }
        if self.has_quick() { names.push("Quick"); }
        names
    }
}
