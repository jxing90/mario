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

### Session 6 — TDD #3 (2026-06-01)

- **Feature #3: Player Controller** (FR-001, FR-002, FR-003) — 玩家水平移动/跳跃/冲刺
- **Status**: failing
- **Dependencies**: [1, 2]
- TDD: green ✓ (R-G-R complete) — 32 tests, categories=FUNC/happy+FUNC/error+BNDRY/edge+INTG/terrain, negative_ratio=45.2%
- Quality: line=88.52%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (3/3 FR-001+FR-002+FR-003)
- current.phase: tdd → st

### Feature #3: Player Controller — PASS
- Completed: 2026-06-01
- TDD: green ✓
- Quality Gates: 88.52% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 31 cases, all PASS
- Inline Check: PASS (P2: 7/7 methods, T2: 31/31 tests, D3: OK, ATS Category: 2/2, §4: 6 files 0 violations)
- Git: b615dc6 feat: feature #3 player-controller — ST passed (31/31 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 7 — Feature Design #4 (2026-06-01)

- **Feature #4: Camera System** (FR-013) — 摄像机平滑跟随 + 死区 + 关卡钳制
- **SRS**: §FR-013 (lines 190-200)
- **Design §2.4**: (lines 170-195)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/4-camera-system.md)
- current.phase: design → tdd

### Session 8 — TDD #4 (2026-06-01)

- **Feature #4: Camera System** (FR-013) — 摄像机平滑跟随 + 死区 + 关卡钳制
- **Status**: failing
- **Dependencies**: [3, 2]
- TDD: green ✓ (R-G-R complete) — 21 tests, categories=FUNC/happy=9+FUNC/error=2+BNDRY/edge=6+BNDRY/invalid=1+INTG/player=1+INTG/level=1+INTG/parallax=1, negative_ratio=0.429
- Quality: line=82.35%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (1/1 FR-013)
- current.phase: tdd → st

### Feature #4: Camera System — PASS
- Completed: 2026-06-01
- TDD: green ✓
- Quality Gates: 82.35% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 20 cases, all PASS
- Inline Check: PASS (P2: 3/3 methods, T2: 17/17 tests, D3: OK, ATS Category: 2/2, §4: 2 files 0 violations)
- Git: 43926fc feat: feature #4 camera-system — ST passed (20/20 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 9 — Feature Design #5 (2026-06-01)

- **Feature #5: Hazards** (FR-009) — 尖刺实体 + 深渊死亡平面
- **SRS**: §FR-009 (lines 154-163)
- **Design §2.5**: (lines 196-214)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/5-hazards.md)
- current.phase: design → tdd

### Session 10 — TDD #5 (2026-06-01)

- **Feature #5: Hazards** (FR-009) — 尖刺实体 + 深渊死亡平面
- **Status**: failing
- **Dependencies**: [2, 3]
- TDD: green ✓ (R-G-R complete) — 14 tests, categories=FUNC/happy+FUNC/error+BNDRY/edge+BNDRY/batch+BNDRY/null+INTG/level+INTG/physics, negative_ratio=57.1%
- Quality: line=88.46%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (1/1 FR-009)
- current.phase: tdd → st

### Feature #5: Hazards — PASS
- Completed: 2026-06-02
- TDD: green ✓
- Quality Gates: 88.46% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 14 cases, all PASS
- Inline Check: PASS (P2: 5/5 methods, T2: 14/14 tests, D3: N/A, ATS Category: 2/2, §4: 0 files 0 violations)
- Git: 15866b5 feat: feature #5 hazards — ST passed (14/14 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 11 — Feature Design #6 (2026-06-02)

- **Feature #6: Life, Death & Win** (FR-014a, FR-014b, FR-014c, FR-015) — 生命/死亡/重生/胜利
- **SRS**: §FR-014a (line 202), §FR-014b (line 213), §FR-014c (line 225), §FR-015 (line 235)
- **Design §2.6**: (lines 215-235)
- **UCD**: §3.13 Game Over Screen (line 166), §3.14 Victory Screen (line 172)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: true (ui_entry=/game-over-overlay, /victory-overlay)
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/6-life-death-win.md)
- current.phase: design → tdd

### Session 12 — TDD #6 (2026-06-02)

- **Feature #6: Life, Death & Win** (FR-014a, FR-014b, FR-014c, FR-015) — 生命/死亡/重生/胜利
- **Status**: failing
- **Dependencies**: [3, 2, 5]
- TDD: green ✓ (R-G-R complete) — 75 tests, categories=FUNC/happy+FUNC/error+BNDRY/edge+UI/render+INTG/player+INTG/physics+INTG/level, negative_ratio=40.8%
- Quality: line=91.00%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (4/4 FR-014a+FR-014b+FR-014c+FR-015)
- current.phase: tdd → st

---

### Feature #6: Life, Death & Win — PASS
- Completed: 2026-06-02
- TDD: green ✓
- Quality Gates: 91.00% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 32 cases, all PASS
- Inline Check: PASS (P2: 21/21 methods, T2: 75/75 tests, D3: OK, ATS Category: 4/4, §4: 0 files 0 violations)
- Git: d02e761 feat: feature #6 life-death-win — ST passed (32/32 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 13 — Feature Design #11 (2026-06-02)

- **Feature #11: 60fps Frame Rate (NFR-001)** — 性能度量仪表 + FPS 计数器
- **SRS**: NFR-001 (§5 非功能需求表，行 362)
- **Design**: §1.5 NFR 对齐摘要 (行 92-94)；无独立 §2.11（首个 NFR 特性）
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/11-60fps-frame-rate-nfr-001.md)
- current.phase: design → tdd

### Session 14 — TDD #11 (2026-06-02)

- **Feature #11: 60fps Frame Rate (NFR-001)** — 性能度量仪表 + FPS 计数器
- **Status**: failing
- **Dependencies**: [1]
- TDD: green ✓ (R-G-R complete) — 17 tests, categories=FUNC/happy+FUNC/error+BNDRY/edge+PERF, negative_ratio=0.471
- Quality: line=92.10%, branch=N/A (MSVC tool limitation), srs_trace_coverage=OK (1/1 NFR-001)
- current.phase: tdd → st

---

### Feature #11: 60fps Frame Rate (NFR-001) — PASS
- Completed: 2026-06-02
- TDD: green ✓
- Quality Gates: 92.10% line, branch=N/A (MSVC tool limitation)
- Feature-ST: 17 cases, all PASS
- Inline Check: PASS (P2: 7/7 methods, T2: 17/17 tests, D3: OK, ATS Category: 3/3, §4: 3 files 0 violations)
- Git: c589b2f feat: feature #11 60fps-frame-rate — ST passed (17/17 cases)
#### Risks
- ⚠ [Coverage] branch N/A — MSVC tool limitation blocked branch coverage measurement; verified manually via test case boundary/path analysis

### Session 15 — Feature Design #7 (2026-06-03)

- **Feature #7: Patrol Enemy** (FR-010) — 巡逻敌人 + 踩踏判定 + 碰撞方向分派
- **SRS**: §FR-010 (line 165-175)
- **Design**: §2.7 (lines 237-255)
- **env-guide §4**: greenfield — no codebase constraints
- **UI**: false
- **Config Gate**: skipped (no required_configs)
- Design: DONE (docs/features/7-patrol-enemy.md)
- current.phase: design → tdd