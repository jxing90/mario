// Feature #8: Collectibles & Blocks — LootTable probability distribution
// Design Reference: docs/features/8-collectibles-blocks.md §4, §6, §8
//
// Stateless module providing deterministic reward selection based on a
// uniform random value in [0.0, 1.0].

use crate::entities::power_up::PowerUpKind;

/// A stateless loot table that maps a uniform random value in [0.0, 1.0]
/// to a PowerUpKind reward.
///
/// §8 Data Model: No state fields. Pure-function module.
///
/// # Probability Distribution (§4, §6)
/// | Interval        | Reward        | Probability |
/// |-----------------|---------------|-------------|
/// | [0.00, 0.65)    | Coin          | 65%         |
/// | [0.65, 0.80)    | SuperMushroom | 15%         |
/// | [0.80, 0.95)    | FireFlower    | 15%         |
/// | [0.95, 1.00]    | Starman       |  5%         |
pub struct LootTable;

impl LootTable {
    /// Deterministic roll using an externally-provided value in [0.0, 1.0].
    ///
    /// # Interval mapping (§4, §6)
    /// - `val < 0.65` → `PowerUpKind::Coin`
    /// - `0.65 <= val < 0.80` → `PowerUpKind::SuperMushroom`
    /// - `0.80 <= val < 0.95` → `PowerUpKind::FireFlower`
    /// - `0.95 <= val <= 1.00` → `PowerUpKind::Starman`
    ///
    /// # Boundary conditions (§Implementation Summary)
    /// - `val = 0.00` → Coin (lower inclusive)
    /// - `val = 0.65` → SuperMushroom (Coin upper exclusive)
    /// - `val = 0.80` → FireFlower (Mushroom upper exclusive)
    /// - `val = 0.95` → Starman (Flower upper exclusive)
    /// - `val = 1.00` → Starman (upper inclusive)
    pub fn roll_with_value(val: f32) -> PowerUpKind {
        if val < 0.65 {
            PowerUpKind::Coin
        } else if val < 0.80 {
            PowerUpKind::SuperMushroom
        } else if val < 0.95 {
            PowerUpKind::FireFlower
        } else {
            PowerUpKind::Starman
        }
    }
}
