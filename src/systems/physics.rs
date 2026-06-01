// Feature #5: Hazards — CollisionEvent enum and Physics collision dispatch
// Design Reference: docs/features/5-hazards.md §4, §6
//
// CollisionEvent is a shared enum consumed by F06 (Life/Death), F07 (Patrol Enemy),
// and F08 (Collectibles). Physics::hazard_check detects spike contact and pit fall.

use crate::entities::player::Player;
use crate::level::Tile;

/// Events produced by the physics collision system and consumed by
/// downstream features (F06 Life/Death, F07 Patrol Enemy, F08 Collectibles).
#[derive(Debug, Clone)]
pub enum CollisionEvent {
    /// Player's collider overlaps a Tile::Spike hazard zone.
    HazardContact,
    /// Player's Y position exceeds the kill-plane threshold (pos.y > kill_y).
    PitFall,
    /// Player's collider overlaps the flagpole trigger zone.
    FlagpoleReached,
}

/// Physics system — collision detection and event emission.
///
/// Unit struct; methods are pure functions acting on borrowed game state.
pub struct Physics;

impl Physics {
    /// Detects hazard-related collision events: spike contact and pit fall.
    ///
    /// # Parameters
    /// - `player`: Reference to the player entity for collider and position queries.
    /// - `terrain`: Slice of terrain tiles from `Level::query_terrain(player.collider())`.
    /// - `kill_y`: Kill-plane Y threshold from `Level::bounds().kill_y`.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing:
    ///   - One `HazardContact` for each `Tile::Spike` whose AABB overlaps the player's collider
    ///   - One `PitFall` if `player.pos().y > kill_y`
    /// - Returns an empty Vec if no hazard conditions are detected.
    ///
    /// # Design contract
    /// - F05 only detects and reports events — it does NOT filter by invulnerability.
    ///   Invulnerability exemption is F06's responsibility when consuming the event.
    /// - Spike checks iterate all terrain tiles first, then pit check runs last
    ///   (consistent with the flowchart TD decision order).
    /// - `Tile::Platform` and `Tile::Empty` variants are ignored (no false positives).
    /// - Empty terrain produces empty Vec (no panic on empty slice).
    pub fn hazard_check(
        player: &Player,
        terrain: &[Tile],
        kill_y: f32,
    ) -> Vec<CollisionEvent> {
        let mut events = Vec::new();

        // Phase 1: Check each terrain tile for spike overlap
        for tile in terrain {
            if let Tile::Spike(spike_aabb) = tile
                && player.collider().intersects(spike_aabb)
            {
                events.push(CollisionEvent::HazardContact);
            }
        }

        // Phase 2: Check pit fall (strict greater-than comparison)
        if player.pos().y > kill_y {
            events.push(CollisionEvent::PitFall);
        }

        events
    }
}
