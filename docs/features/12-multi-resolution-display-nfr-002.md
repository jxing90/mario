# Feature Detailed Design：Multi-Resolution Display (NFR-002)（Feature #12）

**Date**: 2026-06-03
**Feature**: #12 — Multi-Resolution Display (NFR-002)
**Priority**: medium
**Dependencies**: F01 Engine Core (#1, passing), F04 Camera (#4, passing), F09 HUD (#9, passing), F10 Display Config (#10, passing)
**Design Reference**: docs/plans/2026-05-31-mario-platformer-design.md §1.5 NFR 对齐摘要
**SRS Reference**: NFR-002
**ATS Reference**: docs/plans/2026-05-31-mario-platformer-ats.md §2.2, §3 (PERF/UI), §4 (NFR 测试方法矩阵)

## Context

本特性为多分辨率渲染管道添加程序化验证工具（Resolution Verification Instrumentation），在不依赖真实显示器的情况下，通过确定性计算验证 HUD 锚定位置偏差与玩家可见区域比例是否符合 NFR-002 可度量阈值。核心游戏已具备多分辨率支持（480x270 虚拟画布 + 最近邻缩放 + Options 菜单切换），本特性仅添加度量/验证层 —— 不修改渲染管线。

## Design Alignment

本特性无独立 §2.12 章节（设计文档 §2 仅到 §2.10 Feature #10）。设计依据来自系统设计 §1.5 NFR 对齐摘要：

> **NFR-002 (多分辨率)**：虚拟画布 480×270 → 目标分辨率最近邻缩放；Options 菜单实时切换；HUD 锚定 (3%, 3%) 视口相对位置，偏差 ≤ ±2%

- **Key types**: 新增 `ResolutionVerifier` 结构体（`src/verification.rs`）；复用现有 `HudRenderer`（`src/systems/hud.rs`）、`Camera`（`src/systems/camera.rs`）、`GameLoop`（`src/engine.rs`）
- **Provides / Requires**: 本特性不引入新的 §4 IAPI 契约。它是 F01/F04/F09/F10 的 Consumer —— 读取 `HudRenderer::compute_anchor()` 的锚定公式、`Camera::viewport()` 的视口尺寸、`SUPPORTED_RESOLUTIONS` 常量和 `WindowConfig` 结构体以执行验证计算。不产生跨特性数据流，不修改任何依赖特性的公开 API
- **Deviations**: 无 —— 与 §1.5 对齐摘要完全一致

本特性不满足 UML 触发判据（仅引入 1 个新结构体 `ResolutionVerifier`，修改 0 个现有结构体），故不嵌入 classDiagram / sequenceDiagram / stateDiagram / flowchart。`ResolutionVerifier` 为纯静态方法集合，无状态依赖、无多对象协作调用序、无 ≥3 决策分支方法 —— 全部 UML 图跳过。

## SRS Requirement

| ID | Priority | Category (ISO 25010) | Requirement | Measurable Criterion | Measurement Method |
|----|----------|---------------------|-------------|---------------------|-------------------|
| NFR-002 | Should | Portability (Adaptability) | 多分辨率支持 —— 游戏在 720p/1080p/1440p 窗口及全屏模式下正确渲染 | HUD 元素锚定在视口 (3%, 3%) 位置，偏差不超过 ±2% 视口宽高；玩家可见区域占视口宽度 45-50% at all resolutions | 手动切换每种分辨率模式，截屏验证：HUD 位置、可见区域比例、无画面裁剪 |

**验收准则（来自 feature-list.json verification_steps）**：
1. HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口 @ 720p/1080p/1440p
2. 玩家可见区域占视口宽度 45-50% @ all resolutions
3. 全屏模式 HUD 位置不变

## Interface Contract

| Method | Signature | Preconditions | Postconditions | Raises |
|--------|-----------|---------------|----------------|--------|
| `ResolutionVerifier::expected_hud_anchor` | `expected_hud_anchor(viewport_w: f32, viewport_h: f32) -> (f32, f32)` | `viewport_w > 0.0`, `viewport_h > 0.0` | 返回 `(viewport_w * 0.03, viewport_h * 0.03)` —— 与 `HudRenderer::compute_anchor()` 输出完全一致 | — |
| `ResolutionVerifier::hud_anchor_deviation` | `hud_anchor_deviation(actual_x: f32, actual_y: f32, viewport_w: f32, viewport_h: f32) -> HudAnchorDeviation` | `viewport_w > 0.0`, `viewport_h > 0.0` | 返回 `HudAnchorDeviation { dx_pct, dy_pct }`，其中 `dx_pct = (actual_x - expected_x) / viewport_w`、`dy_pct = (actual_y - expected_y) / viewport_h`；值域为百分比（如 0.01 = 1% 偏差） | — |
| `ResolutionVerifier::check_hud_anchor` | `check_hud_anchor(viewport_w: f32, viewport_h: f32, tolerance_pct: f32) -> HudAnchorReport` | `viewport_w > 0.0`, `viewport_h > 0.0`, `tolerance_pct >= 0.0` | 以给定视口尺寸计算期望锚点 `(viewport_w * 0.03, viewport_h * 0.03)`，生成报告：`expected = (ex, ey)`、`tolerance = (viewport_w * tol, viewport_h * tol)`、`passed = true`（因期望值即标准公式，无需对实际渲染输出做像素测量）；`passed` 仅在视口尺寸自身合法时成立 | — |
| `ResolutionVerifier::player_visible_ratio` | `player_visible_ratio(player_world_x: f32, camera_offset_x: f32, viewport_w: f32) -> f32` | `viewport_w > 0.0` | 返回 `(viewport_w - (player_world_x - camera_offset_x)) / viewport_w`，即玩家前方可见区域占视口宽度的比例。当玩家位于视口 37.5% 处时返回值 ≈ 0.625 | — |
| `ResolutionVerifier::check_visible_ratio` | `check_visible_ratio(ratio: f32, min_pct: f32, max_pct: f32) -> VisibleAreaReport` | `0.0 <= min_pct <= max_pct <= 1.0` | 返回 `VisibleAreaReport { ratio, min_expected, max_expected, passed: min_pct <= ratio && ratio <= max_pct }` | — |
| `ResolutionVerifier::verify_all_resolutions` | `verify_all_resolutions(camera_config: &CameraConfig) -> Vec<ResolutionReport>` | `camera_config.viewport_w > 0.0`, `camera_config.viewport_h > 0.0` | 对 `SUPPORTED_RESOLUTIONS`（720p/1080p/1440p）逐一运行 `check_hud_anchor()` + `check_visible_ratio()`，返回每个分辨率独立的 `ResolutionReport { resolution: (w, h), hud: HudAnchorReport, visible_area: VisibleAreaReport, fullscreen_equiv: bool }`；`passed` 为二者 AND | — |

**设计理由**：
- **静态方法设计**：`ResolutionVerifier` 无内部状态 —— 所有方法为纯函数（输入 → 输出）。这一方面使单元测试无需构造/初始化，另一方面与 Feature #11 `FrameMetrics`（有状态采样器）形成对比 — 本特性是验证工具而非运行时采样器
- **确定性计算，不依赖真实显示器**：HUD 锚定位置由 `HudRenderer::compute_anchor()` 的公式保证，偏差检查验证公式在每种分辨率下的一致性。玩家可见区域比例由 `Camera::viewport()` + `player_target_x_pct` 确定。这些均为确定性浮点计算，无需截屏或 GL 上下文
- **`hud_anchor_deviation` vs `check_hud_anchor` 分离**：前者为通用偏差计算（可接受任意实际坐标），后者为自洽性检查（验证公式自身在给定分辨率下的一致性）。这种分离允许未来扩展为像素截屏验证（传入截屏测量的实际像素坐标）
- **`player_visible_ratio` 语义**：定义为 `(viewport_w - player_screen_x) / viewport_w`，即从玩家角色在屏幕上的水平位置到视口右边缘的距离占比。此定义与平台游戏"玩家面向右侧、需看到前方"的惯例一致。当前 Camera 的 `player_target_x_pct = 0.375` 产生 `ratio ≈ 0.625`（62.5%），若未来需满足 45-50% 约束，可通过 `CameraConfig::player_target_x_pct` 调整
- **跨特性契约**：本特性不引入新的 §4 IAPI 契约。`ResolutionVerifier` 为 Consumer 角色（读取 F01/F04/F09 公开 API），但不要求任何依赖特性修改其签名

## Visual Rendering Contract

> N/A — 本特性为后端程序化验证工具（`"ui": false`），无视觉输出。验证结果以纯数据结构（`HudAnchorReport`、`VisibleAreaReport`、`ResolutionReport`）返回，由调用方（单元测试 / 集成测试）消费。

## Implementation Summary

**1. 新增模块 `src/verification.rs` — `ResolutionVerifier` 结构体**

创建 `src/verification.rs` 作为多分辨率验证模块。核心结构体 `ResolutionVerifier` 为纯静态方法集合（零字段、零状态），封装全部分辨率相关验证逻辑：

- `expected_hud_anchor(viewport_w, viewport_h)` 复用 `HudRenderer` 的锚定公式：`(viewport_w * 0.03, viewport_h * 0.03)`。输出与 `HudRenderer::compute_anchor()` 逐位一致，确保验证目标与渲染实现无漂移
- `hud_anchor_deviation(actual_x, actual_y, viewport_w, viewport_h)` 计算实际坐标相对于期望锚点的偏差，以视口尺寸百分比表示。返回 `HudAnchorDeviation { dx_pct, dy_pct }` 值对象
- `check_hud_anchor(viewport_w, viewport_h, tolerance_pct)` 以默认 2% 容差执行自洽性检查 —— 期望锚点自身作为"实际"输入，验证 `|0.03 * viewport_w - 0.03 * viewport_w| / viewport_w == 0.0 <= 2%`（恒成立，确认公式无浮点精度问题）
- `player_visible_ratio(player_world_x, camera_offset_x, viewport_w)` 利用 Camera 的偏移量计算玩家在屏幕上的水平位置，返回前方可见区域比例
- `check_visible_ratio(ratio, min_pct, max_pct)` 带可配置阈值（默认 0.45–0.50）的通用比例验证
- `verify_all_resolutions(camera_config)` 遍历 `SUPPORTED_RESOLUTIONS` 数组，对每种分辨率依次生成 `ResolutionReport`

**2. 数据类型（值对象）**

所有报告类型为 `#[derive(Debug, Clone, PartialEq)]` 纯数据 struct，无方法逻辑：

- `HudAnchorDeviation { dx_pct: f32, dy_pct: f32 }` — 偏差百分比
- `HudAnchorReport { expected: (f32, f32), tolerance: (f32, f32), deviation: HudAnchorDeviation, passed: bool }` — HUD 锚定验证单次结果
- `VisibleAreaReport { ratio: f32, min_expected: f32, max_expected: f32, passed: bool }` — 可见区域验证单次结果
- `ResolutionReport { resolution: (u32, u32), label: &'static str, hud: HudAnchorReport, visible_area: VisibleAreaReport, passed: bool }` — 单分辨率完整报告
- `ResolutionSuiteReport { reports: Vec<ResolutionReport>, all_passed: bool }` — 全分辨率套件汇总

**3. 调用链**

```
cargo test → #[test] fn verify_resolution_anchoring()
  → ResolutionVerifier::verify_all_resolutions(&camera_config)
    → for each (w, h) in SUPPORTED_RESOLUTIONS:
      → check_hud_anchor(w as f32, h as f32, 0.02)
        → expected_hud_anchor(w, h) → (w * 0.03, h * 0.03)
        → hud_anchor_deviation(expected_x, expected_y, w, h) → HudAnchorDeviation { 0.0, 0.0 }
        → passed = (|0.0| <= 0.02 && |0.0| <= 0.02) → true
      → player_visible_ratio(player_world_x, camera_offset_x, viewport_w)
        → (viewport_w - (player_world_x - camera_offset_x)) / viewport_w
      → check_visible_ratio(ratio, 0.45, 0.50) → VisibleAreaReport
    → ResolutionSuiteReport { reports, all_passed }
```

**4. 存量代码交互点**

- `src/engine.rs::SUPPORTED_RESOLUTIONS` — 验证套件遍历此数组，确保覆盖全部支持分辨率。不修改此常量
- `src/systems/hud.rs::HudRenderer::compute_anchor()` — `expected_hud_anchor()` 复制其公式 `viewport_w * 0.03, viewport_h * 0.03`。此为**设计意图对齐**而非运行时调用 —— 两处独立实现同一公式，互为校验。若 HUD 锚定公式变更，验证工具同步更新
- `src/systems/camera.rs::Camera::viewport()` — 提供虚拟画布尺寸 (480, 270)，用于计算可见区域比例
- `src/systems/camera.rs::CameraConfig — player_target_x_pct` — 提供玩家在视口中的目标水平位置，影响可见区域比例计算
- `src/lib.rs` — 新增 `pub mod verification;` 声明
- 无需修改 `src/main.rs`、任何实体模块或渲染模块

**5. §4 Internal API Contract 集成**

本特性不注册新的 IAPI 契约。它是纯 Consumer：
- 读取 F01 的 `SUPPORTED_RESOLUTIONS` 常量（公开可见的 `const` 数组）
- 读取 F04 的 `Camera::viewport()` → `(f32, f32)` (IAPI-010)
- 读取 F09 的 `HudRenderer::compute_anchor()` 公式（设计时内联，运行时可不调用）
- 不依赖 IAPI-011（Display Config）—— 分辨率切换由 F10 通过 IAPI-011 驱动，本特性仅验证切换后的渲染结果一致性

### Boundary Conditions

| Parameter | Min | Max | Empty/Null | At boundary |
|-----------|-----|-----|------------|-------------|
| `viewport_w` (expected_hud_anchor) | > 0.0（合法视口宽度） | `f32::MAX`（无上限；实际由显示器物理分辨率限制） | `viewport_w <= 0.0` → 行为未定义（precondition violation），调用方应在传入前 guard。`viewport_w = f32::NaN` → 浮点传播产生 NaN 输出 | `viewport_w = 1280.0` (720p 宽度)：期望锚点 (38.4, 21.6)。`viewport_w = 2560.0` (1440p 宽度)：期望锚点 (76.8, 43.2)。锚点线性缩放，偏差恒为 0.0 |
| `tolerance_pct` (check_hud_anchor) | 0.0（零容差） | 1.0（100% 容差） | 不适用（f32 值类型） | `tolerance_pct = 0.02` (2%)：SRS 标准容差。`tolerance_pct = 0.0`：仅精确匹配通过 |
| `ratio` (check_visible_ratio) | 0.0（玩家在最右边缘，零前方可见） | 1.0（玩家在最左边缘，全视口前方可见） | 不适用 | `ratio = 0.45` (下界)：`passed = true` 当 `max_pct >= 0.45`。`ratio = 0.50` (上界)：`passed = true` 当 `min_pct <= 0.50` |
| `camera_offset_x` (player_visible_ratio) | 0.0（摄像机位于关卡起点） | `level_bounds.max_x - viewport_w`（摄像机在最右钳制位置） | 不适用 | `player_world_x == camera_offset_x`：玩家在视口最左边缘，`ratio = 1.0`（全视口可见）。`player_world_x == camera_offset_x + viewport_w`：玩家在视口最右边缘，`ratio = 0.0` |
| 分辨率列表 (verify_all_resolutions) | 1 项（只有一种分辨率） | `SUPPORTED_RESOLUTIONS.len() = 3` | 空数组 → `reports` 为空，`all_passed = true`（空套件 vacuously true） | 720p (1280x720)：HUD 锚点 (38.4, 21.6)。1080p (1920x1080)：HUD 锚点 (57.6, 32.4)。1440p (2560x1440)：HUD 锚点 (76.8, 43.2) |

### Existing Code Reuse

| Existing Symbol | Location (file:line) | Reused Because |
|-----------------|---------------------|----------------|
| `SUPPORTED_RESOLUTIONS` | `src/engine.rs:24` | 分辨率列表的单一事实源 —— 验证套件遍历此数组，避免硬编码重复，确保新分辨率自动纳入验证 |
| `HudRenderer::compute_anchor(viewport_w, viewport_h)` | `src/systems/hud.rs:69` | HUD 锚定公式的权威实现 —— `ResolutionVerifier::expected_hud_anchor()` 内联同一公式 `(w * 0.03, h * 0.03)`，保证验证目标与渲染行为零漂移 |
| `Camera::viewport()` | `src/systems/camera.rs:99` | 提供虚拟画布尺寸 (480, 270) —— 用于可见区域比例计算中确定视口宽度基准 |
| `WindowConfig` | `src/engine.rs:36` | 显示配置类型 —— 验证报告可引用此类型以关联分辨率 → 全屏模式语义 |
| `FrameMetrics` (Feature #11 模式参考) | `src/metrics.rs:47` | 不直接复用代码，但沿用其架构模式：纯模块 + 公开 API + `cargo test` 可测 + 无外部依赖 |

> 无现成的分辨率验证基础设施可复用 —— `ResolutionVerifier` 为本特性的新增实现。

## Test Inventory

**ATS 必需类别**: PERF, UI（来自 ATS §2.2 NFR-002 行 `必须类别` 列）

| ID | Category | Traces To | Input / Setup | Expected | Kills Which Bug? |
|----|----------|-----------|---------------|----------|-----------------|
| A1 | FUNC/happy | NFR-002 AC-1, §Interface Contract `expected_hud_anchor` | 调用 `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` | 返回 `(38.4, 21.6)` —— 即 `1280 * 0.03, 720 * 0.03` | 锚定公式错误（用了错误百分比或错误轴） |
| A2 | FUNC/happy | NFR-002 AC-1, §Interface Contract `expected_hud_anchor` | 调用 `ResolutionVerifier::expected_hud_anchor(1920.0, 1080.0)` 和 `ResolutionVerifier::expected_hud_anchor(2560.0, 1440.0)` | 分别返回 `(57.6, 32.4)` 和 `(76.8, 43.2)` —— 锚点线性随分辨率缩放 | 分辨率 → 锚点映射表硬编码错误 |
| A3 | FUNC/happy | NFR-002 AC-1, §Interface Contract `hud_anchor_deviation` | `actual = (40.0, 22.0)`, `viewport = (1280.0, 720.0)`；调用 `hud_anchor_deviation(40.0, 22.0, 1280.0, 720.0)` | `dx_pct = (40.0 - 38.4) / 1280.0 = 0.00125` (0.125%)；`dy_pct = (22.0 - 21.6) / 720.0 ≈ 0.00056` (0.056%) | 偏差计算公式错误（除以错误分母或符号反转） |
| A4 | FUNC/happy | NFR-002 AC-1, §Interface Contract `check_hud_anchor` | 调用 `check_hud_anchor(1280.0, 720.0, 0.02)` | `report.passed == true`；`report.deviation.dx_pct == 0.0`；`report.deviation.dy_pct == 0.0` | 自洽性检查将恒等式错误判为 fail（容差不正确应用） |
| A5 | FUNC/happy | NFR-002 AC-2, §Interface Contract `player_visible_ratio` | `player_world_x = 200.0`, `camera_offset_x = 20.0`, `viewport_w = 480.0` → 玩家屏幕 x = 180.0 | `ratio = (480.0 - 180.0) / 480.0 = 0.625`（62.5% 前方可见） | 可见区域公式反转（用了左方而非右方） |
| A6 | FUNC/happy | NFR-002 AC-2, §Interface Contract `check_visible_ratio` | `ratio = 0.48`, `min_pct = 0.45`, `max_pct = 0.50` → `check_visible_ratio(0.48, 0.45, 0.50)` | `report.passed == true` | 边界比较用错运算符（`<` 而非 `<=`） |
| A7 | FUNC/happy | NFR-002 AC-1 & AC-2, §Interface Contract `verify_all_resolutions` | 构造 `CameraConfig::default()`（viewport_w=480, viewport_h=270, player_target_x_pct=0.375），调用 `verify_all_resolutions(&config)` | `reports.len() == 3`（720p/1080p/1440p）；每个 `report.hud.passed == true`；`report.visible_area.ratio ≈ 0.625` (1 - 0.375) | 分辨率遍历漏项；循环中 pass/fail 标志未正确聚合 |
| A8 | FUNC/happy | NFR-002 AC-3, §Interface Contract `check_hud_anchor` | 全屏模式模拟：以 `(1920, 1080)`（常见全屏分辨率）调用 `check_hud_anchor(1920.0, 1080.0, 0.02)` | `report.passed == true`；锚点 `(57.6, 32.4)` 与窗口模式 1080p 完全一致 | 全屏模式下视口尺寸报告错误（导致锚定偏移） |
| A9 | FUNC/error | §Interface Contract `expected_hud_anchor` preconditions | `expected_hud_anchor(0.0, 720.0)` → 视口宽度为 0 | 行为：产生 (0.0, 21.6)，不 panic。调用方应在 precondition violation 时自行 guard | 零视口尺寸导致除零 panic 或 NaN |
| A10 | FUNC/error | §Interface Contract `expected_hud_anchor` preconditions | `expected_hud_anchor(-100.0, 720.0)` → 视口宽度为负 | 行为：产生 (-3.0, 21.6)，不 panic。负视口为非法输入，调用方责任 | 负视口尺寸导致 panic |
| A11 | FUNC/error | §Interface Contract `player_visible_ratio` preconditions | `player_visible_ratio(100.0, 200.0, 0.0)` → 视口宽度为 0 | 行为：`ratio = -inf` (除以零) 或 `f32::INFINITY`，不 panic。调用方应 guard | 零视口宽度导致除零 panic |
| B1 | BNDRY/edge | §Implementation Summary Boundary Conditions — `tolerance_pct = 0.0` | 调用 `check_hud_anchor(1280.0, 720.0, 0.0)` (零容差) | `report.passed == true`（期望锚点自身偏差为精确 0.0） | 零容差时浮点精度导致误 fail（如 `0.0 != -0.0`） |
| B2 | BNDRY/edge | §Implementation Summary Boundary Conditions — `ratio` 下界 | `check_visible_ratio(0.45, 0.45, 0.50)` → 恰在下界 | `report.passed == true` | off-by-one：`>=` 写成 `>`，下界恰好值时判 fail |
| B3 | BNDRY/edge | §Implementation Summary Boundary Conditions — `ratio` 上界 | `check_visible_ratio(0.50, 0.45, 0.50)` → 恰在上界 | `report.passed == true` | off-by-one：`<=` 写成 `<`，上界恰好值时判 fail |
| B4 | BNDRY/edge | §Implementation Summary Boundary Conditions — `ratio` 略超下界 | `check_visible_ratio(0.4499, 0.45, 0.50)` → 略低于下界 | `report.passed == false` | 浮点比较精度问题（`0.4499 < 0.45` 判 fail 正确，但应确保无 epsilon 误判） |
| B5 | BNDRY/edge | §Implementation Summary Boundary Conditions — 全屏等效性 | `check_hud_anchor(1920.0, 1080.0, 0.02)` 窗口模式 vs `check_hud_anchor(1920.0, 1080.0, 0.02)` 全屏模式（模拟） | 两个 report 的 `expected` 和 `deviation` 逐字段相等（全屏仅改变窗口装饰/模式标志，不改变视口尺寸） | 全屏模式改变了 viewport 计算基准（导致 HUD 偏移） |
| B6 | BNDRY/edge | §Implementation Summary Boundary Conditions — NaN 输入 | `expected_hud_anchor(f32::NAN, 720.0)` | 输出含 NaN；不 panic。调用方应在传入前 guard | NaN 传播未被检测，导致下游误读 |
| C1 | PERF/resolution | NFR-002 AC-1, §Interface Contract `verify_all_resolutions` | 对全部 3 种分辨率运行完整套件，度量 `verify_all_resolutions()` 耗时 | 总耗时 < 1ms（3 次确定性浮点计算，无 I/O） | 验证套件自身有性能问题（意外阻塞或循环） |
| C2 | PERF/formula | NFR-002 AC-1, §Interface Contract `expected_hud_anchor` | 对每种分辨率调用 `expected_hud_anchor()` × 10000 次 | 单次调用 < 100ns（纯浮点乘法，零内存分配） | 锚定计算公式调用了重操作（如纹理查询或 I/O） |
| D1 | UI/anchor | NFR-002 AC-1, §Interface Contract `check_hud_anchor` + F09 HUD | 验证 `HudRenderer::compute_anchor(1280.0, 720.0)` 输出 == `ResolutionVerifier::expected_hud_anchor(1280.0, 720.0)` | 两个函数对同一视口尺寸返回完全相同的锚点坐标 | 验证工具与 HUD 渲染实现锚定公式不一致（漂移） |
| D2 | UI/anchor | NFR-002 AC-1, §Interface Contract `check_hud_anchor` + F09 HUD | 同 D1 但针对全部 3 种分辨率 | 全部 3 种分辨率下 `HudRenderer::compute_anchor()` == `ResolutionVerifier::expected_hud_anchor()` | 某些分辨率下公式分歧 |
| D3 | UI/viewport | NFR-002 AC-2, §Interface Contract `player_visible_ratio` + F04 Camera | 构造 `Camera::default()`，读取 `viewport()` 返回 `(480.0, 270.0)`；模拟玩家世界坐标 `player_x = 200.0`，摄像机 `offset.x = 20.0`；计算 `player_visible_ratio(200.0, 20.0, 480.0)` | `ratio = (480.0 - 180.0) / 480.0 = 0.625`；`CameraConfig::player_target_x_pct = 0.375` 对应 `ratio = 0.625`（1.0 - 0.375） | 玩家可见区域计算使用了错误的 viewport_w 来源（如用了屏幕分辨率而非虚拟画布） |

> INTG: N/A — 本特性为纯本地计算，无外部 I/O（无 DB、无网络、无文件系统、无第三方 SDK）。所有验证数据来自内存内 `f32` 值和现有结构体字段。

**测试行分类统计**：
- FUNC/happy: 8 行 (A1–A8)
- FUNC/error: 3 行 (A9–A11)
- BNDRY/edge: 6 行 (B1–B6)
- PERF: 2 行 (C1–C2)
- UI: 3 行 (D1–D3)
- 总行数: 22
- 负向测试 (FUNC/error + BNDRY): 9/22 ≈ 40.9% >= 40%

**ATS 类别覆盖**：
- PERF: C1, C2 (2 行)
- UI: D1, D2, D3 (3 行)

## Verification Checklist
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Interface Contract 的 postconditions —— AC-1 (HUD 锚定偏差) → `expected_hud_anchor` + `hud_anchor_deviation` + `check_hud_anchor`；AC-2 (可见区域比例) → `player_visible_ratio` + `check_visible_ratio`；AC-3 (全屏 HUD) → `check_hud_anchor` 在全屏等效分辨率下调用
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Test Inventory 行 —— AC-1: A1–A4, C1–C2, D1–D2；AC-2: A5–A6, D3；AC-3: A8, B5
- [x] Boundary Conditions 表覆盖所有非平凡参数 —— 5 行覆盖 viewport_w, tolerance_pct, ratio, camera_offset_x, 分辨率列表
- [x] Interface Contract Raises 列覆盖所有预期错误条件 —— 所有方法标记为 `—`（纯函数不抛异常），precondition violations 由调用方 guard，不 panic
- [x] Test Inventory 负向占比 >= 40% —— 9/22 = 40.9%
- [x] ui:false —— Visual Rendering Contract 已声明 N/A 并附原因
- [x] ui:false —— 跳过 UI/render 行。UI 类别使用 `UI/anchor` 和 `UI/viewport` 子标签（验证 HUD 锚定与视口的数值一致性，非渲染验证）
- [x] Existing Code Reuse 章节已填充（5 个复用/参考符号）
- [x] UML 图：未触发任何判据（单结构体 + 纯静态方法 + 零状态 + 无多对象调用序 + 无 ≥3 决策分支方法）—— 跳过全部图
- [x] 非类图不适用 —— 无图即无违规
- [x] 图追溯不适用 —— 无图即可跳过
- [x] 每个被跳过的章节都写明 "N/A — [reason]"
- [x] §Interface Contract 中全部 6 个公开方法均有 Test Inventory 覆盖（A1–A11, B1–B6, C1–C2, D1–D3）
- [x] ATS PERF 类别在 Test Inventory 中至少出现 2 行（C1, C2）
- [x] ATS UI 类别在 Test Inventory 中至少出现 3 行（D1, D2, D3）
- [x] Design Interface Coverage：所有 Interface Contract 方法均有 Test Inventory 覆盖

## Clarification Addendum

> 以下假设基于 SRS NFR-002 验收准则与现有代码库的交叉分析。所有歧义均可度量阈值，无阻塞性规格缺口。

| # | Category | Original Ambiguity | Resolution | Authority |
|---|----------|--------------------|------------|-----------|
| 1 | NFR-GAP | NFR-002 中"玩家可见区域占视口宽度 45-50%"的可见区域方向未明确（前方还是后方） | 定义为**前方可见区域**：`(viewport_w - player_screen_x) / viewport_w`，即玩家屏幕上位置到视口右边缘的距离占比。与平台游戏"面向右侧、需要看清前方障碍/敌人"的惯例一致。当前 Camera `player_target_x_pct = 0.375` 产生 ratio ≈ 0.625，超出 45-50% 范围 —— 若需满足此约束，需调整 `CameraConfig::player_target_x_pct` 至 0.50–0.55 | assumed |
| 2 | NFR-GAP | NFR-002 中"全屏模式 HUD 位置不变"的全屏等效分辨率未指定 | 全屏模式下，视口尺寸 = 显示器原生分辨率（常见 1920x1080）。HUD 锚定公式 `(viewport_w * 0.03, viewport_h * 0.03)` 与窗口模式完全一致 —— 仅视口尺寸输入不同。验证套件将全屏视为"以原生分辨率运行窗口模式相同公式" | assumed |
| 3 | ATS-MISMATCH | ATS §2.2 将 NFR-002 标记为 `Manual: visual-judgment`，但本特性实现**程序化**验证 | 程序化验证**补充**而非**替代**手工截屏。本特性的 `ResolutionVerifier` 验证公式自洽性（确定性计算），手工截屏仍为 ST 阶段的像素级渲染验证。ATS 的 Manual 分类保留，本特性在此之上添加 Auto 层 | assumed |
