# 测试用例集: Camera System

**Feature ID**: 4
**关联需求**: FR-013
**日期**: 2026-06-01
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 11 |
| boundary | 9 |
| ui | 0 |
| security | 0 |
| performance | 0 |
| **合计** | **20** |

---

### 用例编号

ST-FUNC-004-001

### 关联需求

FR-013（Camera Follow）— AC-1: 水平 8%/帧追赶

### 测试目标

验证摄像机在单帧内沿水平方向以剩余距离 8% 的速率向目标位置收敛。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `CameraConfig` 使用默认 SRS 参数（h_convergence=0.08, player_target_x_pct=0.375, viewport_w=480）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Camera::new(default_camera_config())`，初始 `offset = (0, 0)` | 摄像机创建成功，offset.x = 0.0, offset.y = 0.0 |
| 2 | 设置 `player_pos = (200, 100)`，`bounds = (min_x:0, max_x:2000)` | 输入数据准备完成 |
| 3 | 调用 `camera.update(player_pos, bounds, dt=1/60)` | 不 panic，正常返回 |
| 4 | 读取 `camera.offset().x` | offset.x = (200 - 480*0.375 - 0) * 0.08 = 1.6 (±0.001) |
| 5 | 验证 target_x = player.x - viewport_w * player_target_x_pct = 20.0 | 目标水平位置为 20.0 |

### 验证点

- 水平收敛率为 8%/帧（h_convergence = 0.08）
- 玩家目标位置位于视口 37.5%（player_target_x_pct = 0.375）
- 收敛方向正确（玩家在右，摄像机向右移动）

### 后置检查

- offset 值均为有限浮点数，无 NaN/Inf

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_horizontal_convergence_single_frame`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-002

### 关联需求

FR-013（Camera Follow）— AC-1: 追赶延迟不超过 80 像素

### 测试目标

验证摄像机在玩家持续水平移动时，稳态追赶延迟不超过 80 像素（FR-013 AC-1 硬性上限）。

### 前置条件

- Rust 工具链已安装
- `Camera` 初始 offset = (0, 0)，玩家从 x=200 以 200 px/s 向右移动

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Camera::new(default_config())` | offset = (0, 0) |
| 2 | 玩家每帧向右移动 200*dt = 3.333 px | 模拟最大移动速度下的持续追赶 |
| 3 | 运行 120 帧（2 秒）使摄像机达到稳态 | 摄像机进入稳态追赶状态 |
| 4 | 读取 `target_x = player.x - 480*0.375`，计算 `lag = target_x - offset.x` | lag ≤ 80.0 像素 |
| 5 | 验证 lag ≥ 0（摄像机不超前于玩家） | 摄像机落后于玩家，不会跑到玩家前方 |

### 验证点

- 稳态 lag 不超过 80 像素（FR-013 AC-1 硬性上限）
- 摄像机始终落后于移动中的玩家（不超前）
- 在最大速度（200 px/s）下，稳态 lag 约为 42 像素（理论值），远低于 80px 上限

### 后置检查

- offset 值均为有限浮点数，无 NaN/Inf

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_horizontal_convergence_steady_state_lag`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-003

### 关联需求

FR-013（Camera Follow）— AC-2: 静止时无振荡

### 测试目标

验证当玩家静止且摄像机已收敛至目标位置后，后续多帧的 offset 保持稳定不变（无振荡/漂移）。

### 前置条件

- Rust 工具链已安装
- 玩家位置固定在 (400, 100)
- 摄像机已通过 300 帧充分收敛至目标位置

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，以 `player_x=400, player_y=100` 收敛 300 帧 | offset.x ≈ 400 - 480*0.375 = 220.0 |
| 2 | 记录收敛后的 `converged_x = offset.x` | 作为后续对比基线 |
| 3 | 以相同 player_pos 运行额外 10 帧 | 每帧 offset.x 与 converged_x 差值 < 0.001 |
| 4 | 断言最终 offset.x == converged_x (±epsilon) | 摄像机完全静止，无漂移 |

### 验证点

- 收敛后 offset 不变，满足 "完全静止（无振荡）"
- 连续多帧零 delta，无浮点误差累积导致的漂移

### 后置检查

- 摄像机状态保持稳定

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_no_oscillation_when_converged`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-004

### 关联需求

FR-013（Camera Follow）— AC-5a: 垂直死区上方追踪

### 测试目标

验证当玩家 Y 坐标超出垂直死区上边界时，摄像机以 5%/帧速率向上追赶玩家。

### 前置条件

- Rust 工具链已安装
- Camera offset.y = 0, viewport_h = 270
- 死区上边界 = offset.y + viewport_h * 0.2 = 54

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，offset.y = 0 | 初始垂直偏移为 0 |
| 2 | 设置 `player_pos = (200, 30)`（y=30 < 死区上边界 54） | 玩家在死区上方 |
| 3 | 调用 `camera.update(player_pos, bounds, dt=1/60)` | 触发垂直追踪 |
| 4 | 计算 expected: target_y = 30 - 270/2 = -105; delta = (-105 - 0) * 0.05 = -5.25 | offset.y = 0 + (-5.25) = -5.25 (±0.001) |
| 5 | 验证 offset.y 确实变化（非零 delta） | 摄像机向上追踪，offset.y 减小 |

### 验证点

- 玩家在死区上方时，垂直收敛率为 5%/帧（v_convergence = 0.05）
- target_y = player.y - viewport_h / 2（玩家居中于视口）
- offset.y 向下（负方向）移动以跟踪上方的玩家

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_vertical_tracking_above_dead_zone`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-005

### 关联需求

FR-013（Camera Follow）— AC-5a: 垂直死区下方追踪

### 测试目标

验证当玩家 Y 坐标超出垂直死区下边界时，摄像机以 5%/帧速率向下追赶玩家。

### 前置条件

- Rust 工具链已安装
- Camera offset.y = 0, viewport_h = 270
- 死区下边界 = offset.y + viewport_h * 0.8 = 216

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，offset.y = 0 | 初始垂直偏移为 0 |
| 2 | 设置 `player_pos = (200, 250)`（y=250 > 死区下边界 216） | 玩家在死区下方 |
| 3 | 调用 `camera.update(player_pos, bounds, dt=1/60)` | 触发垂直追踪 |
| 4 | 计算 expected: target_y = 250 - 270/2 = 115; delta = (115 - 0) * 0.05 = 5.75 | offset.y = 0 + 5.75 = 5.75 (±0.001) |
| 5 | 验证 offset.y 确实变化（非零 delta） | 摄像机向下追踪，offset.y 增大 |

### 验证点

- 玩家在死区下方时，垂直收敛率为 5%/帧（v_convergence = 0.05）
- target_y = player.y - viewport_h / 2（玩家居中于视口）
- offset.y 向下（正方向）移动以跟踪下方的玩家

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_vertical_tracking_below_dead_zone`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-006

### 关联需求

FR-013（Camera Follow）— AC-5b: 死区内垂直静止

### 测试目标

验证当玩家 Y 坐标在垂直死区范围内时，摄像机垂直位置保持不变。

### 前置条件

- Rust 工具链已安装
- Camera offset.y = 0, viewport_h = 270
- 死区范围 = [0 + 270*0.2, 0 + 270*0.8] = [54, 216]

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，offset.y = 0 | 初始垂直偏移为 0 |
| 2 | 设置 `player_pos = (200, 135)`（y=135 在死区 [54, 216] 内） | 玩家在死区中心 |
| 3 | 运行 30 帧，每帧调用 `camera.update(player_pos, bounds, dt=1/60)` | 不 panic |
| 4 | 验证最终 offset.y == 0.0 (±0.001) | 垂直偏移完全不变 |
| 5 | 验证所有 30 帧 offset.y 无任何变化 | 死区内不触发垂直追踪 |

### 验证点

- 玩家在垂直死区中央 60% 范围内时，摄像机垂直不移动
- 连续 30 帧零垂直变化，无微幅漂移
- 死区机制有效避免因小幅垂直移动（如平台起伏）导致的画面晃动

### 后置检查

- 摄像机状态保持稳定

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_vertical_still_in_dead_zone`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-007

### 关联需求

FR-013（Camera Follow）— Interface Contract dt 防御

### 测试目标

验证当 `dt = 0` 时，`Camera::update()` 为无害空操作：offset 不变、不 panic、不产生 NaN。

### 前置条件

- Rust 工具链已安装
- Camera 已构造，player_pos = (500, 200)，bounds = (0, 2000)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，记录 old_x = offset.x, old_y = offset.y | 初始 (0, 0) |
| 2 | 调用 `camera.update(player_pos, bounds, dt=0.0)` | 不 panic，正常返回 |
| 3 | 验证 offset.x == old_x (±0.001) | offset.x 不变 |
| 4 | 验证 offset.y == old_y (±0.001) | offset.y 不变 |
| 5 | 验证 offset 值为有限浮点数 | 无 NaN/Inf |

### 验证点

- dt=0 时完全不改变摄像机状态（no-op）
- 无除零或 NaN 传播风险
- 与 `Player::update()` 的防御式 guard clause 风格一致

### 后置检查

- 摄像机状态保持初始值

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_dt_zero_no_op`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-008

### 关联需求

FR-013（Camera Follow）— Interface Contract dt 防御

### 测试目标

验证当 `dt < 0`（负时间步）时，`Camera::update()` 为无害空操作：offset 不变、不反向移动、不 panic。

### 前置条件

- Rust 工具链已安装
- Camera 已构造，player_pos = (500, 200)，bounds = (0, 2000)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，记录 old_x = offset.x, old_y = offset.y | 初始 (0, 0) |
| 2 | 调用 `camera.update(player_pos, bounds, dt=-0.016)` | 不 panic，正常返回 |
| 3 | 验证 offset.x == old_x (±0.001) | offset.x 不变（不反向移动） |
| 4 | 验证 offset.y == old_y (±0.001) | offset.y 不变 |
| 5 | 验证 offset 值为有限浮点数 | 无 NaN/Inf |

### 验证点

- 负 dt 不导致摄像机反向移动
- 负 dt 不 panic
- 防御式编程，与 Player::update() 风格一致

### 后置检查

- 摄像机状态保持初始值

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_dt_negative_no_op`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-009

### 关联需求

FR-013（Camera Follow）— IAPI-007: Player::pos() 集成

### 测试目标

验证 Camera 通过 IAPI-007 正确消费 `Player::pos()` 返回的世界坐标，水平目标计算和垂直死区判定均基于真实 Player 位置。

### 前置条件

- Rust 工具链已安装
- Player 以默认配置构造（初始位置 x=100, y=100）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Player::new(default_config())` → 读取 `player.pos()` | pos = (100, 100)，验证 IAPI-007 返回值 |
| 2 | `Camera::new(default_config())` → `camera.update(player.pos(), bounds, dt)` | 使用真实 Player 位置调用 update |
| 3 | 验证 target_x = 100 - 480*0.375 = -80; delta = (-80 - 0) * 0.08 = -6.4 | offset.x = -6.4 (±0.001) |
| 4 | 验证 player.y=100 在死区[54, 216]内 | offset.y = 0.0（无垂直追踪） |

### 验证点

- Camera 正确读取 Player::pos().x 用于水平目标计算
- Camera 正确读取 Player::pos().y 用于垂直死区判定
- IAPI-007 接口 schema（Vec2 { x, y }）与 Camera::update() 的 player_pos 参数匹配

### 后置检查

- Camera 和 Player 实例可独立释放

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_reads_player_position_via_iapi_007`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-010

### 关联需求

FR-013（Camera Follow）— IAPI-008: Level::bounds() 集成

### 测试目标

验证 Camera 通过 IAPI-008 正确消费 `Level::bounds()` 返回的关卡边界，水平钳制生效。

### 前置条件

- Rust 工具链已安装
- Level 以默认构造（min_x=0, max_x=2000）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` → 读取 `level.bounds()` | bounds.min_x = 0.0, bounds.max_x = 2000.0 |
| 2 | `Camera::new(default_config())` | 摄像机初始 offset=(0,0) |
| 3 | 玩家 x=2500（超右边界），运行 300 帧 | offset.x 钳制至 ≤ max_x - viewport_w = 1520.0 |
| 4 | 玩家 x=-100（超左边界），运行 300 帧 | offset.x 钳制至 ≥ min_x = 0.0 |
| 5 | 验证最终 offset.x 在 [0, 1520] 范围内 | 钳制正确，不显示关卡外区域 |

### 验证点

- Camera 正确读取 Level::bounds() 的 min_x 和 max_x 用于水平钳制
- IAPI-008 接口 schema（LevelBounds { min_x, max_x, ... }）与 Camera::update() 的 bounds 参数匹配
- 左右两方向钳制均生效

### 后置检查

- Camera 和 Level 实例可独立释放

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_respects_level_bounds_via_iapi_008`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-004-011

### 关联需求

FR-013（Camera Follow）— IAPI-010: Camera::offset() 输出集成

### 测试目标

验证 ParallaxLayer 通过 IAPI-010 正确消费 `Camera::offset()` 输出，视差滚动偏移与摄像机偏移成正比。

### 前置条件

- Rust 工具链已安装
- Camera 已收敛至已知 offset，ParallaxLayer 以 speeds = [0.1, 0.3, 0.6] 构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 设置 Camera offset（玩家 x=600 收敛 300 帧） | offset 为有效 Vec2 |
| 2 | 读取 `cam_offset = camera.offset()` | offset.x 和 offset.y 均为有限值 |
| 3 | 对每层 speed ∈ [0.1, 0.3, 0.6]：`layer.update_scroll(cam_offset)` | scroll_offset.x = cam_offset.x * speed |
| 4 | 验证每层 scroll_offset.y == 0.0 | 垂直滚动始终为 0 |
| 5 | 再次读取 camera.offset()，与步骤 2 对比 | offset 不变（纯 getter，无副作用） |

### 验证点

- Camera::offset() 输出为 Vec2 { x, y }，与 IAPI-010 Response schema 一致
- ParallaxLayer::update_scroll(Vec2) 正确接收 Camera::offset() 输出
- 三层背景滚动速度分别为 0.1x, 0.3x, 0.6x
- Camera::offset() 是纯 getter，读取不改变内部状态

### 后置检查

- 摄像机状态不受读取影响

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_parallax_consumes_camera_offset_via_iapi_010`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-001

### 关联需求

FR-013（Camera Follow）— AC-3: 左边界钳制

### 测试目标

验证当玩家移动到关卡最左边界时，摄像机停止在 min_x，视口左侧不显示关卡外区域。

### 前置条件

- Rust 工具链已安装
- bounds.min_x = 0, viewport_w = 480

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，bounds.min_x = 0 | 初始 offset.x = 0 |
| 2 | 设置 player_x = 0, player_y = 100 | 玩家在关卡左边界 |
| 3 | 运行 300 帧充分收敛 | offset.x 收敛并钳制于 bounds.min_x |
| 4 | 验证 offset.x >= 0.0 - epsilon | 不显示关卡左侧外区域 |
| 5 | 验证 offset.x ≈ 0.0 (±0.001) | 停止在左边界 |

### 验证点

- 左边界钳制：offset.x >= bounds.min_x
- 钳制生效后摄像机停止在边界处
- 满足 AC-3 "左侧不显示关卡外区域"

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_left_boundary_clamp`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-002

### 关联需求

FR-013（Camera Follow）— AC-4: 右边界钳制

### 测试目标

验证当玩家移动到关卡最右边界时，摄像机停止在 max_x - viewport_w，视口右侧不显示关卡外区域。

### 前置条件

- Rust 工具链已安装
- bounds.max_x = 2000, viewport_w = 480
- max_offset = 2000 - 480 = 1520

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera | 初始 offset.x = 0 |
| 2 | 设置 player_x = 1990, player_y = 100 | 玩家接近关卡右边界 |
| 3 | 运行 300 帧充分收敛 | offset.x 收敛并钳制于 max_offset |
| 4 | 验证 offset.x <= 1520.0 + epsilon | 不显示关卡右侧外区域 |
| 5 | 验证 offset.x ≈ 1520.0 （收敛足够长时间后） | 停止在右边界 |

### 验证点

- 右边界钳制：offset.x <= bounds.max_x - viewport_w
- 钳制生效后摄像机停止在边界处
- 满足 AC-4 "右侧不显示关卡外区域"

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_right_boundary_clamp`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-003

### 关联需求

FR-013（Camera Follow）— AC-5a/5b: 死区上边界闭区间

### 测试目标

验证玩家恰好在死区上边界（y = offset.y + viewport_h * 0.2）时，视为在死区内——不触发垂直追踪。

### 前置条件

- Rust 工具链已安装
- Camera offset.y = 0, viewport_h = 270
- 死区上边界 = 0 + 270 * 0.2 = 54

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，offset.y = 0 | 初始垂直偏移为 0 |
| 2 | 设置 `player_pos = (200, 54)`（恰好在死区上边界） | 玩家在死区边界线上 |
| 3 | 运行 10 帧，每帧调用 `camera.update(player_pos, bounds, dt=1/60)` | 不 panic |
| 4 | 验证 offset.y == 0.0 (±0.001) | offset.y 不变（边界闭区间，视为在死区内） |

### 验证点

- 死区边界判定使用 <= 语义（闭区间），恰好在线上不触发追踪
- 避免 off-by-one 导致的边界行为不一致
- 边界 inclusive 检查防止在边界处的抖动

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_dead_zone_top_boundary_inclusive`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-004

### 关联需求

FR-013（Camera Follow）— AC-5a/5b: 死区下边界闭区间

### 测试目标

验证玩家恰好在死区下边界（y = offset.y + viewport_h * 0.8）时，视为在死区内——不触发垂直追踪。

### 前置条件

- Rust 工具链已安装
- Camera offset.y = 0, viewport_h = 270
- 死区下边界 = 0 + 270 * 0.8 = 216

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 Camera，offset.y = 0 | 初始垂直偏移为 0 |
| 2 | 设置 `player_pos = (200, 216)`（恰好在死区下边界） | 玩家在死区边界线上 |
| 3 | 运行 10 帧，每帧调用 `camera.update(player_pos, bounds, dt=1/60)` | 不 panic |
| 4 | 验证 offset.y == 0.0 (±0.001) | offset.y 不变（边界闭区间，视为在死区内） |

### 验证点

- 死区边界判定使用 >= 语义（闭区间），恰好在线上不触发追踪
- 避免 off-by-one 导致的边界行为不一致
- 边界 inclusive 检查防止在边界处的抖动

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_dead_zone_bottom_boundary_inclusive`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-005

### 关联需求

FR-013（Camera Follow）— AC-3: 左边界已钳制后继续左移

### 测试目标

验证当摄像机已在左边界（offset.x = min_x），玩家继续向左移动至关卡外时，offset.x 保持钳制不跟随。

### 前置条件

- Rust 工具链已安装
- bounds.min_x = 0
- 摄像机已收敛至左边界 offset.x = 0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 将 Camera 收敛至左边界（player_x=0 跑 300 帧） | offset.x ≈ 0.0 |
| 2 | 设置 player_x = -50（玩家移出关卡左侧） | 玩家在世界坐标负方向 |
| 3 | 运行 10 帧 | offset.x 保持在 0（不跟随到负值） |
| 4 | 验证 offset.x >= 0.0 - epsilon | 钳制持续生效 |

### 验证点

- 已钳制状态下进一步越界移动不突破钳制
- 钳制不是仅初次生效，而是每帧持续保护
- 满足 AC-3 "左侧不显示关卡外区域"

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_left_clamp_already_at_boundary`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-006

### 关联需求

FR-013（Camera Follow）— AC-4: 右边界已钳制后继续右移

### 测试目标

验证当摄像机已在右边界（offset.x = max_x - viewport_w），玩家继续向右移动至关卡外时，offset.x 保持钳制不跟随。

### 前置条件

- Rust 工具链已安装
- bounds.max_x = 2000, viewport_w = 480
- 摄像机已收敛至右边界 offset.x = 1520

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 将 Camera 收敛至右边界（player_x=1990 跑 300 帧） | offset.x ≈ 1520.0 |
| 2 | 设置 player_x = 2500（玩家移出关卡右侧） | 玩家在世界坐标远超右边界 |
| 3 | 运行 10 帧 | offset.x 保持在 1520（不跟随到更大值） |
| 4 | 验证 offset.x <= 1520.0 + epsilon | 钳制持续生效 |

### 验证点

- 已钳制状态下进一步越界移动不突破钳制
- 钳制不是仅初次生效，而是每帧持续保护
- 满足 AC-4 "右侧不显示关卡外区域"

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_right_clamp_already_at_boundary`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-007

### 关联需求

FR-013（Camera Follow）— AC-3/AC-4: 窄关卡（关卡宽度 < 视口宽度）

### 测试目标

验证当关卡宽度小于视口宽度（max_x < viewport_w），max_offset 为负数时，offset.x 正确钳制到 min_x 而非负值。

### 前置条件

- Rust 工具链已安装
- bounds.max_x = 300, viewport_w = 480
- max_offset = 300 - 480 = -180（负值，逻辑无效）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 bounds.max_x = 300（< viewport_w = 480） | 关卡窄于视口 |
| 2 | 设置 player_x = 150（关卡中心），运行 60 帧 | 摄像机更新正常，不 panic |
| 3 | 验证 offset.x >= 0.0 - epsilon | 钳制到 min_x，不是负值 |
| 4 | 验证 offset.x ≈ bounds.min_x = 0.0 (±0.001) | 窄关卡时摄像机锁定于 min_x |

### 验证点

- max_offset 为负时摄像机不产生负偏移
- 钳制逻辑保护：当 max_x - viewport_w < min_x 时使用 min_x
- 无 panic

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_narrow_level_clamp_to_min`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-008

### 关联需求

FR-013（Camera Follow）— Boundary Conditions: 退化边界

### 测试目标

验证当 LevelBounds 非法（min_x >= max_x）时，Camera 不 panic，offset 值保持有限。

### 前置条件

- Rust 工具链已安装
- bounds.min_x = 100, bounds.max_x = 50（退化边界，min_x >= max_x）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 bounds.min_x = 100, bounds.max_x = 50 | 逻辑无效的边界配置 |
| 2 | 调用 `camera.update(player_pos, bounds, dt=1/60)` | 不 panic，正常返回 |
| 3 | 验证 offset.x.is_finite() && offset.y.is_finite() | 无 NaN 或 Inf |
| 4 | 验证 offset.x >= bounds.min_x - epsilon | offset.x 不低于 min_x |

### 验证点

- 退化边界输入不导致 panic 或 NaN
- 防御式 clamp 逻辑：min_x 保护仍然生效
- 优雅降级——不崩溃，保持有限状态

### 后置检查

- offset 值均为有限浮点数

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_degenerate_bounds_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-004-009

### 关联需求

FR-013（Camera Follow）— AC-2: 收敛终止 epsilon

### 测试目标

验证当摄像机与目标距离极小（< 1e-5）时，收敛终止，连续 60 帧无任何浮点漂移。

### 前置条件

- Rust 工具链已安装
- 摄像机已充分收敛至目标位置

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 将 Camera 收敛至玩家 x=400 的目标（300 帧） | offset.x ≈ 220.0 |
| 2 | 验证 |offset.x - target_x| < 0.01 | 已充分收敛 |
| 3 | 连续 60 帧更新，记录每帧 offset.x 的 delta | 每帧 delta < 1e-5 |
| 4 | 验证 max_delta < 1e-5 | 收敛终止，无永不停歇的微幅漂移 |

### 验证点

- 收敛终止：距离足够小时不再产生有意义的 delta
- 无浮点累积漂移：60 帧内零有效变化
- 满足 AC-2 "无振荡" 的长期稳定性

### 后置检查

- 摄像机状态保持稳定

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/camera_system_test.rs::test_camera_convergence_epsilon_termination`
- **Test Type**: Real

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-004-001 | FR-013 AC-1 | AC-1 水平收敛 8%/帧 | test_camera_horizontal_convergence_single_frame | Real | PASS |
| ST-FUNC-004-002 | FR-013 AC-1 | AC-1 追赶延迟 ≤ 80px | test_camera_horizontal_convergence_steady_state_lag | Real | PASS |
| ST-FUNC-004-003 | FR-013 AC-2 | AC-2 静止无振荡 | test_camera_no_oscillation_when_converged | Real | PASS |
| ST-FUNC-004-004 | FR-013 AC-5a | AC-5a 死区上方追踪 | test_camera_vertical_tracking_above_dead_zone | Real | PASS |
| ST-FUNC-004-005 | FR-013 AC-5a | AC-5a 死区下方追踪 | test_camera_vertical_tracking_below_dead_zone | Real | PASS |
| ST-FUNC-004-006 | FR-013 AC-5b | AC-5b 死区内静止 | test_camera_vertical_still_in_dead_zone | Real | PASS |
| ST-FUNC-004-007 | FR-013 IC | dt=0 防御 | test_camera_dt_zero_no_op | Real | PASS |
| ST-FUNC-004-008 | FR-013 IC | dt<0 防御 | test_camera_dt_negative_no_op | Real | PASS |
| ST-FUNC-004-009 | FR-013 + IAPI-007 | IAPI-007 Player::pos() 集成 | test_camera_reads_player_position_via_iapi_007 | Real | PASS |
| ST-FUNC-004-010 | FR-013 + IAPI-008 | IAPI-008 Level::bounds() 集成 | test_camera_respects_level_bounds_via_iapi_008 | Real | PASS |
| ST-FUNC-004-011 | FR-013 + IAPI-010 | IAPI-010 Camera::offset() 集成 | test_parallax_consumes_camera_offset_via_iapi_010 | Real | PASS |
| ST-BNDRY-004-001 | FR-013 AC-3 | AC-3 左边界钳制 | test_camera_left_boundary_clamp | Real | PASS |
| ST-BNDRY-004-002 | FR-013 AC-4 | AC-4 右边界钳制 | test_camera_right_boundary_clamp | Real | PASS |
| ST-BNDRY-004-003 | FR-013 AC-5 | AC-5 死区上边界闭区间 | test_camera_dead_zone_top_boundary_inclusive | Real | PASS |
| ST-BNDRY-004-004 | FR-013 AC-5 | AC-5 死区下边界闭区间 | test_camera_dead_zone_bottom_boundary_inclusive | Real | PASS |
| ST-BNDRY-004-005 | FR-013 AC-3 | AC-3 左边界已钳制后继续左移 | test_camera_left_clamp_already_at_boundary | Real | PASS |
| ST-BNDRY-004-006 | FR-013 AC-4 | AC-4 右边界已钳制后继续右移 | test_camera_right_clamp_already_at_boundary | Real | PASS |
| ST-BNDRY-004-007 | FR-013 AC-3/4 | 窄关卡钳制 | test_camera_narrow_level_clamp_to_min | Real | PASS |
| ST-BNDRY-004-008 | FR-013 BC | 退化边界不 panic | test_camera_degenerate_bounds_no_panic | Real | PASS |
| ST-BNDRY-004-009 | FR-013 AC-2 | AC-2 收敛终止 epsilon | test_camera_convergence_epsilon_termination | Real | PASS |

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 20 |
| Passed | 20 |
| Failed | 0 |
| Pending | 0 |

## Manual Test Case Summary

> 本节不适用 —— 所有测试用例均已自动化（`已自动化: Yes`），无手工测试用例。
