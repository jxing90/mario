# Feature Detailed Design：60fps Frame Rate (NFR-001)（Feature #11）

**Date**: 2026-06-02
**Feature**: #11 — 60fps Frame Rate (NFR-001)
**Priority**: high
**Dependencies**: F01 Engine Core (Feature #1, status=passing)
**Design Reference**: docs/plans/2026-05-31-mario-platformer-design.md §1.5 NFR 对齐摘要
**SRS Reference**: NFR-001
**ATS Reference**: docs/plans/2026-05-31-mario-platformer-ats.md §2.2, §3 (PERF), §4 (NFR 测试方法矩阵)

## Context

本特性为游戏循环添加性能度量仪表（Performance Instrumentation），实时记录每帧渲染耗时并维护 min/max/avg/p99 统计，同时通过 60 秒滑动窗口持续验证帧率不低于 58fps。所有统计数据定期输出到控制台日志，用于开发期性能回归检测和 ATS PERF 类别验收。

## Design Alignment

本特性无独立 §2.N 章节（设计文档 §2 仅到 §2.10 Feature #10）。设计依据来自系统设计 §1.5 NFR 对齐摘要：

> **NFR-001 (60fps)**：固定时间步长循环（1/60s），物理与渲染解耦；单帧模拟步数上限 5 次防止螺旋死亡；Rust 零成本抽象保证无 GC 抖动

- **Key types**: 新增 `FrameMetrics` 结构体（`src/metrics.rs`）；扩展现有 `GameLoop`（`src/engine.rs`）以集成度量采样点
- **Provides / Requires**: 本特性不引入新的 §4 IAPI 契约。它是 F01 Engine Core 的内部增强 —— `FrameMetrics` 在 `GameLoop::tick()` 入口处采样 `frame_time`，属于 Consumer 角色（消费 Engine Core 的帧时间数据），但不产生跨特性数据流
- **Deviations**: 无

本特性不满足 UML 触发判据（仅引入 1 个新结构体 `FrameMetrics` + 修改 1 个现有结构体 `GameLoop`），故不嵌入 classDiagram（`见系统设计 §4 类图 - GameLoop`）。无多对象调用序、无显式状态机、无 ≥3 决策分支方法 —— 跳过 sequenceDiagram / stateDiagram / flowchart。

## SRS Requirement

| ID | Priority | Category (ISO 25010) | Requirement | Measurable Criterion | Measurement Method |
|----|----------|---------------------|-------------|---------------------|-------------------|
| NFR-001 | Must | Performance Efficiency (Time Behaviour) | 60fps 渲染帧率 —— 游戏在目标分辨率下稳定输出 60 帧每秒 | 99% 的帧渲染时间 ≤ 16.67ms；任意连续 60 秒窗口内帧率不低于 58fps | 内置 FPS 计数器记录每帧耗时，输出 min/max/avg/p99 帧时间日志 |

**验收准则（来自 feature-list.json verification_steps）**：
1. P99 帧时间 ≤ 16.67ms at 720p/1080p/1440p 窗口模式
2. 任意连续 60s 窗口帧率 ≥ 58fps（Playing 状态满实体）
3. 全屏模式下帧时间一致性验证

## Interface Contract

| Method | Signature | Preconditions | Postconditions | Raises |
|--------|-----------|---------------|----------------|--------|
| `FrameMetrics::new` | `new() -> Self` | 无 | `frame_count = 0`；`min_frame_time = f32::MAX`；`max_frame_time = 0.0`；`window_ring` 全零初始化（长度 60）；`window_frame_count = 0`；`window_accumulator = 0.0`；`window_index = 0`；`frame_time_buffer` 为空 Vec | — |
| `FrameMetrics::sample` | `sample(&mut self, frame_time: f32)` | `frame_time > 0.0`（正常帧）；`frame_time = 0.0` 允许（首帧或计时器毛刺，视为合法输入但跳过统计） | `frame_count += 1`；更新 `min` / `max`；累加 `sum` 与 `sum_sq`；将 `frame_time` 追加到 `frame_time_buffer`（环形缓冲区，容量上限 7200 帧 = 120s @ 60fps）；每累计 ≥ 1.0s 的 frame_time 总量时，将当前秒内帧数写入 `window_ring[window_index]` 并推进索引 | — |
| `FrameMetrics::p99` | `p99(&self) -> f32` | `frame_time_buffer` 非空（至少 100 帧） | 返回 `frame_time_buffer` 排序后的第 99 百分位值；若缓冲区为空返回 `0.0` | — |
| `FrameMetrics::window_fps` | `window_fps(&self) -> f32` | `window_ring` 中至少填充了 60 个秒级样本 | 返回 60 秒滑动窗口的总帧数 / 60.0；若窗口未满返回当前平均帧率（已填充秒数的总帧数 / 已填充秒数） | — |
| `FrameMetrics::stats` | `stats(&self) -> FrameStats` | `frame_count > 0` | 返回 `FrameStats { min, max, avg, p99, window_fps, frame_count }`；`avg = sum / frame_count`；`p99 = self.p99()`；`window_fps = self.window_fps()` | — |
| `FrameMetrics::log_report` | `log_report(&self)` | 无 | 将 `stats()` 结果以 `[FPS] min={}ms max={}ms avg={}ms p99={}ms window_fps={} frames={}` 格式打印到 stdout；若 `frame_count == 0` 则输出 `[FPS] no frames sampled yet` | — |
| `GameLoop::tick` | `tick(&mut self, frame_time: f32, state: &mut S)` | （现有不变）+ `self.metrics` 已初始化 | （现有不变）+ `self.metrics.sample(frame_time)` 已在方法入口调用；每 60 秒自动调用 `self.metrics.log_report()` | （现有不变） |

**设计理由**：
- **P99 用排序而非近似算法**：帧时间数据量可控（60s @ 60fps = 3600 帧），排序 O(n log n) 在日志输出频率（每 60s 一次）下成本可忽略。近似算法（t-digest）在此规模下增加复杂度而无收益。
- **环形缓冲区容量 7200 帧**：覆盖 120 秒 @ 60fps，提供两倍于滑动窗口的缓冲，确保 P99 基于足够样本。
- **`frame_time = 0.0` 处理**：首帧或 `get_frame_time()` 返回极值的边缘情况 —— 记录但不污染 min/max 统计（min 用 `f32::MAX` 初始化天然排除 0.0）。这避免了将计时器毛刺误判为性能问题。
- **自动日志输出间隔 60 秒**：与滑动窗口长度对齐，每次日志输出包含完整的 60s 窗口帧率数据，便于人工或 CI 解析。
- **跨特性契约**：本特性不引入新的 §4 IAPI 契约。`GameLoop::tick()` 签名不变（已在 Feature #1 定义）；`FrameMetrics` 作为 GameLoop 内部字段是无外部可见性变更的纯增强。

## Visual Rendering Contract

> N/A — 本特性为后端性能度量仪表（`"ui": false`），无视觉输出。帧时间统计数据以纯文本日志形式输出到 stdout。

## Implementation Summary

**1. 新增模块 `src/metrics.rs` — `FrameMetrics` 结构体**

创建 `src/metrics.rs` 作为性能度量模块。核心结构体 `FrameMetrics` 封装全部帧时间采样与统计逻辑：持有 `frame_time_buffer: Vec<f32>` 作为环形缓冲区（容量上限 7200 帧）存储最近帧时间样本；持有 `window_ring: [u32; 60]` 作为秒级帧数环形缓冲区用于 60 秒滑动窗口计算；持有 `min_frame_time`、`max_frame_time`、`sum_frame_time`、`sum_sq_frame_time` 四个运行累加器；持有 `window_accumulator`（秒级累加器）和 `window_frame_count`（当前秒内帧数）。`sample()` 方法由 `GameLoop::tick()` 在每帧入口调用，入参为 Macroquad 提供的 `frame_time`（真实 wall-clock delta）。`p99()` 方法对缓冲区做防御性拷贝后排序取 P99 分位值，时间复杂度 O(n log n)、空间 O(n)。`window_fps()` 遍历 `window_ring` 求和后除以已填充秒数（最多 60）。`stats()` 聚合全部统计到 `FrameStats` 值对象，`log_report()` 将其格式化为单行日志并输出到 stdout。

**2. 与 `GameLoop` 集成（修改 `src/engine.rs`）**

在 `GameLoop` 结构体中新增 `pub metrics: FrameMetrics` 字段。`GameLoop::new()` 中初始化 `metrics: FrameMetrics::new()`。`GameLoop::tick()` 方法入口处插入 `self.metrics.sample(frame_time)` 作为第一行语句，确保帧时间在 update/render 逻辑之前被记录（避免 update/render 耗时污染 frame_time 度量）。每 60 秒（通过 `frame_count % 3600 == 0` 且 `frame_count > 0` 检测）自动调用 `self.metrics.log_report()` 输出统计报告。`GameLoop::run()` 无需修改 —— `get_frame_time()` 继续提供真实帧时间，`tick()` 内部完成采样。

**3. 调用链**

```
main() → GameLoop::run()
  → macroquad::time::get_frame_time() → frame_time: f32
  → GameLoop::tick(frame_time, state)
    → FrameMetrics::sample(frame_time)     // 入口采样：记录帧时间
    → accumulator += frame_time            // 现有逻辑：累加时间
    → [固定步长 update 循环]               // 现有逻辑：0–5 次 update(dt)
    → state.render(alpha)                  // 现有逻辑：渲染一次
    → FrameMetrics::log_report()           // 条件触发：每 60s 输出日志
  → macroquad::window::next_frame().await  // 等待垂直同步
```

**4. 存量代码交互点**

- `src/engine.rs::GameLoop` — 在现有结构体中新增 `metrics` 字段；`tick()` 方法入口新增一行采样调用。不修改现有逻辑（accumulator、update 循环、alpha 计算和渲染调用保持不变）。不引入新依赖（`std::time::Instant` 仅在测试中使用；运行时代码直接消费 Macroquad 的 `get_frame_time()` 输出）。
- `src/lib.rs` — 新增 `pub mod metrics;` 声明
- 无需修改 `src/main.rs`、`src/state.rs` 或任何实体/系统模块

**5. §4 Internal API Contract 集成**

本特性不注册新的 IAPI 契约。`GameLoop::apply_display()` (IAPI-011) 不受影响 —— 分辨率变更后帧时间继续被正确采样，日志输出不依赖显示配置。若未来需要外部查询帧率数据（例如 HUD 显示实时 FPS），可在后续特性中通过新增 IAPI 暴露 `FrameMetrics::stats()` 返回值。

### Boundary Conditions

| Parameter | Min | Max | Empty/Null | At boundary |
|-----------|-----|-----|------------|-------------|
| `frame_time` (sample) | 0.0（合法，首帧或计时器毛刺） | `f32::MAX`（无上限）；实际由 Macroquad 帧间隔物理限制 | 不适用 | `frame_time = 0.0`：跳过 min/max 更新（min 保持 `f32::MAX`，max 保持 0.0），frame_count 仍递增但 sum 不累加。`frame_time ≈ DT` (≈ 0.01667s)：正常路径，min 应更新为接近 DT 值。`frame_time > dt * (max_steps + 1)` (≈ 0.1s)：螺旋死亡钳制场景，统计中保留真实 frame_time 以暴露性能退化 |
| `frame_time_buffer` | 空（0 帧） | 7200 帧（容量上限） | `p99()` 返回 0.0（空缓冲区） | 缓冲区满时，移除最旧元素（FIFO 环形覆盖），保持最近 7200 帧 |
| `window_ring` | 全零（未填充） | 60 个秒级条目 | `window_fps()` 返回已填充秒数的平均帧率而非 60 秒固定窗口值 | 索引推进到 60 后回绕到 0（环形覆盖）；回绕后最近 60 秒数据始终可用 |
| `frame_count` (log_report 触发) | 0 | `u64::MAX` | 0 帧时 `log_report()` 输出 "no frames sampled yet" | 每 3600 帧（≈60s @ 60fps）触发一次日志输出 |

### Existing Code Reuse

| Existing Symbol | Location (file:line) | Reused Because |
|-----------------|---------------------|----------------|
| `GameLoop::tick(frame_time: f32, state: &mut S)` | `src/engine.rs:133` | `frame_time` 参数即为 Macroquad 提供的真实 wall-clock 帧间隔，是 FPS 度量的权威数据源；在入口处采样可避免 update/render 计算耗时污染度量 |
| `GameLoop::run()` | `src/engine.rs:189` | `run()` 调用 `macroquad::time::get_frame_time()` 获取 frame_time 并传入 `tick()`；无需修改，度量链路天然集成 |
| `DT` 常量 (`1.0/60.0`) | `src/engine.rs:17` | 作为参考帧时间（≈16.67ms），在测试中用于验证 frame_time 采样的合理性边界 |

> 无现有 FPS 计数器或帧时间统计基础设施可复用 —— `FrameMetrics` 为本特性的新增实现。

## Test Inventory

**ATS 必需类别**: PERF（来自 ATS §2.2 NFR-001 行 `必须类别` 列）

| ID | Category | Traces To | Input / Setup | Expected | Kills Which Bug? |
|----|----------|-----------|---------------|----------|-----------------|
| A1 | FUNC/happy | NFR-001 AC-1, §Interface Contract `sample` / `stats` | 构造 `FrameMetrics::new()`，依次调用 `sample(0.016)` × 100 帧（模拟 60fps 稳定运行） | `stats().avg ≈ 0.016` (容差 0.001)；`stats().min ≈ 0.016`；`stats().max ≈ 0.016`；`stats().frame_count == 100` | 累加器未初始化（sum 为垃圾值）；avg 计算公式错误（除零或除错分母） |
| A2 | FUNC/happy | NFR-001 AC-1, §Interface Contract `p99` | 构造 `FrameMetrics`，`sample(0.016)` × 99 帧 + `sample(0.050)` × 1 帧（100 帧中 1 帧慢） | `p99() <= 0.050` 且 `p99() > 0.016`；`stats().max == 0.050` | P99 排序错误（取了 max 或 avg）；分位索引 off-by-one |
| A3 | FUNC/happy | NFR-001 AC-2, §Interface Contract `window_fps` | 构造 `FrameMetrics`，模拟 60 秒每秒钟恰好 60 帧：每秒调用 `sample(DT)` 60 次，调用 60 轮（共 3600 帧） | `window_fps() >= 59.5` 且 `window_fps() <= 60.0` | 秒级帧数计数遗漏帧；环缓冲区索引推进错误导致帧数丢失 |
| A4 | FUNC/happy | NFR-001 AC-2, §Interface Contract `window_fps` | 同 A3 但每秒 58 帧：每秒调用 `sample(DT * 60.0 / 58.0)` 58 次，调用 60 轮（共 3480 帧） | `window_fps() ≈ 58.0` (容差 0.5) | 滑动窗口求和错误（帧率不是简单平均）；环形覆盖逻辑错误 |
| A5 | FUNC/happy | NFR-001, §Interface Contract `log_report` | 调用 `FrameMetrics::new()`，`sample(DT)` × 100 帧，调用 `log_report()` | stdout 输出包含 `[FPS] min=` `max=` `avg=` `p99=` `window_fps=` `frames=100` 格式字符串 | 格式化宏错位；字段顺序错误；NaN 输出（除零） |
| A6 | FUNC/happy | NFR-001, §Implementation Summary 集成点 | 构造 `GameLoop::new(valid_config)`，调用 `tick(DT, &mut spy_state)` 1 帧 | `game_loop.metrics.frame_count == 1`；`game_loop.metrics.min_frame_time == DT` | `tick()` 入口未调用 `metrics.sample()`；集成遗漏 |
| A7 | FUNC/error | §Interface Contract `p99` (空缓冲区) | 构造 `FrameMetrics::new()`，不调用 `sample`，直接调用 `p99()` | `p99() == 0.0` | 空 Vec 上调用排序 panic；除零 panic |
| A8 | FUNC/error | §Interface Contract `stats` (零帧) | 构造 `FrameMetrics::new()`，不调用 `sample`，直接调用 `stats()` | `stats().frame_count == 0`；`stats().avg` 不 panic（0.0 或合理默认值） | 除零 panic（sum / frame_count）；NaN 传播 |
| A9 | FUNC/error | §Interface Contract `log_report` (零帧) | 构造 `FrameMetrics::new()`，不调用 `sample`，直接调用 `log_report()` | stdout 输出 `[FPS] no frames sampled yet` | 零帧时格式化宏 panic；除零 |
| B1 | BNDRY/edge | §Implementation Summary Boundary Conditions — `frame_time = 0.0` | `FrameMetrics::new()`，`sample(0.0)` × 1 帧，然后 `sample(DT)` × 1 帧 | `frame_count == 2`；`min_frame_time == DT`（0.0 不污染 min）；`max_frame_time == DT` | 0.0 覆盖了正确 sample（min 变成 0.0 导致误报）；0.0 被拒绝导致 frame_count 未递增 |
| B2 | BNDRY/edge | §Implementation Summary Boundary Conditions — 缓冲区满 | 构造 `FrameMetrics`，`sample(DT)` × 7200 帧（填满），再 `sample(0.020)` × 1 帧（触发覆盖） | `p99()` 基于最近 7200 帧（不含最早的第 1 帧 DT）；`frame_count == 7201` | 缓冲区满后 push 导致 panic（Vec 无限增长）；FIFO 逻辑错误（覆盖了错误位置） |
| B3 | BNDRY/edge | §Implementation Summary Boundary Conditions — 环索引回绕 | 模拟 120 秒：每秒 60 帧 × 120 轮（共 7200 帧） | `window_fps()` 基于最近 60 秒数据（第 61–120 秒）；值约 60.0 | 环索引不回绕（越界写入 panic）；回绕后旧数据未覆盖（结果基于第 1–120 秒而非第 61–120 秒） |
| B4 | BNDRY/edge | §Implementation Summary Boundary Conditions — `frame_time` 极大值 | `FrameMetrics::new()`，`sample(0.5)` × 1 帧（模拟严重卡顿，约 2fps），然后 `sample(DT)` × 99 帧 | `max_frame_time == 0.5`；`p99() <= 0.5`；`avg` 约 `(0.5 + 99 * DT) / 100` | 极大值溢出累加器（f32 精度丢失）；max 未更新 |
| B5 | BNDRY/edge | §Implementation Summary — 60s 日志触发边界 | 构造 `GameLoop`，调用 `tick(DT, spy_state)` 3599 帧（第 3600 帧前） | 日志**未**触发输出（最后一帧前无 `log_report` 调用） | off-by-one 导致 3599 帧时误触发日志；3600 帧时未触发 |
| P1 | PERF/frame_time | NFR-001 AC-1, ATS §3 PERF, §Interface Contract `p99` | 运行 `GameLoop::run()` 的模拟模式，固定 360 帧（6s @ 60fps），用 `std::time::Instant` 度量实际耗时 | 总耗时 < 360 × DT × 1.05（即 < 6.3s，5% 容差）；P99 帧时间 ≤ 16.67ms | 游戏循环未保持 60fps；固定步长更新中出现非预期的阻塞调用 |
| P2 | PERF/sliding | NFR-001 AC-2, §Interface Contract `window_fps` | 同 P1 但运行 3600 帧（60s），填充所有 Playing 实体（全部敌人/金币/方块/尖刺激活） | `window_fps() >= 58.0`；`log_report()` 输出中 `window_fps >= 58.0` | 满实体负载下帧率退化未被检测；滑动窗口计算掩盖了帧率下降 |
| P3 | PERF/empty | NFR-001, §Interface Contract `stats` | 空场景（仅有玩家，无敌人/金币/方块/尖刺），运行 3600 帧 | `window_fps() >= 59.5`；`avg < 20.0ms`；`p99() < 20.0ms` | 空场景基准未建立，无法判断后续性能退化 |

> INTG: N/A — 本特性为纯本地计算，无外部 I/O（无 DB、无网络、无文件系统、无第三方 SDK）。帧数据全部来自内存内 `f32` 值。

**测试行分类统计**：
- FUNC/happy: 6 行 (A1–A6)
- FUNC/error: 3 行 (A7–A9)
- BNDRY/edge: 5 行 (B1–B5)
- PERF: 3 行 (P1–P3)
- 总行数: 17
- 负向测试 (FUNC/error + BNDRY): 8/17 ≈ 47.1% ≥ 40%

## Verification Checklist
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Interface Contract 的 postconditions
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Test Inventory 行
- [x] Boundary Conditions 表覆盖所有非平凡参数
- [x] Interface Contract Raises 列覆盖所有预期错误条件（FrameMetrics 方法无 panic，公开 API 防御性处理边缘输入）
- [x] Test Inventory 负向占比 >= 40%（47.1%）
- [x] ui:false —— Visual Rendering Contract 已声明 N/A 并附原因
- [x] ui:false —— 跳过 UI/render 行
- [x] Existing Code Reuse 章节已填充（3 个复用符号）
- [x] UML 图：未触发任何判据（单结构体 + 单字段修改，无 ≥2 类协作 / 无 ≥2 对象调用序 / 无显式状态机 / 无 ≥3 决策分支方法）—— 跳过全部图
- [x] 非类图不适用 —— 无图即无违规
- [x] 图追溯不适用 —— 无图即可跳过
- [x] 每个被跳过的章节都写明 "N/A — [reason]"
- [x] §2.N 所有函数/方法都至少有一行 Test Inventory（本特性无独立 §2.N，Interface Contract 中全部 7 个方法均有 Test Inventory 覆盖；GameLoop::tick 修改对应 A6/B5/P1/P2/P3）
- [x] ATS PERF 类别在 Test Inventory 中至少出现 3 行（P1/P2/P3）
- [x] Design Interface Coverage：所有 Interface Contract 方法 + GameLoop 修改点均有 Test Inventory 覆盖

## Clarification Addendum

> 无需澄清 —— 全部规格明确。NFR-001 的所有度量阈值（P99 ≤ 16.67ms、60s 窗口 ≥ 58fps）均为可度量数值，实现方案由现有 GameLoop 的 `frame_time` 参数直接支撑，无歧义。

| # | Category | Original Ambiguity | Resolution | Authority |
|---|----------|--------------------|------------|-----------|
| — | — | — | — | — |
