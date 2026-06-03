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
/// | [0.00, 0.70)    | Coin          | 70%         |
/// | [0.70, 0.85)    | SuperMushroom | 15%         |
/// | [0.85, 1.00]    | FireFlower    | 15%         |
pub struct LootTable;

impl LootTable {
    /// Deterministic roll using an externally-provided value in [0.0, 1.0].
    ///
    /// # Interval mapping (§4, §6)
    /// - `val < 0.70` → `PowerUpKind::Coin`
    /// - `0.70 <= val < 0.85` → `PowerUpKind::SuperMushroom`
    /// - `0.85 <= val <= 1.00` → `PowerUpKind::FireFlower`
    ///
    /// # Boundary conditions (§Implementation Summary)
    /// - `val = 0.00` → Coin (lower inclusive)
    /// - `val = 0.70` → SuperMushroom (Coin upper exclusive)
    /// - `val = 0.85` → FireFlower (Mushroom upper exclusive)
    /// - `val = 1.00` → FireFlower (upper inclusive)
    pub fn roll_with_value(val: f32) -> PowerUpKind {
        if val < 0.70 {
            PowerUpKind::Coin
        } else if val < 0.85 {
            PowerUpKind::SuperMushroom
        } else {
            PowerUpKind::FireFlower
        }
    }
}
