# 测试用例集: Life, Death & Win

**Feature ID**: 6
**关联需求**: FR-014a (Death Trigger), FR-014b (Respawn & Invulnerability), FR-014c (Game Over), FR-015 (Win Condition — Flagpole)
**日期**: 2026-06-02
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

> Specification resolutions applied from Feature Design Clarification Addendum (无需澄清 — 全部规格明确).

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 17 |
| boundary | 10 |
| ui | 5 |
| security | 0 |
| performance | 0 |
| **合计** | **32** |

---

### 用例编号

ST-FUNC-006-001

### 关联需求

FR-014a（Death Trigger）— AC-1: 生命数从 N 减至 N-1

### 测试目标

验证玩家接触即死危险时，生命计数器正确递减。

### 前置条件

- Player 创建，lives=3，处于 Playing 状态
- 触发 `CollisionEvent::HazardContact`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 创建 PlayingState，lives=3，检测到 HazardContact 事件 | PlayingState::update 返回 GameState::Dead |
| 2 | 检查 DeadState 中的 lives 字段 | `lives == 2`（从 3 减至 2） |
| 3 | 重复死亡 2 次（共 3 次） | lives 序列：3→2→1→0 |

### 验证点

- 每次死亡 life 递减 1
- 第 3 次死亡后 lives=0，转入 GameOver 而非 Playing

### 后置检查

- 无（纯状态检查，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t01_fun_happy_hazard_contact_decrements_lives, t22_bndry_edge_lives_sequence_three_to_zero
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-002

### 关联需求

FR-014a（Death Trigger）— AC-2: 死亡动画 1.5s 内输入锁定

### 测试目标

验证死亡动画播放期间玩家输入被完全锁定，角色位置不变。

### 前置条件

- DeadState 已激活，death_timer=1.5

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 向 DeadState::update 传入包含 right/jump 的 InputState | update 后 death_timer 递减，但不更新 Player 位置 |
| 2 | 检查 Player 位置是否变化 | Player 位置与死亡瞬间保持一致 |
| 3 | 连续推进 90 帧 (dt=1/60)，始终发送输入 | 整个 1.5s 期间输入均被忽略 |

### 验证点

- DeadState 期间 Player::update 不被调用
- 所有方向键和跳跃输入均被忽略
- 1.5s 后 death_timer <= 0，触发状态转换

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t02_fun_happy_input_locked_during_death_animation
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-003

### 关联需求

FR-014a（Death Trigger）— AC-3: 生命 > 0 时转入重生流程

### 测试目标

验证死亡动画完成后，当 lives > 0 时状态机转换至 Playing（重生）。

### 前置条件

- DeadState 激活，lives=2，checkpoint=Some(pos)
- death_timer 到期（≤ 0）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 推进 death_timer 至 ≤ 0 | DeadState 内部标记 transition |
| 2 | GameState::update 读取转换信号 | 返回 Playing 变体 |
| 3 | 新 PlayingState 中的 invuln_timer | `invuln_timer == 2.0` |

### 验证点

- 返回 GameState::Playing（而非 GameOver）
- invuln_timer 初始化为 2.0
- Player 位置为 checkpoint 位置

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t03_fun_happy_death_timer_expires_lives_gt_0_transitions_to_playing
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-004

### 关联需求

FR-014a（Death Trigger）— AC-4: 生命 = 0 时转入 Game Over

### 测试目标

验证死亡动画完成后，当 lives = 0 时状态机转换至 GameOver。

### 前置条件

- DeadState 激活，lives=0（已在进入 DeadState 前减至 0）
- death_timer 到期（≤ 0）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 推进 death_timer 至 ≤ 0 | DeadState 内部标记 transition |
| 2 | GameState::update 读取转换信号 | 返回 GameOver 变体 |
| 3 | 检查 GameOverState.coins | coins 等于死亡时的金币数 |

### 验证点

- 返回 GameState::GameOver（而非 Playing）
- GameOverState 持有正确的 coins 值
- lives=1 时死亡 → GameOver（最后一次机会耗尽）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t04_fun_happy_lives_reach_zero_transitions_to_game_over
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-005

### 关联需求

FR-014a（Death Trigger）— AC-3 补充: 死亡与旗杆同一帧时死亡优先

### 测试目标

验证同一帧内同时触发 HazardContact 和 FlagpoleReached 时，死亡流程优先于胜利。

### 前置条件

- PlayingState，玩家位置同时满足 HazardContact 和 Flagpole AABB 重叠
- lives=2

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造同时包含 HazardContact 和 FlagpoleReached 的事件列表 | 两个事件都会产生 |
| 2 | PlayingState::update 处理事件 | 优先检查并触发死亡流程 |
| 3 | 检查转换后的状态 | 返回 GameState::Dead，lives=1 |

### 验证点

- 死亡触发（而非 Victory 触发）
- FlagpoleReached 事件在死亡帧被忽略
- lives 正确递减

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t15_fun_happy_death_priority_over_flagpole_in_same_frame
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-006

### 关联需求

FR-014a（Death Trigger）— 防御: 死亡动画期间不二次触发死亡

### 测试目标

验证 DeadState 期间再次坠落超过 kill_y 不会二次触发死亡流程。

### 前置条件

- DeadState 已激活，death_timer=0.5（动画播放中）
- 玩家 Y 坐标超过 kill_y

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 在 DeadState 中推进 death_timer | death_timer 正常递减 |
| 2 | 模拟 Physics 返回 PitFall 事件（实际由 GameState 阻止） | DeadState 不处理致命事件 |
| 3 | 检查 lives 是否被二次递减 | lives 保持不变 |

### 验证点

- DeadState 期间不会触发新的死亡流程
- lives 不被二次递减
- death_timer 到期后按正确路径转换

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t21_fun_error_no_death_retrigger_during_dead_state
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-007

### 关联需求

FR-014a（Death Trigger）— 防御: Space 键不跳过死亡动画

### 测试目标

验证死亡动画期间按下 Space 不会跳过动画或提前转换状态。

### 前置条件

- DeadState 激活，death_timer=0.8

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 向 DeadState::update 传入 jump_just=true 的 InputState | death_timer 正常递减，不加速 |
| 2 | 连续推进动画，每帧发送 Space | 动画完整播放 1.5s |
| 3 | 动画结束后按正確逻辑转换 | lives>0 → Playing，lives=0 → GameOver |

### 验证点

- Space 不加速或跳过 death_timer
- death_timer 经历完整 1.5s
- 状态转换时机不受 Space 影响

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t18_fun_error_space_does_not_skip_death_animation
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-008

### 关联需求

FR-014b（Respawn & Invulnerability）— AC-1: 在激活的检查点位置重生

### 测试目标

验证玩家在激活检查点后死亡，重生位置为检查点坐标。

### 前置条件

- 已激活检查点，LifeState.checkpoint = Some(Vec2{x:500, y:300})
- lives=2，死亡动画完成

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | DeadState 转换至 Playing，checkpoint=Some(500,300) | Player.pos 设为 (500, 300) |
| 2 | 验证 Player 重生位置 | `player.pos() == Vec2(500.0, 300.0)` |
| 3 | 验证 invuln_timer | `invuln_timer == 2.0` |

### 验证点

- 重生位置精确等于检查点位置
- invuln_timer 初始化为 2.0
- 玩家碰撞检测正常运行

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t05_fun_happy_respawn_at_checkpoint_position
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-009

### 关联需求

FR-014b（Respawn & Invulnerability）— AC-1: 无检查点时在关卡起点重生

### 测试目标

验证玩家未激活任何检查点时，死亡后重生至关卡起点。

### 前置条件

- LifeState.checkpoint = None
- lives=2，死亡动画完成

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | DeadState 转换至 Playing，checkpoint=None | Player.pos 设为关卡起点 (100, 100) |
| 2 | 验证 Player 重生位置 | `player.pos() == 关卡起点坐标` |

### 验证点

- 重生位置为关卡起点
- checkpoint 保持 None
- invuln_timer 初始化正确

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t06_fun_happy_respawn_at_level_start_when_no_checkpoint
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-010

### 关联需求

FR-014b（Respawn & Invulnerability）— AC-1: 连续死亡未经过检查点始终在起点重生

### 测试目标

验证连续多次死亡且未经过检查点时，每次都重生至关卡起点（而非上次重生位置）。

### 前置条件

- 从未激活任何检查点
- 连续死亡 3 次

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 第 1 次死亡 → 重生 | pos = 关卡起点 |
| 2 | 第 2 次死亡 → 重生 | pos = 关卡起点（非上次重生位置） |
| 3 | 第 3 次死亡 → 重生 | pos = 关卡起点 |

### 验证点

- 每次重生位置均为关卡起点
- checkpoint 始终为 None
- 不存在"隐式缓存"的检查点

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t19_fun_error_consecutive_deaths_without_checkpoint_respawn_at_level_start, t20_fun_error_checkpoint_stays_none_after_no_checkpoint_respawn
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-011

### 关联需求

FR-014b（Respawn & Invulnerability）— AC-2: 无敌期间接触危险不受伤

### 测试目标

验证重生后 2 秒无敌期间，玩家接触尖刺或坠落深渊不触发死亡。

### 前置条件

- PlayingState，invuln_timer > 0（无敌状态中）
- Physics 返回 HazardContact 或 PitFall

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | invuln_timer=1.5, 检测到 HazardContact | 事件被丢弃，lives 不变，继续 Playing |
| 2 | invuln_timer=1.0, 检测到 PitFall | 事件被丢弃，lives 不变 |
| 3 | invuln_timer 持续递减 | 无敌持续到 0 |

### 验证点

- HazardContact 和 PitFall 均被无敌豁免
- lives 不递减
- GameState 保持 Playing

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t07_fun_happy_invulnerable_player_ignores_hazard_events, t16_fun_error_invulnerability_blocks_pit_fall, t17_fun_error_invulnerability_blocks_hazard_contact
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-012

### 关联需求

FR-014b（Respawn & Invulnerability）— AC-3: 无敌结束后碰撞恢复

### 测试目标

验证 2 秒无敌到期后，下一次危险接触正常触发死亡。

### 前置条件

- PlayingState，invuln_timer 刚从正数递减至 ≤ 0
- 之后发生 HazardContact

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | invuln_timer 从 0.0167 递减至 0.0 | invuln_timer 到达 0 |
| 2 | 之后触发 HazardContact | 事件未被过滤，正常触发死亡流程 |
| 3 | 验证 lives 递减 | lives 减 1 |

### 验证点

- invuln_timer ≤ 0 后碰撞恢复
- 死亡流程正常触发
- lives 正常递减

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t08_fun_happy_invulnerability_expiry_restores_collision
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-013

### 关联需求

FR-014c（Game Over）— AC-1, AC-2: Game Over 画面 + Space 重置

### 测试目标

验证生命归零后显示 Game Over 画面，且空格键触发完全重置。

### 前置条件

- lives=0，死亡动画完成
- GameOverState 已创建，coins=5

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | GameOverState::update(dt), 无 Space 输入 | 状态保持 GameOver，blink_phase 推进 |
| 2 | GameOverState::update(dt), jump_just=true (Space) | 返回 reset 信号 |
| 3 | GameState::full_reset | lives=3, coins=0, checkpoint=None, pos=关卡起点 |

### 验证点

- GameOver 画面显示且不自动消失
- Space 触发完全重置
- 重置后: lives=3, coins=0, checkpoint=None, pos=关卡起点
- 状态转换至 Playing

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t09_fun_happy_game_over_renders_with_expected_values, t10_fun_happy_space_on_game_over_triggers_full_reset, t11_fun_happy_game_over_persists_without_space_input
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-014

### 关联需求

FR-015（Win Condition）— AC-1, AC-2, AC-3: 旗杆触发→胜利画面→Space 重置

### 测试目标

验证旗杆触发滑下动画、动画完成后显示 Victory 画面、Space 重置。

### 前置条件

- PlayingState，玩家碰撞体与旗杆 AABB 重叠
- coins=42

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | PlayingState::check_flagpole() | 返回 true |
| 2 | Flagpole.phase 切换至 Sliding，slide_progress 推进 | 动画完成时 phase=Done |
| 3 | 构造 VictoryState(coins=42) | VictoryState.coins == 42 |
| 4 | VictoryState::update(dt), jump_just=true | 返回 reset 信号 → Playing (lives=3, coins=0) |

### 验证点

- 旗杆 AABB 重叠检测正确 (16×80 触发区域)
- Flagpole.phase 从 Idle→Sliding→Done 状态转移
- Victory 画面渲染金币数 "Coins: 042"
- Space 触发完全重置

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t12_fun_happy_flagpole_overlap_triggers_slide, t13_fun_happy_flagpole_complete_victory_shows_coins, t14_fun_happy_space_on_victory_triggers_full_reset
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-001

### 关联需求

FR-014a（Death Trigger）— 边界: lives 递减序列

### 测试目标

验证 lives 从 3→2→1→0 完整递减序列无 off-by-one 错误。

### 前置条件

- lives=3
- 连续 3 次触发死亡

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 第 1 次死亡 → lives=2 | 重生至 Playing |
| 2 | 第 2 次死亡 → lives=1 | 重生至 Playing |
| 3 | 第 3 次死亡 → lives=0 | GameOver（非重生） |

### 验证点

- lives 不出现负值或 u32 下溢
- 第 3 次死亡后状态为 GameOver 而非 Playing
- 每次递减均为精确 1

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t22_bndry_edge_lives_sequence_three_to_zero
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-002

### 关联需求

FR-014a（Death Trigger）— 边界: death_timer 90 帧精确

### 测试目标

验证 1.5s 死亡计时器在恰好 90 帧后触发转换（dt=1/60s）。

### 前置条件

- DeadState 激活，death_timer=1.5

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 推进 89 帧 (dt=1/60 × 89) | death_timer > 0，状态保持 Dead |
| 2 | 推进第 90 帧 | death_timer <= 0，状态转换触发 |

### 验证点

- 90 帧后精确触发（非 89 帧或 91 帧）
- 浮点累积误差可接受（使用 <= 0.0 判断）
- dt=1/60, 1.5s = 90 帧

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t23_bndry_edge_death_timer_90_frames_exactly
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-003

### 关联需求

FR-014a（Death Trigger）— 边界: kill_y 坠落判定

### 测试目标

验证 Y 坐标相对于 kill_y 的边界判定：> kill_y 触发 PitFall，< kill_y 不触发。

### 前置条件

- kill_y = 2500.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | Player Y = kill_y + 1.0 (> kill_y) | PitFall 事件触发 |
| 2 | Player Y = kill_y - 1.0 (< kill_y) | 无 PitFall，安全 |

### 验证点

- 边界 off-by-one 正确：Y > kill_y → 致命
- 边界 off-by-one 正确：Y < kill_y → 安全
- Y == kill_y 的边界行为一致

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t25_bndry_edge_y_greater_than_kill_y_triggers_pit_fall, t26_bndry_edge_y_less_than_kill_y_is_safe
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-004

### 关联需求

FR-014b（Respawn & Invulnerability）— 边界: invuln_timer 120 帧精确

### 测试目标

验证 2.0s 无敌计时器在恰好 120 帧后到期。

### 前置条件

- PlayingState，invuln_timer=2.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 推进 119 帧 | invuln_timer > 0，无敌状态保持 |
| 2 | 推进第 120 帧 | invuln_timer <= 0，碰撞恢复 |

### 验证点

- 2.0s = 120 帧 @ 60fps
- 无敌到期后碰撞正常恢复

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t24_bndry_edge_invuln_timer_120_frames_exactly
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-005

### 关联需求

FR-014b（Respawn & Invulnerability）— 边界: 重生位置精确匹配

### 测试目标

验证重生时玩家位置严格等于检查点坐标（Vec2 精确值）。

### 前置条件

- checkpoint = Some(Vec2{x:500.0, y:300.0})
- 重生完成

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 读取 Player.pos | player.pos == Vec2{x:500.0, y:300.0} |

### 验证点

- x, y 均为精确值（浮点比较使用 epsilon=0.001）
- 无坐标拷贝/赋值错误导致的偏移

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t27_bndry_edge_checkpoint_position_exact_copy
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-006

### 关联需求

FR-015（Win Condition）— 边界: 旗杆碰撞边界接触

### 测试目标

验证玩家 AABB 右边界刚好等于旗杆 AABB 左边界时算碰撞。

### 前置条件

- 旗杆碰撞体 AABB: (x, y, 16, 80)
- 玩家 AABB 右边界 == 旗杆 AABB 左边界

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | player.collider().intersects(&flagpole.collider()) | 返回 true（边界接触算重叠） |
| 2 | 玩家 AABB 右边界 = 旗杆左边界 - 1px | 返回 false（1px 间隙不触发） |

### 验证点

- AABB intersects 使用 <= / >= （而非严格 < / >）
- 1px 间隙不误触发 Victory

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t28_bndry_edge_flagpole_edge_contact_counts_as_overlap, t29_bndry_edge_one_pixel_gap_from_flagpole_no_trigger
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-007

### 关联需求

FR-014b（Respawn & Invulnerability）— 边界: 闪烁相位 4Hz 阈值

### 测试目标

验证无敌闪烁在 flicker_phase 周期 0.25s 下准确切换可见/不可见。

### 前置条件

- invuln_timer > 0，flicker_phase 递增

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | flicker_phase 从 0.124 推进至 0.126（跨过 0.125） | 可见性在 0.125 边界翻转 |
| 2 | 完整周期 0.250s | 完成一次可见+不可见循环（4Hz） |

### 验证点

- flicker_phase % 0.25 < 0.125 → 可见
- flicker_phase % 0.25 >= 0.125 → 不可见
- 值在阈值边界精确翻转

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t30_bndry_edge_flicker_phase_4hz_toggle_at_boundary
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-008

### 关联需求

FR-014c（Game Over）— 边界: lives=0 不溢出

### 测试目标

验证 lives=0 后不会再递减（防止 u32 下溢为 u32::MAX）。

### 前置条件

- lives=0，GameOverState 已激活

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | GameOver 状态下不触发额外死亡事件检查 | lives 保持 0 |
| 2 | 系统保持 GameOver | 不会因 lives 下溢回归 Playing |

### 验证点

- lives 保持在 0
- 不会变成 u32::MAX（下溢）
- GameOver 状态稳定

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t32_bndry_edge_lives_zero_no_underflow
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-009

### 关联需求

FR-014c, FR-015（Game Over / Victory）— 边界: 完全重置验证

### 测试目标

验证 GameOver/Victory → Space → full_reset 后所有字段恢复初始值。

### 前置条件

- GameOverState 或 VictoryState，coins 非 0
- 执行完全重置

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | GameOver → Space → full_reset | lives=3, coins=0, checkpoint=None, pos=关卡起点 |
| 2 | Victory → Space → full_reset | lives=3, coins=0, checkpoint=None, pos=关卡起点（与 GameOver 重置一致） |
| 3 | LifeState::reset() 单独调用 | lives=3, checkpoint=None, coins=0 |

### 验证点

- 两条重置路径结果一致
- 所有字段均重置无遗漏
- checkpoint 和 coins 均被清除

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t33_bndry_edge_full_reset_from_game_over_all_fields, t34_bndry_edge_victory_reset_identical_to_gameover_reset, t35_bndry_edge_life_state_reset_all_fields
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-006-010

### 关联需求

FR-014c（Game Over）— 边界: 提示闪烁 2Hz 阈值

### 测试目标

验证 "Press Space to Restart" 提示以 2Hz 正确闪烁。

### 前置条件

- GameOverState 或 VictoryState 激活
- blink_phase 推进

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | blink_phase 从 0.49 推进至 0.51（跨过 0.5） | 可见性在 0.5 边界翻转 |
| 2 | 连续采样 2.0s @ 60fps | 约一半帧可见、一半不可见（2Hz 方波 50% 占空比） |

### 验证点

- blink_phase % 1.0 < 0.5 → 文字可见
- blink_phase % 1.0 >= 0.5 → 文字不可见
- 2Hz = 0.5s 可见 / 0.5s 不可见

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t31_bndry_edge_blink_phase_2hz_toggle_at_boundary
- **Test Type**: Real

---

### 用例编号

ST-UI-006-001

### 关联需求

FR-014c（Game Over）— UI: Game Over 叠加层渲染

### 测试目标

验证 GameOverState 渲染逻辑产生正确的视觉参数（矩形尺寸、颜色、文字位置）。

### 前置条件

- GameOverState 已创建，viewport=480×270

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 GameOverState::render(alpha) | draw_rectangle 参数: (0,0,480,270), rgba(0,0,0,0.65) |
| 2 | 检查标题文字 "GAME OVER" 位置计算 | Y ≈ 108 (= 270 × 0.40), 字号 16px, 颜色白色 |
| 3 | 检查提示文字可见性 (blink_phase=0.3 < 0.5) | 提示文字 "Press Space to Restart" 可见，字号 8px |

### 验证点

- 叠加层覆盖全视口
- 标题位置 Y ≈ viewport.h × 0.40
- alpha 值 = 0.65
- 提示文字在 blink_phase < 0.5 时渲染

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: ui
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t36_ui_render_game_over_overlay_full_viewport, t37_ui_render_game_over_title_position_and_style, t38_ui_render_press_space_prompt_visible_at_correct_time
- **Test Type**: Real

---

### 用例编号

ST-UI-006-002

### 关联需求

FR-014c（Game Over）— UI: 提示闪烁 50% 占空比

### 测试目标

验证 GameOver 提示文字闪烁为精确 2Hz 50% 占空比方波。

### 前置条件

- GameOverState，blink_phase 从 0.0 推进至 2.0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 采样 2.0s 内每帧的可见性标志 | 可见帧数与不可见帧数大致相等 |
| 2 | 验证每次可见/不可见持续时长 | 每次约 0.5s (30 帧) |

### 验证点

- 2Hz 方波: 周期 1.0s
- 50% 占空比: 0.5s 可见, 0.5s 不可见
- blink_phase % 1.0 逻辑正确

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: ui
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t39_ui_render_blink_50_percent_duty_cycle_over_2_seconds
- **Test Type**: Real

---

### 用例编号

ST-UI-006-003

### 关联需求

FR-015（Win Condition）— UI: Victory 叠加层渲染

### 测试目标

验证 VictoryState 渲染逻辑产生正确的视觉参数。

### 前置条件

- VictoryState 已创建，coins=42
- viewport=480×270

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 VictoryState::render(alpha) | draw_rectangle 覆盖全视口, rgba(0,0,0,0.65) |
| 2 | 检查 "VICTORY!" 标题颜色 | RGB = (248, 184, 0) aka #F8B800 金色 |
| 3 | 检查金币文字 | "Coins: 042" 格式（3 位零填充） |

### 验证点

- 标题使用金色 RGB(248, 184, 0)
- 金币格式化 "Coins: NNN"（3 位零填充）
- 颜色常量为 UCD Token --color-coin-gold #F8B800

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: ui
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t40_ui_render_victory_overlay_renders_correct_content, t41_ui_render_victory_title_gold_color
- **Test Type**: Real

---

### 用例编号

ST-UI-006-004

### 关联需求

FR-014a, FR-014b（Death & Respawn）— UI: 死亡动画与无敌闪烁

### 测试目标

验证死亡动画（弹起+下落渐隐）和无敌闪烁（4Hz）的渲染参数计算。

### 前置条件

- DeadState 或 PlayingState（invuln_timer > 0）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | death_timer=1.2 (>1.0 → 弹起阶段) | bounce_offset > 0 (幅度 0-16px) |
| 2 | death_timer=0.5 (≤1.0 → 下落阶段) | Player Y 增加 + alpha < 1.0 (渐隐) |
| 3 | invuln_timer=1.0, 采样 1.0s | 精灵可见/不可见交替约 4 次 (4Hz) |
| 4 | Flagpole.phase=Sliding, slide_progress=0.5 | Player Y = pole_top + slide_progress × pole_height |

### 验证点

- 弹起阶段 bounce_offset ∈ [0, 16]
- 下落阶段: Y 递增, alpha 递减
- flicker_phase 驱动 4Hz 闪烁正确
- 旗杆滑下: 玩家精灵沿旗杆 Y 轴移动

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: ui
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t42_ui_render_death_bounce_animation, t43_ui_render_death_fall_fade_alpha_decreases, t44_ui_render_invulnerability_flicker_4hz, t45_ui_render_flagpole_slide_animation_player_position
- **Test Type**: Real

---

### 用例编号

ST-UI-006-005

### 关联需求

FR-014c, FR-015（Game Over / Victory）— UI: 叠加层 z-order

### 测试目标

验证叠加层的渲染 z-order 确保完全覆盖下层游戏画面。

### 前置条件

- PlayingState 正在渲染 + GameOver 叠加层

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | GameState::render 先调用 PlayingState::render，再绘制叠加层 | 叠加层矩形后绘制，覆盖游戏元素 |
| 2 | 验证渲染顺序 | alpha=0.65 的黑色矩形在精灵之上 |

### 验证点

- 叠加层在游戏元素之后渲染
- 无"穿模"——游戏元素不透过叠加层显示
- render 调用顺序正确: 游戏画面 → 叠加层

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: ui
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t46_ui_render_overlay_z_order_above_game_elements
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-015

### 关联需求

FR-014a（Death Trigger）— INTG: Player stats 实时反映 lives 变化

### 测试目标

验证 Player.lives 递减后，Player::stats() 返回正确的 lives 值（跨组件数据流）。

### 前置条件

- PlayingState 检测到 HazardContact
- Player.lives 减 1

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | PlayingState 在检测到致命事件后递减 Player.lives | lives = 原值 - 1 |
| 2 | 调用 Player::stats() | stats.lives == 递减后的值 |

### 验证点

- lives 通过 IAPI-009 PlayerStats 正确暴露
- HUD (F09) 可通过 Player::stats() 读取最新值
- 同一条路径下没有数据不一致

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: integration
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t47_intg_player_lives_decrement_reflected_in_player_stats
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-016

### 关联需求

FR-014a（Death Trigger）— INTG: 多危险事件去重

### 测试目标

验证同一帧内 Physics 返回多个致命事件时，PlayingState 只触发一次死亡。

### 前置条件

- Physics::hazard_check() 同时返回 [HazardContact, PitFall]

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | PlayingState 接收多个致命事件 | 只触发一次死亡流程 |
| 2 | 检查 lives | lives 减 1（非 2） |

### 验证点

- 多事件去重: lives 只减 1
- DeadState 只创建一次
- 不会因同帧多事件导致 lives 多减

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: integration
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t48_intg_physics_multiple_hazard_events_single_death
- **Test Type**: Real

---

### 用例编号

ST-FUNC-006-017

### 关联需求

FR-014a（Death Trigger）— INTG: Level bounds 提供 kill_y

### 测试目标

验证 Level::bounds() 返回的 kill_y 被 Physics 正确使用以检测坠落。

### 前置条件

- Level 已创建，kill_y=2500.0
- 玩家 Y = 2501.0 (超过 kill_y)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | Level::bounds() 返回 kill_y=2500 | bounds.kill_y == 2500.0 |
| 2 | Physics::hazard_check(player, terrain, kill_y) | 检测到 PitFall |
| 3 | PitFall 事件触发死亡流程 | GameState → Dead |

### 验证点

- kill_y 从 Level 正确读取（非硬编码默认值）
- Physics 使用 Level::bounds() 返回值
- PitFall 触发条件: Y > kill_y

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: integration
- **已自动化**: Yes
- **测试引用**: tests/life_death_win_test.rs::t49_intg_level_kill_y_from_level_bounds_triggers_death
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-006-001 | FR-014a | AC-1: lives 从 N 减至 N-1 | t01_fun_happy_hazard_contact_decrements_lives | Real | PASS |
| ST-FUNC-006-002 | FR-014a | AC-2: 输入锁定 1.5s | t02_fun_happy_input_locked_during_death_animation | Real | PASS |
| ST-FUNC-006-003 | FR-014a | AC-3: 生命>0 → 重生 | t03_fun_happy_death_timer_expires_lives_gt_0_transitions_to_playing | Real | PASS |
| ST-FUNC-006-004 | FR-014a | AC-4: 生命=0 → Game Over | t04_fun_happy_lives_reach_zero_transitions_to_game_over | Real | PASS |
| ST-FUNC-006-005 | FR-014a, FR-015 | 死亡优先于胜利 (FR-015 AC-4) | t15_fun_happy_death_priority_over_flagpole_in_same_frame | Real | PASS |
| ST-FUNC-006-006 | FR-014a | 死亡动画期间不二次触发 | t21_fun_error_no_death_retrigger_during_dead_state | Real | PASS |
| ST-FUNC-006-007 | FR-014a | Space 不跳过死亡动画 | t18_fun_error_space_does_not_skip_death_animation | Real | PASS |
| ST-FUNC-006-008 | FR-014b | AC-1: 检查点位置重生 | t05_fun_happy_respawn_at_checkpoint_position | Real | PASS |
| ST-FUNC-006-009 | FR-014b | AC-1: 无检查点 → 关卡起点重生 | t06_fun_happy_respawn_at_level_start_when_no_checkpoint | Real | PASS |
| ST-FUNC-006-010 | FR-014b | AC-1: 连续无检查点死亡始终起点 | t19_fun_error_consecutive_deaths_without_checkpoint_respawn_at_level_start | Real | PASS |
| ST-FUNC-006-011 | FR-014b | AC-2: 无敌期间豁免危险 | t07_fun_happy_invulnerable_player_ignores_hazard_events, t16_fun_error_invulnerability_blocks_pit_fall, t17_fun_error_invulnerability_blocks_hazard_contact | Real | PASS |
| ST-FUNC-006-012 | FR-014b | AC-3: 无敌结束后碰撞恢复 | t08_fun_happy_invulnerability_expiry_restores_collision | Real | PASS |
| ST-FUNC-006-013 | FR-014c | AC-1, AC-2: Game Over 画面 + Space 重置 | t09_fun_happy_game_over_renders_with_expected_values, t10_fun_happy_space_on_game_over_triggers_full_reset, t11_fun_happy_game_over_persists_without_space_input | Real | PASS |
| ST-FUNC-006-014 | FR-015 | AC-1, AC-2, AC-3: 旗杆 → Victory → Space 重置 | t12_fun_happy_flagpole_overlap_triggers_slide, t13_fun_happy_flagpole_complete_victory_shows_coins, t14_fun_happy_space_on_victory_triggers_full_reset | Real | PASS |
| ST-BNDRY-006-001 | FR-014a | lives 3→0 序列 | t22_bndry_edge_lives_sequence_three_to_zero | Real | PASS |
| ST-BNDRY-006-002 | FR-014a | death_timer 90 帧精确 | t23_bndry_edge_death_timer_90_frames_exactly | Real | PASS |
| ST-BNDRY-006-003 | FR-014a | kill_y 边界判定 (> / <) | t25_bndry_edge_y_greater_than_kill_y_triggers_pit_fall, t26_bndry_edge_y_less_than_kill_y_is_safe | Real | PASS |
| ST-BNDRY-006-004 | FR-014b | invuln_timer 120 帧精确 | t24_bndry_edge_invuln_timer_120_frames_exactly | Real | PASS |
| ST-BNDRY-006-005 | FR-014b | 重生位置精确匹配 | t27_bndry_edge_checkpoint_position_exact_copy | Real | PASS |
| ST-BNDRY-006-006 | FR-015 | 旗杆碰撞边界接触/1px 间隙 | t28_bndry_edge_flagpole_edge_contact_counts_as_overlap, t29_bndry_edge_one_pixel_gap_from_flagpole_no_trigger | Real | PASS |
| ST-BNDRY-006-007 | FR-014b | 闪烁相位 4Hz 阈值 | t30_bndry_edge_flicker_phase_4hz_toggle_at_boundary | Real | PASS |
| ST-BNDRY-006-008 | FR-014c | lives=0 不溢出 | t32_bndry_edge_lives_zero_no_underflow | Real | PASS |
| ST-BNDRY-006-009 | FR-014c, FR-015 | 完全重置全部字段 | t33_bndry_edge_full_reset_from_game_over_all_fields, t34_bndry_edge_victory_reset_identical_to_gameover_reset, t35_bndry_edge_life_state_reset_all_fields | Real | PASS |
| ST-BNDRY-006-010 | FR-014c | 提示闪烁 2Hz 阈值 | t31_bndry_edge_blink_phase_2hz_toggle_at_boundary | Real | PASS |
| ST-UI-006-001 | FR-014c | UI: Game Over 叠加层渲染逻辑 | t36_ui_render_game_over_overlay_full_viewport, t37_ui_render_game_over_title_position_and_style, t38_ui_render_press_space_prompt_visible_at_correct_time | Real | PASS |
| ST-UI-006-002 | FR-014c | UI: 提示闪烁 50% 占空比 | t39_ui_render_blink_50_percent_duty_cycle_over_2_seconds | Real | PASS |
| ST-UI-006-003 | FR-015 | UI: Victory 叠加层渲染逻辑 | t40_ui_render_victory_overlay_renders_correct_content, t41_ui_render_victory_title_gold_color | Real | PASS |
| ST-UI-006-004 | FR-014a, FR-014b, FR-015 | UI: 死亡动画+无敌闪烁+旗杆滑下 | t42_ui_render_death_bounce_animation, t43_ui_render_death_fall_fade_alpha_decreases, t44_ui_render_invulnerability_flicker_4hz, t45_ui_render_flagpole_slide_animation_player_position | Real | PASS |
| ST-UI-006-005 | FR-014c, FR-015 | UI: 叠加层 z-order | t46_ui_render_overlay_z_order_above_game_elements | Real | PASS |
| ST-FUNC-006-015 | FR-014a | INTG: PlayerStats 实时反映 lives | t47_intg_player_lives_decrement_reflected_in_player_stats | Real | PASS |
| ST-FUNC-006-016 | FR-014a | INTG: 多危险事件去重 | t48_intg_physics_multiple_hazard_events_single_death | Real | PASS |
| ST-FUNC-006-017 | FR-014a | INTG: Level bounds 提供 kill_y | t49_intg_level_kill_y_from_level_bounds_triggers_death | Real | PASS |

> 结果 valid values: `PENDING`, `PASS`, `FAIL`, `MANUAL-PASS`, `MANUAL-FAIL`, `BLOCKED`, `PENDING-MANUAL`

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 32 |
| Passed | 32 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` — must be fixed and re-executed.

---

> **Note on UI Layer Detection Warnings**: The validation script reports missing Layer 1/2/3 detection for UI test cases.
> This is expected behavior — Feature #6 is a native Macroquad desktop application (not browser-based), so Chrome DevTools
> MCP tools (`navigate_page`, `evaluate_script`, `list_console_messages`, `take_snapshot`) are not applicable per
> env-guide.md §5. UI render logic tests (ST-UI-006-001 through ST-UI-006-005) validate render-driving calculations
> (positions, colors, alpha values, timer phases) through automated `cargo test` assertions. These tests validate
> the Visual Rendering Contract parameters that drive the actual GPU rendering pipeline, which is the appropriate
> approach for a native desktop game application where the rendering output is a canvas 2D surface, not DOM/CSS.

> **Note on Manual Visual Verification**: Feature #6 is a native Macroquad desktop application (not browser-based).
> Per env-guide.md §5, Chrome DevTools MCP is not applicable. UI/render tests (ST-UI-006-001 through ST-UI-006-005)
> validate render-driving logic (state values, timer calculations, position calculations, color constants)
> through automated `cargo test` assertions. Pixel-level visual quality (UCD compliance of Game Over / Victory overlays)
> is deferred to System ST where the game binary is launched and screenshots are taken for manual comparison.
> SRS traceability requirement for FR-014c/FR-015 manually verified visual output is documented here for completeness
> but falls outside the scope of Feature-ST automated execution.
