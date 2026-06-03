# Release Notes — Mario 2D Platformer Demo

## [Unreleased]

### Added
- Initial project scaffold
- Feature #1: Engine Core — 60fps fixed-timestep game loop with accumulator (max 5 catch-up steps), virtual 480x270 render target, display config support (IAPI-011), 13/13 ST cases passing
- Feature #2: Level & Background — single-level platform geometry, 3-layer parallax background, terrain query (IAPI-005) and level bounds (IAPI-008), 17/17 ST cases passing
- Feature #3: Player Controller — horizontal movement (acceleration/friction/reversal), variable-height jump, sprint (1.5x speed), power-up state machine (Small/Super/Fire), IAPI-007/IAPI-009 provider, 31/31 ST cases passing
- Feature #4: Camera System — horizontal 8%/frame lerp convergence (player at 37.5% viewport left), vertical 60% dead-zone with 5%/frame tracking, level bounds clamping (IAPI-010 provider), 20/20 ST cases passing
- Feature #5: Hazards — spike entity collision detection (16x8 px), kill-plane pit fall detection, CollisionEvent emission (HazardContact/PitFall) for F06 consumption, 14/14 ST cases passing
- Feature #6: Life, Death & Win — lives counter (initial 3), checkpoint activation, death animation (1.5s input lock) → respawn at checkpoint (2s invulnerability + 4Hz flicker) or Game Over (lives=0), flagpole trigger → slide animation → Victory screen with coin total + restart prompt, full game reset via Space, 32/32 ST cases passing

- Feature #7: Patrol Enemy — enemy that patrols between two waypoints at constant speed (reverses at endpoint ≤1 frame), stomp-kill from above with player bounce, side/below contact triggers death, 17/17 ST cases passing
- Feature #8: Collectibles & Blocks — coin collection on player collision (deactivate + counter +1, reset on death), question blocks hit from below (loot table: Coin 70%/Super Mushroom 15%/Fire Flower 15%), power-ups (Super Mushroom bouncing + growth, Fire Flower static + fireball ability), fireball projectile (enemy elimination), 39/39 ST cases passing
- Feature #11: 60fps Frame Rate (NFR-001) — built-in FPS counter with min/max/avg/p99 frame-time statistics and 60-second sliding window FPS validation, 17/17 ST cases passing
- Feature #9: HUD — screen-space overlay rendering PlayerStats (coins, lives) at viewport (3%, 3%) with 1px black-outline pixel font, coin/heart icons (8x8px pixel art), instant frame update via IAPI-009, 24/24 ST cases passing
- Feature #10: Display Config — ESC toggles options menu overlay with resolution selection (720p/1080p/1440p) and fullscreen toggle, Arrow keys navigate with wrap-around, Enter confirms via IAPI-011 with nearest-neighbor scaling, ESC closes without changes, 31/31 ST cases passing
- Feature #12: Multi-Resolution Display (NFR-002) — ResolutionVerifier programmatic validation tool with HUD anchor (3%, 3%) self-consistency checks and player visible area ratio computations across 720p/1080p/1440p, 22/22 automated cases passing + 3 manual visual-judgment cases, 25/25 ST cases passing
- Feature #13: Pixel Art Rendering (NFR-003) — nearest-neighbor texture filtering (apply_pixel_art_filter), sprite coordinate rounding (round_sprite_pos with banker's rounding), palette verification (SpritePalette ≤16 colors, PNG byte-level decoding for GL-context-free testing), coin.png and heart.png palette compliance verified, 19/19 automated + 1 manual visual-judgment, 20/20 ST cases passing

### Changed
- (none yet)

### Fixed
- (none yet)

### System Test
- **Date**: 2026-06-04
- **Verdict**: Go (Conditional)
- **ST Report**: docs/plans/2026-06-04-st-report.md
- **ST Plan**: docs/plans/2026-06-04-st-plan.md
- **Summary**: 374 total tests (0 failures), 27 new System ST tests (13 integration + 5 smoke + 9 E2E), line coverage 86.26% (≥80% gate), ATS strict mode compliant, 0 Critical/Major defects, 7 manual visual-judgment cases pending
- **Categories executed**: Regression, Integration, Smoke, E2E, Performance, Security audit, Compatibility, Exploratory
- **Examples**: 5 scenario-based usage examples generated (13/13 features covered)

---

_Format: [Keep a Changelog](https://keepachangelog.com/) — Updated after every git commit._
