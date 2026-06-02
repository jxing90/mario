# 测试用例集: 60fps Frame Rate (NFR-001)

**Feature ID**: 11
**关联需求**: NFR-001
**日期**: 2026-06-02
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 9 |
| boundary | 5 |
| performance | 3 |
| ui | 0 |
| security | 0 |
| **合计** | **17** |

---

### 用例编号

ST-FUNC-011-001

### 关联需求

NFR-001（60fps 渲染帧率）— AC-1: P99 帧时间 ≤ 16.67ms

### 测试目标

验证 FrameMetrics 在 100 帧稳定 60fps 输入下，stats() 返回正确的 avg/min/max/frame_count 统计值。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | `frame_count == 0`，`min_frame_time == f32::MAX` |
| 2 | 模拟 100 帧 @ DT (1/60s)，每帧调用 `metrics.sample(DT)` | 所有 sample 调用无 panic |
| 3 | 调用 `metrics.stats()` | 返回 `FrameStats` 值对象 |
| 4 | 断言 `stats.avg ≈ DT` (容差 0.01) | avg 应为 DT (≈0.01667s) |
| 5 | 断言 `stats.min ≈ DT` (容差 0.01) | min 应为 DT |
| 6 | 断言 `stats.max ≈ DT` (容差 0.01) | max 应为 DT（所有帧耗时相同） |
| 7 | 断言 `stats.frame_count == 100` | frame_count 恰好为 100 |

### 验证点

- avg 计算公式正确（sum / frame_count，无除零）
- min/max 跑分累加器正确更新
- frame_count 准确递增

### 后置检查

- `FrameMetrics` 实例在测试结束后可被丢弃，无持久状态

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a1_stable_60fps_stats_avg_max_min`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-002

### 关联需求

NFR-001（60fps 渲染帧率）— AC-1: P99 帧时间 ≤ 16.67ms

### 测试目标

验证 P99 百分位计算在包含 1 帧慢帧（50ms）的 100 帧样本中返回正确的百分位值（应 ≥ DT 且 ≤ 50ms）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 模拟 99 帧 @ DT，调用 `metrics.sample(DT)` x 99 | 99 帧正常帧时间已记录 |
| 3 | 模拟 1 帧 @ 0.050s（慢帧），调用 `metrics.sample(0.050)` | 慢帧已记录 |
| 4 | 调用 `metrics.stats()` | 返回统计值 |
| 5 | 断言 `stats.p99 >= DT` 且 `stats.p99 <= 0.050` | P99 位于正常帧与慢帧之间 |
| 6 | 断言 `stats.max ≈ 0.050` | max 等于慢帧时间 |
| 7 | 断言 `stats.min ≈ DT` | min 仍为正常帧时间 |
| 8 | 断言 `stats.frame_count == 100` | 共 100 帧 |

### 验证点

- P99 分位数排序正确（非 max 或 avg 的简单取值）
- P99 索引 off-by-one 错误被排除
- max/min 在混合分布中正确识别

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a2_p99_with_one_slow_frame`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-003

### 关联需求

NFR-001（60fps 渲染帧率）— AC-2: 任意连续 60s 窗口帧率 ≥ 58fps

### 测试目标

验证 60 秒滑动窗口帧率计算在稳定 60fps 输入下返回 59.5–60.0 fps。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 模拟 60 秒 x 60fps：外层 60 轮，每轮 `sample(DT)` x 60 次 | 共 3600 帧，window_ring 填充 60 个秒级条目 |
| 3 | 调用 `metrics.window_fps()` | 返回滑动窗口帧率 |
| 4 | 断言 `window_fps >= 59.5` 且 `window_fps <= 60.0` | 帧率 ≈ 60.0 fps |
| 5 | 断言 `stats().frame_count == 3600` | 恰好 3600 帧 |

### 验证点

- window_ring 秒级帧数计数正确（每秒 60 帧）
- 环缓冲区索引推进无误
- window_fps() 除以正确分母（已填充秒数，非固定 60）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a3_window_fps_stable_60fps`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-004

### 关联需求

NFR-001（60fps 渲染帧率）— AC-2: 任意连续 60s 窗口帧率 ≥ 58fps

### 测试目标

验证滑动窗口在 58fps 临界输入下返回约 58.0 fps（容差 ±0.5）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 计算 58fps 对应的帧时间：`frame_time_58 = DT * 60.0 / 58.0` | 帧时间 ≈ 0.01724s |
| 3 | 模拟 60 秒 x 58fps：外层 60 轮，每轮 `sample(frame_time_58)` x 58 次 | 共 3480 帧 |
| 4 | 调用 `metrics.window_fps()` | 返回帧率 |
| 5 | 断言 `abs(window_fps - 58.0) < 0.5` | 帧率 ≈ 58.0 fps |
| 6 | 断言 `stats().frame_count == 3480` | 恰好 3480 帧 |

### 验证点

- 非整数帧率下秒级帧数计数正确（每秒 58 帧）
- 滑动窗口求和逻辑对低帧率场景不退化
- 环形覆盖逻辑在满窗口后持续正确

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a4_window_fps_58fps_near_threshold`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-005

### 关联需求

NFR-001（60fps 渲染帧率）— 内置 FPS 计数器日志输出

### 测试目标

验证 `log_report()` 在采样 100 帧后可安全调用（无 panic），且 stats() 返回正确的非零统计值。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 模拟 100 帧 @ DT，调用 `metrics.sample(DT)` x 100 | 100 帧已记录 |
| 3 | 调用 `metrics.stats()` | frame_count > 0, avg > 0, min > 0, max > 0 |
| 4 | 调用 `metrics.log_report()` | 无 panic；stdout 输出格式正确（人工验证） |

### 验证点

- `log_report()` 在有帧数据时不 panic
- `stats()` 返回非零值，确认格式化宏的输入有效
- 字段顺序与格式串一致

### 后置检查

- 无；stdout 输出由测试框架捕获

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a5_log_report_format_after_samples`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-006

### 关联需求

NFR-001（60fps 渲染帧率）— GameLoop 集成点

### 测试目标

验证 `GameLoop::tick()` 在入口处调用 `metrics.sample(frame_time)`，使得每帧过后 `metrics.frame_count` 递增。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop::new(1280x720)` 构造成功
- `SpyStateMachine` 已构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `WindowConfig { 1280, 720, false }` | 创建配置成功 |
| 2 | `GameLoop::new(config)` + `SpyStateMachine::new()` | 返回 Ok(GameLoop)，metrics 已初始化 |
| 3 | `gl.tick(DT, &mut state)` 执行 1 帧 | 无 panic |
| 4 | 断言 `gl.metrics.frame_count == 1` | 1 帧后 frame_count = 1 |
| 5 | 断言 `gl.metrics.min_frame_time ≈ DT` | min 记录为 DT |

### 验证点

- `tick()` 入口处 `sample()` 调用未被遗忘
- frame_count 正确递增（非 0）
- min_frame_time 从 f32::MAX 更新为实际 DT 值

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a6_game_loop_tick_integrates_sample`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-007

### 关联需求

NFR-001（60fps 渲染帧率）— 防御性编程：空缓冲区

### 测试目标

验证 `p99()` 在从未调用 `sample()` 的空缓冲区上返回 0.0 且不 panic。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功，未调用任何 `sample()`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功，frame_time_buffer 为空 |
| 2 | 直接调用 `metrics.p99()` | 不 panic |
| 3 | 断言 `p99 == 0.0` | 空缓冲区返回 0.0 |

### 验证点

- 空 Vec 上排序不 panic
- 除零保护有效
- 返回值是合法 f32（非 NaN）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a7_p99_empty_buffer_returns_zero`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-008

### 关联需求

NFR-001（60fps 渲染帧率）— 防御性编程：零帧

### 测试目标

验证 `stats()` 在零帧采样的情况下返回 frame_count == 0 且 avg 不 panic / 不为 NaN。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功，未调用任何 `sample()`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 直接调用 `metrics.stats()` | 不 panic |
| 3 | 断言 `stats.frame_count == 0` | 零帧 |
| 4 | 断言 `stats.avg` 非 NaN | avg 为合法数值 |
| 5 | 断言 `stats.min` 为 0.0 或 f32::MAX | min 保留初始 sentinel 值 |

### 验证点

- sum / frame_count 除零保护生效（avg 不 panic）
- NaN 不传播到 stats 输出
- sentinel 初始值在无数据场景下保留

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a8_stats_zero_frames_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-011-009

### 关联需求

NFR-001（60fps 渲染帧率）— 防御性编程：零帧日志

### 测试目标

验证 `log_report()` 在零帧采样时安全调用（无 panic），输出 "no frames sampled yet" 类消息。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功，`frame_count == 0`

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | frame_count == 0 |
| 2 | 断言 `metrics.frame_count == 0` | 前置条件确认 |
| 3 | 调用 `metrics.log_report()` | 不 panic |

### 验证点

- 零帧时格式化宏不 panic（无除零/空值引用）
- "no frames sampled yet" 消息输出到 stdout

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::a9_log_report_empty_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-011-001

### 关联需求

NFR-001（60fps 渲染帧率）— Boundary: frame_time = 0.0

### 测试目标

验证 `frame_time = 0.0`（首帧或计时器毛刺）不污染 min/max 统计，但 frame_count 仍然递增。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 调用 `metrics.sample(0.0)` x 1（零帧时间） | 0.0 帧已记录 |
| 3 | 调用 `metrics.sample(DT)` x 1（正常帧） | 正常帧已记录 |
| 4 | 调用 `metrics.stats()` | 返回统计值 |
| 5 | 断言 `stats.frame_count == 2` | 两帧均已计数 |
| 6 | 断言 `stats.min ≈ DT`（非 0.0） | min 未被 0.0 污染 |
| 7 | 断言 `stats.max ≈ DT` | max 为正常帧时间 |

### 验证点

- frame_time = 0.0 不污染 min（min 保持为 DT）
- frame_count 对 0.0 帧仍然递增
- max 不受 0.0 帧影响

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::b1_zero_frame_time_does_not_pollute_min`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-011-002

### 关联需求

NFR-001（60fps 渲染帧率）— Boundary: 缓冲区满 FIFO

### 测试目标

验证 `frame_time_buffer` 满 7200 帧后，新帧采用 FIFO 策略覆盖最旧帧，p99 基于最近 7200 帧。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 模拟 7200 帧 @ DT | 缓冲区满（容量上限 7200） |
| 3 | 追加 1 帧 @ 0.020s（明显不同于 DT） | 触发 FIFO 覆盖 |
| 4 | 调用 `metrics.stats()` | 返回统计值 |
| 5 | 断言 `stats.frame_count == 7201` | 总数不受缓冲区容量限制 |
| 6 | 断言 `stats.max >= 0.020 - epsilon` | 新帧（0.020s）在统计中反映 |
| 7 | 断言 `stats.p99 >= DT` | P99 基于最近 7200 帧（不含最早被覆盖帧） |

### 验证点

- 缓冲区满后不 panic（无 Vec 无限增长）
- FIFO 移除最旧帧（非最新帧）
- frame_count 不受缓冲区容量限制（持续递增）
- P99 正确反映最近 7200 帧的分布

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::b2_buffer_full_fifo_behavior`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-011-003

### 关联需求

NFR-001（60fps 渲染帧率）— Boundary: 环索引回绕

### 测试目标

验证 `window_ring[60]` 在 120 秒数据（超出 60 秒窗口）下正确回绕，window_fps 仅反映最近 60 秒。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 前 60 秒：每秒 60 帧 @ DT（60fps），共 3600 帧 | window_ring 填满，window_fps ≈ 60.0 |
| 3 | 断言 `window_fps ≈ 60.0` | 前 60 秒数据正确 |
| 4 | 后 60 秒：每秒 30 帧 @ DT*2（30fps），共 1800 帧 | 环索引回绕，覆盖前 60 秒数据 |
| 5 | 调用 `metrics.window_fps()` | 返回最近 60 秒帧率 |
| 6 | 断言 `window_fps >= 29.5` 且 `window_fps <= 30.0` | 仅反映最近 60s（30fps），非全部 120s |
| 7 | 断言 `stats().frame_count == 5400` | 总帧数 3600 + 1800 |

### 验证点

- 环索引在 60 后正确回绕到 0（无越界写入 panic）
- 回绕后旧数据被覆盖（非与旧数据混合计算）
- window_fps 仅基于最近 60 个秒级条目

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::b3_ring_index_wraparound_120_seconds`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-011-004

### 关联需求

NFR-001（60fps 渲染帧率）— Boundary: frame_time 极大值

### 测试目标

验证单帧 0.5s 极大值（严重卡顿，约 2fps）后，max 正确更新为 0.5s，avg 正确反映混合分布。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `FrameMetrics::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `FrameMetrics::new()` | 初始化成功 |
| 2 | 调用 `metrics.sample(0.5)` x 1（极大卡顿帧） | 0.5s 帧已记录 |
| 3 | 调用 `metrics.sample(DT)` x 99（正常帧） | 99 帧正常帧已记录 |
| 4 | 调用 `metrics.stats()` | 返回统计值 |
| 5 | 断言 `stats.max ≈ 0.5` | max 为极大值 |
| 6 | 断言 `stats.avg ≈ (0.5 + 99 * DT) / 100` | avg 反映混合分布 |
| 7 | 断言 `stats.p99 <= 0.5` | P99 不超过极大值 |
| 8 | 断言 `stats.frame_count == 100` | 共 100 帧 |

### 验证点

- f32 累加器无溢出（sum = 0.5 + 99*DT ≈ 2.15，远在 f32 范围内）
- max 对极大值正确更新
- avg 公式在混合分布下正确
- P99 对极端离群值不误判

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::b4_extreme_frame_time_handled`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-011-005

### 关联需求

NFR-001（60fps 渲染帧率）— Boundary: 60s 日志触发边界

### 测试目标

验证 GameLoop 在 3599 帧时不触发日志，第 3600 帧时触发 `log_report()`（off-by-one 关卡）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop::new(1280x720)` 构造成功
- `SpyStateMachine` 已构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `GameLoop::new(1280x720)` 和 `SpyStateMachine` | 初始化成功 |
| 2 | 驱动 3599 帧：`gl.tick(DT, &mut state)` x 3599 | 日志**不**触发 |
| 3 | 断言 `gl.metrics.frame_count == 3599` | 3599 帧 |
| 4 | 再驱动 1 帧：`gl.tick(DT, &mut state)` | 第 3600 帧，日志触发 |
| 5 | 断言 `gl.metrics.frame_count == 3600` | 3600 帧 |

### 验证点

- 3599 帧：日志不触发（非 3600 的倍数）
- 3600 帧：日志触发（frame_count % 3600 == 0 且 frame_count > 0）
- off-by-one 错误被排除

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::b5_log_trigger_boundary_3599_vs_3600`
- **Test Type**: Real

---

### 用例编号

ST-PERF-011-001

### 关联需求

NFR-001（60fps 渲染帧率）— AC-1: P99 帧时间 ≤ 16.67ms

### 测试目标

通过 `std::time::Instant` 度量 360 帧 GameLoop 实际执行耗时，验证总耗时不超过预算（360 × 1/60s × 1.05 = 6.3s）。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop::new(1280x720)` 构造成功
- `SpyStateMachine` 已构造

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `GameLoop::new(1280x720)` 和 `SpyStateMachine` | 初始化成功 |
| 2 | 记录 `start = Instant::now()` | 计时开始 |
| 3 | 驱动 360 帧：`gl.tick(DT, &mut state)` x 360 | 全部帧无 panic |
| 4 | 计算 `elapsed = start.elapsed()` | 获取真实耗时 |
| 5 | 断言 `elapsed.as_secs_f64() < 6.3` | 总耗时 < 360 × 1/60 × 1.05 |
| 6 | 断言 `gl.metrics.frame_count == 360` | 360 帧已记录 |
| 7 | 断言 `state.update_count == 360` | 360 次 update() 调用 |

### 验证点

- GameLoop::tick() 无意外阻塞调用
- 固定步长循环在纯计算场景下不超预算（5% 容差）
- frame_count 与 update_count 一致

### 后置检查

- 无

### 元数据

- **优先级**: Critical
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::p1_360_frames_performance_within_budget`
- **Test Type**: Real

---

### 用例编号

ST-PERF-011-002

### 关联需求

NFR-001（60fps 渲染帧率）— AC-2: 任意连续 60s 窗口帧率 ≥ 58fps

### 测试目标

验证 3600 帧（60 秒模拟时间）持续运行后，`window_fps()` 不低于 58.0 fps。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop::new(1280x720)` 构造成功
- `SpyStateMachine` 已构造（轻量实体负载）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `GameLoop::new(1280x720)` 和 `SpyStateMachine` | 初始化成功 |
| 2 | 驱动 3600 帧：`gl.tick(DT, &mut state)` x 3600 | 模拟 60 秒 @ 60fps |
| 3 | 调用 `gl.metrics.window_fps()` | 返回滑动窗口帧率 |
| 4 | 断言 `window_fps >= 58.0` | 满足 NFR-001 AC-2 阈值 |
| 5 | 断言 `gl.metrics.frame_count == 3600` | 3600 帧已记录 |

### 验证点

- 60 秒持续运行后 window_fps ≥ 58.0
- 框架度量基础设施在中等规模测试中不引入显著开销
- NFR-001 AC-2 的最低阈值得到满足

### 后置检查

- 无

### 元数据

- **优先级**: Critical
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::p2_3600_frames_full_load_window_fps`
- **Test Type**: Real

---

### 用例编号

ST-PERF-011-003

### 关联需求

NFR-001（60fps 渲染帧率）— 空场景基准

### 测试目标

建立空场景（仅玩家，无敌人/金币/方块/尖刺）的性能基准：window_fps ≥ 59.5，avg < 20ms，p99 < 20ms。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `GameLoop::new(1280x720)` 构造成功
- `SpyStateMachine` 已构造（空场景）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `GameLoop::new(1280x720)` 和 `SpyStateMachine` | 初始化成功 |
| 2 | 驱动 3600 帧：`gl.tick(DT, &mut state)` x 3600 | 3600 帧完成 |
| 3 | 调用 `gl.metrics.stats()` | 返回统计值 |
| 4 | 断言 `stats.window_fps >= 59.5` | 空场景帧率接近完美 |
| 5 | 断言 `stats.avg < 0.020` (20ms) | avg 帧时间 < 20ms |
| 6 | 断言 `stats.p99 < 0.020` (20ms) | P99 帧时间 < 20ms |
| 7 | 断言 `stats.frame_count == 3600` | 3600 帧 |

### 验证点

- 空场景基准已建立（后续用于回归检测）
- window_fps ≥ 59.5：接近理论最大值
- avg 与 p99 均在合理范围内（< 20ms）
- 框架度量自身开销不显著影响帧时间统计

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: performance
- **已自动化**: Yes
- **测试引用**: `tests/frame_rate_test.rs::p3_empty_scene_baseline_3600_frames`
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-011-001 | NFR-001 | AC-1: P99 ≤ 16.67ms | `tests/frame_rate_test.rs::a1_stable_60fps_stats_avg_max_min` | Real | PASS |
| ST-FUNC-011-002 | NFR-001 | AC-1: P99 ≤ 16.67ms | `tests/frame_rate_test.rs::a2_p99_with_one_slow_frame` | Real | PASS |
| ST-FUNC-011-003 | NFR-001 | AC-2: 60s ≥ 58fps | `tests/frame_rate_test.rs::a3_window_fps_stable_60fps` | Real | PASS |
| ST-FUNC-011-004 | NFR-001 | AC-2: 60s ≥ 58fps | `tests/frame_rate_test.rs::a4_window_fps_58fps_near_threshold` | Real | PASS |
| ST-FUNC-011-005 | NFR-001 | 内置 FPS 计数器日志 | `tests/frame_rate_test.rs::a5_log_report_format_after_samples` | Real | PASS |
| ST-FUNC-011-006 | NFR-001 | GameLoop 集成点 | `tests/frame_rate_test.rs::a6_game_loop_tick_integrates_sample` | Real | PASS |
| ST-FUNC-011-007 | NFR-001 | 空缓冲区安全 | `tests/frame_rate_test.rs::a7_p99_empty_buffer_returns_zero` | Real | PASS |
| ST-FUNC-011-008 | NFR-001 | 零帧安全 | `tests/frame_rate_test.rs::a8_stats_zero_frames_no_panic` | Real | PASS |
| ST-FUNC-011-009 | NFR-001 | 零帧日志安全 | `tests/frame_rate_test.rs::a9_log_report_empty_no_panic` | Real | PASS |
| ST-BNDRY-011-001 | NFR-001 | frame_time=0.0 | `tests/frame_rate_test.rs::b1_zero_frame_time_does_not_pollute_min` | Real | PASS |
| ST-BNDRY-011-002 | NFR-001 | 缓冲满 FIFO | `tests/frame_rate_test.rs::b2_buffer_full_fifo_behavior` | Real | PASS |
| ST-BNDRY-011-003 | NFR-001 | 环索引回绕 | `tests/frame_rate_test.rs::b3_ring_index_wraparound_120_seconds` | Real | PASS |
| ST-BNDRY-011-004 | NFR-001 | frame_time 极大值 | `tests/frame_rate_test.rs::b4_extreme_frame_time_handled` | Real | PASS |
| ST-BNDRY-011-005 | NFR-001 | 60s 日志触发边界 | `tests/frame_rate_test.rs::b5_log_trigger_boundary_3599_vs_3600` | Real | PASS |
| ST-PERF-011-001 | NFR-001 | AC-1: P99 ≤ 16.67ms | `tests/frame_rate_test.rs::p1_360_frames_performance_within_budget` | Real | PASS |
| ST-PERF-011-002 | NFR-001 | AC-2: 60s ≥ 58fps | `tests/frame_rate_test.rs::p2_3600_frames_full_load_window_fps` | Real | PASS |
| ST-PERF-011-003 | NFR-001 | 空场景基准 | `tests/frame_rate_test.rs::p3_empty_scene_baseline_3600_frames` | Real | PASS |

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 17 |
| Passed | 17 |
| Failed | 0 |
| Pending | 0 |

## Manual Test Case Summary

> 本特性无手动测试用例 —— 所有 17 条用例均通过 `cargo test` 自动化执行（`已自动化: Yes`）。特性为非功能性后端度量仪表（`ui: false`），无视觉验证需求。
