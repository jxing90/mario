# 测试用例集: Player Controller

**Feature ID**: 3
**关联需求**: FR-001, FR-002, FR-003
**日期**: 2026-06-01
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

> Specification resolutions applied from Feature Design Clarification Addendum (6 items: SRS-VAGUE FR-001 AC-4, NFR-GAP FR-002 AC-1/FR-003 AC-2, PlayerConfig defaults, InputState location, coins/lives pub access).

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 21 |
| boundary | 10 |
| ui | 0 |
| security | 0 |
| performance | 0 |
| **合计** | **31** |

> UI: N/A -- `"ui": false`, 本特性为纯逻辑/物理模块，无图形界面。
> SEC: N/A -- 独立桌面游戏，无用户认证、外部输入验证或安全威胁面。
> PERF: N/A -- 无性能指标需求；60fps 性能由 Feature #1 覆盖。

---

### 用例编号

ST-FUNC-003-001

### 关联需求

FR-001（Player Horizontal Movement）— AC-1: 加速至最大速度

### 测试目标

验证玩家从静止状态按住右键后，在 0.3s 内加速至配置的最大向右速度并保持匀速。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 玩家初始速度为零（`vel.x = 0.0`）
- 使用默认 `PlayerConfig`（`max_speed = 200.0` px/s）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(PlayerConfig::default())`，设置 `on_ground = true` | 玩家创建成功，`pos = (100.0, 100.0)`, `vel = (0, 0)` |
| 2 | 构造 `InputState { right: true, ..default() }`，持续 0.3s（18帧 @ 60fps）逐帧调用 `player.update(dt, &input, &[])` | `player.vel.x` 逐步增长至 `config.max_speed`（200.0 px/s），容差 epsilon=1.0 |
| 3 | 继续 5 帧保持 `input.right = true` | `player.vel.x` 稳定在 `200.0`，不继续增长（速度钳制正确） |

### 验证点

- 0.3s 后 `vel.x ≈ 200.0` px/s（容差 1.0）
- 达到最大速度后不再增长（速度钳制生效）
- `pos.x` 随速度正向增加（实际位移与速度一致）

### 后置检查

- 无 panic；所有物理量为有限浮点数（非 NaN/Inf）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t01_accelerate_to_max_speed_in_0_3_seconds`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-002

### 关联需求

FR-001（Player Horizontal Movement）— AC-2: 摩擦减速至静止

### 测试目标

验证玩家释放所有水平方向输入后，在 0.2s 内因摩擦力减速至完全静止。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 玩家以最大速度移动（`vel.x = config.max_speed = 200.0`）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`vel.x = 200.0`, `on_ground = true` | 玩家向右移动中 |
| 2 | 构造 `InputState::default()`（无任何方向输入），持续 0.2s（12帧 @ 60fps）逐帧调用 `player.update(dt, &input, &[])` | `player.vel.x` 逐步减小至 0.0，容差 epsilon=1.0 |
| 3 | 继续 3 帧保持无输入 | `player.vel.x` 维持在 0.0，不反向移动（无负值） |

### 验证点

- 0.2s 后 `vel.x ≈ 0.0`（容差 1.0）
- 减速过程中 `vel.x` 始终 >= 0（不跨越零变为负值，不在零点振荡）
- 最终静止后 `pos.x` 不再变化

### 后置检查

- 无 panic；`vel.x` 精度正常（无浮点误差累积导致漂移）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t02_friction_decelerates_to_zero_in_0_2_seconds`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-003

### 关联需求

FR-001（Player Horizontal Movement）— AC-3: 反向加速

### 测试目标

验证玩家从最大向右速度按住左键后，在 0.3s 内从向右最大速度反向加速至向左最大速度。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 玩家初始速度 `vel.x = +200.0`（最大向右速度）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`vel.x = 200.0`, `on_ground = true` | 玩家向右高速移动中 |
| 2 | 构造 `InputState { left: true, ..default() }`，持续 0.3s（18帧）逐帧调用 `player.update(dt, &input, &[])` | `player.vel.x` 从 `+200.0` 平滑过渡至 `-200.0`（先减速至零，再反向加速） |
| 3 | 继续 5 帧保持 `input.left = true` | `player.vel.x` 稳定在 `-200.0`（向左最大速度钳制正确） |

### 验证点

- 0.3s 后 `vel.x ≈ -200.0`（容差 1.0）
- 速度变化是连续的（不跳变，先减速后反向加速）
- 反向加速时间与正向加速时间一致（~0.3s）

### 后置检查

- 无 panic；`facing` 应在加速过程中从 1 更新为 -1

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t03_reverse_acceleration_from_max_right_to_max_left`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-004

### 关联需求

FR-001（Player Horizontal Movement）— AC-4: 双键冲突处理

### 测试目标

验证同时按住左右方向键时，系统按"最后按下方向优先"策略处理；同时按下时减速至静止。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 使用默认 `PlayerConfig`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`on_ground = true` | 玩家创建成功 |
| 2 | 前一帧 `left=false, right=false`，当前帧 `left=true, right=true`（左键刚按下，右键保持按下） | `player` 获得向左加速度（最后按下的左键优先） |
| 3 | 前一帧 `left=false, right=false`，当前帧 `left=true, right=true`（两键同时从 false→true） | 净加速度 = 0（两键同时变化），`vel.x` 向 0 摩擦减速 |

### 验证点

- 有明确最后按下方向时，优先该方向
- 两键同时按下时，不偏向任何方向，减速至静止
- 无 panic 或 NaN

### 后置检查

- 状态机无振荡（双键持续按下时 `facing` 不每帧翻转）

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t04a_dual_key_last_pressed_left_priority`, `tests/player_controller_test.rs::t04b_dual_key_both_simultaneous_no_movement`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-005

### 关联需求

FR-001（Player Horizontal Movement）— AC-5: 空中水平控制

### 测试目标

验证玩家在空中时水平加速度为地面加速度的 60%。

### 前置条件

- 玩家处于空中（`on_ground = false`）
- 使用默认 `PlayerConfig`（`air_control_factor = 0.6`）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`on_ground = false`, `vel.x = 0.0` | 玩家在空中静止 |
| 2 | `input.right = true`，执行 1 帧 `player.update(dt, &input, &[])` | `vel.x` 增量 = `acceleration * air_control_factor * dt`（即地面加速度的 60%） |
| 3 | 对照：相同 `input` 但 `on_ground = true` 时执行 1 帧 | 地面 `vel.x` 增量 > 空中 `vel.x` 增量（比值 ≈ 0.6） |

### 验证点

- 空中水平加速度 = 地面加速度 * 0.6
- `air_control_factor = 0.6` 时空中转向灵敏度明显低于地面

### 后置检查

- 无 panic

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t05_air_control_is_60_percent_of_ground_acceleration`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-006

### 关联需求

FR-002（Player Jump）— AC-1: 短按跳跃

### 测试目标

验证轻按空格键（<100ms）时玩家上升至最大跳跃高度的约 40% 后开始下落。

### 前置条件

- 玩家站立在平台上（`on_ground = true`）
- 使用默认 `PlayerConfig`（`jump_initial_velocity = -420.0`, `gravity = 1200.0`）
- 基于 Clarification Addendum #2: 短跳高度容差为中心值的 35-45%

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `input.jump_just = true`，触发起跳 | `vel.y = -420.0`（向上），`jump_timer = 0.0`, `jump_held = true`, `on_ground = false` |
| 2 | 在 `jump_timer < 0.1s` 时设置 `input.jump = false`（模拟 < 100ms 释放） | sustain 力停止，`jump_held = false` |
| 3 | 等待玩家到达最高点（`vel.y` 从负变正） | 达到的最大高度 ≈ 最大跳跃高度的 35-45% |

### 验证点

- 短跳高度 ≈ 最大高度的 35-45%（容差范围内）
- 初始脉冲 `jump_initial_velocity` 提供的上升高度 ≈ `jump_initial_velocity^2 / (2 * gravity)` ≈ 40% 最大高度

### 后置检查

- 下落阶段 `vel.y` 持续受重力加速
- 无空中再次起跳

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t06_short_jump_reaches_approximately_40_percent_of_max_height`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-007

### 关联需求

FR-002（Player Jump）— AC-2: 长按最大跳跃

### 测试目标

验证按住空格不放时玩家上升至最大跳跃高度，并在 `max_jump_duration` 耗尽后停止 sustain 力。

### 前置条件

- 玩家站立在平台上（`on_ground = true`）
- `max_jump_duration = 0.35s`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `input.jump_just = true, input.jump = true`，触发起跳 | `vel.y = -420.0`, `jump_timer = 0.0`, `jump_held = true` |
| 2 | 持续 `input.jump = true`，每帧 `jump_timer += dt`，直到 `jump_timer == max_jump_duration`（0.35s, 21帧） | 持续施加 sustain 力，`vel.y` 保持负值（向上）更久 |
| 3 | 当 `jump_timer >= max_jump_duration` 时 | `jump_held = false`，sustain 力停止，重力开始主导 |

### 验证点

- 最大跳跃高度 > 短跳高度（短跳约 40%）
- `jump_timer` 到达 `max_jump_duration` 后不再增长
- 到达最大高度时 `vel.y == 0`（顶点瞬间悬停），随后 `vel.y > 0`（开始下落）

### 后置检查

- `jump_timer` 不溢出（不超过 `max_jump_duration`）；无 panic

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t07_max_jump_reaches_full_height_with_sustain`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-008

### 关联需求

FR-002（Player Jump）— AC-3: 禁止空中二段跳

### 测试目标

验证玩家在空中时按下空格键不会再次起跳。

### 前置条件

- 玩家处于空中（`on_ground = false`）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`on_ground = false`, `vel.y = 100.0`（下落中） | 玩家在空中 |
| 2 | `input.jump_just = true, input.jump = true` | `vel.y` 不因 `jump_just` 变化；`player` 忽略输入，继续下落 |
| 3 | 检查 `jump_timer` 和 `jump_held` | 保持原有值不变（不触发新跳跃状态） |

### 验证点

- 空中 `jump_just` 被完全忽略（无 `vel.y` 突然变负）
- `on_ground` 守卫正确工作（仅地面可起跳）
- 无跳跃状态机误触发

### 后置检查

- 无 panic；一帧内 `jump_just` 残留不会误触发下一次起跳

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t08_no_double_jump_when_airborne`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-009

### 关联需求

FR-002（Player Jump）— AC-4: 天花板碰撞

### 测试目标

验证玩家在跳跃过程中头顶撞击天花板时垂直速度立即归零并开始下落。

### 前置条件

- 玩家正在向上移动（`vel.y < 0`）
- 头顶存在平台 tile（天花板）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`vel.y = -300.0`（向上），terrain 包含头顶 `Tile::Platform(AABB)` 位于玩家头顶 1px 处 | 玩家接近天花板 |
| 2 | 执行 `player.update(dt, &input, &terrain)` 进行碰撞解析 | 碰撞检测命中天花板 → `vel.y = 0.0`（垂直速度归零，停止上升） |
| 3 | 检查碰撞后状态 | `player.pos.y` 被修正至平台底部下方（不穿入平台）；`on_ground = false`（仅头碰顶，非着陆） |

### 验证点

- 天花板碰撞后 `vel.y = 0.0`（向上运动立即停止）
- `pos.y` 正确修正（不穿入天花板）
- `on_ground` 保持 `false`（不误判为着陆）
- 碰撞后重力重新施加，玩家开始下落

### 后置检查

- 无 panic；未产生位置穿透

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t09_ceiling_collision_stops_upward_motion`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-010

### 关联需求

FR-002（Player Jump）— AC-5: 从平台边缘走出

### 测试目标

验证玩家从平台边缘走出后失去支撑、重力立即作用、`on_ground` 变为 false。

### 前置条件

- 玩家站立在平台上（`on_ground = true`）
- 地形查询结果为玩家下方无支撑平台

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`on_ground = true`，terrain 为空切片 `&[]`（无平台支撑） | 玩家下方无任何地形 |
| 2 | 执行 1 帧 `player.update(dt, &InputState::default(), &[])` | `player.on_ground` 变为 `false`；`player.vel.y` 增加 `gravity * dt`（开始下落） |
| 3 | 再执行 10 帧更新，持续无平台支撑 | `vel.y` 持续受重力加速增加；玩家持续下落 |

### 验证点

- `on_ground` 正确从 `true` 变为 `false`
- 重力立即施加，无延迟帧
- 连续多帧下落后 `vel.y` 持续增加

### 后置检查

- 无 panic；`on_ground` 状态转换正确（无悬空 `on_ground=true`）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t10_walk_off_ledge_applies_gravity`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-011

### 关联需求

FR-003（Sprint）— AC-1: 冲刺速度 1.5x

### 测试目标

验证按住 Shift 冲刺时，玩家最大水平速度提升至基础速度的 1.5 倍。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- `sprint_multiplier = 1.5`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`on_ground = true` | 玩家在地面 |
| 2 | `input.right = true, input.sprint = true`，持续更新至达到最大速度 | `player.vel.x` 钳制于 `config.max_speed * 1.5 = 300.0` px/s |
| 3 | 释放 `input.sprint`（保持 `input.right = true`） | `vel.x` 从 300.0 降至 200.0（最大速度恢复正常） |

### 验证点

- 冲刺时 `max_speed` 乘 1.5
- 释放 Shift 后速度上限恢复为 1.0x
- 冲刺仅影响速度上限，不影响加速度值

### 后置检查

- 无 panic

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t11_sprint_increases_max_speed_by_1_5x`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-012

### 关联需求

FR-003（Sprint）— AC-2: 冲刺跳跃距离增加 50%

### 测试目标

验证冲刺中起跳时水平位移约为不冲刺的 1.5 倍。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 基于 Clarification Addendum #3: 容差范围 1.4-1.6 倍

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player::new(default_config)`，`on_ground = true`, `input.right = true, input.sprint = true, input.jump_just = true` | 冲刺跳跃触发 |
| 2 | 从起跳至落地，记录水平位移 | 水平位移 ≈ 不冲刺跳跃的 1.4-1.6 倍 |
| 3 | 对照：相同条件但不冲刺（`input.sprint = false`） | 冲刺跳跃距离 / 普通跳跃距离 ≈ 1.5x |

### 验证点

- 空中水平速度保持 `max_speed * sprint_multiplier`
- 跳跃距离成比例增加（因空中时间相同）
- 浮点积分误差不导致比例偏离容差范围

### 后置检查

- 冲刺速度加成在跳跃全程保持（不因进入空中而重置）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t12_sprint_jump_has_approximately_1_5x_horizontal_distance`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-013

### 关联需求

FR-003（Sprint）— AC-3: 冲刺不影响摩擦力

### 测试目标

验证释放方向键后，冲刺状态下的减速行为与不冲刺完全一致。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- 玩家以冲刺最大速度移动（`vel.x = 300.0`）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`vel.x = 300.0`（冲刺最大速度），`on_ground = true` | 玩家高速移动中 |
| 2 | 释放所有水平输入（`left=false, right=false`） | 玩家减速至 0，减速时间 ≈ 0.2s（与非冲刺一致） |
| 3 | 对比 `friction` 参数值 | 不受 `sprint` 状态修改 |

### 验证点

- 减速至 0 用时 ≈ 0.2s（冲刺不改变 `friction` 参数）
- 释放 sprint 键时速度不瞬间跳回（平滑过渡到较低的最大速度）
- `sprint` 仅修改 `effective_max_speed`，不影响摩擦力计算

### 后置检查

- 无 panic

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t13_sprint_release_friction_same_as_normal`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-014

### 关联需求

FR-001, FR-002（IAPI-007 提供方）

### 测试目标

验证 `Player::pos()` 返回当前世界坐标（纯 getter，无副作用）。

### 前置条件

- 玩家已构造并位于特定坐标

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player` at `(150.0, 300.0)` | 玩家创建成功 |
| 2 | 调用 `player.pos()` | 返回 `Vec2 { x: 150.0, y: 300.0 }` |
| 3 | 再次调用 `player.pos()` | 连续调用返回值一致（纯 getter） |

### 验证点

- `pos()` 返回值与内部坐标精确一致
- 无副作用（不修改内部状态）
- 返回值类型为 `Vec2`

### 后置检查

- 返回值引用的字段在多次调用间无变化（除非 `update()` 执行）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t23_pos_returns_current_world_coordinates`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-015

### 关联需求

FR-001, FR-002（IAPI-009 提供方）

### 测试目标

验证 `Player::stats()` 返回当前 `PlayerStats { coins, lives }`（纯 getter）。

### 前置条件

- 玩家已构造，`coins` 和 `lives` 已设置

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，设置 `coins = 5`, `lives = 2` | 内部字段已更新 |
| 2 | 调用 `player.stats()` | 返回 `PlayerStats { coins: 5, lives: 2 }` |
| 3 | 修改内部值后再次调用 | 返回值反映最新值（非 stale） |

### 验证点

- 返回值与内部字段一致
- 无副作用（纯 getter）
- 不产生不必要的堆分配

### 后置检查

- `stats()` 读取字段后返回值的生命周期独立于 `&self`

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t24_stats_returns_current_player_stats`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-016

### 关联需求

FR-001, FR-002（Interface Contract: `Player::new`）

### 测试目标

验证 `Player::new` 使用无效配置参数（`max_speed = 0.0`）时构造成功且不 panic。

### 前置条件

- 无

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `PlayerConfig { max_speed: 0.0, ..default() }` | 配置创建成功 |
| 2 | `Player::new(config)` | 构造成功，不 panic |
| 3 | 尝试 `update()` 并水平移动 | `vel.x` 始终钳制于 0.0（玩家无法水平移动），行为定义明确 |

### 验证点

- `max_speed = 0` 不会导致除零错误
- 构造成功返回有效 `Player` 实例
- 后续物理计算不 panic

### 后置检查

- 所有参数为负值时的行为定义

### 元数据

- **优先级**: Low
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t14_invalid_config_max_speed_zero_does_not_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-017

### 关联需求

Interface Contract: `Player::apply_powerup`

### 测试目标

验证 `apply_powerup(Super)` 从 Small 状态升级为 Super 状态，碰撞体尺寸正确更新且脚底对齐。

### 前置条件

- 玩家处于 `PlayerState::Small`
- 碰撞体为 16x16 (脚底对齐)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `player.state = Small`，获取 `collider()` | 返回 AABB { w=16, h=16 } |
| 2 | `player.apply_powerup(PlayerState::Super)` | `player.state == Super` |
| 3 | 再次获取 `collider()` | 返回 AABB { w=16, h=32 }（宽度不变，高度加倍），`pos.y` 不变（脚底对齐） |

### 验证点

- `state` 正确更新为 `Super`
- 碰撞体高度从 16 变为 32
- 脚底世界坐标不变（`pos.y` 未因碰撞体扩展而移动）

### 后置检查

- 无 panic

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t25_apply_powerup_small_to_super`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-018

### 关联需求

Interface Contract: `Player::take_damage` (Super -> Small)

### 测试目标

验证 Super 状态受到伤害后降级为 Small 且不触发死亡。

### 前置条件

- 玩家处于 `PlayerState::Super`
- 碰撞体为 16x32

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `player.state = Super` | 玩家处于 Super 状态 |
| 2 | 调用 `player.take_damage()` | 返回 `false`（未死亡） |
| 3 | 检查 `player.state` 和 `collider()` | `state == Small`；`collider()` 返回 16x16 AABB（碰撞体尺寸正确缩回） |

### 验证点

- `take_damage()` 返回值 `false`（不触发死亡流程）
- `state` 正确降级为 `Small`
- 碰撞体从 16x32 缩回 16x16（脚底对齐）

### 后置检查

- 降级后不影响后续水平移动、跳跃等操作

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t26_take_damage_super_downgrades_to_small`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-019

### 关联需求

Interface Contract: `Player::take_damage` (Small -> Death)

### 测试目标

验证 Small 状态受到伤害后返回 `true`（应触发死亡流程）。

### 前置条件

- 玩家处于 `PlayerState::Small`
- 玩家非无敌（无敌由 Feature #6 管理，不在本特性范围）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `player.state = Small` | 玩家处于 Small 状态 |
| 2 | 调用 `player.take_damage()` | 返回 `true`（应触发死亡） |
| 3 | 检查 `player.state` | 保持 `Small`（不 panic，由外部调用方处理死亡流程） |

### 验证点

- `take_damage()` 对 Small 状态返回 `true`
- 不 panic（如 unwrap 空值）
- `player.state` 仍为 `Small`（状态不变，死亡处理由 Feature #6 负责）

### 后置检查

- 无副作用影响其他字段

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t27_take_damage_small_returns_death`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-020

### 关联需求

Interface Contract: `Player::update` 异常输入

### 测试目标

验证 `dt = 0.0`（零时间步长）时 `update()` 不 panic 且不修改状态。

### 前置条件

- 玩家已构造
- 使用正常 `InputState` 和 terrain

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 记录 `player.pos` 和 `player.vel` 初始值 | 基线值已记录 |
| 2 | 执行 `player.update(0.0, &input, &terrain)` | 不 panic |
| 3 | 比较更新后 `pos` 和 `vel` | 与更新前完全一致（无任何变化） |
| 4 | 检查 `on_ground` | 保持原值不变 |

### 验证点

- `dt = 0` 不导致除零、NaN 传播或 panic
- `pos`、`vel`、`on_ground` 均不变
- 所有物理计算在 `dt = 0` 时安全短路

### 后置检查

- 无 panic

### 元数据

- **优先级**: Low
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t28_zero_dt_does_not_panic_or_modify_state`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-003-021

### 关联需求

FR-001, FR-002, FR-006（INTG/terrain: Player + Level 集成）

### 测试目标

验证玩家在真实关卡平台上正确站立（集成 IAPI-005 地形查询 + Player::update 碰撞解析）。

### 前置条件

- 使用真实 `Level` 实例（Feature #2 提供）
- Playing 状态编排 `query_terrain → player.update` 数据流

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Level::new()`（真实关卡）和 `Player::new(default_config)` | 玩家初始位置在关卡有效区域内 |
| 2 | `terrain = level.query_terrain(player.collider())`，调用 `player.update(dt, &input, &terrain)` | 玩家与平台正确碰撞 |
| 3 | 验证碰撞结果 | `player.on_ground == true`（站在地面上）；`player.vel.y == 0`；玩家 `pos` 在平台顶面上方 |

### 验证点

- Player 正确站在 Level 提供的平台上
- `on_ground` 反映真实地形接触
- 地形 tile 格式匹配（`Tile::Platform` 被 Player 正确解析）
- collider() 的 AABB 坐标系统与 Level 世界坐标对齐

### 后置检查

- 无 panic；集成通路数据流完整

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t31_integration_player_stands_on_real_level_platform`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-001

### 关联需求

FR-001（Boundary: `vel.x` at max_speed）

### 测试目标

验证速度恰好达到最大速度时被正确钳制，不超出上限。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- `max_speed = 200.0`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`vel.x = 199.99`（接近最大速度） | 玩家接近速度上限 |
| 2 | `input.right = true`，执行 1 帧 `update` | `vel.x` 钳制于 `200.0`（不超出） |
| 3 | 检查钳制逻辑 | 使用 `<=` 比较而非 `<`，恰好等于 max 时不再加速 |

### 验证点

- 速度不超过 `max_speed`
- 加速到等于或超过 `max_speed` 时被正确钳制
- 浮点误差不导致速度缓慢漂移超出上限

### 后置检查

- 多帧后速度仍稳定在 `max_speed` 不漂移

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t15_speed_clamped_at_max_when_already_at_max`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-002

### 关联需求

FR-001（Boundary: `vel.x` 过零）

### 测试目标

验证极低速度下摩擦减速时 `vel.x` 恰好过零时被钳制为 0，不产生振荡。

### 前置条件

- 玩家站立在地面上（`on_ground = true`）
- `vel.x = 0.5`（极低速度）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`vel.x = 0.5` | 极低速移动中 |
| 2 | `input = InputState::default()`（无任何输入），执行摩擦减速 | `vel.x` 变为 0.0 |
| 3 | 继续执行几帧 | `vel.x` 稳定在 0.0（不跨越零变为负值） |

### 验证点

- 速度从正值减速至 0，不出现负值
- 过零后钳制为 0（不在零点附近振荡）
- 微小速度最终收敛至 0（不永远停在 0.0001 等微小值）

### 后置检查

- 无恒久微小速度漂移

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t16_velocity_zero_crossing_clamped_to_zero`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-003

### 关联需求

FR-002（Boundary: `jump_timer == max_jump_duration`）

### 测试目标

验证 `jump_timer` 精确到达 `max_jump_duration` 时 sustain 力停止施加，且 timer 不再增长。

### 前置条件

- 玩家在地面起跳（`on_ground = true`）
- `max_jump_duration = 0.35s`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `jump_just = true, jump = true` 触发起跳 | `jump_timer = 0.0` |
| 2 | 持续更新至 `jump_timer` 恰好等于 `max_jump_duration`（第 21 帧） | 当帧 sustain 力停止施加（使用 `>=` 而非 `>` 比较，避免多施加一帧） |
| 3 | 下一帧检查 `jump_timer` | 不再继续增长（停止在 `max_jump_duration`） |

### 验证点

- `jump_timer == max_jump_duration` 时精确停止 sustain
- 比较操作符正确（不使用 `>` 导致多施一帧）
- `jump_timer` 不溢出（停在 `max_jump_duration`）

### 后置检查

- 无 panic；`jump_held` 变为 `false`

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t17_jump_timer_at_max_duration_stops_sustain`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-004

### 关联需求

FR-002（Boundary: `on_ground` 着陆精确接触）

### 测试目标

验证玩家下落恰好接触地面平台时的碰撞解析精度：不移位、不穿透、on_ground 正确设置。

### 前置条件

- 玩家正在下落（`vel.y > 0`）
- 地面平台 AABB 恰好位于 `player.pos.y`（精确接触）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`vel.y = +200.0`（下落），terrain 包含地面 `Tile::Platform` 位于脚底位置 | 玩家即将着陆 |
| 2 | 执行 `player.update(dt, &input, &terrain)` 碰撞解析 | `vel.y = 0.0`；`on_ground = true`；`pos.y` 等于平台顶面 Y（精确修正） |
| 3 | 下一帧更新 | `on_ground` 保持 `true`（不误判为空中） |

### 验证点

- 浮点比较无穿透（`pos.y` 不超过平台顶面）
- 不嵌入平台（`pos.y` 不偏大）
- `on_ground` 正确设为 `true`
- 连续多帧 `on_ground = true` 稳定

### 后置检查

- 无浮点误差导致的 jitter（位置抖动）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t18_landing_on_platform_sets_on_ground_and_zeroes_vertical_velocity`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-005

### 关联需求

FR-001, FR-002（Boundary: terrain 空切片）

### 测试目标

验证无地形约束时玩家自由下落，不 panic。

### 前置条件

- 玩家在空中（`on_ground = false`）
- `terrain = &[]`（空地形切片）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Player`，`on_ground = false`，`terrain = &[]` | 玩家自由落体 |
| 2 | 执行多帧 `player.update(dt, &input, &[])` | `vel.y` 持续受重力增加；`on_ground` 保持 `false`；`pos` 持续下落 |
| 3 | 检查无 panic | 空切片遍历不导致索引越界或 unwrap panic |

### 验证点

- 空 terrain 安全处理（不 crash）
- `on_ground` 不被误设为 `true`
- 重力正常施加

### 后置检查

- 即使长时间空地形更新也不产生性能退化

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t19_empty_terrain_allows_continuous_falling`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-006

### 关联需求

Interface Contract（Boundary: `collider()` 尺寸切换）

### 测试目标

验证 Small 与 Super 状态下 `collider()` 返回正确的 AABB 尺寸，脚底始终对齐。

### 前置条件

- 玩家初始状态为 Small

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `player.state = Small`，获取 `collider()` | AABB { w=16, h=16 }，脚底 = `pos.y` |
| 2 | `player.state = Super`，获取 `collider()` | AABB { w=16, h=32 }，脚底 = `pos.y`（不变，脚底对齐） |
| 3 | 再次切换回 `Small` 验证 | AABB { w=16, h=16 }，脚底对齐 |

### 验证点

- Small: w=16, h=16
- Super/Fire: w=16, h=32
- `pos.y` 在状态切换前后不变（脚底不移动）
- 碰撞体向上扩展而非向下（避免脚底嵌入地面）

### 后置检查

- 无 panic；Fire 状态与 Super 碰撞体一致

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t20_collider_size_changes_with_powerup_state_foot_aligned`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-007

### 关联需求

FR-001（Boundary: `air_control_factor = 0.0`）

### 测试目标

验证 `air_control_factor = 0.0` 时在空中完全无法水平操控。

### 前置条件

- 玩家在空中（`on_ground = false`）
- `config.air_control_factor = 0.0`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `PlayerConfig { air_control_factor: 0.0, ..default() }` | 配置空中操控为 0 |
| 2 | `on_ground = false`，`input.right = true`，执行 `update` | 空中 `vel.x` 不变（无加速度/减速度） |
| 3 | 玩家沿弹道轨迹飞行 | 水平速度保持初始值，仅受重力改变垂直速度 |

### 验证点

- 空中水平加速度为 0（`acceleration * 0.0 = 0`）
- 玩家水平速度在起跳后保持不变（无空中操控）

### 后置检查

- 无 panic

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t21_air_control_factor_zero_prevents_air_steering`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-008

### 关联需求

FR-001（Boundary: `air_control_factor = 1.0`）

### 测试目标

验证 `air_control_factor = 1.0` 时空中加速度等于地面加速度（无惩罚）。

### 前置条件

- 玩家在空中（`on_ground = false`）
- `config.air_control_factor = 1.0`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `PlayerConfig { air_control_factor: 1.0, ..default() }` | 配置空中操控为 100% |
| 2 | `on_ground = false`，`input.right = true`，执行 `update` | 空中 `vel.x` 增量 = `ground_acceleration`（与 `on_ground=true` 完全一致） |
| 3 | 对照：相同条件 `on_ground = true` | 两者加速度值相等 |

### 验证点

- 空中加速度 = 地面加速度（`air_control_factor = 1.0` 不受缩减）
- 极值正确（factor 范围 [0.0, 1.0] 均可工作）

### 后置检查

- 无 panic

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t22_air_control_factor_one_equals_ground_acceleration`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-009

### 关联需求

FR-001, FR-002（Boundary: 多平台同时碰撞）

### 测试目标

验证玩家被夹在两个相邻平台之间时，两侧均正确阻挡，不穿透、不振荡。

### 前置条件

- 玩家位于两个相邻平台之间（水平方向恰好为玩家宽度）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 terrain 含两个相邻 `Tile::Platform` 分别位于玩家左右侧（间距 = 16px，玩家宽度） | 玩家在狭窄间隙中 |
| 2 | 执行 `player.update(dt, &input, &terrain)` 碰撞解析 | 玩家 `pos` 被夹在两个平台之间；`vel.x = 0`（两侧均受阻） |
| 3 | 多次更新验证 | 无振荡（不左推→右推→左推交替）；位置稳定 |

### 验证点

- 多平台碰撞全部处理（非仅处理第一个）
- 不穿透任一平台
- 不产生振荡（位置稳定）
- 两侧速度均钳制为 0

### 后置检查

- 无 panic；多帧后位置稳定（无 drift）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t29_multi_platform_collision_prevents_penetration`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-003-010

### 关联需求

FR-001（Boundary: `facing` 更新）

### 测试目标

验证 `facing` 随方向输入正确更新：左= -1, 右= 1, 无输入保持前值。

### 前置条件

- 玩家已构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `input.left = true, input.right = false`，执行 `update` | `player.facing == -1`（朝左） |
| 2 | `input.left = false, input.right = true`，执行 `update` | `player.facing == 1`（朝右） |
| 3 | `input.left = false, input.right = false`（无输入），执行 `update` | `player.facing` 保持前一帧的值（不重置为默认值） |

### 验证点

- 左键: `facing = -1`
- 右键: `facing = 1`
- 无输入: `facing` 保持前值不变
- 双键同时按下: `facing` 不振荡（仅更新到成功那方向）

### 后置检查

- `facing` 始终为 -1 或 1（无 0 或其他值）

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/player_controller_test.rs::t30_facing_updates_with_directional_input`
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-003-001 | FR-001 AC-1 | 加速至 max_speed 0.3s | t01_accelerate_to_max_speed_in_0_3_seconds | Real | PASS |
| ST-FUNC-003-002 | FR-001 AC-2 | 摩擦减速至静止 0.2s | t02_friction_decelerates_to_zero_in_0_2_seconds | Real | PASS |
| ST-FUNC-003-003 | FR-001 AC-3 | 反向加速 max→-max | t03_reverse_acceleration_from_max_right_to_max_left | Real | PASS |
| ST-FUNC-003-004 | FR-001 AC-4 | 双键冲突去抖 | t04a, t04b | Real | PASS |
| ST-FUNC-003-005 | FR-001 AC-5 | 空中操控 60% | t05_air_control_is_60_percent_of_ground_acceleration | Real | PASS |
| ST-FUNC-003-006 | FR-002 AC-1 | 短跳 ~40% 高度 | t06_short_jump | Real | PASS |
| ST-FUNC-003-007 | FR-002 AC-2 | 最大跳跃高度 | t07_max_jump_reaches_full_height_with_sustain | Real | PASS |
| ST-FUNC-003-008 | FR-002 AC-3 | 禁止二段跳 | t08_no_double_jump_when_airborne | Real | PASS |
| ST-FUNC-003-009 | FR-002 AC-4 | 天花板碰撞 | t09_ceiling_collision_stops_upward_motion | Real | PASS |
| ST-FUNC-003-010 | FR-002 AC-5 | 平台边缘走出坠落 | t10_walk_off_ledge_applies_gravity | Real | PASS |
| ST-FUNC-003-011 | FR-003 AC-1 | 冲刺 1.5x 速度 | t11_sprint_increases_max_speed_by_1_5x | Real | PASS |
| ST-FUNC-003-012 | FR-003 AC-2 | 冲刺跳跃距离 +50% | t12_sprint_jump_has_approximately_1_5x_horizontal_distance | Real | PASS |
| ST-FUNC-003-013 | FR-003 AC-3 | 冲刺摩擦不变 | t13_sprint_release_friction_same_as_normal | Real | PASS |
| ST-FUNC-003-014 | IAPI-007 | pos() getter | t23_pos_returns_current_world_coordinates | Real | PASS |
| ST-FUNC-003-015 | IAPI-009 | stats() getter | t24_stats_returns_current_player_stats | Real | PASS |
| ST-FUNC-003-016 | Contract `new` | 无效 config 不 panic | t14_invalid_config_max_speed_zero_does_not_panic | Real | PASS |
| ST-FUNC-003-017 | Contract `apply_powerup` | Small→Super 碰撞体更新 | t25_apply_powerup_small_to_super | Real | PASS |
| ST-FUNC-003-018 | Contract `take_damage` | Super→Small 降级 | t26_take_damage_super_downgrades_to_small | Real | PASS |
| ST-FUNC-003-019 | Contract `take_damage` | Small→death | t27_take_damage_small_returns_death | Real | PASS |
| ST-FUNC-003-020 | Contract `update` | dt=0 安全处理 | t28_zero_dt_does_not_panic_or_modify_state | Real | PASS |
| ST-FUNC-003-021 | FR-001,FR-002,FR-006 | INTG: 真实关卡站立 | t31_integration_player_stands_on_real_level_platform | Real | PASS |
| ST-BNDRY-003-001 | FR-001 | vel.x at max 钳制 | t15_speed_clamped_at_max_when_already_at_max | Real | PASS |
| ST-BNDRY-003-002 | FR-001 | vel.x 过零钳制 | t16_velocity_zero_crossing_clamped_to_zero | Real | PASS |
| ST-BNDRY-003-003 | FR-002 | jump_timer at max | t17_jump_timer_at_max_duration_stops_sustain | Real | PASS |
| ST-BNDRY-003-004 | FR-002 | 着陆精确接触 | t18_landing_on_platform_sets_on_ground_and_zeroes_vertical_velocity | Real | PASS |
| ST-BNDRY-003-005 | FR-001,FR-002 | 空地形自由下落 | t19_empty_terrain_allows_continuous_falling | Real | PASS |
| ST-BNDRY-003-006 | Contract `collider` | 碰撞体尺寸切换脚底对齐 | t20_collider_size_changes_with_powerup_state_foot_aligned | Real | PASS |
| ST-BNDRY-003-007 | FR-001 | air_control_factor=0 | t21_air_control_factor_zero_prevents_air_steering | Real | PASS |
| ST-BNDRY-003-008 | FR-001 | air_control_factor=1 | t22_air_control_factor_one_equals_ground_acceleration | Real | PASS |
| ST-BNDRY-003-009 | FR-001,FR-002 | 多平台同时碰撞 | t29_multi_platform_collision_prevents_penetration | Real | PASS |
| ST-BNDRY-003-010 | FR-001 | facing 方向更新 | t30_facing_updates_with_directional_input | Real | PASS |

> 结果 valid values: `PENDING`, `PASS`, `FAIL`, `MANUAL-PASS`, `MANUAL-FAIL`, `BLOCKED`, `PENDING-MANUAL`

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 31 |
| Passed | 31 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` -- must be fixed and re-executed.
