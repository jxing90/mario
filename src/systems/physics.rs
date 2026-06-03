// Feature #5: Hazards — CollisionEvent enum and Physics collision dispatch
// Design Reference: docs/features/5-hazards.md §4, §6
//
// CollisionEvent is a shared enum consumed by F06 (Life/Death), F07 (Patrol Enemy),
// and F08 (Collectibles). Physics::hazard_check detects spike contact and pit fall.
//
// Feature #8: Collectibles & Blocks — extended with CoinCollect, QuestionBlockHit,
// PowerUpCollect, FireballHitEnemy variants + 4 new detection methods.

use crate::entities::coin::Coin;
use crate::entities::enemy::Enemy;
use crate::entities::fireball::Fireball;
use crate::entities::player::Player;
use crate::entities::power_up::PowerUp;
use crate::entities::question_block::QuestionBlock;
use crate::level::{AABB, Tile};

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
    /// Player stomped an enemy from above (index into enemies Vec).
    EnemyStomp(usize),
    /// Player contacted an enemy from the side or below (index into enemies Vec).
    EnemyContact(usize),
    /// Player collected a coin (index into coins Vec). Feature #8.
    CoinCollect(usize),
    /// Player hit a question block from below (index into blocks Vec). Feature #8.
    QuestionBlockHit(usize),
    /// Player collected a PowerUp item (index into power_ups Vec). Feature #8.
    PowerUpCollect(usize),
    /// Fireball hit an enemy (fireball_index, enemy_index). Feature #8.
    FireballHitEnemy(usize, usize),
}

/// Physics system — collision detection and event emission.
///
/// Unit struct; methods are pure functions acting on borrowed game state.
pub struct Physics;

/// 2-pixel tolerance for stomp detection: if the player's previous-frame bottom edge
/// is within 2px above the enemy's top edge, it counts as "was above" (§4 Interface Contract).
const STOMP_TOLERANCE: f32 = 2.0;

/// 4-pixel tolerance for question-block head-proximity check: player head must be
/// within this distance of the block bottom surface to count as a "hit from below"
/// (§4 Interface Contract, §Implementation Summary §3).
const HEAD_TOLERANCE: f32 = 4.0;

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

    /// Detects player-enemy collision events: stomp (from above) vs contact (side/below).
    ///
    /// # Parameters (from §4 Interface Contract)
    /// - `player`: Reference to the player entity for collider, position, and velocity queries.
    /// - `enemies`: Slice of all living enemies in the current frame.
    /// - `dt`: Fixed timestep (`1.0/60.0`), used to compute previous-frame player position.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing `EnemyStomp(i)` or `EnemyContact(i)` for each
    ///   overlapping pair.
    /// - Returns empty Vec if no collisions, enemies slice is empty, or enemy is dead.
    ///
    /// # Stomp detection (per flowchart TD branch#5)
    /// A stomp requires BOTH:
    /// 1. `player.vel.y > 0.0` (strictly downward velocity)
    /// 2. Player was above the enemy before this step:
    ///    `prev_player_bottom = player.collider().bottom - vel.y * dt`
    ///    `prev_player_bottom <= enemy.collider().top + 2.0`
    ///
    /// If both conditions are met → `EnemyStomp(i)`. Otherwise → `EnemyContact(i)`.
    pub fn enemy_check(
        player: &Player,
        enemies: &[Enemy],
        dt: f32,
    ) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        let player_col = player.collider();

        for (i, enemy) in enemies.iter().enumerate() {
            // Skip dead enemies (flowchart branch#2: CheckAlive / false → skip)
            if !enemy.alive {
                continue;
            }

            let enemy_col = enemy.collider();

            // Check AABB overlap (flowchart branch#3: CheckIntersect)
            if !player_col.intersects(&enemy_col) {
                continue;
            }

            // Determine stomp vs contact
            // Condition 1: player must be falling (vel.y > 0 in Y-down coords)
            // Condition 2: player must have been above enemy before this step
            let player_bottom = player_col.y + player_col.h;
            let prev_player_bottom = player_bottom - player.vel.y * dt;
            let enemy_top = enemy_col.y;

            let is_stomp = player.vel.y > 0.0
                && prev_player_bottom <= enemy_top + STOMP_TOLERANCE;

            if is_stomp {
                events.push(CollisionEvent::EnemyStomp(i));
            } else {
                events.push(CollisionEvent::EnemyContact(i));
            }
        }

        events
    }

    /// Detects coin collection events: player collider overlaps any uncollected coin.
    ///
    /// # Parameters (§4 Interface Contract)
    /// - `player`: Reference to the player entity for collider queries.
    /// - `coins`: Slice of all coins in the current frame.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing `CoinCollect(i)` for each uncollected coin
    ///   whose AABB overlaps the player's collider.
    /// - Returns empty Vec if coins slice is empty or no overlaps.
    ///
    /// # Design contract (§6)
    /// - Coins are pure triggers — no direction check, any AABB overlap triggers collection.
    /// - Already-collected coins (`collected == true`) are skipped.
    /// - Empty coins slice returns empty Vec (no panic).
    pub fn coin_check(player: &Player, coins: &[Coin]) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        let player_col = player.collider();

        for (i, coin) in coins.iter().enumerate() {
            if coin.collected {
                continue;
            }
            if player_col.intersects(&coin.collider()) {
                events.push(CollisionEvent::CoinCollect(i));
            }
        }

        events
    }

    /// Detects question-block activation events: player hitting a block from below.
    ///
    /// # Parameters (§4 Interface Contract)
    /// - `player`: Reference to the player entity for collider and velocity queries.
    /// - `blocks`: Slice of all question blocks in the current frame.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing `QuestionBlockHit(i)` for each unused block
    ///   that is hit from below by the player's head.
    /// - Returns empty Vec if blocks slice is empty or no valid activations.
    ///
    /// # Activation conditions (§6 flowchart TD + §3 key decisions)
    /// A block activates only when ALL of the following are true:
    /// 1. `block.used == false` (not already consumed)
    /// 2. Player collider overlaps block collider (AABB intersection)
    /// 3. `player.vel.y < 0.0` (strictly upward movement in Y-down coords)
    /// 4. Player head is near block bottom: `player_col.y <= block_bottom + HEAD_TOLERANCE`
    ///
    /// Conditions 3+4 together ensure only head-on hits from below trigger activation,
    /// not side-contact or stationary contact.
    pub fn question_block_check(
        player: &Player,
        blocks: &[QuestionBlock],
    ) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        let player_col = player.collider();

        for (i, block) in blocks.iter().enumerate() {
            // CheckUsed: skip already-used blocks
            if block.used {
                continue;
            }

            let block_col = block.collider();

            // CheckOverlap: player must intersect the block (AABB expanded by HEAD_TOLERANCE
            // on the bottom side to allow activation when the player's head is slightly
            // below the block surface — per §3 key decisions, head can be up to 4px below
            // the block bottom and still count as "hitting from below").
            let expanded_block = AABB {
                x: block_col.x,
                y: block_col.y,
                w: block_col.w,
                h: block_col.h + HEAD_TOLERANCE,
            };
            if !player_col.intersects(&expanded_block) {
                continue;
            }

            // CheckVelocity: must be moving upward (vel.y strictly < 0.0)
            if player.vel.y >= 0.0 {
                continue;
            }

            // CheckHeadPos: player head must be near block bottom (two-sided tolerance).
            // The head must be within ±HEAD_TOLERANCE of the block's true bottom edge
            // to prevent side-contact false positives while allowing slight penetration.
            let block_bottom = block_col.y + block_col.h;
            if (player_col.y - block_bottom).abs() > HEAD_TOLERANCE {
                continue;
            }

            events.push(CollisionEvent::QuestionBlockHit(i));
        }

        events
    }

    /// Detects power-up collection events: player collider overlaps any PowerUp.
    ///
    /// # Parameters (§4 Interface Contract)
    /// - `player`: Reference to the player entity for collider queries.
    /// - `power_ups`: Slice of all active PowerUp entities in the current frame.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing `PowerUpCollect(i)` for each PowerUp
    ///   whose AABB overlaps the player's collider.
    /// - Returns empty Vec if power_ups slice is empty or no overlaps.
    ///
    /// # Design contract (§6)
    /// - PowerUps are triggers (no direction check), same as coins.
    /// - Empty slice returns empty Vec (no panic).
    pub fn powerup_check(
        player: &Player,
        power_ups: &[PowerUp],
    ) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        let player_col = player.collider();

        for (i, power_up) in power_ups.iter().enumerate() {
            if player_col.intersects(&power_up.collider()) {
                events.push(CollisionEvent::PowerUpCollect(i));
            }
        }

        events
    }

    /// Detects fireball-enemy collision events: any alive fireball overlapping any alive enemy.
    ///
    /// # Parameters (§4 Interface Contract)
    /// - `fireballs`: Slice of all active fireballs in the current frame.
    /// - `enemies`: Slice of all enemies in the current frame.
    ///
    /// # Returns
    /// - `Vec<CollisionEvent>` containing `FireballHitEnemy(fi, ei)` for each pair
    ///   (alive fireball, alive enemy) whose AABBs overlap.
    /// - Returns empty Vec if either slice is empty or no overlapping alive pairs.
    ///
    /// # Design contract (§6)
    /// - Both fireball AND enemy must be alive for a hit to register.
    /// - Fireball does not collide with terrain (penetrates platforms).
    /// - Empty slices return empty Vec (no panic).
    pub fn fireball_enemy_check(
        fireballs: &[Fireball],
        enemies: &[Enemy],
    ) -> Vec<CollisionEvent> {
        let mut events = Vec::new();

        for (fi, fireball) in fireballs.iter().enumerate() {
            if !fireball.alive {
                continue;
            }
            let fb_col = fireball.collider();
            for (ei, enemy) in enemies.iter().enumerate() {
                if !enemy.alive {
                    continue;
                }
                if fb_col.intersects(&enemy.collider()) {
                    events.push(CollisionEvent::FireballHitEnemy(fi, ei));
                }
            }
        }

        events
    }
}
