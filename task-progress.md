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

### Session 1 — Feature Design #1 (2026-05-31)

- **Feature #1: Engine Core** (FR-018) — 60fps fixed-timestep game loop
- **SRS**: §FR-018 (lines 269-278)
- **Design §2.1**: (lines 98-117)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/1-engine-core.md)
- current.phase: design → tdd

### Session 2 — TDD #1 (2026-05-31)

- **Feature #1: Engine Core** (FR-018) — 60fps fixed-timestep game loop
- **Status**: failing
- **Dependencies**: none
- TDD: green ✓ (R-G-R complete) — 12 tests, categories=FUNC/happy+BNDRY/edge+PERF/frame-time, negative_ratio=0.50
- Quality: line=80.56%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (1/1 FR-018)
- current.phase: tdd → st

### Feature #1: Engine Core — PASS
- Completed: 2026-05-31
- TDD: green ✓
- Quality Gates: 80.56% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 13 cases, all PASS
- Inline Check: PASS (P2: 3/3 methods, T2: 13/13 tests, D3: OK, ATS Category: 3/3, §4: 4 files 0 violations)
- Git: 30a2c5e feat: feature #1 engine-core — ST passed (13/13 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 3 — Feature Design #2 (2026-05-31)

- **Feature #2: Level & Background** (FR-006) — 关卡几何 + 3层视差背景
- **SRS**: §FR-006 (lines 133-142)
- **Design §2.2**: (lines 119-141)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/2-level-background.md)
- current.phase: design → tdd

### Session 4 — TDD #2 (2026-05-31)

- **Feature #2: Level & Background** (FR-006) — 关卡几何 + 3层视差背景
- **Status**: failing
- **Dependencies**: none
- TDD: green ✓ (R-G-R complete) — 17 tests, categories=FUNC/happy=9+FUNC/error=1+BNDRY/edge=7, negative_ratio=0.47
- Quality: line=89.02%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (1/1 FR-006)
- current.phase: tdd → st

### Feature #2: Level & Background — PASS
- Completed: 2026-06-01
- TDD: green ✓
- Quality Gates: 89.02% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 17 cases, all PASS
- Inline Check: PASS (P2: 6/6 methods, T2: 17/17 tests, D3: N/A, ATS Category: 2/2, §4: 0 files 0 violations)
- Git: 47840fb feat: feature #2 level-background — ST passed (17/17 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 5 — Feature Design #3 (2026-06-01)

- **Feature #3: Player Controller** (FR-001, FR-002, FR-003) — 玩家水平移动/跳跃/冲刺
- **SRS**: §FR-001 (lines 99-108), §FR-002 (lines 111-121), §FR-003 (lines 123-131)
- **Design §2.3**: (lines 143-169)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/3-player-controller.md)
- current.phase: design → tdd