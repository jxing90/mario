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

### Changed
- (none yet)

### Fixed
- (none yet)

---

_Format: [Keep a Changelog](https://keepachangelog.com/) — Updated after every git commit._
