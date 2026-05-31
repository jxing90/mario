// Feature #2: Level & Background — TDD Red Phase
//
// [no integration test] — pure data structures with in-memory queries, no external I/O.
// Parallax rendering integration tested via Playing state integration (INT-004).
// See long-task-guide.md Real Test Convention for pure-function exemption.
//
// Test Inventory Reference: docs/features/2-level-background.md §7
// All tests are expected to FAIL (RED phase) — implementation not yet written.
//
// Category coverage (Rule 1):
//   FUNC/happy:  T01, T02, T03, T04, T05, T06, T07, T08, T16
//   FUNC/error:  T09
//   BNDRY/edge:  T10, T11, T12, T13, T14, T15, T17
//   SEC: N/A — pure data module, no user-facing input or network exposure
//   INTG: N/A — pure function, no external I/O (per design §7, Test Inventory INTG note)

use mario_platformer::level::{AABB, Level, LevelBounds, Platform, Tile, Vec2};
use mario_platformer::parallax::ParallaxLayer;

// ============================================================================
// Constants
// ============================================================================

/// Allowed tolerance for f32 comparisons.
const EPSILON: f32 = 1e-5;

// ============================================================================
// Helpers
// ============================================================================

/// Returns true if two f32 values are within EPSILON of each other.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Returns the AABB of the first platform in the level, panicking if none exist.
fn first_platform_aabb(level: &Level) -> AABB {
    assert!(
        !level.platforms().is_empty(),
        "Level must have at least one platform"
    );
    level.platforms()[0].aabb.clone()
}

/// Returns the midpoint of an AABB.
fn aabb_midpoint(aabb: &AABB) -> (f32, f32) {
    (
        aabb.x + aabb.w * 0.5,
        aabb.y + aabb.h * 0.5,
    )
}

// ============================================================================
// T01 — FUNC/happy
// Traces To: FR-006 AC-1, §Interface Contract query_terrain
// Kills: terrain query misses platform the player is standing on → falling through.
// Wrong-impl challenge:
//   - Wrong: query_terrain always returns empty → FAIL (we expect Platform found)
//   - Wrong: query_terrain uses strict-less-than for overlap → boundary miss → FAIL
//   - Wrong: query_terrain checks x-axis only → platform below feet missed → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t01_query_terrain_player_standing_on_platform() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // Player AABB: 16x16 box with bottom edge resting on platform top surface.
    // Player feet (max_y) == platform top (p_aabb.y).
    let player_x = p_aabb.x + 4.0;
    let player_y = p_aabb.y - 16.0; // player on top: player.max_y == platform.y
    let player_aabb = AABB {
        x: player_x,
        y: player_y,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&player_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T01: query_terrain with player standing on platform must return Platform tile.\n\
         Player AABB: ({}, {}, 16, 16), Platform AABB: ({}, {}, {}, {})",
        player_x, player_y, p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h
    );
}

// ============================================================================
// T02 — FUNC/happy
// Traces To: FR-006 AC-2, §Interface Contract query_terrain
// Kills: terrain query reports phantom platform in empty air → player floats.
// Wrong-impl challenge:
//   - Wrong: query returns all platforms regardless of overlap → FAIL
//   - Wrong: overlap check inverted → returns tiles that DON'T overlap → FAIL
//   - Wrong: AABB intersects treats any horizontal overlap as full overlap → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t02_query_terrain_player_in_air_returns_empty() {
    let level = Level::new();

    // Player AABB high above any platform (y = -500.0 — well above ground).
    let air_aabb = AABB {
        x: 100.0,
        y: -500.0,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&air_aabb);
    assert!(
        result.is_empty(),
        "T02: query_terrain in open air must return empty Vec (player is not on any platform).\n\
         Got {} tile(s): {:?}",
        result.len(),
        result
    );
}

// ============================================================================
// T03 — FUNC/happy
// Traces To: FR-006 AC-3, §Interface Contract query_terrain
// Kills: platform not detected from below → player can jump through ceiling.
// Wrong-impl challenge:
//   - Wrong: query_terrain only checks top surface → below query returns empty → FAIL
//   - Wrong: y-axis overlap uses player_y only, ignoring height → below contact missed → FAIL
//   - Wrong: collision is directional in query_terrain → query should be direction-agnostic → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t03_query_terrain_player_head_hits_platform_bottom() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // Player AABB: head (min_y) touching platform bottom (max_y).
    // Player stands below the platform, jumping up so head touches the bottom.
    let player_x = p_aabb.x + 4.0;
    let player_y = p_aabb.y + p_aabb.h; // player.max_y == p_aabb.y + p_aabb.h → player head at platform bottom
    let player_aabb = AABB {
        x: player_x,
        y: player_y,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&player_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T03: query_terrain with player head hitting platform bottom must return Platform tile.\n\
         Player AABB: ({}, {}, 16, 16), Platform AABB: ({}, {}, {}, {})",
        player_x, player_y, p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h
    );
}

// ============================================================================
// T04 — FUNC/happy
// Traces To: FR-006 AC-4, §Interface Contract query_terrain
// Kills: platform not detected from side → player can walk through walls.
// Wrong-impl challenge:
//   - Wrong: query_terrain only checks vertical overlap → side contact missed → FAIL
//   - Wrong: AABB intersects uses strict inequality → boundary touching returns false → FAIL
//   - Wrong: platforms are ordered and query stops early → side platform skipped → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t04_query_terrain_player_side_collides_with_wall() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // Player AABB: right side (max_x) touching platform left edge (p_aabb.x).
    // Player stands at same vertical level as platform, walking right into it.
    let player_x = p_aabb.x - 16.0; // player.max_x == p_aabb.x
    let player_y = p_aabb.y + 2.0;
    let player_aabb = AABB {
        x: player_x,
        y: player_y,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&player_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T04: query_terrain with player side-touching platform wall must return Platform tile.\n\
         Player AABB: ({}, {}, 16, 16), Platform AABB: ({}, {}, {}, {})",
        player_x, player_y, p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h
    );
}

// ============================================================================
// T05 — FUNC/happy
// Traces To: §Interface Contract bounds, IAPI-008
// Kills: bounds values wrong → camera clamping broken or kill_y in visible area.
// Wrong-impl challenge:
//   - Wrong: min_x and max_x swapped → camera logic inverted → FAIL
//   - Wrong: kill_y above or at max_y → player dies standing on ground → FAIL
//   - Wrong: bounds returns different values on each call → non-deterministic clamping → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t05_level_bounds_returns_valid_level_bounds() {
    let level = Level::new();
    let bounds = level.bounds();

    // min_x must be <= max_x (otherwise the level is inside-out).
    assert!(
        bounds.min_x <= bounds.max_x,
        "T05: bounds.min_x ({}) must be <= bounds.max_x ({})",
        bounds.min_x,
        bounds.max_x
    );

    // The level should have a positive width (max_x > min_x).
    let level_width = bounds.max_x - bounds.min_x;
    assert!(
        level_width > 0.0,
        "T05: Level width ({}) must be > 0.0 (got min_x={}, max_x={})",
        level_width,
        bounds.min_x,
        bounds.max_x
    );

    // kill_y must be >= the visible area bottom (max_y), so player only dies below screen.
    // The visible area extends from min_y to max_y. kill_y should be below max_y.
    let max_y = bounds.min_y + level_width; // visible area bottom
    // We check kill_y >= max_y — if kill_y < max_y, the kill plane is in the visible area.
    assert!(
        bounds.kill_y >= max_y,
        "T05: kill_y ({}) must be >= visible area bottom ({}) — otherwise player dies on screen.\n\
         bounds = {{ min_x: {}, max_x: {}, min_y: {}, kill_y: {} }}",
        bounds.kill_y,
        max_y,
        bounds.min_x,
        bounds.max_x,
        bounds.min_y,
        bounds.kill_y
    );

    // Second call must return identical values (pure function).
    let bounds2 = level.bounds();
    assert!(
        approx_eq(bounds2.min_x, bounds.min_x) && approx_eq(bounds2.max_x, bounds.max_x),
        "T05: bounds() must return identical values on every call (pure function).\
         \n  Call 1: ({}, {}, {}, {})\
         \n  Call 2: ({}, {}, {}, {})",
        bounds.min_x, bounds.max_x, bounds.min_y, bounds.kill_y,
        bounds2.min_x, bounds2.max_x, bounds2.min_y, bounds2.kill_y
    );
}

// ============================================================================
// T06 — FUNC/happy
// Traces To: §Interface Contract ParallaxLayer::update_scroll
// Kills: scroll multiplier applied to wrong axis or wrong factor → parallax desync.
// Wrong-impl challenge:
//   - Wrong: scroll_offset.x = camera_offset.x (no multiplication) → no parallax effect → FAIL
//   - Wrong: scroll_offset applied to y-axis too → vertical drift → FAIL
//   - Wrong: scroll_offset = camera_offset * (1.0 - speed) instead of * speed → wrong speed → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t06_parallax_layer_scroll_offset_calculation() {
    let mut layer = ParallaxLayer::new(0.3);
    let camera_offset = Vec2 { x: 100.0, y: 50.0 };

    layer.update_scroll(camera_offset);

    assert!(
        approx_eq(layer.scroll_offset.x, 30.0),
        "T06: scroll_offset.x must be camera_offset.x * speed = 100.0 * 0.3 = 30.0, got {}",
        layer.scroll_offset.x
    );
    assert!(
        approx_eq(layer.scroll_offset.y, 0.0),
        "T06: scroll_offset.y must remain 0.0 (horizontal-only parallax), got {}",
        layer.scroll_offset.y
    );
}

// ============================================================================
// T07 — FUNC/happy
// Traces To: §Interface Contract Level::new postcondition
// Kills: wrong number of parallax layers or wrong speed assignments → visual defects.
// Wrong-impl challenge:
//   - Wrong: only 2 layers created → FAIL (we expect 3)
//   - Wrong: speeds in wrong order (e.g., 0.6, 0.3, 0.1) → near layer scrolls faster → FAIL
//   - Wrong: all layers have same speed → no parallax effect → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t07_level_creates_three_parallax_layers_with_correct_speeds() {
    let level = Level::new();
    let layers = level.parallax_layers();

    assert_eq!(
        layers.len(),
        3,
        "T07: Level must have exactly 3 parallax layers, got {}",
        layers.len()
    );

    let expected_speeds: [f32; 3] = [0.1, 0.3, 0.6];
    for (i, (&expected, layer)) in expected_speeds.iter().zip(layers.iter()).enumerate() {
        assert!(
            approx_eq(layer.speed, expected),
            "T07: ParallaxLayer[{}].speed must be {}, got {}",
            i,
            expected,
            layer.speed
        );
    }
}

// ============================================================================
// T08 — FUNC/happy
// Traces To: §Interface Contract AABB::intersects
// Kills: overlap detection fundamentally broken → all collision detection fails.
// Wrong-impl challenge:
//   - Wrong: intersects uses `||` instead of `&&` for x/y overlap → any proximity returns true → FAIL
//   - Wrong: axes check order is swapped (x overlap on y dimension) → FAIL
//   - Wrong: intersects returns always false → no collisions ever → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t08_aabb_intersects_detects_overlapping_boxes() {
    // Two AABBs with clear overlap: (0,0,10,10) and (5,5,10,10)
    // Overlap region: x=[5,10], y=[5,10]
    let a = AABB {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    let b = AABB {
        x: 5.0,
        y: 5.0,
        w: 10.0,
        h: 10.0,
    };

    assert!(
        a.intersects(&b),
        "T08: AABB (0,0,10,10) must intersect (5,5,10,10) — they share region [5-10],[5-10]"
    );
    // Symmetry check: intersects must be commutative.
    assert!(
        b.intersects(&a),
        "T08: intersects must be commutative: b.intersects(a) should == a.intersects(b)"
    );
}

// ============================================================================
// T09 — FUNC/error
// Traces To: §Interface Contract query_terrain postcondition (no panic),
//            §Boundary Conditions query_terrain out-of-bounds
// Kills: out-of-bounds query panics or returns garbage → crash or corruption.
// Wrong-impl challenge:
//   - Wrong: no bounds check → negative coordinates cause index panic → FAIL
//   - Wrong: returns phantom platforms at out-of-bounds coordinates → FAIL
//   - Wrong: infinite loop iterating over tile grid at negative coords → timeout → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t09_query_terrain_far_outside_level_returns_empty() {
    let level = Level::new();

    // AABB completely outside the level, far to the left and above.
    let far_out_aabb = AABB {
        x: -10000.0,
        y: -10000.0,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&far_out_aabb);
    assert!(
        result.is_empty(),
        "T09: query_terrain far outside level bounds must return empty Vec (not panic).\n\
         AABB at ({}, {}, 16, 16) — got {} tile(s)",
        far_out_aabb.x,
        far_out_aabb.y,
        result.len()
    );

    // Also test far to the right and below.
    let far_right_aabb = AABB {
        x: 50000.0,
        y: 50000.0,
        w: 16.0,
        h: 16.0,
    };
    let result2 = level.query_terrain(&far_right_aabb);
    assert!(
        result2.is_empty(),
        "T09: query_terrain far right+below must also return empty Vec for coords ({}, {}), got {} tile(s)",
        far_right_aabb.x,
        far_right_aabb.y,
        result2.len()
    );
}

// ============================================================================
// T10 — BNDRY/edge
// Traces To: §Boundary Conditions query_terrain zero-size AABB (w=0, h=0)
// Kills: zero-size AABB excluded → point-query fails for precise collision detection.
// Wrong-impl challenge:
//   - Wrong: empty check rejects w=0 or h=0 → zero-size query excluded → FAIL
//   - Wrong: AABB intersects uses strict comparison for zero-size → false → FAIL
//   - Wrong: guard clause returns early for zero-size → always empty result → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t10_zero_size_aabb_inside_platform_returns_platform() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // Zero-size AABB at the exact center of the platform.
    let (cx, cy) = aabb_midpoint(&p_aabb);
    let point_aabb = AABB {
        x: cx,
        y: cy,
        w: 0.0,
        h: 0.0,
    };

    let result = level.query_terrain(&point_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T10: Zero-size AABB (w=0, h=0) at platform center ({:.1}, {:.1}) must return Platform tile.\n\
         Platform AABB: ({}, {}, {}, {}), got {} tile(s)",
        cx, cy,
        p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h,
        result.len()
    );
}

// ============================================================================
// T11 — BNDRY/edge
// Traces To: §Boundary Conditions query_terrain inclusive boundary (max_x == platform.min_x)
// Kills: exclusive boundary → 1-pixel gap between adjacent platforms → player falls through.
// Wrong-impl challenge:
//   - Wrong: AABB::intersects uses `other.x >= self.max_x` (strict) instead of `>=` → false → FAIL
//   - Wrong: query_terrain skips tiles whose min edge equals query max edge → false → FAIL
//   - Wrong: f32 comparison with exact `==` instead of `>=` → floating point exact match fails → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t11_boundary_touch_right_edge_equals_platform_left_edge() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // AABB whose right edge (max_x) is exactly at platform left edge (p_aabb.x).
    let player_right_x = p_aabb.x; // player.max_x == p_aabb.x (exact boundary contact)
    let player_aabb = AABB {
        x: player_right_x - 16.0,
        y: p_aabb.y + 2.0,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&player_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T11: AABB with max_x exactly at platform min_x ({}) must return Platform tile (inclusive boundary).\n\
         Player AABB: ({}, {}, 16, 16), Platform AABB: ({}, {}, {}, {})",
        p_aabb.x,
        player_aabb.x, player_aabb.y,
        p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h
    );
}

// ============================================================================
// T12 — BNDRY/edge
// Traces To: §Boundary Conditions query_terrain inclusive boundary (max_y == platform.min_y)
// Kills: exclusive boundary → player piercing platform by 1 pixel → visible artifact.
// Wrong-impl challenge:
//   - Wrong: AABB::intersects uses `other.y > self.max_y` (strict) → false at exact edge → FAIL
//   - Wrong: query_terrain uses player bottom strictly less than platform top → FAIL
//   - Wrong: y-axis check swapped with x-axis → body contact condition inverted → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t12_boundary_touch_bottom_edge_equals_platform_top_edge() {
    let level = Level::new();
    let p_aabb = first_platform_aabb(&level);

    // AABB whose bottom edge (max_y) is exactly at platform top edge (p_aabb.y).
    let player_bottom_y = p_aabb.y; // player.max_y == p_aabb.y (exact landing)
    let player_aabb = AABB {
        x: p_aabb.x + 4.0,
        y: player_bottom_y - 16.0,
        w: 16.0,
        h: 16.0,
    };

    let result = level.query_terrain(&player_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T12: AABB with max_y exactly at platform min_y ({}) must return Platform tile (inclusive boundary).\n\
         Player AABB max_y = {}, Platform min_y = {}",
        p_aabb.y,
        player_bottom_y,
        p_aabb.y
    );
}

// ============================================================================
// T13 — BNDRY/edge
// Traces To: §Boundary Conditions query_terrain partial out-of-bounds
// Kills: panic or overflow when AABB spans outside level bounds → crash.
// Wrong-impl challenge:
//   - Wrong: out-of-bounds coordinate causes index overflow → panic → FAIL
//   - Wrong: early return if any part of AABB is out of bounds → misses valid overlap → FAIL
//   - Wrong: ignores in-bounds platforms when AABB extends outside → returns empty → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t13_partial_out_of_bounds_aabb_returns_only_intersecting_platforms() {
    let level = Level::new();

    // Assert at least one platform exists with x > 0 so it might be found.
    assert!(
        !level.platforms().is_empty(),
        "T13 requires at least one platform"
    );

    // Pick the first platform that is entirely within level bounds.
    let p_aabb = first_platform_aabb(&level);

    // Create an AABB that starts far left (x = -500.0) and extends right
    // so that it overlaps with the first platform whose x >= 0.
    // The part OUTSIDE the level (negative x) must not cause a panic.
    let half_out_aabb = AABB {
        x: -500.0,
        y: p_aabb.y,
        w: 516.0 + p_aabb.w, // extends from -500 to 16+p_aabb.w, overlapping the platform
        h: p_aabb.h,
    };

    let result = level.query_terrain(&half_out_aabb);
    let has_platform = result.iter().any(|t| matches!(t, Tile::Platform(_)));
    assert!(
        has_platform,
        "T13: AABB half outside level (starts at x=-500) must still detect platforms in overlapping region.\n\
         Platform AABB: ({}, {}, {}, {}), got {} tile(s)",
        p_aabb.x, p_aabb.y, p_aabb.w, p_aabb.h,
        result.len()
    );
}

// ============================================================================
// T14 — BNDRY/edge
// Traces To: §Boundary Conditions AABB::intersects zero-size AABBs at same point
// Kills: zero-size AABBs incorrectly return false → point collision detection broken.
// Wrong-impl challenge:
//   - Wrong: early return false if w=0 or h=0 on either AABB → FAIL
//   - Wrong: overlap condition uses `>` instead of `>=` → zero overlap area → false → FAIL
//   - Wrong: uses `c > a` instead of `c >= a` for overlap → zero extents never match → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t14_two_zero_size_aabbs_at_same_coordinate_intersect() {
    // Two point-like AABBs at exactly the same coordinates.
    let a = AABB {
        x: 5.0,
        y: 5.0,
        w: 0.0,
        h: 0.0,
    };
    let b = AABB {
        x: 5.0,
        y: 5.0,
        w: 0.0,
        h: 0.0,
    };

    assert!(
        a.intersects(&b),
        "T14: Two zero-size AABBs at same coordinate (5,5) must intersect (point-in-point)."
    );
    // Symmetry: same result in reverse.
    assert!(
        b.intersects(&a),
        "T14: Intersection of zero-size AABBs must be commutative."
    );
}

// ============================================================================
// T15 — BNDRY/edge
// Traces To: §Boundary Conditions AABB::intersects non-overlapping
// Kills: non-overlapping boxes incorrectly report collision → false positive collisions.
// Wrong-impl challenge:
//   - Wrong: always returns true → all objects collide with everything → FAIL
//   - Wrong: x-overlap OR y-overlap instead of AND → distant boxes collide → FAIL
//   - Wrong: off-by-one on overlap check → gap of 1 pixel treated as overlap → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t15_non_overlapping_aabbs_do_not_intersect() {
    // Two AABBs separated by 10 units in both axes — no overlap.
    let a = AABB {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    // a occupies [0,10] × [0,10]
    // b starts at x=20, y=20 — 10-unit gap on both axes.
    let b = AABB {
        x: 20.0,
        y: 20.0,
        w: 10.0,
        h: 10.0,
    };
    // b occupies [20,30] × [20,30]

    assert!(
        !a.intersects(&b),
        "T15: AABB (0,0,10,10) must NOT intersect (20,20,10,10) — gap of 10 on both axes."
    );
    assert!(
        !b.intersects(&a),
        "T15: Non-intersection must be commutative."
    );

    // Additional case: horizontal-only separation (same y, different x).
    let c = AABB {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    let d = AABB {
        x: 15.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    assert!(
        !c.intersects(&d),
        "T15: AABB (0,0,10,10) must NOT intersect (15,0,10,10) — 5-unit horizontal gap."
    );

    // Vertical-only separation (same x, different y).
    let e = AABB {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    let f = AABB {
        x: 0.0,
        y: 15.0,
        w: 10.0,
        h: 10.0,
    };
    assert!(
        !e.intersects(&f),
        "T15: AABB (0,0,10,10) must NOT intersect (0,15,10,10) — 5-unit vertical gap."
    );
}

// ============================================================================
// T16 — FUNC/happy
// Traces To: §Interface Contract Level::new postcondition (platforms.len() >= 1)
// Kills: empty level or zero-area platforms → player instantly falls to kill_y.
// Wrong-impl challenge:
//   - Wrong: Level::new returns empty platforms vec → FAIL
//   - Wrong: platform w=0 or h=0 → invalid collision geometry → FAIL
//   - Wrong: platform AABB fields are NaN or infinite → undefined behavior → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t16_level_platforms_have_valid_dimensions() {
    let level = Level::new();
    let platforms = level.platforms();

    assert!(
        !platforms.is_empty(),
        "T16: Level must have at least 1 platform. Got 0 platforms — player cannot stand anywhere."
    );

    for (i, platform) in platforms.iter().enumerate() {
        let aabb = &platform.aabb;
        assert!(
            aabb.w > 0.0,
            "T16: Platform[{}].aabb.w ({}) must be > 0.0",
            i,
            aabb.w
        );
        assert!(
            aabb.h > 0.0,
            "T16: Platform[{}].aabb.h ({}) must be > 0.0",
            i,
            aabb.h
        );
        assert!(
            aabb.w.is_finite() && aabb.h.is_finite(),
            "T16: Platform[{}].aabb dimensions must be finite (w={}, h={})",
            i,
            aabb.w,
            aabb.h
        );
        assert!(
            aabb.x.is_finite() && aabb.y.is_finite(),
            "T16: Platform[{}].aabb position must be finite (x={}, y={})",
            i,
            aabb.x,
            aabb.y
        );
    }
}

// ============================================================================
// T17 — BNDRY/edge
// Traces To: §Boundary Conditions query_terrain full-level AABB
// Kills: large query skips platforms → incomplete collision for full-level scan.
// Wrong-impl challenge:
//   - Wrong: query limits result to first N platforms → returns subset → FAIL
//   - Wrong: max bounds computed incorrectly → misses platforms at edge → FAIL
//   - Wrong: iteration terminates early on first non-overlap → misses later platforms → FAIL
// ============================================================================

// real_test (feature #2)
#[test]
fn t17_full_level_aabb_returns_all_platforms() {
    let level = Level::new();
    let bounds = level.bounds();
    let platform_count = level.platforms().len();

    // Skip if only 1 platform (test is trivially correct).
    if platform_count <= 1 {
        // For a single-platform level, verify the full query finds it.
        let full_aabb = AABB {
            x: bounds.min_x,
            y: bounds.min_y,
            w: bounds.max_x - bounds.min_x,
            h: bounds.kill_y - bounds.min_y, // full vertical span
        };
        let result = level.query_terrain(&full_aabb);
        let found = result.iter().filter(|t| matches!(t, Tile::Platform(_))).count();
        assert_eq!(
            found, platform_count,
            "T17: Full-level AABB query must return all {} platform(s), got {}",
            platform_count, found
        );
        return;
    }

    // Multi-platform level: create AABB covering the entire level.
    let full_aabb = AABB {
        x: bounds.min_x,
        y: bounds.min_y,
        w: bounds.max_x - bounds.min_x,
        h: bounds.kill_y - bounds.min_y,
    };

    let result = level.query_terrain(&full_aabb);

    // Count Platform tiles in the result.
    let found_platform_count = result
        .iter()
        .filter(|t| matches!(t, Tile::Platform(_)))
        .count();

    assert_eq!(
        found_platform_count,
        platform_count,
        "T17: Full-level AABB query ({:.0}x{:.0} starting at {:.0},{:.0}) must return ALL {} platforms.\n\
         Got {} Platform tiles. Missing platforms indicate incomplete overlap detection.",
        full_aabb.w, full_aabb.h, full_aabb.x, full_aabb.y,
        platform_count,
        found_platform_count
    );
}
