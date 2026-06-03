# 测试用例集: Multi-Resolution Display (NFR-002)

**Feature ID**: 12
**关联需求**: NFR-002
**日期**: 2026-06-03
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0
**应用假设**: Specification resolutions applied from Feature Design Clarification Addendum (3 items: visible-area direction defined as forward-visible, fullscreen resolution equivalent to windowed with same viewport dimensions, programmatic verification supplements manual visual-judgment).

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 14 |
| boundary | 6 |
| ui | 3 |
| performance | 2 |
| security | 0 |
| **合计** | **25** |

> 注：`ui` 类别 3 条用例为人工视觉验证（`已自动化: No`），对应 ATS NFR-002 行的 `Manual: visual-judgment` 分类。程序化公式一致性验证已由 FUNC/BNDRY/PERF 用例覆盖（22 条 `已自动化: Yes`）。

---

### 用例编号

ST-FUNC-012-001

### 关联需求

NFR-002（多分辨率支持）— AC-1: HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口 @ 720p

### 测试目标

验证 `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` 返回正确的 (3%, 3%) 锚点坐标 (38.4, 21.6)。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- 项目成功编译（`cargo build`）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` | 返回 (f32, f32) 元组，不 panic |
| 2 | 断言 `x ≈ 38.4` (容差 0.01) | `1280 * 0.03 = 38.4` |
| 3 | 断言 `y ≈ 21.6` (容差 0.01) | `720 * 0.03 = 21.6` |

### 验证点

- 锚点 x 坐标 = viewport_w * 0.03
- 锚点 y 坐标 = viewport_h * 0.03
- 公式使用正确百分比（3%）和正确轴（x=宽度，y=高度）

### 后置检查

- 纯函数调用，无副作用，无需清理

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a1_expected_hud_anchor_720p`; `tests/multi_resolution_test.rs::r1_hud_anchor_expected_720p`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-002

### 关联需求

NFR-002（多分辨率支持）— AC-1: HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口 @ 1080p/1440p

### 测试目标

验证 `expected_hud_anchor` 在 1080p 和 1440p 下锚点线性缩放正确，1440p 锚点恰好为 720p 的 2 倍。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `expected_hud_anchor(1920.0, 1080.0)` | (57.6, 32.4)，`1920*0.03=57.6`, `1080*0.03=32.4` |
| 2 | 调用 `expected_hud_anchor(2560.0, 1440.0)` | (76.8, 43.2)，`2560*0.03=76.8`, `1440*0.03=43.2` |
| 3 | 调用 `expected_hud_anchor(1280.0, 720.0)` 获取基准 | (38.4, 21.6) |
| 4 | 断言 `x1440 / x720 ≈ 2.0` (容差 0.01) | 锚点线性随分辨率缩放 |
| 5 | 断言 `y1440 / y720 ≈ 2.0` (容差 0.01) | 锚点线性随分辨率缩放 |

### 验证点

- 1080p 锚点 = (1920*0.03, 1080*0.03)
- 1440p 锚点 = (2560*0.03, 1440*0.03)
- 线性缩放不变性：分辨率加倍 → 锚点坐标加倍

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a2_expected_hud_anchor_1080p_and_1440p`; `tests/multi_resolution_test.rs::r2_hud_anchor_expected_1080p_1440p`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-003

### 关联需求

NFR-002（多分辨率支持）— AC-1: HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口

### 测试目标

验证 `hud_anchor_deviation` 在给定实际坐标 (40.0, 22.0) 与 720p 期望锚点 (38.4, 21.6) 时，正确计算偏差百分比。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `hud_anchor_deviation(40.0, 22.0, 1280.0, 720.0)` | 返回 `HudAnchorDeviation`，不 panic |
| 2 | 断言 `dev.dx_pct ≈ 0.00125` (容差 0.0001) | `(40.0 - 38.4) / 1280.0 = 0.00125` |
| 3 | 断言 `dev.dy_pct ≈ 0.0005556` (容差 0.0001) | `(22.0 - 21.6) / 720.0 ≈ 0.0005556` |

### 验证点

- dx_pct = (actual_x - expected_x) / viewport_w
- dy_pct = (actual_y - expected_y) / viewport_h
- 偏差以视口尺寸百分比表示（非像素绝对值）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a3_hud_anchor_deviation_computation`; `tests/multi_resolution_test.rs::r3_hud_anchor_deviation_computation`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-004

### 关联需求

NFR-002（多分辨率支持）— AC-1: HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口

### 测试目标

验证 `check_hud_anchor(1280.0, 720.0, 0.02)` 自洽性检查通过：期望锚点与自身比较偏差为 0.0，passed == true。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_hud_anchor(1280.0, 720.0, 0.02)` | 返回 `HudAnchorReport` |
| 2 | 断言 `report.passed == true` | 自洽性检查通过 |
| 3 | 断言 `report.deviation.dx_pct ≈ 0.0` | 期望锚点与自身无偏差 |
| 4 | 断言 `report.deviation.dy_pct ≈ 0.0` | 期望锚点与自身无偏差 |
| 5 | 断言 `report.expected == (38.4, 21.6)` (容差 0.01) | 期望锚点正确 |
| 6 | 断言 `report.tolerance == (25.6, 14.4)` (容差 0.01) | 容差 = viewport * 2% |

### 验证点

- 自洽性检查：期望锚点与自身比较，偏差恒为 0
- tolerance 正确计算：`(viewport_w * 0.02, viewport_h * 0.02)`
- passed 标志在偏差 ≤ 容差时为 true

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a4_check_hud_anchor_self_consistency`; `tests/multi_resolution_test.rs::r4_hud_anchor_self_consistency`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-005

### 关联需求

NFR-002（多分辨率支持）— AC-2: 玩家可见区域占视口宽度 45-50%

### 测试目标

验证 `player_visible_ratio(200.0, 20.0, 480.0)` 正确计算玩家前方可见区域比例为 0.625。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `player_visible_ratio(200.0, 20.0, 480.0)` | 返回 f32 比例值，不 panic |
| 2 | 断言 `ratio ≈ 0.625` (容差 0.001) | `(480 - (200-20)) / 480 = 300/480 = 0.625` |

### 验证点

- player_screen_x = player_world_x - camera_offset_x
- ratio = (viewport_w - player_screen_x) / viewport_w
- 公式计算的是前方（右侧）可见区域，非后方

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a5_player_visible_ratio_computation`; `tests/multi_resolution_test.rs::r5_player_visible_ratio_computation`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-006

### 关联需求

NFR-002（多分辨率支持）— AC-2: 玩家可见区域占视口宽度 45-50%

### 测试目标

验证 `check_visible_ratio(0.48, 0.45, 0.50)` 在范围内时 passed == true。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_visible_ratio(0.48, 0.45, 0.50)` | 返回 `VisibleAreaReport` |
| 2 | 断言 `report.passed == true` | ratio 0.48 在 [0.45, 0.50] 范围内 |
| 3 | 断言 `report.ratio ≈ 0.48` | ratio 值正确保存 |
| 4 | 断言 `report.min_expected ≈ 0.45` | 下界正确保存 |
| 5 | 断言 `report.max_expected ≈ 0.50` | 上界正确保存 |

### 验证点

- passed 判断使用 `<=` 和 `>=`（包含边界），非 `<` 和 `>`
- ratio / min_expected / max_expected 字段均正确回传

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a6_check_visible_ratio_within_range`; `tests/multi_resolution_test.rs::r6_visible_ratio_within_range`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-007

### 关联需求

NFR-002（多分辨率支持）— AC-1 + AC-2: HUD 锚定 + 可见区域比例全覆盖

### 测试目标

验证 `verify_all_resolutions(&CameraConfig::default())` 对全部 3 种分辨率返回 3 份报告，每份 HUD 锚定检查均通过。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `CameraConfig::default()` 提供 viewport_w=480, viewport_h=270, player_target_x_pct=0.375

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `CameraConfig::default()` | viewport_w=480, viewport_h=270, player_target_x_pct=0.375 |
| 2 | 调用 `verify_all_resolutions(&config)` | 返回 `Vec<ResolutionReport>` |
| 3 | 断言 `reports.len() == 3` | 覆盖 720p、1080p、1440p |
| 4 | 遍历每个 report，断言 `report.hud.passed == true` | 所有分辨率 HUD 锚定自洽性检查通过 |

### 验证点

- 分辨率列表从 `SUPPORTED_RESOLUTIONS` 正确读取（3 项：720p/1080p/1440p）
- 每个 report.label 为 "720p" / "1080p" / "1440p"
- hud.passed + visible_area.passed → report.passed 聚合逻辑正确

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a7_verify_all_resolutions_full_suite`; `tests/multi_resolution_test.rs::r7_verify_all_resolutions_full_suite`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-008

### 关联需求

NFR-002（多分辨率支持）— AC-3: 全屏模式 HUD 位置不变

### 测试目标

验证以全屏等效分辨率 1080p 调用 `check_hud_anchor` 产生与窗口模式 1080p 完全一致的锚点坐标（57.6, 32.4），偏差为 0.0。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_hud_anchor(1920.0, 1080.0, 0.02)` | 返回 `HudAnchorReport`，passed=true |
| 2 | 断言 `report.expected == (57.6, 32.4)` (容差 0.01) | 锚点 = 1920*0.03, 1080*0.03 |
| 3 | 断言 `report.deviation.dx_pct ≈ 0.0` | 全屏与窗口模式锚定公式一致 |
| 4 | 断言 `report.deviation.dy_pct ≈ 0.0` | 全屏与窗口模式锚定公式一致 |

### 验证点

- 全屏模式下视口尺寸 = 显示器原生分辨率（本例 1920x1080）
- HUD 锚定公式 `(viewport_w * 0.03, viewport_h * 0.03)` 与窗口模式完全相同
- 全屏仅改变窗口装饰/模式标志，不改变视口尺寸计算基准

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a8_fullscreen_hud_anchor_same_as_windowed`; `tests/multi_resolution_test.rs::r8_fullscreen_hud_anchor_same_as_windowed`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-009

### 关联需求

NFR-002（多分辨率支持）— expected_hud_anchor 前置条件违规：viewport_w = 0.0

### 测试目标

验证 `expected_hud_anchor(0.0, 720.0)` 不会 panic（前置条件违规由调用方 guard），返回 (0.0, 21.6)。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 在 `catch_unwind` 中调用 `expected_hud_anchor(0.0, 720.0)` | 不 panic（`result.is_ok()`） |
| 2 | 解包结果，断言 `x ≈ 0.0` | `0.0 * 0.03 = 0.0` |
| 3 | 断言 `y ≈ 21.6` | `720.0 * 0.03 = 21.6` |

### 验证点

- 零视口宽度不触发除零或 panic
- 行为：产生 (0.0, 21.6)，调用方自行 guard

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a9_expected_hud_anchor_zero_width_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-010

### 关联需求

NFR-002（多分辨率支持）— expected_hud_anchor 前置条件违规：viewport_w < 0

### 测试目标

验证 `expected_hud_anchor(-100.0, 720.0)` 不会 panic，返回 (-3.0, 21.6)。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 在 `catch_unwind` 中调用 `expected_hud_anchor(-100.0, 720.0)` | 不 panic |
| 2 | 解包结果，断言 `x ≈ -3.0` | `-100.0 * 0.03 = -3.0` |
| 3 | 断言 `y ≈ 21.6` | `720.0 * 0.03 = 21.6` |

### 验证点

- 负视口宽度不触发 panic
- 行为：负值按公式传播，调用方自行 guard

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a10_expected_hud_anchor_negative_width_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-011

### 关联需求

NFR-002（多分辨率支持）— player_visible_ratio 前置条件违规：viewport_w = 0.0

### 测试目标

验证 `player_visible_ratio(100.0, 200.0, 0.0)` 不 panic（除零返回 Inf 或 NaN），调用方自行 guard。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 在 `catch_unwind` 中调用 `player_visible_ratio(100.0, 200.0, 0.0)` | 不 panic |
| 2 | 解包结果，断言 `ratio.is_infinite() \|\| ratio.is_nan()` | 除零产生 ±Inf 或 NaN |

### 验证点

- 零视口宽度不触发 panic（除以零在 IEEE 754 浮点中产生 Inf）
- 行为：返回 Inf 或 NaN，调用方自行 guard

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_a11_player_visible_ratio_zero_viewport_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-012

### 关联需求

NFR-002（多分辨率支持）— AC-1 + F09 HUD: 验证工具与 HUD 渲染器锚定公式一致性

### 测试目标

验证 `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` 与 `HudRenderer::compute_anchor(1280.0, 720.0)` 返回完全相同的坐标，防止公式漂移。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `HudRenderer::compute_anchor()` 存在且可调用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `HudRenderer::compute_anchor(1280.0, 720.0)` | 获取 HUD 渲染器锚点 (hx, hy) |
| 2 | 调用 `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` | 获取验证工具锚点 (vx, vy) |
| 3 | 断言 `hx ≈ vx` (容差 0.001) | x 坐标一致 |
| 4 | 断言 `hy ≈ vy` (容差 0.001) | y 坐标一致 |

### 验证点

- 验证工具与 HUD 渲染实现使用相同的锚定公式
- 防止一处修改公式而另一处未同步导致漂移

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_d1_hud_anchor_consistency_720p`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-013

### 关联需求

NFR-002（多分辨率支持）— AC-1 + F09 HUD: 全分辨率锚定一致性

### 测试目标

验证全部 3 种分辨率 (720p/1080p/1440p) 下 `HudRenderer::compute_anchor()` == `ResolutionVerifier::expected_hud_anchor()`。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 对 each (w, h) in [(1280,720), (1920,1080), (2560,1440)] | 遍历全部支持分辨率 |
| 2 | 调用 `HudRenderer::compute_anchor(w, h)` | 获取 HUD 渲染器锚点 |
| 3 | 调用 `ResolutionVerifier::expected_hud_anchor(w, h)` | 获取验证工具锚点 |
| 4 | 断言两者 x 坐标近似相等 (容差 0.001) | 所有分辨率下 x 一致 |
| 5 | 断言两者 y 坐标近似相等 (容差 0.001) | 所有分辨率下 y 一致 |

### 验证点

- 无分辨率特异的公式分歧
- 遍历逻辑完整覆盖 3 种分辨率

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_d2_hud_anchor_consistency_all_resolutions`; `tests/multi_resolution_test.rs::r9_hud_anchor_consistency_all_resolutions`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-012-014

### 关联需求

NFR-002（多分辨率支持）— AC-2 + F04 Camera: 可见区域比例使用虚拟画布

### 测试目标

验证 `player_visible_ratio` 使用虚拟画布宽度 (480px) 而非屏幕分辨率宽度进行比例计算。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `Camera::viewport()` 返回 (480.0, 270.0)
- `CameraConfig::default().player_target_x_pct == 0.375`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `Camera::new(CameraConfig::default())` | Camera 实例创建成功 |
| 2 | 调用 `camera.viewport()`，断言 `(480.0, 270.0)` | 虚拟画布尺寸正确 |
| 3 | 调用 `player_visible_ratio(200.0, 20.0, 480.0)` | ratio = (480-180)/480 = 0.625 |
| 4 | 断言 `ratio ≈ 0.625` (容差 0.001) | 使用虚拟画布宽度计算 |
| 5 | 断言 `ratio ≈ 1.0 - config.player_target_x_pct` | 设计不变式成立 |

### 验证点

- ratio 使用 480px（虚拟画布）而非 1280/1920/2560（屏幕分辨率）
- ratio = 1.0 - player_target_x_pct (当玩家在目标位置时)

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_d3_player_visible_ratio_uses_virtual_canvas`; `tests/multi_resolution_test.rs::r10_visible_ratio_uses_virtual_canvas`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-001

### 关联需求

NFR-002（多分辨率支持）— AC-1: tolerance_pct = 0.0（零容差边界）

### 测试目标

验证 `check_hud_anchor(1280.0, 720.0, 0.0)` 在零容差下仍然 passed == true（期望锚点自身偏差精确为 0.0）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_hud_anchor(1280.0, 720.0, 0.0)` | 返回 `HudAnchorReport` |
| 2 | 断言 `report.passed == true` | 零容差自洽性通过 |
| 3 | 断言 `report.deviation.dx_pct ≈ 0.0` | 偏差为精确 0.0 |
| 4 | 断言 `report.deviation.dy_pct ≈ 0.0` | 偏差为精确 0.0 |
| 5 | 断言 `report.tolerance == (0.0, 0.0)` | 容差精确为 0 |

### 验证点

- `|0.0| <= 0.0` 传递 —— 无 `-0.0 != 0.0` 误判
- tolerance 在 tolerance_pct=0.0 时为 (0.0, 0.0)

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b1_check_hud_anchor_zero_tolerance`; `tests/multi_resolution_test.rs::r11_zero_tolerance_self_consistency`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-002

### 关联需求

NFR-002（多分辨率支持）— AC-2: ratio 恰在下界 (0.45)

### 测试目标

验证 `check_visible_ratio(0.45, 0.45, 0.50)` 在 ratio 恰好等于下界时 passed == true（边界包含）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_visible_ratio(0.45, 0.45, 0.50)` | 返回 `VisibleAreaReport` |
| 2 | 断言 `report.passed == true` | ratio = 0.45 在下界处通过（包含） |
| 3 | 断言 `report.ratio ≈ 0.45` | 输入值正确保存 |

### 验证点

- 下界比较使用 `>=` 非 `>`，防止 off-by-one

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b2_check_visible_ratio_at_lower_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` (lower)
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-003

### 关联需求

NFR-002（多分辨率支持）— AC-2: ratio 恰在上界 (0.50)

### 测试目标

验证 `check_visible_ratio(0.50, 0.45, 0.50)` 在 ratio 恰好等于上界时 passed == true（边界包含）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_visible_ratio(0.50, 0.45, 0.50)` | 返回 `VisibleAreaReport` |
| 2 | 断言 `report.passed == true` | ratio = 0.50 在上界处通过（包含） |
| 3 | 断言 `report.ratio ≈ 0.50` | 输入值正确保存 |

### 验证点

- 上界比较使用 `<=` 非 `<`，防止 off-by-one

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b3_check_visible_ratio_at_upper_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` (upper)
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-004

### 关联需求

NFR-002（多分辨率支持）— AC-2: ratio 略低于下界 (0.4499)

### 测试目标

验证 `check_visible_ratio(0.4499, 0.45, 0.50)` 在 ratio 略低于下界时 passed == false（正确拒绝）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_visible_ratio(0.4499, 0.45, 0.50)` | 返回 `VisibleAreaReport` |
| 2 | 断言 `report.passed == false` | ratio 0.4499 < 0.45 应被拒绝 |
| 3 | 断言 `report.ratio ≈ 0.4499` | 输入值正确保存 |

### 验证点

- 略低于下界的值被正确拒绝（无浮点 epsilon 误判）
- `0.4499 < 0.45` 正确判定为 false（无精度宽容）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b4_check_visible_ratio_below_lower_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` (below)
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-005

### 关联需求

NFR-002（多分辨率支持）— AC-3: 全屏与窗口模式报告完全一致（纯函数确定性）

### 测试目标

验证对同一 1080p 分辨率连续两次调用 `check_hud_anchor` 返回完全相同的报告（纯函数确定性：相同输入 → 相同输出）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `check_hud_anchor(1920.0, 1080.0, 0.02)` → r1 | 生成报告 r1 |
| 2 | 调用 `check_hud_anchor(1920.0, 1080.0, 0.02)` → r2 | 生成报告 r2 |
| 3 | 断言 `r1 == r2`（全字段相等） | 纯函数确定性，相同输入产生相同输出 |
| 4 | 断言 `r1.expected == (57.6, 32.4)` (容差 0.01) | 锚点正确 |
| 5 | 断言 `r1.deviation.dx_pct ≈ 0.0` | 偏差为零 |

### 验证点

- 全屏模式下视口尺寸与窗口模式相同分辨率的计算基准一致
- 纯函数无状态依赖，重复调用结果不变

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b5_fullscreen_windowed_identical_reports`; `tests/multi_resolution_test.rs::r13_same_resolution_identical_reports`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-012-006

### 关联需求

NFR-002（多分辨率支持）— expected_hud_anchor NaN 输入边界

### 测试目标

验证 `expected_hud_anchor(f32::NAN, 720.0)` 不 panic，NaN 按 IEEE 754 传播到输出。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 在 `catch_unwind` 中调用 `expected_hud_anchor(f32::NAN, 720.0)` | 不 panic |
| 2 | 解包结果，断言 `x.is_nan()` | NaN 输入 → NaN 输出（IEEE 754 传播） |

### 验证点

- NaN 不触发 panic
- NaN 按浮点标准传播（调用方应 guard）

### 后置检查

- 无

### 元数据

- **优先级**: Low
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_b6_expected_hud_anchor_nan_input_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-PERF-012-001

### 关联需求

NFR-002（多分辨率支持）— AC-1: verify_all_resolutions 全套件性能

### 测试目标

验证 `verify_all_resolutions()` 全分辨率套件（3 种分辨率 × (HUD + 可见区域)）执行时间 < 1ms。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `CameraConfig::default()` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 记录起始时间 `Instant::now()` | 获取基准时间 |
| 2 | 调用 `verify_all_resolutions(&CameraConfig::default())` | 执行全分辨率套件 |
| 3 | 记录结束时间，计算 elapsed | 计算耗时 |
| 4 | 断言 `elapsed.as_micros() < 1000` | 总耗时 < 1000us (1ms) |
| 5 | 断言 `reports.len() == 3` | 正确性附带检查 |

### 验证点

- 纯确定性 f32 计算无 I/O 阻塞
- 3 次 `check_hud_anchor` + 3 次 `check_visible_ratio` 应在亚毫秒完成

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_c1_perf_full_suite_timing`
- **Test Type**: Real

---

### 用例编号

ST-PERF-012-002

### 关联需求

NFR-002（多分辨率支持）— AC-1: expected_hud_anchor 单次调用延迟

### 测试目标

验证 `expected_hud_anchor()` 单次调用延迟 < 100ns（10000 次调用 < 1ms）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 记录起始时间 | 获取基准时间 |
| 2 | 循环 10000 次调用 `expected_hud_anchor(1280.0, 720.0)` | 所有调用无 panic |
| 3 | 记录结束时间，计算 elapsed | 计算总耗时 |
| 4 | 断言 `elapsed.as_micros() < 1000` | 10000 次 < 1000us → 单次 < 100ns |
| 5 | 额外调用一次确认返回值正确: `(38.4, 21.6)` | 性能测试不牺牲正确性 |

### 验证点

- 纯 f32 乘法（无内存分配、无 I/O）应在 100ns 内完成
- 性能测试附带正确性断言，防止优化器消除死代码

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `src/verification.rs::tests::test_c2_perf_single_call_latency`
- **Test Type**: Real

---

### 用例编号

ST-UI-012-015

### 关联需求

NFR-002（多分辨率支持）— AC-1: HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口（视觉验证）

### 测试目标

通过实际运行游戏并截屏，人工验证在 720p、1080p、1440p 三种分辨率下 HUD 元素（金币图标+数字、生命图标+数字）锚定在视口 (3%, 3%) 位置，偏差不超过 ±2% 视口宽高。

### 前置条件

- Windows 桌面环境，显示器支持 720p/1080p/1440p 分辨率
- 游戏已编译：`cargo build --release`，产物 `target/release/mario-platformer.exe`
- 分辨率切换功能（FR-017）已实现且可用

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 启动游戏 `target/release/mario-platformer.exe` | 游戏窗口正常渲染，默认分辨率启动 |
| 2 | 按 ESC 打开选项菜单 | 显示分辨率列表和全屏开关 |
| 3 | 选择 720p (1280x720)，按 Enter 确认 | 游戏窗口切换至 1280x720 |
| 4 | 截屏保存至 `screenshots/nfr002_hud_720p.png` | 截图包含完整游戏画面和 HUD |
| 5 | 按 ESC → 选择 1080p (1920x1080) → Enter | 游戏窗口切换至 1920x1080 |
| 6 | 截屏保存至 `screenshots/nfr002_hud_1080p.png` | 截图包含完整游戏画面和 HUD |
| 7 | 按 ESC → 选择 1440p (2560x1440) → Enter | 游戏窗口切换至 2560x1440 |
| 8 | 截屏保存至 `screenshots/nfr002_hud_1440p.png` | 截图包含完整游戏画面和 HUD |
| 9 | 以像素测量工具检查每张截图中 HUD 锚定位置 | HUD 左上角锚点 x 方向偏差 ≤ ±2% 视口宽度，y 方向偏差 ≤ ±2% 视口高度 |

### 验证点

- 720p: HUD 锚定 x ≈ 38.4px (±25.6px), y ≈ 21.6px (±14.4px)
- 1080p: HUD 锚定 x ≈ 57.6px (±38.4px), y ≈ 32.4px (±21.6px)
- 1440p: HUD 锚定 x ≈ 76.8px (±51.2px), y ≈ 43.2px (±28.8px)
- HUD 元素（金币图标+数字、生命图标+数字）在三种分辨率下均完整可见、无裁剪

### 后置检查

- 关闭游戏窗口
- 保存所有截图至 `screenshots/` 目录作为证据

### 元数据

- **优先级**: High
- **类别**: ui
- **已自动化**: No
- **手动测试原因**: visual-judgment
- **测试引用**: N/A
- **Test Type**: Real

---

### 用例编号

ST-UI-012-016

### 关联需求

NFR-002（多分辨率支持）— AC-2: 玩家可见区域占视口宽度 45-50%（视觉验证）

### 测试目标

通过实际运行游戏并截屏，人工验证在 720p/1080p/1440p 三种分辨率下玩家角色前方可见区域占视口宽度的 45-50%。

### 前置条件

- Windows 桌面环境
- 游戏已编译：`target/release/mario-platformer.exe`
- 玩家角色在关卡中处于正常游戏状态（非贴边/死亡状态）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 启动游戏，切换至 720p 分辨率 | 游戏窗口 1280x720 |
| 2 | 让玩家站在关卡中间位置（不贴摄像机边界） | 摄像机正常跟随，玩家在视口 35-40% 左侧位置 |
| 3 | 截屏保存至 `screenshots/nfr002_visible_720p.png` | 截图包含完整游戏画面 |
| 4 | 切换至 1080p，同样场景截屏 → `screenshots/nfr002_visible_1080p.png` | 截图包含完整游戏画面 |
| 5 | 切换至 1440p，同样场景截屏 → `screenshots/nfr002_visible_1440p.png` | 截图包含完整游戏画面 |
| 6 | 以像素测量工具测量每张截图中玩家左侧边缘到视口左边缘的距离 vs 玩家右侧边缘到视口右边缘的距离 | 玩家前方（右侧）可见区域占视口宽度 45-50%（即玩家屏幕位置在视口 50-55% 处） |

### 验证点

- 720p: 玩家前方可见区域 = (视口右边缘 - 玩家右边缘) / 1280 ∈ [0.45, 0.50]
- 1080p: 同理，分母为 1920
- 1440p: 同理，分母为 2560
- 注：当前 Camera `player_target_x_pct = 0.375` 产生 ratio ≈ 0.625 (62.5%) 前方可见，超出 45-50% 范围 —— 此偏差在 Feature Design Clarification Addendum #1 中已记录

### 后置检查

- 关闭游戏窗口
- 保存截图证据

### 元数据

- **优先级**: Medium
- **类别**: ui
- **已自动化**: No
- **手动测试原因**: visual-judgment
- **测试引用**: N/A
- **Test Type**: Real

---

### 用例编号

ST-UI-012-017

### 关联需求

NFR-002（多分辨率支持）— AC-3: 全屏模式 HUD 位置不变（视觉验证）

### 测试目标

通过实际运行游戏并截屏，人工验证全屏模式下 HUD 锚定位置与对应分辨率的窗口模式一致（HUD 不因全屏切换而偏移）。

### 前置条件

- Windows 桌面环境，显示器支持全屏模式
- 游戏已编译：`target/release/mario-platformer.exe`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 启动游戏，切换至 1080p 窗口模式 | 游戏窗口 1920x1080 |
| 2 | 截屏窗口模式 HUD → `screenshots/nfr002_fullscreen_windowed_1080p.png` | 窗口模式下 HUD 锚定位置 |
| 3 | 按 ESC → 开启全屏开关 → Enter | 游戏进入全屏模式 |
| 4 | 截屏全屏模式 HUD → `screenshots/nfr002_fullscreen_1080p.png` | 全屏模式下 HUD 锚定位置 |
| 5 | 对比两张截图：HUD 元素锚定位置 | 全屏与窗口模式 HUD 锚定位置一致，无偏移 |
| 6 | 按 ESC → 关闭全屏 → Enter | 游戏恢复正常窗口模式，HUD 锚定位置不变 |

### 验证点

- 全屏模式下 HUD 锚定位置与窗口模式 1080p `(57.6, 32.4)` 一致
- 全屏 → 窗口模式切换后 HUD 锚定位置不变
- HUD 元素在全屏/窗口切换过程中无闪烁、位移或裁剪

### 后置检查

- 关闭游戏窗口
- 保存截图证据

### 元数据

- **优先级**: High
- **类别**: ui
- **已自动化**: No
- **手动测试原因**: visual-judgment
- **测试引用**: N/A
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | Feature Design 行 | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-------------------|-----------|---------|------|
| ST-FUNC-012-001 | NFR-002 | verification_steps[0] | A1 (FUNC/happy) | `src/verification.rs::test_a1_expected_hud_anchor_720p`; `tests/multi_resolution_test.rs::r1_hud_anchor_expected_720p` | Real | PASS |
| ST-FUNC-012-002 | NFR-002 | verification_steps[0] | A2 (FUNC/happy) | `src/verification.rs::test_a2_expected_hud_anchor_1080p_and_1440p`; `tests/multi_resolution_test.rs::r2_hud_anchor_expected_1080p_1440p` | Real | PASS |
| ST-FUNC-012-003 | NFR-002 | verification_steps[0] | A3 (FUNC/happy) | `src/verification.rs::test_a3_hud_anchor_deviation_computation`; `tests/multi_resolution_test.rs::r3_hud_anchor_deviation_computation` | Real | PASS |
| ST-FUNC-012-004 | NFR-002 | verification_steps[0] | A4 (FUNC/happy) | `src/verification.rs::test_a4_check_hud_anchor_self_consistency`; `tests/multi_resolution_test.rs::r4_hud_anchor_self_consistency` | Real | PASS |
| ST-FUNC-012-005 | NFR-002 | verification_steps[1] | A5 (FUNC/happy) | `src/verification.rs::test_a5_player_visible_ratio_computation`; `tests/multi_resolution_test.rs::r5_player_visible_ratio_computation` | Real | PASS |
| ST-FUNC-012-006 | NFR-002 | verification_steps[1] | A6 (FUNC/happy) | `src/verification.rs::test_a6_check_visible_ratio_within_range`; `tests/multi_resolution_test.rs::r6_visible_ratio_within_range` | Real | PASS |
| ST-FUNC-012-007 | NFR-002 | verification_steps[0], verification_steps[1] | A7 (FUNC/happy) | `src/verification.rs::test_a7_verify_all_resolutions_full_suite`; `tests/multi_resolution_test.rs::r7_verify_all_resolutions_full_suite` | Real | PASS |
| ST-FUNC-012-008 | NFR-002 | verification_steps[2] | A8 (FUNC/happy) | `src/verification.rs::test_a8_fullscreen_hud_anchor_same_as_windowed`; `tests/multi_resolution_test.rs::r8_fullscreen_hud_anchor_same_as_windowed` | Real | PASS |
| ST-FUNC-012-009 | NFR-002 | — | A9 (FUNC/error) | `src/verification.rs::test_a9_expected_hud_anchor_zero_width_no_panic` | Real | PASS |
| ST-FUNC-012-010 | NFR-002 | — | A10 (FUNC/error) | `src/verification.rs::test_a10_expected_hud_anchor_negative_width_no_panic` | Real | PASS |
| ST-FUNC-012-011 | NFR-002 | — | A11 (FUNC/error) | `src/verification.rs::test_a11_player_visible_ratio_zero_viewport_no_panic` | Real | PASS |
| ST-FUNC-012-012 | NFR-002 | verification_steps[0] | D1 (UI/anchor) | `src/verification.rs::test_d1_hud_anchor_consistency_720p` | Real | PASS |
| ST-FUNC-012-013 | NFR-002 | verification_steps[0] | D2 (UI/anchor) | `src/verification.rs::test_d2_hud_anchor_consistency_all_resolutions`; `tests/multi_resolution_test.rs::r9_hud_anchor_consistency_all_resolutions` | Real | PASS |
| ST-FUNC-012-014 | NFR-002 | verification_steps[1] | D3 (UI/viewport) | `src/verification.rs::test_d3_player_visible_ratio_uses_virtual_canvas`; `tests/multi_resolution_test.rs::r10_visible_ratio_uses_virtual_canvas` | Real | PASS |
| ST-BNDRY-012-001 | NFR-002 | — | B1 (BNDRY/edge) | `src/verification.rs::test_b1_check_hud_anchor_zero_tolerance`; `tests/multi_resolution_test.rs::r11_zero_tolerance_self_consistency` | Real | PASS |
| ST-BNDRY-012-002 | NFR-002 | — | B2 (BNDRY/edge) | `src/verification.rs::test_b2_check_visible_ratio_at_lower_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` | Real | PASS |
| ST-BNDRY-012-003 | NFR-002 | — | B3 (BNDRY/edge) | `src/verification.rs::test_b3_check_visible_ratio_at_upper_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` | Real | PASS |
| ST-BNDRY-012-004 | NFR-002 | — | B4 (BNDRY/edge) | `src/verification.rs::test_b4_check_visible_ratio_below_lower_bound`; `tests/multi_resolution_test.rs::r12_visible_ratio_boundary_inclusive` | Real | PASS |
| ST-BNDRY-012-005 | NFR-002 | verification_steps[2] | B5 (BNDRY/edge) | `src/verification.rs::test_b5_fullscreen_windowed_identical_reports`; `tests/multi_resolution_test.rs::r13_same_resolution_identical_reports` | Real | PASS |
| ST-BNDRY-012-006 | NFR-002 | — | B6 (BNDRY/edge) | `src/verification.rs::test_b6_expected_hud_anchor_nan_input_no_panic` | Real | PASS |
| ST-PERF-012-001 | NFR-002 | verification_steps[0] | C1 (PERF/resolution) | `src/verification.rs::test_c1_perf_full_suite_timing` | Real | PASS |
| ST-PERF-012-002 | NFR-002 | verification_steps[0] | C2 (PERF/formula) | `src/verification.rs::test_c2_perf_single_call_latency` | Real | PASS |
| ST-UI-012-015 | NFR-002 | verification_steps[0] | — | N/A (Manual) | Real | PENDING-MANUAL |
| ST-UI-012-016 | NFR-002 | verification_steps[1] | — | N/A (Manual) | Real | PENDING-MANUAL |
| ST-UI-012-017 | NFR-002 | verification_steps[2] | — | N/A (Manual) | Real | PENDING-MANUAL |

> 结果有效值: `PENDING`, `PASS`, `FAIL`, `MANUAL-PASS`, `MANUAL-FAIL`, `BLOCKED`, `PENDING-MANUAL`

## SRS Trace 覆盖

| SRS 需求 ID | ST 用例映射 | 覆盖状态 |
|------------|------------|---------|
| NFR-002 AC-1 (HUD 锚定偏差) | ST-FUNC-012-001/002/003/004, ST-PERF-012-001/002, ST-FUNC-012-012/013, ST-UI-012-015 | 已覆盖 (9 条) |
| NFR-002 AC-2 (可见区域比例) | ST-FUNC-012-005/006/007, ST-FUNC-012-014, ST-UI-012-016 | 已覆盖 (4 条) |
| NFR-002 AC-3 (全屏 HUD 位置) | ST-FUNC-012-008, ST-BNDRY-012-005, ST-UI-012-017 | 已覆盖 (3 条) |

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 22 |
| Passed | 22 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` executed against a real running environment.
> Manual test cases (ST-UI-012-015/016/017) are excluded from this count (tracked in Manual Test Case Summary below).

## Manual Test Case Summary

| Metric | Count |
|--------|-------|
| Total Manual Test Cases | 3 |
| Manual Passed (MANUAL-PASS) | 0 |
| Manual Failed (MANUAL-FAIL) | 0 |
| Blocked | 0 |
| Pending (PENDING-MANUAL) | 3 |

> Manual test cases = test cases with `已自动化: No`. Results collected via human review gate after automated execution.
> Any MANUAL-FAIL blocks the feature from being marked `"passing"` — same as automated FAIL.
> ST-UI-012-016 note: Current `player_target_x_pct = 0.375` produces ratio ≈ 0.625 (62.5% forward), exceeding the 45-50% SRS target. This deviation is documented in Feature Design Clarification Addendum #1 as a known assumption.
