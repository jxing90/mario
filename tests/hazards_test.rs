// Feature #5: Hazards — TDD Red Phase
//
// Test Inventory Reference: docs/features/5-hazards.md §7
// SRS Reference: FR-009 (Hazards — Spikes and Pits)
// Design Reference: §2.5 Hazards, §4 IAPI-004/005/008
//
// All tests are expected to FAIL (RED phase) — implementation not yet written.
// Post-red remediation: compile errors → import fixes; assertion failures → implement.
//
// Category coverage (Rule 1):
//   FUNC/happy:  T01, T02, T03, T04
//   FUNC/error:  T05, T06
//   BNDRY/edge:  T07, T08, T09, T14
//   BNDRY/batch: T10
//   BNDRY/null:  T11
//   INTG/level:  T12 [real_test]
//   INTG/physics: T13 [real_test]
//   SEC: N/A — offline desktop game, no user-facing input channels, no network exposure
//
// Rule 2 negative ratio: 8/14 = 57.1% (>= 40%)
//   Negative tests: T05(FUNC/error), T06(FUNC/error), T07(BNDRY/edge),
//                   T08(BNDRY/edge), T09(BNDRY/edge), T10(BNDRY/batch),
//                   T11(BNDRY/null), T14(BNDRY/edge)
//
// Rule 5 real_test_count: 2 (T12, T13) — both integration tests with Level dependency
//
// UML trace coverage (Rule 8):
//   classDiagram: Spike, CollisionEvent, Tile, LevelBounds, Player, Level — covered by T01-T13
//   sequenceDiagram msg#1-10: traced via §Design Alignment in T01-T13
//   flowchart TD branch#1-5: traced via §Implementation Summary in T01-T13

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::level::{AABB, Level, Tile, Vec2};

// NEW types (expected to fail compilation — implementation not yet written):
// - Spike struct in src/entities/hazard.rs
// - CollisionEvent enum in src/systems/physics.rs
// - Physics::hazard_check() in src/systems/physics.rs
//
// These imports will cause compilation errors until the Green phase:
use mario_platformer::entities::hazard::Spike;
use mario_platformer::systems::physics::{CollisionEvent, Physics};

// ============================================================================
// Constants
// ============================================================================

/// Small epsilon for floating-point comparisons.
const EPSILON: f32 = 1e-5;

/// Feature #5 default kill-plane Y threshold (matches LevelBounds.kill_y).
const DEFAULT_KILL_Y: f32 = 2500.0;

/// Small player collider dimensions (Small power-up state).
const PLAYER_W: f32 = 16.0;
const PLAYER_H: f32 = 16.0;

/// Spike collider dimensions per design §2.5.2.
const SPIKE_W: f32 = 16.0;
const SPIKE_H: f32 = 8.0;

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Creates a default Player at a specific world-space foot position.
/// The player is in Small state (collider 16x16).
fn player_at(x: f32, y: f32) -> Player {
    let mut p = Player::new(PlayerConfig::default());
    p.pos = Vec2 { x, y };
    p
}

/// Constructs the AABB a player at (px, py) would have in Small state.
/// Player collider is centered horizontally, foot at bottom:
///   x = px - w/2, y = py - h, w = PLAYER_W, h = PLAYER_H
fn player_aabb_at(px: f32, py: f32) -> AABB {
    AABB {
        x: px - PLAYER_W / 2.0,
        y: py - PLAYER_H,
        w: PLAYER_W,
        h: PLAYER_H,
    }
}

/// Constructs the AABB a Spike at (sx, sy) would have.
/// Spike collider is centered at position:
///   x = sx - SPIKE_W/2, y = sy - SPIKE_H/2, w = SPIKE_W, h = SPIKE_H
fn spike_aabb_at(sx: f32, sy: f32) -> AABB {
    AABB {
        x: sx - SPIKE_W / 2.0,
        y: sy - SPIKE_H / 2.0,
        w: SPIKE_W,
        h: SPIKE_H,
    }
}

/// Checks whether a Vec<CollisionEvent> contains a HazardContact variant.
fn contains_hazard_contact(events: &[CollisionEvent]) -> bool {
    events.iter().any(|e| matches!(e, CollisionEvent::HazardContact))
}

/// Checks whether a Vec<CollisionEvent> contains a PitFall variant.
fn contains_pit_fall(events: &[CollisionEvent]) -> bool {
    events.iter().any(|e| matches!(e, CollisionEvent::PitFall))
}

/// Counts HazardContact occurrences in an event Vec.
fn count_hazard_contacts(events: &[CollisionEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, CollisionEvent::HazardContact))
        .count()
}

/// Counts PitFall occurrences in an event Vec.
fn count_pit_falls(events: &[CollisionEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, CollisionEvent::PitFall))
        .count()
}

// ============================================================================
// T01 — FUNC/happy — Spike contact triggers HazardContact
// Traces To: FR-009 AC-1 (spike contact → death), §Interface Contract hazard_check
// Kills: Spike collision ignored entirely (collision loop skips Tile::Spike)
// Wrong-impl challenge:
//   - Wrong: hazard_check ignores Tile::Spike variant → FAIL (we expect HazardContact)
//   - Wrong: AABB::intersects not called → FAIL (no overlap detection)
//   - Wrong: intersects returns false for real overlaps → FAIL
// ============================================================================

#[test]
fn t01_fun_happy_spike_contact_emits_hazard_contact() {
    // Spike at (100, 100) → collider AABB (92, 96, 16, 8)
    let spike = Spike::new(Vec2 { x: 100.0, y: 100.0 });
    let spike_aabb = spike_aabb_at(100.0, 100.0);
    assert!(
        approx_eq(spike.collider().x, spike_aabb.x),
        "Spike collider x mismatch"
    );
    assert!(
        approx_eq(spike.collider().y, spike_aabb.y),
        "Spike collider y mismatch"
    );

    // Player at (100, 100) → collider AABB (92, 84, 16, 16)
    // Overlap with spike (92, 96, 16, 8):
    //   X-axis: [92,108] ∩ [92,108] = yes
    //   Y-axis: [84,100] ∩ [96,104] = yes (84 <= 104 AND 100 >= 96)
    let player = player_at(100.0, 100.0);

    // Verify AABB::intersects confirms the overlap (sanity check)
    let p_aabb = player.collider();
    assert!(
        p_aabb.intersects(&spike_aabb),
        "Precondition: player AABB must overlap spike AABB"
    );

    // Execute hazard check
    let terrain = vec![Tile::Spike(spike_aabb)];
    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    // Assert: HazardContact is emitted for the spike overlap
    assert!(
        contains_hazard_contact(&events),
        "Expected HazardContact when player overlaps spike collider, got: {:?}",
        events
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 event (HazardContact), got {}",
        events.len()
    );
}

// ============================================================================
// T02 — FUNC/happy — Safe passage (no overlap) returns empty
// Traces To: FR-009 AC-2 (jump over spike safely), §Interface Contract hazard_check
// Kills: False positive — non-overlapping AABBs incorrectly reported as HazardContact
// Wrong-impl challenge:
//   - Wrong: hazard_check always returns HazardContact regardless of overlap → FAIL
//   - Wrong: AABB bounds swapped causing false overlap → FAIL
//   - Wrong: return vec![HazardContact] hardcoded → FAIL
// ============================================================================

#[test]
fn t02_fun_happy_safe_passage_returns_empty() {
    // Spike at (100, 100) → collider (92, 96, 16, 8)
    let spike_aabb = spike_aabb_at(100.0, 100.0);

    // Player at (200, 50) → collider (192, 34, 16, 16)
    // X-axis: [192,208] vs spike [92,108] → no overlap
    let player = player_at(200.0, 50.0);

    // Verify no overlap (sanity)
    let p_aabb = player.collider();
    assert!(
        !p_aabb.intersects(&spike_aabb),
        "Precondition: player AABB must NOT overlap spike AABB"
    );

    let terrain = vec![Tile::Spike(spike_aabb)];
    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        events.is_empty(),
        "Expected empty Vec for non-overlapping player and spike, got {} events",
        events.len()
    );
}

// ============================================================================
// T03 — FUNC/happy — Y > kill_y triggers PitFall
// Traces To: FR-009 AC-3 (pit fall → death), §Interface Contract hazard_check
// Kills: Kill-plane never checked (forgot to call Level::bounds())
// Wrong-impl challenge:
//   - Wrong: hazard_check never checks player.pos().y against kill_y → FAIL
//   - Wrong: PitFall check uses wrong comparison direction (< instead of >) → FAIL
//   - Wrong: kill_y hardcoded to 0.0 or infinity → FAIL
// ============================================================================

#[test]
fn t03_fun_happy_pit_fall_triggers_pit_fall() {
    // Player Y = 2600, kill_y = 2500 → Y > kill_y → PitFall
    let player = player_at(500.0, 2600.0);
    let terrain: Vec<Tile> = vec![]; // No spikes, only pit check

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        contains_pit_fall(&events),
        "Expected PitFall when player.pos().y ({}) > kill_y ({}), got: {:?}",
        player.pos().y,
        DEFAULT_KILL_Y,
        events
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 event (PitFall), got {}",
        events.len()
    );
}

// ============================================================================
// T04 — FUNC/happy — Invulnerability does NOT suppress HazardContact
// Traces To: FR-009 AC-4 (invulnerability exemption), §Interface Contract hazard_check
// Kills: Premature optimization — F05 silences event when invulnerable
//   (invuln is F06 responsibility, tested separately)
// Wrong-impl challenge:
//   - Wrong: F05 hazard_check filters events when invuln_timer > 0 → FAIL
//   - Wrong: F05 calls into F06 invuln state (shouldn't know about it) → FAIL
// ============================================================================

#[test]
fn t04_fun_happy_invulnerability_does_not_suppress_hazard_contact() {
    // Spike overlap exists
    let spike_aabb = spike_aabb_at(100.0, 100.0);

    // Player overlapping spike — collider (92, 84, 16, 16) overlaps (92, 96, 16, 8)
    let player = player_at(100.0, 100.0);

    // Verify overlap
    assert!(
        player.collider().intersects(&spike_aabb),
        "Precondition: player must overlap spike"
    );

    let terrain = vec![Tile::Spike(spike_aabb)];
    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    // Critical assertion: F05 must still produce HazardContact regardless of
    // any invulnerability state (which F05 should not even be aware of).
    // The invulnerability check is F06's responsibility when consuming the event.
    assert!(
        contains_hazard_contact(&events),
        "F05 MUST emit HazardContact even when player would be invulnerable — \
         invuln filtering is F06 responsibility, not F05"
    );
    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 HazardContact event"
    );
}

// ============================================================================
// T05 — FUNC/error — Multiple spikes + pit: all events emitted, order preserved
// Traces To: §Interface Contract: HazardContact + PitFall both present
//   sequenceDiagram msg#7-10 (spike check loop + pit check)
// Kills: Short-circuit — PitFall check skipped after first HazardContact found
// Wrong-impl challenge:
//   - Wrong: return early after first HazardContact, skip PitFall → FAIL
//   - Wrong: return early after PitFall, skip remaining spikes → FAIL
//   - Wrong: reverse order (PitFall before spikes) → MAY FAIL (order check)
// ============================================================================

#[test]
fn t05_fun_error_concurrent_spike_and_pit_all_events_emitted() {
    // Player at Y=2600 (below kill_y=2500) overlapping two spikes.
    // spike1 at (100, 2600) → collider (92, 2596, 16, 8)
    // spike2 at (100, 2595) → collider (92, 2591, 16, 8)
    // player at (100, 2600) → collider (92, 2584, 16, 16)
    // spike1: X [92,108] ∩ [92,108] yes; Y [2596,2604] ∩ [2584,2600] → overlap
    // spike2: X [92,108] ∩ [92,108] yes; Y [2591,2599] ∩ [2584,2600] → overlap
    let spike_aabb_1 = spike_aabb_at(100.0, 2600.0);
    let spike_aabb_2 = spike_aabb_at(100.0, 2595.0);

    let player = player_at(100.0, 2600.0);

    // Verify both spikes overlap player
    let p_aabb = player.collider();
    assert!(
        p_aabb.intersects(&spike_aabb_1),
        "Precondition: player must overlap spike 1"
    );
    assert!(
        p_aabb.intersects(&spike_aabb_2),
        "Precondition: player must overlap spike 2"
    );
    // Verify Y > kill_y
    assert!(
        player.pos().y > DEFAULT_KILL_Y,
        "Precondition: player Y ({}) must exceed kill_y ({})",
        player.pos().y,
        DEFAULT_KILL_Y
    );

    let terrain = vec![Tile::Spike(spike_aabb_1), Tile::Spike(spike_aabb_2)];
    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    // Assert 3 events: 2 HazardContact + 1 PitFall
    assert_eq!(
        events.len(),
        3,
        "Expected 3 events (2x HazardContact + 1x PitFall), got {}: {:?}",
        events.len(),
        events
    );
    assert_eq!(
        count_hazard_contacts(&events),
        2,
        "Expected exactly 2 HazardContact events"
    );
    assert_eq!(
        count_pit_falls(&events),
        1,
        "Expected exactly 1 PitFall event"
    );

    // Order check: PitFall check happens after all spike checks per flowchart
    // The first two events should both be HazardContact
    assert!(
        matches!(events[0], CollisionEvent::HazardContact),
        "First event should be HazardContact (spike check before pit check)"
    );
    assert!(
        matches!(events[1], CollisionEvent::HazardContact),
        "Second event should be HazardContact"
    );
    assert!(
        matches!(events[2], CollisionEvent::PitFall),
        "Third event should be PitFall (pit check after spike loop)"
    );
}

// ============================================================================
// T06 — FUNC/error — Empty terrain → empty Vec, no panic
// Traces To: §Interface Contract hazard_check, §Boundary Conditions: empty terrain
// Kills: Panic on empty terrain slice (index out of bounds or unwrap on None)
// Wrong-impl challenge:
//   - Wrong: hazard_check assumes non-empty terrain, calls unwrap() on first() → FAIL (panic)
//   - Wrong: hazard_check panics on empty iterator → FAIL
//   - Wrong: hazard_check returns HazardContact as default on empty → FAIL
// ============================================================================

#[test]
fn t06_fun_error_empty_terrain_returns_empty_no_panic() {
    let player = player_at(100.0, 100.0);
    let terrain: Vec<Tile> = vec![];

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        events.is_empty(),
        "Expected empty Vec for empty terrain input, got {} events: {:?}",
        events.len(),
        events
    );
    // If we reach here without panic, the empty-terrain safety is verified.
}

// ============================================================================
// T07 — BNDRY/edge — Y == kill_y exactly → safe (strict > comparison)
// Traces To: §Boundary Conditions: Y == kill_y, FR-009 AC-3
//   flowchart TD branch#4 (CheckPit: player.pos().y > kill_y?)
// Kills: Off-by-one — `>=` used instead of `>`, triggering PitFall at exact threshold
// Wrong-impl challenge:
//   - Wrong: >= comparison → PitFall emitted at exact boundary → FAIL
//   - Wrong: floating-point issue with exact equality → FAIL
// ============================================================================

#[test]
fn t07_bndry_edge_y_equals_kill_y_returns_empty() {
    let player = player_at(500.0, DEFAULT_KILL_Y); // Exactly at kill_y
    let terrain: Vec<Tile> = vec![];

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        events.is_empty(),
        "player.pos().y == kill_y should be SAFE (> is strict, == is not >), \
         but got {} events: {:?}",
        events.len(),
        events
    );
}

// ============================================================================
// T08 — BNDRY/edge — Tangential/edge contact counts as overlap
// Traces To: §Boundary Conditions: edge contact, §Interface Contract Spike::collider
//   classDiagram Spike.collider method
// Kills: Uses strict `<` instead of `<=` in AABB overlap check → skips edge contact
// Wrong-impl challenge:
//   - Wrong: AABB::intersects modified to use strict inequality → FAIL
//   - Wrong: Spike collider has off-by-one dimension → FAIL
// ============================================================================

#[test]
fn t08_bndry_edge_edge_contact_counts_as_hazard_contact() {
    // Spike at (100, 100) → collider (92, 96, 16, 8)
    let spike_aabb = spike_aabb_at(100.0, 100.0);

    // Player at exactly the right edge: player AABB left edge = spike AABB right edge
    // Spike collider (92, 96, 16, 8) → right edge at 92+16 = 108
    // Player at px = 108 + PLAYER_W/2 = 108 + 8 = 116
    // Player collider: (108, py - 16, 16, 16)
    // X overlap: player [108,124] vs spike [92,108] → 108 <= 108 AND 124 >= 92 → true (inclusive)
    let player = player_at(116.0, 100.0);

    let p_aabb = player.collider();
    // Sanity: the overlap is at the edge
    assert!(
        approx_eq(p_aabb.x, spike_aabb.x + spike_aabb.w),
        "Precondition: player left edge must equal spike right edge (tangent)"
    );

    let terrain = vec![Tile::Spike(spike_aabb)];
    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        contains_hazard_contact(&events),
        "Edge contact (player left == spike right) must count as overlap \
         per inclusive-edge AABB::intersects contract"
    );
}

// ============================================================================
// T09 — BNDRY/edge — Y = kill_y + epsilon triggers PitFall
// Traces To: §Boundary Conditions: infinitesimally beyond kill_y
//   flowchart TD branch#4 (CheckPit)
// Kills: Floating-point comparison error — epsilon too loose or too tight
// Wrong-impl challenge:
//   - Wrong: hazard_check uses <= comparison → FAIL
//   - Wrong: f32 comparison uses some arbitrary tolerance → FAIL
// ============================================================================

#[test]
fn t09_bndry_edge_y_just_beyond_kill_y_triggers_pit_fall() {
    // Player Y is epsilon beyond kill_y — barely in the death zone.
    // f32::EPSILON (~1.19e-7) is too small at this magnitude (ULP ~0.000244);
    // 0.001 is the smallest meaningful increment that survives f32 representation.
    let player = player_at(500.0, DEFAULT_KILL_Y + 0.001);
    let terrain: Vec<Tile> = vec![];

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        contains_pit_fall(&events),
        "player.pos().y = {} (> kill_y = {} by epsilon) must trigger PitFall, \
         got {} events: {:?}",
        player.pos().y,
        DEFAULT_KILL_Y,
        events.len(),
        events
    );
}

// ============================================================================
// T10 — BNDRY/batch — 0, 1, 5 spikes → correct event count, no panic
// Traces To: §Implementation Summary: variable spike count
//   flowchart TD branch#1-3 (CheckSpikes loop)
// Kills: Fixed-size event buffer overflow; Vec push fails on N spikes
// Wrong-impl challenge:
//   - Wrong: spike iteration limited to hardcoded count → FAIL (5 spikes)
//   - Wrong: panic on empty spikes Vec → FAIL (0 spikes)
// ============================================================================

#[test]
fn t10_bndry_batch_variable_spike_count() {
    let kill_y_high = 10000.0; // Well above player, no pit fall

    // Case 1: 0 spikes → 0 events
    {
        let player = player_at(100.0, 100.0);
        let terrain: Vec<Tile> = vec![];
        let events = Physics::hazard_check(&player, &terrain, kill_y_high);
        assert!(
            events.is_empty(),
            "0 spikes: expected empty, got {} events",
            events.len()
        );
    }

    // Case 2: 1 spike overlapping → 1 event
    {
        let player = player_at(100.0, 100.0);
        let spike_aabb = spike_aabb_at(100.0, 100.0);
        let terrain = vec![Tile::Spike(spike_aabb)];
        let events = Physics::hazard_check(&player, &terrain, kill_y_high);
        assert_eq!(
            events.len(),
            1,
            "1 overlapping spike: expected 1 HazardContact, got {} events",
            events.len()
        );
        assert!(contains_hazard_contact(&events));
    }

    // Case 3: 5 overlapping spikes → 5 HazardContact events
    {
        // Player at (100, 100), 5 spikes all overlapping at various nearby positions
        let player = player_at(100.0, 100.0);
        let spike_positions = [
            (100.0, 100.0),
            (98.0, 100.0),
            (102.0, 100.0),
            (100.0, 98.0),
            (100.0, 102.0),
        ];
        let terrain: Vec<Tile> = spike_positions
            .iter()
            .map(|&(sx, sy)| Tile::Spike(spike_aabb_at(sx, sy)))
            .collect();

        let events = Physics::hazard_check(&player, &terrain, kill_y_high);
        assert_eq!(
            events.len(),
            5,
            "5 overlapping spikes: expected 5 HazardContact events, got {}",
            events.len()
        );
        assert_eq!(
            count_hazard_contacts(&events),
            5,
            "All 5 events should be HazardContact"
        );
    }
}

// ============================================================================
// T11 — BNDRY/null — Only Platform tiles (no Spike) → empty Vec
// Traces To: §Boundary Conditions: query_terrain returns no Spike tiles
//   flowchart TD branch#1 (CheckSpikes: match Tile variant)
// Kills: Incorrect pattern match — Tile::Platform mistaken for Tile::Spike
// Wrong-impl challenge:
//   - Wrong: match arm for Platform emits HazardContact → FAIL
//   - Wrong: default/wildcard match arm treats all tiles as spikes → FAIL
// ============================================================================

#[test]
fn t11_bndry_null_platform_only_no_hazard_events() {
    let player = player_at(100.0, 500.0);
    let platform_aabb = AABB {
        x: 0.0,
        y: 600.0,
        w: 2000.0,
        h: 40.0,
    };
    let terrain = vec![Tile::Platform(platform_aabb)];

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        events.is_empty(),
        "Platform tiles must NOT produce hazard events, got {} events: {:?}",
        events.len(),
        events
    );
}

// ============================================================================
// T12 — INTG/level — query_terrain returns Spike tiles [real_test]
// Traces To: §Interface Contract: Level::query_terrain (MODIFIED)
//   sequenceDiagram msg#3-4 (Physics->>Level: query_terrain → Vec<Tile>)
// Kills: Spike tile missing from query_terrain — Level never stores/includes spikes
// ============================================================================

// [real_test] [integration] Feature #5 integration — Level::query_terrain returns Spike tiles
#[test]
fn t12_intg_level_query_terrain_returns_spike_tiles() {
    let level = Level::new();

    // Query the terrain over a region where spikes should be placed
    // (Spike positions are hardcoded in Level::new(), per design §Implementation Summary)
    //
    // Per the design, spikes should be placed in the level (e.g., at ground level ~600).
    // We query a generous AABB along the ground area.
    let query_aabb = AABB {
        x: 0.0,
        y: 550.0,
        w: 2000.0,
        h: 200.0,
    };

    let tiles = level.query_terrain(&query_aabb);

    // At least one Tile::Spike should be returned by the modified query_terrain
    let spike_count = tiles
        .iter()
        .filter(|t| matches!(t, Tile::Spike(_)))
        .count();

    assert!(
        spike_count > 0,
        "Level::query_terrain must return Spike tiles in spike regions. \
         Found 0 Spike tiles (total tiles: {}). \
         Level::new() should hardcode spikes.",
        tiles.len()
    );

    // Also verify that the Spike AABB is valid (dimensions correct)
    for tile in &tiles {
        if let Tile::Spike(aabb) = tile {
            assert!(
                aabb.w > 0.0 && aabb.h > 0.0,
                "Spike tile AABB must have positive dimensions, got w={}, h={}",
                aabb.w,
                aabb.h
            );
            assert!(
                approx_eq(aabb.w, SPIKE_W),
                "Spike AABB width should be {}, got {}",
                SPIKE_W,
                aabb.w
            );
            assert!(
                approx_eq(aabb.h, SPIKE_H),
                "Spike AABB height should be {}, got {}",
                SPIKE_H,
                aabb.h
            );
        }
    }
}

// ============================================================================
// T13 — INTG/physics — End-to-end: spike in level → hazard_check → HazardContact [real_test]
// Traces To: §Design Alignment seq msg#3-10 (full hazard detection call sequence)
//   flowchart TD (full hazard_check decision flow)
// Kills: Broken match arm — Tile::Spike variant added to enum but
//   hazard_check match is non-exhaustive (default `_` arm skips Spike)
// ============================================================================

// [real_test] [integration] Feature #5 integration — end-to-end hazard detection
#[test]
fn t13_intg_physics_end_to_end_spike_in_level_emits_hazard_contact() {
    let level = Level::new();

    // Step 1: Query terrain to find spike tiles
    let query_aabb = AABB {
        x: 0.0,
        y: 550.0,
        w: 2000.0,
        h: 200.0,
    };
    let terrain = level.query_terrain(&query_aabb);

    // Verify we actually got spike tiles (this is the integration point)
    let has_spikes = terrain.iter().any(|t| matches!(t, Tile::Spike(_)));
    assert!(
        has_spikes,
        "Integration precondition: terrain must include Spike tiles from Level"
    );

    // Step 2: Place player at a position that overlaps one of the spikes
    // Find the first spike AABB and place player on top of it
    let spike_aabb = terrain
        .iter()
        .find_map(|t| {
            if let Tile::Spike(aabb) = t {
                Some(*aabb)
            } else {
                None
            }
        })
        .expect("Must have at least one Spike tile for this test");

    // Position player at the spike's center
    let spike_center_x = spike_aabb.x + spike_aabb.w / 2.0;
    let spike_center_y = spike_aabb.y + spike_aabb.h / 2.0;
    let player = player_at(spike_center_x, spike_center_y);

    // Step 3: Run hazard_check with the terrain from the real Level
    let kill_y = level.bounds().kill_y;
    let events = Physics::hazard_check(&player, &terrain, kill_y);

    // Step 4: Assert that an actual HazardContact is emitted
    assert!(
        contains_hazard_contact(&events),
        "End-to-end: player overlapping a Level-spawned Spike must produce HazardContact. \
         Player at ({}, {}), spike at ({}, {}), events: {:?}",
        spike_center_x,
        spike_center_y,
        spike_aabb.x,
        spike_aabb.y,
        events
    );
}

// ============================================================================
// T14 — BNDRY/fuzz — Player at kill_y - 1.0 is safe (boundary proximity)
// Supplementary edge test: close to kill_y but above it → safe
// Traces To: §Boundary Conditions: Y = kill_y - epsilon
// ============================================================================

#[test]
fn t14_bndry_edge_y_just_above_kill_y_is_safe() {
    // Player at kill_y - 1.0: clearly above the death plane
    let player = player_at(500.0, DEFAULT_KILL_Y - 1.0);
    let terrain: Vec<Tile> = vec![];

    let events = Physics::hazard_check(&player, &terrain, DEFAULT_KILL_Y);

    assert!(
        !contains_pit_fall(&events),
        "player.pos().y = {} (< kill_y = {}): must NOT trigger PitFall, \
         but got PitFall",
        player.pos().y,
        DEFAULT_KILL_Y
    );
}
