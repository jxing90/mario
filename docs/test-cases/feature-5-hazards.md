# 测试用例集: Hazards

**Feature ID**: 5
**关联需求**: FR-009 (Hazards — Spikes and Pits)
**日期**: 2026-06-02
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

> Specification resolutions applied from Feature Design Clarification Addendum (无需澄清 — 全部规格明确).

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 8 |
| boundary | 6 |
| ui | 0 |
| security | 0 |
| performance | 0 |
| **合计** | **14** |

---

### 用例编号

ST-FUNC-005-001

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-1: 尖刺接触死亡

### 测试目标

验证玩家碰撞体与尖刺（Spike）AABB 重叠时，`Physics::hazard_check` 返回包含 `HazardContact` 的事件 Vec。

### 前置条件

- Spike 实例已创建，位置 (100.0, 100.0)，碰撞体 AABB 为 (92.0, 96.0, 16.0, 8.0)
- Player 已创建在 Small 状态，位置 (100.0, 100.0)，碰撞体与 Spike AABB 重叠 8px (X 轴) + 4px (Y 轴)
- `kill_y = 2500.0`，player.pos().y (= 100.0) 远小于 kill_y

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Spike::new(Vec2 { x: 100.0, y: 100.0 })` | 返回 Spike 实例，`collider()` 返回 AABB (92.0, 96.0, 16.0, 8.0) |
| 2 | 验证 `player.collider().intersects(&spike_aabb)` | 返回 `true`（重叠确认） |
| 3 | `Physics::hazard_check(&player, &terrain, 2500.0)`，terrain = [Tile::Spike(spike_aabb)] | 返回 `[CollisionEvent::HazardContact]`，长度为 1 |

### 验证点

- `contains_hazard_contact(&events)` 为 `true`
- `events.len() == 1`
- 无 PitFall 事件（Y << kill_y）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t01_fun_happy_spike_contact_emits_hazard_contact
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-002

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-2: 安全越过尖刺

### 测试目标

验证玩家不与尖刺碰撞体重叠时（安全越过），`Physics::hazard_check` 返回空 Vec。

### 前置条件

- Spike 位置 (100.0, 100.0)，Player 位置 (200.0, 50.0)，两者 AABB 无重叠
- `kill_y = 2500.0`，player.pos().y (= 50.0) 远小于 kill_y

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 `player.collider().intersects(&spike_aabb)` | 返回 `false`（无重叠） |
| 2 | `Physics::hazard_check(&player, &terrain, 2500.0)`，terrain = [Tile::Spike(spike_aabb)] | 返回空 Vec `[]` |

### 验证点

- `events.is_empty()` 为 `true`
- 无 HazardContact（未重叠）
- 无 PitFall（Y << kill_y）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t02_fun_happy_safe_passage_returns_empty
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-003

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-3: 坠入深渊即死

### 测试目标

验证玩家 Y 坐标超过 kill_y 时，`Physics::hazard_check` 返回包含 `PitFall` 的事件 Vec。

### 前置条件

- Player 位置 (500.0, 2600.0)，`player.pos().y (= 2600.0) > kill_y (= 2500.0)`
- terrain 为空（无 Spike），仅检测坠落

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[], 2500.0)` | 返回 `[CollisionEvent::PitFall]`，长度为 1 |

### 验证点

- `contains_pit_fall(&events)` 为 `true`
- `events.len() == 1`
- 无 HazardContact（空 terrain）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t03_fun_happy_pit_fall_triggers_pit_fall
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-004

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-4: 无敌豁免

### 测试目标

验证 F05 在检测到危险时**不**因玩家无敌状态而抑制事件 — 无敌豁免是 F06 的职责。`Physics::hazard_check` 应无条件返回检测到的事件。

### 前置条件

- Player 与 Spike AABB 重叠确认
- F05 无权访问无敌状态（设计决策：invulnerability check 归 F06）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 确认 `player.collider().intersects(&spike_aabb)` | 返回 `true` |
| 2 | `Physics::hazard_check(&player, &[Tile::Spike(spike_aabb)], 2500.0)` | 返回 `[CollisionEvent::HazardContact]` — 事件**未被抑制** |

### 验证点

- `contains_hazard_contact(&events)` 为 `true`
- F05 不感知无敌状态，仅负责检测并报告

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t04_fun_happy_invulnerability_does_not_suppress_hazard_contact
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-005

### 关联需求

FR-009（Hazards — Spikes and Pits）— 多重危险并发

### 测试目标

验证同时存在多个 Spike 重叠 + PitFall 条件时，所有事件均被发射且顺序正确（Spike 检查先于 Pit）。

### 前置条件

- Player 在 (100.0, 2600.0)，2 个 Spike 均与 player AABB 重叠
- `player.pos().y (= 2600.0) > kill_y (= 2500.0)`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证两个 Spike 均与 player 重叠 | `intersects()` 两次均为 `true` |
| 2 | 验证 `player.pos().y > DEFAULT_KILL_Y` | `true` |
| 3 | `Physics::hazard_check(&player, &terrain, 2500.0)` | 返回 3 个事件：`[HazardContact, HazardContact, PitFall]` |

### 验证点

- `events.len() == 3`
- `count_hazard_contacts(&events) == 2`
- `count_pit_falls(&events) == 1`
- `events[0]` 和 `events[1]` 均为 `HazardContact`（spike 检查先于 pit）
- `events[2]` 为 `PitFall`（pit 检查在 spike 循环之后）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t05_fun_error_concurrent_spike_and_pit_all_events_emitted
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-006

### 关联需求

FR-009（Hazards — Spikes and Pits）— 空 terrain 安全处理

### 测试目标

验证 terrain 为空切片时，`hazard_check` 不 panic 并返回空 Vec。

### 前置条件

- Player 位置 (100.0, 100.0)，Y << kill_y
- terrain = `[]`（空切片）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[], 2500.0)` | 返回空 Vec `[]`，无 panic |

### 验证点

- `events.is_empty()` 为 `true`
- 无 panic（执行到达断言处即验证成功）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t06_fun_error_empty_terrain_returns_empty_no_panic
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-007

### 关联需求

FR-009（Hazards — Spikes and Pits）— Level::query_terrain 返回 Spike tiles

### 测试目标

验证 `Level::query_terrain()`（MODIFIED）在 Spike 区域查询时返回 `Tile::Spike(AABB)` 变体。

### 前置条件

- `Level::new()` 成功构造
- Spike 位置在 Level 构造中硬编码，查询 AABB 覆盖地面区域 (0, 550, 2000, 200)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` | 返回 Level 实例 |
| 2 | `level.query_terrain(&query_aabb)`，query_aabb = AABB{x:0, y:550, w:2000, h:200} | 返回 Vec 中包含至少 1 个 `Tile::Spike(aabb)` |
| 3 | 检查每个 `Tile::Spike` 变体的 AABB 维度 | `aabb.w == 16.0`，`aabb.h == 8.0`，w > 0 且 h > 0 |

### 验证点

- `spike_count > 0` — 至少返回 1 个 Spike tile
- 每个 Spike AABB 的宽高为 16x8 像素
- 无 `Tile::Spike` 被错误映射为其他变体

### 后置检查

- 无（Level 实例在测试结束时释放）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t12_intg_level_query_terrain_returns_spike_tiles
- **Test Type**: Real

---

### 用例编号

ST-FUNC-005-008

### 关联需求

FR-009（Hazards — Spikes and Pits）— 端到端集成流水线

### 测试目标

验证端到端集成：Spike 位于 Level 中 → `query_terrain` 返回 Spike tile → `hazard_check` 匹配并发射 `HazardContact`。

### 前置条件

- `Level::new()` 成功构造含 Spike 的关卡
- Player 放置位置与 Level 中 Spike 重叠

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` → `level.query_terrain()` | terrain 中包含 `Tile::Spike` 变体 |
| 2 | 找到第一个 Spike AABB，将 Player 放置于其中心位置 | Player 与 Spike 重叠 |
| 3 | `Physics::hazard_check(&player, &terrain, level.bounds().kill_y)` | 返回包含 `HazardContact` 的事件 |

### 验证点

- terrain 包含至少 1 个 `Tile::Spike`（集成前提）
- `contains_hazard_contact(&events)` 为 `true`
- 完整流水线：Level → query_terrain → hazard_check 正确运行

### 后置检查

- 无（Level 实例在测试结束时释放）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t13_intg_physics_end_to_end_spike_in_level_emits_hazard_contact
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-001

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-3: Y == kill_y 边界

### 测试目标

验证 `player.pos().y == kill_y` 恰好相等时**不**触发 PitFall（使用严格 `>` 比较）。

### 前置条件

- Player 位置 (500.0, 2500.0)，`player.pos().y == kill_y == 2500.0`
- terrain 为空

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[], 2500.0)`，其中 player.y == kill_y | 返回空 Vec `[]` |

### 验证点

- `events.is_empty()` 为 `true`
- PitFall **不**被发射（`==` 不满足 FR-009 "超过" 的严格 `>` 语义）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t07_bndry_edge_y_equals_kill_y_returns_empty
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-002

### 关联需求

FR-009（Hazards — Spikes and Pits）— 边缘接触（切线）重叠

### 测试目标

验证边界接触（玩家 AABB 右边缘 == Spike AABB 左边缘）按包容性边界语义计为重叠，触发 HazardContact。

### 前置条件

- Spike 位置 (100.0, 100.0)，AABB 右边缘 at x = 108.0
- Player 左边缘恰好 = spike 右边缘（108.0）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 `approx_eq(player.collider().x, spike_aabb.x + spike_aabb.w)` | `true`（切线接触） |
| 2 | `Physics::hazard_check(&player, &[Tile::Spike(spike_aabb)], 2500.0)` | 返回 `[CollisionEvent::HazardContact]` |

### 验证点

- `contains_hazard_contact(&events)` 为 `true` — 包容性边界的边缘接触被正确识别
- AABB::intersects 使用包容性语义 (`<=`) 而非严格 `<`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t08_bndry_edge_edge_contact_counts_as_hazard_contact
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-003

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-3: epsilon 超越 kill_y

### 测试目标

验证 `player.pos().y = kill_y + 0.001`（微小正增量）触发 PitFall。

### 前置条件

- Player 位置 (500.0, 2500.0 + 0.001)，Y 微超 kill_y
- terrain 为空

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[], 2500.0)` | 返回 `[CollisionEvent::PitFall]` |

### 验证点

- `contains_pit_fall(&events)` 为 `true`
- 任何正增量 (> 0.0) 触发 PitFall
- 无浮点精度导致误判

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t09_bndry_edge_y_just_beyond_kill_y_triggers_pit_fall
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-004

### 关联需求

FR-009（Hazards — Spikes and Pits）— 可变 Spike 数量（0/1/N）

### 测试目标

验证 0 个、1 个、5 个 Spike 时 `hazard_check` 返回正确数量的事件，无 panic。

### 前置条件

- kill_y = 10000.0（远高于玩家，排除 PitFall 干扰）
- 三种场景：terrain 含 0 个 / 1 个 / 5 个 Spike，均与玩家重叠

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `hazard_check` with 0 spikes terrain `[]` | 返回空 Vec，0 个事件 |
| 2 | `hazard_check` with 1 overlapping spike | 返回 1 个 HazardContact |
| 3 | `hazard_check` with 5 overlapping spikes | 返回 5 个 HazardContact |

### 验证点

- 0 spikes → `events.is_empty()`
- 1 spike → `events.len() == 1` 且 `contains_hazard_contact`
- 5 spikes → `events.len() == 5` 且 `count_hazard_contacts == 5`
- 无 panic，无固定大小缓冲区溢出

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t10_bndry_batch_variable_spike_count
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-005

### 关联需求

FR-009（Hazards — Spikes and Pits）— 仅有 Platform 无 Spike

### 测试目标

验证 terrain 仅含 `Tile::Platform` 变体时，`hazard_check` 返回空 Vec（无假阳性）。

### 前置条件

- Player 位置 (100.0, 500.0)
- terrain = `[Tile::Platform(AABB { x:0, y:600, w:2000, h:40 })]`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[Tile::Platform(...)], 2500.0)` | 返回空 Vec `[]` |

### 验证点

- `events.is_empty()` 为 `true`
- `Tile::Platform` 不被误识别为 `Tile::Spike`
- 匹配分支仅触发于 `Tile::Spike` 变体

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t11_bndry_null_platform_only_no_hazard_events
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-005-006

### 关联需求

FR-009（Hazards — Spikes and Pits）— AC-3: kill_y 上方安全

### 测试目标

验证玩家 Y 坐标刚好在 kill_y 上方 (kill_y - 1.0) 时，不触发 PitFall。

### 前置条件

- Player 位置 (500.0, 2499.0)，`player.pos().y (= 2499.0) < kill_y (= 2500.0)`
- terrain 为空

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::hazard_check(&player, &[], 2500.0)` | 返回空 Vec 或不含 PitFall 的 Vec |

### 验证点

- `!contains_pit_fall(&events)` 为 `true`
- 靠近 kill_y 但在其上方时安全

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/hazards_test.rs::t14_bndry_edge_y_just_above_kill_y_is_safe
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-005-001 | FR-009 AC-1 (spike contact death) | feature-list FR-009 AC[0] | t01_fun_happy_spike_contact_emits_hazard_contact | Real | PASS |
| ST-FUNC-005-002 | FR-009 AC-2 (safe dodge) | feature-list FR-009 AC[1] | t02_fun_happy_safe_passage_returns_empty | Real | PASS |
| ST-FUNC-005-003 | FR-009 AC-3 (pit fall) | feature-list FR-009 AC[2] | t03_fun_happy_pit_fall_triggers_pit_fall | Real | PASS |
| ST-FUNC-005-004 | FR-009 AC-4 (invuln exemption) | feature-list FR-009 AC[3] | t04_fun_happy_invulnerability_does_not_suppress_hazard_contact | Real | PASS |
| ST-FUNC-005-005 | FR-009 (concurrent hazards) | — | t05_fun_error_concurrent_spike_and_pit_all_events_emitted | Real | PASS |
| ST-FUNC-005-006 | FR-009 (empty terrain) | — | t06_fun_error_empty_terrain_returns_empty_no_panic | Real | PASS |
| ST-FUNC-005-007 | FR-009 (Spike in Level) | — | t12_intg_level_query_terrain_returns_spike_tiles | Real | PASS |
| ST-FUNC-005-008 | FR-009 (end-to-end) | — | t13_intg_physics_end_to_end_spike_in_level_emits_hazard_contact | Real | PASS |
| ST-BNDRY-005-001 | FR-009 AC-3 (Y == kill_y) | feature-list FR-009 AC[2] | t07_bndry_edge_y_equals_kill_y_returns_empty | Real | PASS |
| ST-BNDRY-005-002 | FR-009 (edge contact) | — | t08_bndry_edge_edge_contact_counts_as_hazard_contact | Real | PASS |
| ST-BNDRY-005-003 | FR-009 AC-3 (epsilon beyond) | feature-list FR-009 AC[2] | t09_bndry_edge_y_just_beyond_kill_y_triggers_pit_fall | Real | PASS |
| ST-BNDRY-005-004 | FR-009 (variable spike count) | — | t10_bndry_batch_variable_spike_count | Real | PASS |
| ST-BNDRY-005-005 | FR-009 (platform only) | — | t11_bndry_null_platform_only_no_hazard_events | Real | PASS |
| ST-BNDRY-005-006 | FR-009 AC-3 (just above kill_y) | feature-list FR-009 AC[2] | t14_bndry_edge_y_just_above_kill_y_is_safe | Real | PASS |

> SRS Trace 覆盖：4/4 FR-009 AC 已全部追溯 — 通过
> ATS 类别覆盖：FUNC (8 cases) >= 1 OK, BNDRY (6 cases) >= 1 OK

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 14 |
| Passed | 14 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` — must be fixed and re-executed.

## Manual Test Case Summary

> 本特性无手动测试用例（全部 14 条已自动化，`ui: false`）。
