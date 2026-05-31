# Mario 2D Platformer Demo — UCD Style Guide

**Date**: 2026-05-31
**Status**: Approved
**SRS Reference**: docs/plans/2026-05-31-mario-platformer-srs.md

## 1. Visual Style Direction

**Style**: Modern Pixel Art

**Mood**: 复古像素精灵结合现代渲染技术——清晰的硬边像素角色和场景图块，搭配动态粒子特效、视差滚动背景和平滑摄像机。致敬经典马里奥但不囿于 NES 硬件限制。

**References**: Celeste, Shovel Knight, The Messenger

## 2. Style Tokens

### 2.1 Color Palette

| Token | Hex | Usage |
|-------|-----|-------|
| --color-sky | `#5C94FC` | Sky background |
| --color-sky-top | `#4A7CE4` | Sky top gradient |
| --color-cloud | `#FFFFFF` | Clouds |
| --color-mountain-far | `#A8C8E8` | Far hills |
| --color-mountain-mid | `#78A878` | Mid hills |
| --color-mountain-near | `#588030` | Near hills |
| --color-ground | `#C84C0C` | Ground/brick main |
| --color-ground-dark | `#A03808` | Ground shadow |
| --color-brick-mortar | `#E8D8B8` | Brick mortar |
| --color-platform | `#80A050` | Platform surface (grass green) |
| --color-platform-dark | `#588030` | Platform side |
| --color-player-red | `#E80000` | Player cap/shirt |
| --color-player-skin | `#FCA044` | Player skin |
| --color-player-overalls | `#0038F8` | Player overalls |
| --color-enemy-brown | `#A84800` | Enemy body |
| --color-coin-gold | `#F8B800` | Coin main |
| --color-coin-shine | `#FCE048` | Coin highlight |
| --color-block-q | `#F8B800` | Question block |
| --color-block-used | `#888888` | Used block |
| --color-spike | `#E0E0E0` | Spike hazard |
| --color-flag-green | `#00A800` | Flag |
| --color-flag-pole | `#D0D0D0` | Flagpole |
| --color-pipe-green | `#00A844` | Pipe main |
| --color-pipe-dark | `#007030` | Pipe shadow |
| --color-mushroom-red | `#E82000` | Super Mushroom cap |
| --color-mushroom-stem | `#F8D8B0` | Mushroom stem |
| --color-fire-flower-orange | `#F85800` | Fire Flower petals |
| --color-ui-text | `#FFFFFF` | HUD/menu text |
| --color-ui-text-outline | `#000000` | Text outline |
| --color-ui-overlay | `rgba(0,0,0,0.65)` | Overlay background |
| --color-ui-bg | `#1A1A2E` | Menu background |

### 2.2 Sprite Size Specification

| Entity | Sprite Resolution | Notes |
|--------|-------------------|-------|
| Player (small) | 16×16 px | Base form |
| Player (large) | 16×32 px | After Mushroom power-up |
| Enemy | 16×16 px | Patrol mushroom creature |
| Coin | 8×8 px | With rotation frames |
| Question Block | 16×16 px | Active / used states |
| Platform Tile | 16×16 px | Seamlessly tileable |
| Super Mushroom | 16×16 px | Bouncing pickup |
| Fire Flower | 16×16 px | Static pickup |
| Fireball | 8×8 px | Horizontal projectile |
| Flagpole | 16×80 px | End-of-level marker |
| Spike | 16×8 px | Ground hazard |
| Pipe | 32×32 px | Tileable |
| Cloud | 16×16 px | Background decoration |
| Tree | 16×16 px | Background decoration |

### 2.3 Typography

| Token | Font Style | Effective Size | Usage |
|-------|-----------|----------------|-------|
| --font-hud | Pixel bitmap font | 8px equivalent | HUD coin/lives counter |
| --font-overlay-title | Pixel outline font | 16px equivalent | "Game Over" / "Victory!" titles |
| --font-overlay-body | Pixel outline font | 8px equivalent | Action prompts |
| --font-menu-label | Pixel outline font | 10px equivalent | Menu items |
| --font-menu-title | Pixel outline font | 12px equivalent | Menu title |

All text uses 1px black outline for readability against any background.

### 2.4 Iconography & Effects

- **Coin collection**: 4-frame rotation shrink animation (32ms/frame)
- **Death animation**: Player bounce up 16px then fall offscreen (1.5s total)
- **Invulnerability flicker**: 4Hz show/hide alternation (250ms period)
- **Question block**: 3-frame "?" flicker (unactivated state)
- **Particles**: 2×2 px white particles on jump/land (6-10 particles, random velocity decay)

## 3. Component Prompts

### 3.1 Player Character (Small Form)
**SRS Trace**: FR-001, FR-002, FR-003
**Variants**: Idle, Walk (2-frame), Jump, Sprint

> A 16×16 pixel art Mario-like character sprite, side view, facing right. Red cap and shirt (#E80000), skin-colored face and hands (#FCA044), blue overalls (#0038F8), brown shoes. Modern pixel art style with subtle anti-aliasing on edges, 16-color palette. Standing pose with slight animation-ready stance. Clean pixel clusters, no jpeg artifacts. White background.

### 3.2 Patrol Enemy (Mushroom Creature)
**SRS Trace**: FR-010
**Variants**: Walk frame 1, Walk frame 2, Stomped (flattened)

> A 16×16 pixel art enemy sprite resembling a brown mushroom creature, side view. Round brown cap (#A84800) with lighter underside, small angry eyes, stubby feet. Simple walk cycle (2 frames: feet apart / feet together). Modern pixel art, 12-color palette. White background.

### 3.3 Coin
**SRS Trace**: FR-008
**Variants**: Frame 1 (full), Frame 2 (¾), Frame 3 (½), Frame 4 (¼)

> An 8×8 pixel art gold coin sprite, isometric angled view. Bright gold (#F8B800) body with lighter highlight (#FCE048) on top edge. 4-frame spin animation (coin narrowing to 2px wide then widening back). Modern pixel art with anti-aliased edges within 8×8 grid. White background.

### 3.4 Ground/Platform Tile
**SRS Trace**: FR-006
**Variants**: Ground tile, Underground tile

> A 16×16 pixel art ground tile. Orange-brown brick body (#C84C0C) with subtle darker mortar lines (#A03808). Grass surface on the top 4px: vibrant green (#80A050) with darker grass blade accents. Seamlessly tileable horizontally and vertically. Modern pixel art style.

### 3.5 Question Block
**SRS Trace**: FR-011
**Variants**: Active (with "?"), Used (dark)

> A 16×16 pixel art question block. Active state: golden (#F8B800) block with "?" symbol in center, 3-frame flicker animation (bright/dim/bright). Used state: dark gray (#888888) block, no symbol. Modern pixel art with subtle highlight on top-left edge. White background.

### 3.6 Super Mushroom Pickup
**SRS Trace**: FR-011
**Variants**: Default

> A 16×16 pixel art Super Mushroom power-up item. Red cap with white spots (#E82000 on #FFFFFF), cream-colored stem (#F8D8B0), small black eyes on stem. Modern pixel art, side view. White background.

### 3.7 Fire Flower Pickup
**SRS Trace**: FR-011
**Variants**: Default

> A 16×16 pixel art Fire Flower power-up item. Orange petals (#F85800) arranged in a simple flower shape, small green stem at base. Modern pixel art, top-down/flat view for clear pickup identification. White background.

### 3.8 Flagpole
**SRS Trace**: FR-015
**Variants**: Default, Flag raised

> A vertical flagpole sprite, 16×80 px. Silver-gray pole (#D0D0D0), green triangular flag (#00A800) at the top. Simple pixel art design with clean edges. Modern pixel art style.

### 3.9 Spike Hazard
**SRS Trace**: FR-009
**Variants**: Default

> A 16×8 px spike hazard sprite. Silver-white triangular spikes (#E0E0E0) arranged in a row with darker base. 3 spikes per 16px width. Modern pixel art, ground-level placement.

### 3.10 Pipe
**SRS Trace**: FR-012 (deferred, but sprite defined for future use)
**Variants**: Pipe body, Pipe rim

> A 32×32 px pixel art pipe tile. Green body (#00A844) with darker shadow on right side (#007030). Top rim is wider with highlight on left edge. Seamlessly tileable vertically. Modern pixel art style.

### 3.11 HUD Elements
**SRS Trace**: FR-016
**Variants**: Default (updating dynamically)

> Top-left corner game HUD overlay. Small gold coin icon (8×8px pixel art) beside white pixel font number displaying count. Below it, a small red heart icon (8×8px pixel art) beside white pixel font number displaying lives. All text has 1px black outline. Anchored at 3% from left and 3% from top edges of viewport. Transparent background.

### 3.12 Options Menu
**SRS Trace**: FR-017
**Variants**: Default, Resolution selected, Fullscreen toggled

> In-game options menu overlay. Dark semi-transparent background (#1A1A2E at 85% opacity). Centered panel with "OPTIONS" title in 12px white pixel font with black outline. Resolution selector list showing "720p / 1080p / 1440p" in 10px white pixel font, current selection highlighted gold (#F8B800). Fullscreen toggle "ON/OFF" below. "Press ESC to close" hint at bottom in 8px white pixel font. Clean layout, no decorative elements.

### 3.13 Game Over Screen
**SRS Trace**: FR-014c
**Variants**: Default

> Full-screen overlay at 65% black opacity. Centered large "GAME OVER" text in 16px white pixel font with 2px black outline. Below: "Press Space to Restart" in 8px white pixel font with 1px black outline, blinking at 2Hz. Dark mood. No other visual elements.

### 3.14 Victory Screen
**SRS Trace**: FR-015
**Variants**: Default

> Full-screen overlay at 65% black opacity. Centered large "VICTORY!" text in 16px gold (#F8B800) pixel font with 2px black outline. Below: "Coins: 000" in 10px white pixel font with 1px black outline. Below: "Press Space to Play Again" in 8px white pixel font with 1px black outline, blinking at 2Hz. Celebratory but minimalist — no complex effects.

## 4. Page Prompts

### 4.1 Playing Screen
**SRS Trace**: FR-001..FR-016
**User Persona**: Local Player

#### Full-Page Prompt
> A 2D side-scrolling pixel art platformer gameplay screenshot. Three-layer parallax background: farthest layer is gradient blue sky (#5C94FC to #4A7CE4) with scattered white pixel clouds, middle layer shows rolling green hills (#78A878) at 0.3× scroll speed, near background has darker green hills (#588030) at 0.6× scroll speed. Foreground: brown brick platforms with green grass tops at various heights spanning left to right, golden question blocks floating in the air, gold coins placed on platforms and in air, a brown mushroom enemy patrolling on a lower platform, and a checkered flagpole visible in the far right distance. A small red-capped pixel art player character stands on a mid-height platform at center-left of frame. Top-left corner HUD shows gold coin icon with count and heart icon with count in white pixel font with black outlines. Rendered at virtual resolution 480×270 with nearest-neighbor upscaling. Modern pixel art aesthetic with subtle particle effects.

#### Key Interactions
- Left/Right Arrow or A/D: Move player horizontally
- Space: Jump (variable height)
- Shift: Sprint
- ESC: Open options menu

### 4.2 Parallax Background Layers
**SRS Trace**: NFR-003
**User Persona**: N/A (environmental)

#### Full-Page Prompt
> Three distinct parallax layers for a 2D pixel art platformer, each on transparent background for compositing. Layer 1 (far, scroll 0.1×): gradient blue sky with 8 cloud sprites at 16×16 px each, scattered at varying heights. Layer 2 (mid, scroll 0.3×): rolling green hill silhouettes at 32px height, repeating pattern, soft blue-green tint. Layer 3 (near, scroll 0.6×): darker green hill silhouettes at 48px height with occasional 16×16 px tree sprites. Modern pixel art, each layer seamlessly tileable horizontally.

## 5. Style Rules & Constraints

| Rule | Description |
|------|-------------|
| **Scaling** | Nearest-neighbor interpolation only; bilinear/trilinear scaling prohibited |
| **Render resolution** | Virtual canvas 480×270 (16:9 base), scaled to target 720p/1080p/1440p |
| **Pixel alignment** | All sprite positions rounded to integer coordinates; no sub-pixel rendering |
| **Font outline** | All HUD/UI text uses 1px black outline for readability on any background (equivalent WCAG AA) |
| **Color depth** | Each sprite limited to 16 colors; maintains pixel art aesthetic |
| **Animation framerate** | Character animations 8-12 fps (2-4 frame loops); effects 16 fps |
| **UI framework** | All UI custom-drawn via Macroquad; no external UI library |
| **Parallax** | 3-layer parallax with distinct scroll speeds (0.1×, 0.3×, 0.6× camera speed) |
| **Virtual resolution** | Internal rendering at 480×270, upscaled to display resolution |
