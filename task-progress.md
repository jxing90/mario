# Task Progress — Mario 2D Platformer Demo

> Session log only. Current project state (which feature is locked, which
> phase it's in, how many features are passing) lives in
> `feature-list.json` — single source of truth. Query it with:
>
>     python scripts/count_pending.py feature-list.json

## Session Log

### Session 0 — Init (2026-05-31)

**Input Documents**:
- SRS: `docs/plans/2026-05-31-mario-platformer-srs.md`
- Design: `docs/plans/2026-05-31-mario-platformer-design.md`
- UCD: `docs/plans/2026-05-31-mario-platformer-ucd.md`
- ATS: `docs/plans/2026-05-31-mario-platformer-ats.md`

**Project Scaffold**:
- 13 features (10 core + 3 NFR)
- 3 UI features (F06 Life/Death/Win, F09 HUD, F10 Display Config)
- 0 required_configs (offline desktop game)
- Rust edition 2024 + Macroquad 0.4 + cargo test + cargo-tarpaulin

**Sizing**: 3 large features retained per Design rationale (F03 Player, F06 Life/Death/Win, F08 Collectibles & Blocks)

**Artifacts**: feature-list.json, env-guide.md, long-task-guide.md, init.sh, init.ps1, .env.example, .gitignore, scripts/, Cargo.toml, src/ skeleton