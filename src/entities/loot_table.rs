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
/// | [0.00, 0.60)    | Coin          | 60%         |
/// | [0.60, 0.75)    | SuperMushroom | 15%         |
/// | [0.75, 0.90)    | FireFlower    | 15%         |
/// | [0.90, 0.95)    | Starman       |  5%         |
/// | [0.95, 1.00]    | OneUpMushroom |  5%         |
pub struct LootTable;

impl LootTable {
    /// Deterministic roll using an externally-provided value in [0.0, 1.0].
    ///
    /// # Interval mapping (§4, §6)
    /// - `val < 0.60` → `PowerUpKind::Coin`
    /// - `0.60 <= val < 0.75` → `PowerUpKind::SuperMushroom`
    /// - `0.75 <= val < 0.90` → `PowerUpKind::FireFlower`
    /// - `0.90 <= val < 0.95` → `PowerUpKind::Starman`
    /// - `0.95 <= val <= 1.00` → `PowerUpKind::OneUpMushroom`
    pub fn roll_with_value(val: f32) -> PowerUpKind {
        if val < 0.60 {
            PowerUpKind::Coin
        } else if val < 0.75 {
            PowerUpKind::SuperMushroom
        } else if val < 0.90 {
            PowerUpKind::FireFlower
        } else if val < 0.95 {
            PowerUpKind::Starman
        } else {
            PowerUpKind::OneUpMushroom
        }
    }
}
