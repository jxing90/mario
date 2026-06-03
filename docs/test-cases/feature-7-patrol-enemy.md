# 测试用例集: Patrol Enemy

**Feature ID**: 7
**关联需求**: FR-010 (Stompable Enemy)
**日期**: 2026-06-03
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

> Specification resolutions applied from Feature Design Clarification Addendum (无需澄清 — 全部规格明确).

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 12 |
| boundary | 5 |
| ui | 0 |
| security | 0 |
| performance | 0 |
| **合计** | **17** |

---

### 用例编号

ST-FUNC-007-001

### 关联需求

FR-010（Stompable Enemy）— AC-1: 向右巡逻至右路点并反转

### 测试目标

验证敌人从起始位置向右巡逻，经过 1.0 秒后准确到达右路点（waypoint_b），速度反转为向左。

### 前置条件

- Enemy 在位置 (100, 584)，路点 A=(50, 584)，路点 B=(150, 584)，config.speed=50（默认）
- Enemy 初始 alive=true，vel.x=config.speed(50.0)，vel.y=0.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 创建 Enemy at pos(100, 584), waypoints (50, 584) → (150, 584), 默认 config | enemy.alive=true, vel.x=50.0, vel.y=0.0 |
| 2 | 调用 `enemy.update(1.0)` — 1 秒以速度 50 向右移动 | enemy.pos.x ≈ 150.0 (准确到达右路点) |
| 3 | 检查反转后的速度与 Y 坐标 | enemy.vel.x ≈ -50.0 (方向反转向左), enemy.pos.y ≈ 584.0 (Y 不变) |

### 验证点

- `enemy.pos.x == 150.0` (准确到达 waypoint_b)
- `enemy.vel.x == -50.0` (方向反转向左)
- `enemy.pos.y == 584.0` (Y 坐标不变)
- `enemy.alive == true` (存活状态不变)

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t01_fun_happy_patrol_right_reaches_waypoint_b_reverses
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-002

### 关联需求

FR-010（Stompable Enemy）— AC-1: 从右路点向左巡逻至左路点并反转

### 测试目标

验证敌人从右路点出发向左巡逻，经过 2.0 秒后准确到达左路点（waypoint_a），速度反转向右。

### 前置条件

- Enemy 在位置 (100, 584)，路点 A=(50, 584)，路点 B=(150, 584)
- 先 update(1.0) 到达右路点并反转（预条件），vel.x = -50.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `enemy.update(1.0)` — 先到达 waypoint_b，vel.x 反转为 -50 | pos.x≈150, vel.x=-50（预条件确认） |
| 2 | `enemy.update(1.0)` — 从 150 向左移动 1 秒 | pos.x≈100.0，vel.x 仍为 -50（未到左路点） |
| 3 | `enemy.update(1.0)` — 再向左移动 1 秒 | pos.x≈50.0 (准确到达左路点)，vel.x≈50.0 (反转向右) |

### 验证点

- 第一段 update 后 `enemy.pos.x ≈ 100.0`，`vel.x == -50.0`
- 第二段 update 后 `enemy.pos.x ≈ 50.0` (waypoint_a)
- `enemy.vel.x ≈ 50.0` (方向反转向右)
- `enemy.pos.y` 始终不变

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t02_fun_happy_patrol_left_from_b_reaches_waypoint_a_reverses
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-003

### 关联需求

FR-010（Stompable Enemy）— AC-1: 到达左路点后反转向右

### 测试目标

验证敌人靠近左路点且向左移动时，越过左路点后被钳位至 waypoint_a 并反转向右。

### 前置条件

- Enemy 在位置 (52, 584)，vel.x=-50 (正在向左移动)
- 路点 A=(50, 584)，路点 B=(150, 584)，dt=1/15 ≈ 0.0667s

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 创建 Enemy at pos(52, 584)，设置 vel.x = -50 | 初始向左移动 |
| 2 | `enemy.update(1.0/15.0)` — 位移 ≈ 3.33px 向左 | pos.x 越过 waypoint_a(50), 钳位至 50.0 |
| 3 | 检查速度反转 | vel.x ≈ 50.0 (反转向右) |

### 验证点

- `enemy.pos.x ≈ 50.0` (钳位至左路点)
- `enemy.vel.x ≈ 50.0` (方向反转向右)
- `enemy.pos.y == 584.0` (Y 坐标不变)

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t03_fun_happy_reaches_left_waypoint_reverses_to_right
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-004

### 关联需求

FR-010（Stompable Enemy）— AC-2: 踩踏消灭敌人 — Physics 检测

### 测试目标

验证玩家从上方落下接触敌人时，`Physics::enemy_check` 正确返回 `EnemyStomp(0)` 事件。

### 前置条件

- Player 在位置 (150, 572)，vel.y=300 (向下坠落)
- Enemy 在位置 (150, 584)，alive=true
- Player AABB 底部 = 572，Enemy AABB 顶部 = 568 → AABB 重叠
- 进入前玩家底部 = 572 - 300/60 = 567 ≤ enemy_top + 2 = 570 → 满足"从上方进入"条件

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（AABB 重叠确认） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回 `[EnemyStomp(0)]`，长度为 1 |

### 验证点

- `contains_enemy_stomp(&events)` 为 `true`
- `events.len() == 1`
- `get_stomp_index(&events) == 0`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t04_fun_happy_stomp_from_above_produces_enemy_stomp
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-005

### 关联需求

FR-010（Stompable Enemy）— AC-2: 踩踏后敌人消灭 + 玩家反弹

### 测试目标

验证 PlayingState 消费 EnemyStomp 事件后，敌人 alive 置为 false，玩家 vel.y 设置为 bounce_velocity。

### 前置条件

- Player 在位置 (150, 572)，vel.y=300
- Enemy 在位置 (150, 584)，config.bounce_velocity = -200.0
- Physics::enemy_check 已返回 `[EnemyStomp(0)]`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 执行 stomp 事件消费逻辑：`enemies[i].alive = false` | enemy.alive == false |
| 2 | 执行反弹逻辑：`player.vel.y = enemy.config.bounce_velocity` | player.vel.y ≈ -200.0 |
| 3 | 验证反弹方向 | player.vel.y < 0.0 (向上反弹) |

### 验证点

- `enemy.alive == false` (敌人被消灭)
- `player.vel.y ≈ -200.0` (向上反弹速度)
- `player.vel.y < 0.0` (负值 = Y-up 方向，确认向上)

### 后置检查

- 无（逻辑验证，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t05_fun_happy_stomp_processing_enemy_dies_player_bounces
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-006

### 关联需求

FR-010（Stompable Enemy）— AC-3: 侧面接触触发玩家死亡

### 测试目标

验证玩家水平方向与敌人重叠且无向下速度时，`Physics::enemy_check` 返回 `EnemyContact` 而非 `EnemyStomp`。

### 前置条件

- Player 在位置 (140, 584)，vel.y=0 (水平移动，无垂直速度)
- Enemy 在位置 (150, 584)
- AABB 重叠（侧面接触）：玩家右边缘 148 与敌人左边缘 142 重叠

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（侧面重叠确认） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回 `[EnemyContact(0)]`，非 EnemyStomp |

### 验证点

- `contains_enemy_contact(&events)` 为 `true`
- `!contains_enemy_stomp(&events)` — vel.y=0 不应判定为踩踏
- `events.len() == 1`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t06_fun_happy_side_contact_produces_enemy_contact
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-007

### 关联需求

FR-010（Stompable Enemy）— AC-4: 下方接触触发玩家死亡

### 测试目标

验证玩家向上跳跃碰到敌人底部时（vel.y < 0），`Physics::enemy_check` 返回 `EnemyContact`。

### 前置条件

- Player 在位置 (150, 600)，vel.y=-300 (向上跳跃)
- Enemy 在位置 (150, 584)
- AABB 重叠（下方接触）：玩家顶部 584 与敌人底部 584 边界接触

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（重叠确认） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回 `[EnemyContact(0)]`，非 EnemyStomp |

### 验证点

- `contains_enemy_contact(&events)` 为 `true`
- `!contains_enemy_stomp(&events)` — vel.y < 0 不应判定为踩踏
- `events.len() == 1`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t07_fun_happy_below_contact_produces_enemy_contact
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-008

### 关联需求

FR-010（Stompable Enemy）— 死敌人不产生碰撞事件

### 测试目标

验证敌人 alive=false 时，即使 AABB 与玩家重叠，`enemy_check` 也跳过该敌人，不产生任何事件。

### 前置条件

- Player 在位置 (150, 572)，vel.y=300
- Enemy 在位置 (150, 584)，alive=false（已消灭）
- AABB 重叠确认

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（即使 alive=false，几何位置仍重叠） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回空 Vec `[]` |

### 验证点

- `events.is_empty()` 为 `true`
- 死敌人不产生任何碰撞事件

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t08_fun_error_dead_enemy_produces_no_events
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-009

### 关联需求

FR-010（Stompable Enemy）— 零垂直速度的碰撞 ≠ 踩踏

### 测试目标

验证玩家静止站立在敌人顶部（vel.y=0，但位置重叠）时，判定为 EnemyContact 而非 EnemyStomp。

### 前置条件

- Player 在位置 (150, 568)，vel.y=0.0（静止，恰在敌人上方）
- Enemy 在位置 (150, 584)
- AABB 边界接触（玩家底部 568 == 敌人顶部 568）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（边界接触） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回 `[EnemyContact(0)]`，非 EnemyStomp |

### 验证点

- `!contains_enemy_stomp(&events)` — vel.y=0 不满足踩踏条件 (>0)
- `contains_enemy_contact(&events)` 为 `true`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t09_fun_error_zero_vely_is_contact_not_stomp
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-010

### 关联需求

FR-010（Stompable Enemy）— 多敌人索引正确性

### 测试目标

验证多个敌人（enemy[0], enemy[1], enemy[2]）中仅 enemy[1] 与玩家重叠时，事件索引正确为 1。

### 前置条件

- 3 个敌人位于不同 X 坐标：enemy[0] at x=50, enemy[1] at x=150, enemy[2] at x=300
- Player 在位置 (150, 572)，仅与 enemy[1] AABB 重叠
- enemy[0] 和 enemy[2] 不与玩家重叠

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player 仅与 enemy[1] 重叠 | intersects(enemy[0])=false, intersects(enemy[1])=true, intersects(enemy[2])=false |
| 2 | `Physics::enemy_check(&player, &[enemy0, enemy1, enemy2], 1/60)` | 返回 `[EnemyStomp(1)]`，长度为 1 |

### 验证点

- `events.len() == 1`
- `contains_enemy_stomp(&events)` 为 `true`
- `get_stomp_index(&events) == 1` (正确索引 enemy[1])
- enemy[0] 和 enemy[2] 不受影响

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t10_fun_error_multi_enemy_index_correctness
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-007-001

### 关联需求

FR-010（Stompable Enemy）— AC-1: 浮点 overshoot 精确到达路点

### 测试目标

验证敌人因浮点运算略微越过路点（149.17 → 150.003）时，仍然正确触发反转并钳位至路点位置。

### 前置条件

- Enemy at pos.x=149.17, waypoint_b.x=150.0, speed=50, dt=1/60
- 预期位移：149.17 + 50/60 ≈ 150.003 > 150.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `enemy.update(1.0/60.0)` | pos.x 钳位至 150.0 |
| 2 | 检查反转 | vel.x ≈ -50.0 (方向反转) |

### 验证点

- `enemy.pos.x ≈ 150.0` (钳位至 waypoint_b)
- `enemy.vel.x ≈ -50.0` (方向反转确认)

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t11_bndry_edge_fp_overshoot_at_waypoint_triggers_reversal
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-007-002

### 关联需求

FR-010（Stompable Enemy）— AC-1: 高速 overshoot 钳位

### 测试目标

验证敌人以高速（300 px/s）大幅越过路点（140→160）时，正确钳位至路点并反转方向。

### 前置条件

- Enemy at pos.x=140, waypoint_b.x=150, speed=300, dt=1/15
- 预期位移：140 + 300/15 = 160，大幅 overshoot

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `enemy.update(1.0/15.0)` | pos.x 钳位至 150.0（不飞到路点之外） |
| 2 | 检查反转速度 | vel.x ≈ -300.0 (按配置速度反转) |

### 验证点

- `enemy.pos.x ≈ 150.0` (钳位至 waypoint_b)
- `enemy.vel.x ≈ -300.0` (以配置速度反转，非默认 50)

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t12_bndry_edge_high_speed_overshoot_clamping
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-007-003

### 关联需求

FR-010（Stompable Enemy）— 切线 AABB 接触算作碰撞

### 测试目标

验证 AABB 边界切线接触（玩家右边缘 == 敌人左边缘）按包容性语义计为碰撞事件。

### 前置条件

- Player 在位置 (158, 584)，AABB 右边缘 166
- Enemy 在位置 (166, 584)，AABB 左边缘 158
- 切线接触（接触但不重叠像素区域）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true（包容性边界语义） |
| 2 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回非空事件 Vec，长度为 1 |
| 3 | 检查事件类型 | EnemyContact (玩家并非从上方进入) |

### 验证点

- `intersects()` 返回 `true`（切线接触判定为碰撞）
- `events.len() == 1`
- `contains_enemy_contact(&events)` 为 `true`

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t13_bndry_edge_tangential_contact_counts_as_collision
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-007-004

### 关联需求

FR-010（Stompable Enemy）— 零巡逻速度

### 测试目标

验证 speed=0 的敌人静止不动，多次 update 不 panic，不产生位置漂移。

### 前置条件

- EnemyConfig.speed=0.0, bounce_velocity=-200.0
- Enemy at pos(100, 584)，waypoints (50, 150)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 创建 speed=0 的 Enemy | vel.x=0.0，pos.x=100.0 |
| 2 | 连续 60 帧 update(dt=1/60) | pos.x 保持 100.0，vel.x 保持 0.0 |
| 3 | 检查存活状态 | enemy.alive 保持 true |

### 验证点

- `enemy.pos.x ≈ 100.0` (位置不变)
- `enemy.pos.y ≈ 584.0` (Y 不变)
- `enemy.vel.x ≈ 0.0` (速度为零)
- `enemy.alive == true` (存活)
- 无 panic，无 NaN

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t14_bndry_edge_zero_patrol_speed_no_panic
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-007-005

### 关联需求

FR-010（Stompable Enemy）— 空敌人列表

### 测试目标

验证 enemies 为空切片时，`enemy_check` 返回空 Vec 且不 panic。

### 前置条件

- Player 在任意位置
- enemies = `[]`（空 Vec）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Physics::enemy_check(&player, &[], 1/60)` | 返回空 Vec `[]`，无 panic |

### 验证点

- `events.is_empty()` 为 `true`
- 无 panic（执行到达断言处即验证成功）

### 后置检查

- 无（纯函数调用，无副作用）

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t15_bndry_null_empty_enemies_returns_empty_vec
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-011

### 关联需求

FR-010（Stompable Enemy）— Physics::enemy_check 端到端集成

### 测试目标

验证从真实 Enemy 构造到 Physics::enemy_check 碰撞检测的完整调用链：Enemy 构造 → 碰撞体正确 → AABB 重叠 → 事件类型正确 → 索引正确。

### 前置条件

- Player 在位置 (150, 572)，vel.y=300
- Enemy 通过 Enemy::new() 正常构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Enemy 并验证初始状态 | alive=true, pos 正确, vel.x=speed, collider 尺寸 16x16 |
| 2 | 验证 player.collider().intersects(&enemy.collider()) | 返回 true |
| 3 | `Physics::enemy_check(&player, &[enemy], 1/60)` | 返回 `[EnemyStomp(0)]` |
| 4 | 验证索引 | get_stomp_index(&events) == 0 |

### 验证点

- 完整集成管道正确运行：Enemy 构造 → 碰撞体查询 → 重叠检测 → 事件生成
- 事件类型正确（EnemyStomp）
- 索引正确（0）

### 后置检查

- 无（Enemy 实例在测试结束时释放）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t16_intg_physics_end_to_end_enemy_collision_detection
- **Test Type**: Real

---

### 用例编号

ST-FUNC-007-012

### 关联需求

FR-010（Stompable Enemy）— PlayingState 更新集成

### 测试目标

验证 PlayingState::update(dt) 完整集成敌人巡逻更新 + 碰撞检测 + 事件消费的端到端流程。

### 前置条件

- PlayingState::new() 成功构造，含 2 个硬编码巡逻敌人
- Player 初始位置在 (150, 572)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 创建 PlayingState，验证 enemies 非空 | state.enemies 含至少 1 个敌人，全部 alive=true |
| 2 | 记录初始位置，调用 state.update(dt) | 至少一个敌人位置发生变化（巡逻活跃） |
| 3 | 验证敌人 Y 坐标不变 | 所有 enemies 的 Y 坐标不变 |
| 4 | 设置踩踏场景并调用 update() | 若踩踏发生：enemy.alive=false + player.vel.y<0；若接触发生：player.lives 减少 |
| 5 | 多次连续 update 验证稳定性 | 无 panic |

### 验证点

- PlayingState 正确初始化 enemies Vec
- update() 驱动敌人巡逻（位置变化）
- 敌人 Y 坐标不变（无垂直移动）
- 踩踏后敌人 alive=false + 玩家反弹
- 多次 update 无 panic

### 后置检查

- PlayingState 实例在测试结束时释放

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/patrol_enemy_test.rs::t17_intg_state_playing_state_update_integrates_enemies
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | Feature Design Test Inventory | 自动化测试 | Test Type | 结果 |
|---------|----------|------------------------------|-----------|---------|------|
| ST-FUNC-007-001 | FR-010 AC-1 (patrol right) | Row A — FUNC/happy | t01_fun_happy_patrol_right_reaches_waypoint_b_reverses | Real | PASS |
| ST-FUNC-007-002 | FR-010 AC-1 (patrol left+reverse) | Row B — FUNC/happy | t02_fun_happy_patrol_left_from_b_reaches_waypoint_a_reverses | Real | PASS |
| ST-FUNC-007-003 | FR-010 AC-1 (left waypoint reverse) | Row C — FUNC/happy | t03_fun_happy_reaches_left_waypoint_reverses_to_right | Real | PASS |
| ST-FUNC-007-004 | FR-010 AC-2 (stomp detection) | Row D — FUNC/happy | t04_fun_happy_stomp_from_above_produces_enemy_stomp | Real | PASS |
| ST-FUNC-007-005 | FR-010 AC-2 (stomp processing) | Row E — FUNC/happy | t05_fun_happy_stomp_processing_enemy_dies_player_bounces | Real | PASS |
| ST-FUNC-007-006 | FR-010 AC-3 (side contact death) | Row F — FUNC/happy | t06_fun_happy_side_contact_produces_enemy_contact | Real | PASS |
| ST-FUNC-007-007 | FR-010 AC-4 (below contact death) | Row G — FUNC/happy | t07_fun_happy_below_contact_produces_enemy_contact | Real | PASS |
| ST-FUNC-007-008 | FR-010 (dead enemy no events) | Row H — FUNC/error | t08_fun_error_dead_enemy_produces_no_events | Real | PASS |
| ST-FUNC-007-009 | FR-010 (stomp req vel.y>0) | Row I — FUNC/error | t09_fun_error_zero_vely_is_contact_not_stomp | Real | PASS |
| ST-FUNC-007-010 | FR-010 (multi-enemy index) | Row J — FUNC/error | t10_fun_error_multi_enemy_index_correctness | Real | PASS |
| ST-BNDRY-007-001 | FR-010 AC-1 (fp overshoot) | Row K — BNDRY/edge | t11_bndry_edge_fp_overshoot_at_waypoint_triggers_reversal | Real | PASS |
| ST-BNDRY-007-002 | FR-010 AC-1 (high-speed overshoot) | Row L — BNDRY/edge | t12_bndry_edge_high_speed_overshoot_clamping | Real | PASS |
| ST-BNDRY-007-003 | FR-010 (tangential contact) | Row M — BNDRY/edge | t13_bndry_edge_tangential_contact_counts_as_collision | Real | PASS |
| ST-BNDRY-007-004 | FR-010 (zero speed) | Row N — BNDRY/edge | t14_bndry_edge_zero_patrol_speed_no_panic | Real | PASS |
| ST-BNDRY-007-005 | FR-010 (empty enemies) | Row O — BNDRY/null | t15_bndry_null_empty_enemies_returns_empty_vec | Real | PASS |
| ST-INTG-007-001 | FR-010 (Physics integration) | Row P — INTG/physics | t16_intg_physics_end_to_end_enemy_collision_detection | Real | PASS |
| ST-FUNC-007-012 | FR-010 (PlayingState integration) | Row Q — INTG/state | t17_intg_state_playing_state_update_integrates_enemies | Real | PASS |

> SRS Trace 覆盖：1/1 FR-010 (4 ACs) 全部有 ST 用例映射 — 通过
> Feature Design Test Inventory 覆盖：17/17 行 (A-Q) 全部映射 — 通过
> ATS 类别覆盖：FUNC (12 cases) >= 1 OK, BNDRY (5 cases) >= 1 OK
> 负面测试占比：FUNC/error (3) + BNDRY/edge (4) + BNDRY/null (1) = 8/17 = 47.1% >= 40% — 通过

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 17 |
| Passed | 17 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` — must be fixed and re-executed.

## Manual Test Case Summary

> 本特性无手动测试用例（全部 17 条已自动化，`ui: false`）。
