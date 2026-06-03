// System ST: PlayingState::update() pipeline integration tests
//
// These tests exercise the FULL update() pipeline — not isolated functions.
// They verify that side effects (coin collection, block activation, brick
// breakage, collision) actually happen when called through PlayingState::update().
//
// Gap this fills: previous tests called Physics::coin_check() etc. in isolation
// but never verified the order-of-operations within update().

use mario_platformer::entities::player::{Player, PlayerConfig, PlayerState};
use mario_platformer::level::Vec2;
use mario_platformer::states::LifeState;
use mario_platformer::states::PlayingState;
use mario_platformer::states::playing::Brick;

// ============================================================================
// Coin collection through update() pipeline
// ============================================================================

/// Player overlapping coin → update() → coin.collected == true, player.coins += 1
#[test]
fn update_pipeline_coin_collection_overlap_triggers_increment() {
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 200.0, y: 600.0 }; // feet on ground
    player.on_ground = true;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Place a coin exactly where the player is standing
    state.coins[0].pos = state.player.pos();
    state.coins[0].collected = false;
    let initial_coins = state.player.coins;

    state.update(1.0 / 60.0);

    assert!(state.coins[0].collected, "Coin should be collected after overlapping player");
    assert_eq!(state.player.coins, initial_coins + 1, "Player coin counter should increment");
}

/// Player NOT overlapping coin → update() → coin stays uncollected
#[test]
fn update_pipeline_coin_far_away_stays_uncollected() {
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Place coin far from player
    state.coins[0].pos = Vec2 { x: 1500.0, y: 500.0 };
    state.coins[0].collected = false;

    state.update(1.0 / 60.0);

    assert!(!state.coins[0].collected, "Far-away coin should not be collected");
}

/// Coin already collected → update() → not collected again, count unchanged
#[test]
fn update_pipeline_already_collected_coin_not_double_counted() {
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 200.0, y: 600.0 };
    player.on_ground = true;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.coins[0].pos = state.player.pos();
    state.coins[0].collected = true; // already collected
    state.player.coins = 5;

    state.update(1.0 / 60.0);

    assert_eq!(state.player.coins, 5, "Already-collected coin should not increment again");
}

// ============================================================================
// Question block hit through update() pipeline
// ============================================================================

/// Player hits question block from below → update() → block.used == true
#[test]
fn update_pipeline_question_block_hit_from_below_activates() {
    let mut player = Player::new(PlayerConfig::default());
    // Place player below a question block, moving upward
    player.pos = Vec2 { x: 450.0, y: 440.0 }; // just below block at (450, 400)
    player.vel.y = -200.0; // moving upward
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Set block to known state directly under player path
    state.question_blocks[0].pos = Vec2 { x: 450.0, y: 412.0 }; // 32px above player
    state.question_blocks[0].used = false;

    // Run one frame — player moves up, should hit block
    state.update(1.0 / 60.0);

    assert!(
        state.question_blocks[0].used,
        "Question block should be activated when player hits from below via update()"
    );
    // Player should be bounced down
    assert!(
        state.player.vel.y > 0.0,
        "Player velocity should be bounced downward after hitting block. vel.y={}",
        state.player.vel.y
    );
}

/// Player touches block from above → update() → block NOT activated (stands on it)
#[test]
fn update_pipeline_question_block_from_above_does_not_activate() {
    let mut player = Player::new(PlayerConfig::default());
    // Player standing ON TOP of a question block
    player.pos = Vec2 { x: 450.0, y: 400.0 }; // feet right at block top
    player.vel.y = 0.0;
    player.on_ground = true;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.question_blocks[0].pos = Vec2 { x: 450.0, y: 400.0 }; // block top at 400
    state.question_blocks[0].used = false;

    state.update(1.0 / 60.0);

    assert!(
        !state.question_blocks[0].used,
        "Block should NOT activate when player stands on top"
    );
}

/// Question block already used → hit again → stays used, no effect
#[test]
fn update_pipeline_already_used_block_no_reactivation() {
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 450.0, y: 440.0 };
    player.vel.y = -300.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.question_blocks[0].pos = Vec2 { x: 450.0, y: 412.0 };
    state.question_blocks[0].used = true; // already used
    let coins_before = state.player.coins;

    state.update(1.0 / 60.0);

    assert!(state.question_blocks[0].used, "Already-used block stays used");
    assert_eq!(state.player.coins, coins_before, "No coin reward from used block");
}

// ============================================================================
// Brick breakage through update() pipeline
// ============================================================================

/// Small Mario hits brick from below → update() → brick NOT broken, player bounces
#[test]
fn update_pipeline_small_mario_hits_brick_does_not_break() {
    let mut player = Player::new(PlayerConfig::default());
    assert!(matches!(player.state, PlayerState::Small));
    player.pos = Vec2 { x: 300.0, y: 475.0 }; // below brick
    player.vel.y = -300.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Add a test brick just above the player
    state.bricks.push(Brick { pos: Vec2 { x: 300.0, y: 445.0 }, broken: false });
    let brick_idx = state.bricks.len() - 1;

    state.update(1.0 / 60.0);

    assert!(!state.bricks[brick_idx].broken,
        "Small Mario should NOT break bricks, got broken=true");
    // Small Mario hits a solid ceiling — terrain collision stops upward motion
    assert!(state.player.vel.y >= 0.0,
        "Small Mario should be stopped by brick ceiling. vel.y={}", state.player.vel.y);
}

/// Super Mario hits brick from below → update() → brick IS broken
#[test]
fn update_pipeline_super_mario_breaks_brick() {
    let mut player = Player::new(PlayerConfig::default());
    player.apply_powerup(PlayerState::Super);
    assert!(matches!(player.state, PlayerState::Super));
    player.pos = Vec2 { x: 300.0, y: 475.0 };
    player.vel.y = -300.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.bricks.push(Brick { pos: Vec2 { x: 300.0, y: 445.0 }, broken: false });
    let brick_idx = state.bricks.len() - 1;

    state.update(1.0 / 60.0);

    assert!(state.bricks[brick_idx].broken,
        "Super Mario should break bricks from below, got broken=false");
}

/// Fire Mario hits brick from below → update() → brick IS broken
#[test]
fn update_pipeline_fire_mario_breaks_brick() {
    let mut player = Player::new(PlayerConfig::default());
    player.apply_powerup(PlayerState::Fire);
    assert!(matches!(player.state, PlayerState::Fire));
    player.pos = Vec2 { x: 300.0, y: 475.0 };
    player.vel.y = -300.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.bricks.push(Brick { pos: Vec2 { x: 300.0, y: 445.0 }, broken: false });
    let brick_idx = state.bricks.len() - 1;

    state.update(1.0 / 60.0);

    assert!(state.bricks[brick_idx].broken,
        "Fire Mario should break bricks from below, got broken=false");
}

/// Already-broken brick → hit again → stays broken, no double-break
#[test]
fn update_pipeline_already_broken_brick_ignored() {
    let mut player = Player::new(PlayerConfig::default());
    player.apply_powerup(PlayerState::Super);
    player.pos = Vec2 { x: 300.0, y: 475.0 };
    player.vel.y = -300.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    state.bricks.push(Brick { pos: Vec2 { x: 300.0, y: 445.0 }, broken: true });
    let brick_idx = state.bricks.len() - 1;

    state.update(1.0 / 60.0);

    // Should still be broken — no crash, no re-break
    assert!(state.bricks[brick_idx].broken, "Broken brick stays broken");
}

// ============================================================================
// Terminal velocity / platform tunneling prevention
// ============================================================================

/// Player falling at high speed → must not tunnel through a platform
#[test]
fn update_pipeline_terminal_velocity_prevents_platform_tunneling() {
    let mut player = Player::new(PlayerConfig::default());
    // Place player VERY high, give them extreme downward velocity
    player.pos = Vec2 { x: 400.0, y: 100.0 };
    player.vel.y = 590.0; // near terminal velocity (600)
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Platform at y=450 — player should land ON it, not pass through
    // The level default platforms include one near here
    let initial_y = 100.0_f32;

    // Run enough frames to reach the platform area
    for _ in 0..120 {
        state.update(1.0 / 60.0);
    }

    // Player should have landed on SOMETHING — not fallen through world
    let bounds = state.level.bounds();
    assert!(
        state.player.pos.y < bounds.kill_y,
        "Player should not fall through to kill plane. Y={:.1}, kill_y={:.1}",
        state.player.pos.y, bounds.kill_y
    );
    // With terminal velocity, the player should NOT have gone past
    // the ground at 600 since there are platforms in the way
    assert!(
        state.player.pos.y < 1200.0,
        "Player should have landed on a platform. Y={:.1} — falling through?",
        state.player.pos.y
    );
}

/// Verify max_fall_speed cap is applied in gravity
#[test]
fn update_pipeline_fall_speed_capped_at_max() {
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 500.0, y: 100.0 };
    player.vel.y = 0.0;
    player.on_ground = false;

    // Let player fall for many frames — velocity should cap at max_fall_speed
    for _ in 0..300 {
        let input = mario_platformer::input::InputState::default();
        let terrain = vec![];
        player.update(1.0 / 60.0, &input, &terrain);
    }

    assert!(
        player.vel.y <= 600.0 + 1.0, // max_fall_speed + small epsilon
        "Fall speed should be capped at 600. vel.y={}", player.vel.y
    );
}

// ============================================================================
// Order-of-operations: hit detection BEFORE terrain collision pushback
// ============================================================================

/// Coin hit detection must work even after player.update() resolves terrain.
/// This is the regression test for the order-of-operations bug.
#[test]
fn update_pipeline_coin_hit_before_terrain_pushback() {
    let mut player = Player::new(PlayerConfig::default());
    // Player positioned so their collider overlaps a coin AND a platform
    // The collision resolution will push the player, but hit detection
    // should have already run and collected the coin.
    player.pos = Vec2 { x: 200.0, y: 590.0 }; // very close to ground
    player.vel.y = 5.0; // falling slightly
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Place coin at player's position
    state.coins[0].pos = state.player.pos();
    state.coins[0].collected = false;

    state.update(1.0 / 60.0);

    // Coin should be collected even though terrain collision pushed player
    assert!(
        state.coins[0].collected,
        "Coin should be collected before terrain collision pushes player away"
    );
}

/// Block hit detection must work even when block AABB is also in terrain.
/// Regression test for: "terrain collision erased the overlap before hit check"
#[test]
fn update_pipeline_block_hit_before_terrain_pushback() {
    let mut player = Player::new(PlayerConfig::default());
    // Player below a block, moving up — the block AABB is in the terrain list
    // Terrain collision would push player down, but hit detection must fire first.
    player.pos = Vec2 { x: 450.0, y: 445.0 }; // close to block
    player.vel.y = -250.0;
    player.on_ground = false;
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    // Block right above the player
    state.question_blocks[0].pos = Vec2 { x: 450.0, y: 415.0 };
    state.question_blocks[0].used = false;

    state.update(1.0 / 60.0);

    // Must be activated despite terrain collision pushing player away
    assert!(
        state.question_blocks[0].used,
        "Block hit detection must fire before terrain collision pushback"
    );
}
