# 测试用例集: Engine Core

**Feature ID**: 1
**关联需求**: FR-018
**日期**: 2026-05-31
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 8 |
| boundary | 4 |
| performance | 1 |
| ui | 0 |
| security | 0 |
| **合计** | **13** |

---

### 用例编号

ST-FUNC-001-001

### 关联需求

FR-018（Fixed-Timestep Game Loop）— AC-1: 60 steps/sec

### 测试目标

验证游戏循环在 1 秒真实时间内恰好执行 60 次固定步长更新（dt = 1/60s）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop` 构造成功（1280x720 窗口配置）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `WindowConfig { width: 1280, height: 720, fullscreen: false }` | 创建配置成功 |
| 2 | `GameLoop::new(config)` | 返回 `Ok(GameLoop)`，`accumulator == 0.0`，`dt == 1.0/60.0` |
| 3 | 以 `frame_time=1/60s` 驱动 60 帧（`tick(DT, state)` x 60） | 每帧累加并消耗 1 步，总共 1.0s 真实时间 |
| 4 | 断言 `state.update_count == 60` | 恰好 60 次 `update(dt)` 调用 |
| 5 | 遍历 `state.update_dt_values`，断言每个 `dt == 1.0/60.0 +- epsilon` | 所有更新的 `dt` 值恒为固定步长 |
| 6 | 断言 `state.render_count == 60` | 60 帧各产生 1 次 `render()` 调用 |

### 验证点

- 1 秒真实时间 = 60 次固定步长模拟更新
- 每次 `update()` 收到的 `dt` 参数一致（固定步长不变性）
- 每帧恰好 1 次 `render()`，与 update 次数解耦

### 后置检查

- 无残留状态；累加器在测试结束后可被丢弃

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t1_fixed_timestep_60_steps_per_second`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-002

### 关联需求

FR-018（Fixed-Timestep Game Loop）— AC-2: Frame rate independence

### 测试目标

验证物理模拟速度不受渲染帧率波动影响：注入可变帧间隔序列（0.008s, 0.025s, 0.033s, 0.012s），运行总真实时间约 1s，物体位移接近 100px（vx=100 px/s，epsilon=0.5 px）。

### 前置条件

- Rust 工具链已安装
- `GameLoop` 构造成功，`SpyStateMachine` 以 `vx = 100.0` 模拟匀速运动

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `GameLoop::new(1280x720 window)` | 初始化成功 |
| 2 | 注入可变帧序列 `[0.008, 0.025, 0.033, 0.012]` x 13 周期 | 总真实时间约 1.014s |
| 3 | 断言 `state.position in [99.5, 101.7]` | 物体位移 ~ 100 px（不因帧率波动而漂移） |
| 4 | 验证所有 `state.update_dt_values[i] == 1/60 +- 1e-5` | `dt` 恒定，可变帧时间未泄漏至模拟步长 |
| 5 | 验证 `state.render_count == 52`（13 周期 x 4 帧/周期） | 每帧恰好 1 次 render |

### 验证点

- 帧率波动不改变物理模拟速度：总位移 ~ 100 px
- 模拟步长 `dt` 始终为 1/60s，不受帧时间变化影响
- 渲染次数等于帧数（独立于每帧 update 次数）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t2_frame_rate_independent_motion`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-003

### 关联需求

FR-018（Fixed-Timestep Game Loop）— AC-4: Accumulator initialization

### 测试目标

验证 `GameLoop::new()` 正确初始化所有字段：`accumulator = 0.0`，`dt = 1.0/60.0`，`max_steps = 5`。

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `GameLoop::new(WindowConfig { 1280, 720, false })` | 返回 `Ok(GameLoop)` |
| 2 | 断言 `accumulator ~ 0.0 +- 1e-5` | 累加器初始化为 0 |
| 3 | 断言 `dt ~ 1.0/60.0 +- 1e-5` | 固定步长常量 |
| 4 | 断言 `max_steps == 5` | 最大追赶步数 |

### 验证点

- 累加器从 0 开始，保证首帧无多余步进
- `dt` 与 `max_steps` 均为硬编码常量，不可变
- 构造函数不 panic、不返回错误（有效输入下）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t3_game_loop_constructor_initializes_correctly`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-004

### 关联需求

FR-018（Fixed-Timestep Game Loop）— IAPI-011: apply_display happy path

### 测试目标

验证 `apply_display(w, h, fullscreen)` 在传入支持的分辨率时正确更新 `WindowConfig`。

### 前置条件

- `GameLoop` 以 1280x720 窗口模式构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `gl.apply_display(1920, 1080, false)` | 窗口配置更新：`width=1920, height=1080, fullscreen=false` |
| 2 | 分别断言 `config.width`, `config.height`, `config.fullscreen` | 全部匹配新值 |
| 3 | `gl.apply_display(2560, 1440, true)` | 窗口配置更新：`width=2560, height=1440, fullscreen=true` |
| 4 | 再次断言三个字段 | 全部匹配第二次设置的新值 |

### 验证点

- 支持的分辨率（1280x720 / 1920x1080 / 2560x1440）均被接受
- `fullscreen` 标志正确切换
- 窗口配置字段即时更新

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t4_apply_display_supported_resolution`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-005

### 关联需求

FR-018（Fixed-Timestep Game Loop）— IAPI-011: apply_display error path

### 测试目标

验证 `apply_display()` 对不支持的分辨率（800x600）保持静默 no-op：不 panic、不修改配置。

### 前置条件

- `GameLoop` 以 1280x720 构造成功，记录原始配置

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 记录原始 `width`, `height`, `fullscreen` | 保存基线值 |
| 2 | `gl.apply_display(800, 600, true)` | 不 panic，不返回错误 |
| 3 | 断言 `width == 原始宽度` | 未变更 |
| 4 | 断言 `height == 原始高度` | 未变更 |
| 5 | 断言 `fullscreen == 原始全屏状态` | 未变更 |

### 验证点

- 不支持的分辨率被静默忽略（非 panic / 非 Err）
- 所有窗口配置字段保持不变
- 符合设计 S4 的 error code 行为："记录警告并忽略"

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t5_apply_display_unsupported_resolution_is_noop`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-006

### 关联需求

FR-018（Fixed-Timestep Game Loop）— AC-3: Max catch-up guard

### 测试目标

验证单帧 extreme frame_time（0.2s）时系统最多执行 5 步 update，超出部分的时间被丢弃。

### 前置条件

- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `gl.tick(0.2, state)` — 注入 0.2s 极端帧时间 | 累加器被钳制至 `dt*(max_steps+1) ~ 0.1s` |
| 2 | 断言 `update_count == 5` | 恰好执行 5 次 update（非 12 次，非 4 次） |
| 3 | 断言 `accumulator < DT + epsilon` | 剩余累积时间被限制在 1 步以内 |
| 4 | 断言 `render_count == 1` | 单帧内仍仅一次 render |

### 验证点

- 累加器钳制防止螺旋死亡：0.2s 钳制至 ~0.1s
- max_steps=5 硬限制：不多不少恰好 5 步
- 超出的时间被丢弃，不影响后续帧

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t6_spike_frame_clamped_to_max_5_steps`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-007

### 关联需求

FR-018（Fixed-Timestep Game Loop）— AC-2: Render per frame independence

### 测试目标

验证 `render()` 每帧恰好调用 1 次，与 `update()` 调用次数（0/1/2）无关。

### 前置条件

- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | Frame 1: `tick(0.035, state)` 导致 2 次 update | `update_count = 2`, `render_count = 1` |
| 2 | Frame 2: `tick(0.018, state)` 导致 1 次 update | `update_count = 3`, `render_count = 2` |
| 3 | Frame 3: `tick(0.001, state)` 导致 0 次 update | `update_count = 3`（不变），`render_count = 3` |
| 4 | 断言总计: `update_count == 3`, `render_count == 3` | render 不因 update=0 而被跳过 |

### 验证点

- render 不在 while 循环内部（否则每步 update 会多一次 render）
- 0 次 update 的帧仍执行 render（避免黑屏首帧）
- 总 render 次数恒等于帧数

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t12_render_once_per_frame_regardless_of_update_count`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-001-008

### 关联需求

FR-018（Fixed-Timestep Game Loop）— IAPI-011: WindowError::UnsupportedResolution

### 测试目标

验证 `GameLoop::new()` 和 `GameWindow::new()` 对不支持的分辨率返回 `WindowError::UnsupportedResolution`。

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `GameLoop::new(WindowConfig { 800, 600, false })` | 返回 `Err(WindowError::UnsupportedResolution)` |
| 2 | `GameLoop::new(WindowConfig { 1024, 768, false })` | 返回 `Err(WindowError::UnsupportedResolution)` |
| 3 | 验证错误不 panic — 正常 `Result::Err` 返回 | 调用方可处理错误 |

### 验证点

- 构造函数对不支持分辨率返回 Err，不 panic
- 错误类型匹配 `WindowError::UnsupportedResolution`
- 三种支持分辨率（1280x720 / 1920x1080 / 2560x1440）之外的输入均被拒绝

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t_st_func_001_008_unsupported_resolution_error`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-001-001

### 关联需求

FR-018（Fixed-Timestep Game Loop）— Boundary: accumulator at dt edge

### 测试目标

验证累加器恰好等于 dt 时（边界情况），系统执行恰好 1 次 update，累加器归零，alpha=0.0。

### 前置条件

- `GameLoop` 构造成功，手动设置 `accumulator = DT`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 手动 `gl.accumulator = DT` | 累加器设为恰好边界值 |
| 2 | `gl.tick(0.0, state)` — 不再追加时间 | 仅消耗现有累加器 |
| 3 | 断言 `update_count == 1` | 累加器 >= dt，执行恰好 1 步 |
| 4 | 断言 `accumulator ~ 0.0 +- 1e-5` | 完全消耗，无浮点余数累积 |
| 5 | 断言 `alpha ~ 0.0` — 首个 render_alpha | 插值因子为 0（恰在步进边界） |

### 验证点

- 比较运算符使用 `>=`（非 `==`，否则边界跳步）
- 减法 `-= dt` 后无微小浮点漂移
- alpha 在边界处为 0.0

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t7_accumulator_exactly_at_dt_boundary`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-001-002

### 关联需求

FR-018（Fixed-Timestep Game Loop）— Boundary: frame_time = 0

### 测试目标

验证 frame_time = 0 时（首帧或计时器异常），系统执行 0 次 update 但仍渲染 1 次（alpha=0.0），不发生除零/NaN/panic。

### 前置条件

- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `gl.tick(0.0, state)` | 不 panic，不产生 NaN |
| 2 | 断言 `update_count == 0` | 无时间累积，无更新 |
| 3 | 断言 `render_count == 1` | 仍然渲染（避免黑屏） |
| 4 | 断言 `alpha == 0.0` | 插值因子正确 |

### 验证点

- frame_time=0 不触发除零错误
- 0 次 update 时渲染不跳过
- 系统稳定，后续帧可正常继续

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t8_zero_frame_time_produces_no_update_still_renders`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-001-003

### 关联需求

FR-018（Fixed-Timestep Game Loop）— Boundary: spiral death clamp

### 测试目标

验证极端掉帧恢复（frame_time=10s，模拟系统挂起）时累加器被钳制，单帧最多 5 步 update，系统恢复后可正常继续。

### 前置条件

- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `gl.tick(10.0, state)` — 注入 10s 挂起恢复 | 累加器被钳制至 `dt*(max_steps+1) ~ 0.1s` |
| 2 | 断言 `update_count == 5` | 恰好 5 步（非 600 步，非 4 步） |
| 3 | 断言 `accumulator < dt*(max_steps+1)` | 钳制生效，超限时间被丢弃 |
| 4 | `gl.tick(DT, state)` — 下一帧正常驱动 | `update_count > 5`（正常恢复继续） |

### 验证点

- 钳制阈值公式：`dt * (max_steps + 1)` 而非 `dt * max_steps`
- 系统未死锁：恢复后可继续处理后续帧
- 超出的累积时间被丢弃，不累积到下一帧

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t9_spiral_death_recovery_clamps_accumulator`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-001-004

### 关联需求

FR-018（Fixed-Timestep Game Loop）— Boundary: dt immutability

### 测试目标

验证 `dt` 在 360 帧运行期间恒为常量 `1.0/60.0`，不被可变帧时间覆盖或意外修改。

### 前置条件

- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 用可变帧序列 `[0.005, 0.010, 0.016667, 0.020, 0.025, 0.033]` x 60 驱动 `tick` | 共 360 帧 |
| 2 | 每周期定点检查 `gl.dt ~ 1.0/60.0 +- 1e-5` | 字段值不变 |
| 3 | 遍历所有 `state.update_dt_values[i]`，断言 `~ 1/60` | 传入 update 的每个 dt 值均为常量 |
| 4 | 断言 `render_count == 360` | 360 帧 = 360 次渲染 |

### 验证点

- `dt` 字段不可变（编译期 `const` / 不可变绑定）
- 可变帧时间未泄漏至模拟步长
- 长期运行无漂移

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t10_dt_is_immutable_constant_across_many_frames`
- **Test Type**: Real

---

### 用例编号

ST-PERF-001-001

### 关联需求

FR-018（Fixed-Timestep Game Loop）— NFR-001: Frame time budget

### 测试目标

验证空 update 循环的运行开销在预算内：360 步模拟总耗时 < 360 x (1/60)s x 1.05 = 6.3s。

### 前置条件

- Rust 工具链已安装
- `GameLoop` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 记录 `std::time::Instant::now()` | 开始计时 |
| 2 | 以 `frame_time=DT` 驱动 360 帧 `tick(DT, state)` | 模拟 6 秒游戏时间 |
| 3 | 计算 `elapsed = now.elapsed()` | 结束计时 |
| 4 | 断言 `elapsed < 6.3s`（360 x 1/60 x 1.05） | 空循环开销不超过 5% 预算 |
| 5 | 断言 `update_count == 360` | 模拟步数正确 |

### 验证点

- 固定时间步长循环本身的 CPU 开销可忽略
- 不会因迭代器/分支预测等导致异常耗时
- 为渲染和游戏逻辑留出充足的帧预算

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `tests/engine_core_test.rs::t11_empty_update_loop_performance_within_budget`
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-001-001 | FR-018 AC-1 | verification_steps[0] | t1_fixed_timestep_60_steps_per_second | Real | PASS |
| ST-FUNC-001-002 | FR-018 AC-2 | verification_steps[1] | t2_frame_rate_independent_motion | Real | PASS |
| ST-FUNC-001-003 | FR-018 AC-4 | verification_steps[3] | t3_game_loop_constructor_initializes_correctly | Real | PASS |
| ST-FUNC-001-004 | FR-018 IAPI-011 | verification_steps[2] | t4_apply_display_supported_resolution | Real | PASS |
| ST-FUNC-001-005 | FR-018 IAPI-011 | verification_steps[2] | t5_apply_display_unsupported_resolution_is_noop | Real | PASS |
| ST-FUNC-001-006 | FR-018 AC-3 | verification_steps[2] | t6_spike_frame_clamped_to_max_5_steps | Real | PASS |
| ST-FUNC-001-007 | FR-018 AC-2 | verification_steps[1] | t12_render_once_per_frame_regardless_of_update_count | Real | PASS |
| ST-FUNC-001-008 | FR-018 IAPI-011 | — | t_st_func_001_008_unsupported_resolution_error | Real | PASS |
| ST-BNDRY-001-001 | FR-018 AC-1 | — | t7_accumulator_exactly_at_dt_boundary | Real | PASS |
| ST-BNDRY-001-002 | FR-018 AC-4 | — | t8_zero_frame_time_produces_no_update_still_renders | Real | PASS |
| ST-BNDRY-001-003 | FR-018 AC-3 | — | t9_spiral_death_recovery_clamps_accumulator | Real | PASS |
| ST-BNDRY-001-004 | FR-018 AC-2 | — | t10_dt_is_immutable_constant_across_many_frames | Real | PASS |
| ST-PERF-001-001 | FR-018 AC-1, NFR-001 | — | t11_empty_update_loop_performance_within_budget | Real | PASS |

> srs_trace 4 AC 全覆盖：
> - AC-1 (60 steps/sec): ST-FUNC-001-001, ST-BNDRY-001-001, ST-PERF-001-001
> - AC-2 (frame independence): ST-FUNC-001-002, ST-FUNC-001-007, ST-BNDRY-001-004
> - AC-3 (max-5 catch-up): ST-FUNC-001-006, ST-BNDRY-001-003
> - AC-4 (accumulator init): ST-FUNC-001-003, ST-BNDRY-001-002
> - IAPI-011 (display config): ST-FUNC-001-004, ST-FUNC-001-005, ST-FUNC-001-008

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 13 |
| Passed | 13 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` — must be fixed and re-executed.
