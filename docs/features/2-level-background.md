# Feature Detailed Design: Level & Background (Feature #2)

**Date**: 2026-05-31
**Feature**: #2 — Level & Background
**Priority**: high
**Dependencies**: none
**Design Reference**: docs/plans/2026-05-31-mario-platformer-design.md §2.2
**SRS Reference**: FR-006

## Context

本特性定义单关卡的静态平台几何、关卡边界与三层视差背景。作为所有依赖关卡几何的功能（物理碰撞、摄像机钳制、敌人路点、危险物摆放）的数据源，提供 `query_terrain` (IAPI-005) 地形查询和 `bounds` (IAPI-008) 边界查询两个公开接口。关卡数据以 Rust 编译期常量硬编码，无需运行时加载。

## Design Alignment

### 概览（Overview）

以硬编码常量定义单个关卡的平台几何、关卡边界 (min_x, max_x, kill_y)、视差背景纹理。提供 `query_terrain(AABB) → Vec<Tile>` 地形查询接口供物理系统使用，提供 `bounds()` 供摄像机钳制。

### 关键类型（Key Types）

- `Level` — 持有 `Vec<Platform>`、`LevelBounds`、`KillPlane` 阈值
- `Platform` — 单个平台：`AABB` + 精灵类型
- `ParallaxLayer` — 单层视差：纹理句柄 + 滚动速度倍率 (`scroll: 0.1 | 0.3 | 0.6`)
- `Tile` — 枚举：`Empty` / `Platform(AABB)` / `Spike(AABB)`

### 集成面（Integration Surface）

**Provides**:

| Consumer Feature(s) | Contract ID | Endpoint / Method | Response |
|---------------------|-------------|-------------------|----------|
| F05 Hazards, F04 Camera | IAPI-005 | `Level::query_terrain(aabb) → Vec<Tile>` | 与查询 AABB 相交的地形 tile 列表 |
| F04 Camera | IAPI-008 | `Level::bounds() → LevelBounds` | `{ min_x, max_x, min_y, kill_y }` |

**Requires**: Self-contained — no external integration surface.

**Deviations**: 无。

**UML 嵌入**: 不触发 — 本特性为数据模型 + 纯查询方法，无 ≥2 类/模块协作序列、无状态机、无 ≥3 决策分支方法。

## SRS Requirement

### FR-006: Fixed Platforms
**优先级（Priority）**: Must
**EARS**: The system shall provide static platform geometry at various (x, y) positions within the level bounds; while the player's collision body overlaps a platform's top surface from above with downward velocity ≤ 0, the system shall resolve the player to stand on the platform surface and reset airborne state.
**可视化输出（Visual output）**: 固定平台以像素艺术精灵渲染在场景中；玩家可见站立、行走、跳跃于不同高度平台之间。
**验收准则（Acceptance Criteria）**:
- Given 玩家从上方向下落接触平台顶部表面, When 玩家碰撞体底部与平台顶部碰撞体重叠, Then 玩家停在平台表面且垂直速度归零，可水平移动和跳跃。
- Given 玩家站在平台上, When 玩家水平移动至平台边缘之外, Then 玩家失去平台支撑并开始下落。
- Given 玩家在平台下方, When 玩家向上跳跃头顶接触到平台底部, Then 玩家垂直速度归零并开始下落（不可穿越平台）。
- Given 玩家水平移动, When 玩家侧面碰撞平台侧边碰撞体, Then 玩家水平移动被阻挡（不可穿墙）。
**来源（Source）**: Raw requirement #6 — 场景元素: 固定平台

## Interface Contract

| Method | Signature | Preconditions | Postconditions | Raises |
|--------|-----------|---------------|----------------|--------|
| `Level::new` | `Level::new() -> Self` | 无 | 返回包含硬编码平台列表、关卡边界、视差层的完整 `Level` 实例。`bounds()` 返回 `LevelBounds { min_x: 0.0, max_x: <关卡总宽>, min_y: 0.0, kill_y: <关卡总高 + 200.0> }`。`platforms.len() ≥ 1`。`parallax_layers.len() == 3`，速度倍率分别为 0.1, 0.3, 0.6。 | — |
| `Level::query_terrain` | `Level::query_terrain(&self, aabb: &AABB) -> Vec<Tile>` | `aabb` 为有效的 AABB（w ≥ 0.0, h ≥ 0.0） | 返回与 `aabb` 相交的所有地形 tile 列表。对每个 `Platform` 若其 `AABB` 与查询 `aabb` 在 x 和 y 轴均有重叠（含边界），则返回 `Tile::Platform(platform.aabb)`。若无平台相交，返回空 `Vec`。结果顺序与 `platforms` 存储顺序一致。 | — |
| `Level::bounds` | `Level::bounds(&self) -> LevelBounds` | 无 | 返回关卡边界 `LevelBounds { min_x, max_x, min_y, kill_y }`，为 `Level::new()` 中定义的硬编码值。每次调用返回相同值（纯函数）。 | — |
| `AABB::intersects` | `AABB::intersects(&self, other: &AABB) -> bool` | 两个 AABB 均为有效（w ≥ 0.0, h ≥ 0.0） | 当且仅当 `self` 与 `other` 在 x 轴和 y 轴均有重叠时返回 `true`。边界接触（如 self.max_x == other.min_x）视为相交。 | — |
| `ParallaxLayer::new` | `ParallaxLayer::new(speed: f32) -> Self` | `speed` 为有限正浮点数 | 创建 `ParallaxLayer`，`speed` 字段为传入值，`scroll_offset` 初始为 (0.0, 0.0)。 | — |
| `ParallaxLayer::update_scroll` | `ParallaxLayer::update_scroll(&mut self, camera_offset: Vec2)` | `camera_offset` 为摄像机当前世界偏移量 | `scroll_offset.x = camera_offset.x * self.speed`，垂直方向 `scroll_offset.y` 保持为 0.0（背景仅水平滚动）。 | — |

**方法状态依赖**: 无状态依赖 — 所有方法均为无状态查询或构造器。`Level` 创建后不可变。

**Design rationale**:
- `query_terrain` 返回 `Vec<Tile>` 而非 `&[Tile]`：允许调用方拥有结果，避免借用 `Level` 的同时进行后续操作。对于本游戏规模的平台数量（预计 < 100），分配开销可忽略。
- AABB 相交采用含边界语义（inclusive edges）：避免相邻平台间出现单像素间隙导致的碰撞漏检，符合 2D 平台游戏的惯例。
- 关卡数据硬编码于 `Level::new()`：符合 ASM-003（单关卡）、ASM-004（无运行时编辑器），且编译期即完成初始化，零运行时加载开销。
- 三层视差速度倍率 0.1/0.3/0.6：从远到近递增，符合透视投影直觉；最远层几乎不动（10% 摄像机速度），最近层有明显位移感（60%）。
- **跨特性契约对齐**：
  - IAPI-005 Provider：`Level::query_terrain(aabb: &AABB) -> Vec<Tile>` 签名与 §4 定义的 `AABB → Vec<Tile>` 完全一致。
  - IAPI-008 Provider：`Level::bounds() -> LevelBounds` 返回 `LevelBounds { min_x, max_x, min_y, kill_y }`，与 §4 schema 完全一致。
  - IAPI-010 Consumer：`ParallaxLayer::update_scroll` 接收 `camera_offset: Vec2`，对应 Camera 的 `Camera::offset() → Vec2`。本特性仅消费该值用于视差计算，不依赖 Camera 模块编译（参数传递式松耦合）。

## Visual Rendering Contract

N/A — `"ui": false`。本特性为纯数据/后端模块，提供地形查询和边界查询。平台精灵的实际渲染由渲染管线（Playing 状态）在 physics→render 阶段完成，不属于本特性范围。视差背景的视觉验收在 Feature ST 阶段通过集成测试（INT-004）覆盖。

## Implementation Summary

### 1. 主要类/函数与文件布局

本特性新增两个源文件：
- `src/level.rs` — 定义 `AABB`、`Vec2`、`Tile` 枚举、`Platform` 结构体、`LevelBounds` 结构体、`Level` 结构体及其所有方法。
- `src/parallax.rs` — 定义 `ParallaxLayer` 结构体及其构造和方法。

`lib.rs` 新增 `pub mod level;` 和 `pub mod parallax;` 声明。`AABB`、`Vec2`、`Tile`、`LevelBounds` 从 `level` 模块公开导出，供其他 feature 使用（物理碰撞、摄像机、实体等均依赖这些基础类型）。

### 2. 调用链

运行时调用链如下：
```
Playing::update(dt)
  → Physics::resolve(player, entities, &level)
    → level.query_terrain(player.aabb())   // IAPI-005 — 获取玩家周围地形
    → [对每个 tile 执行碰撞检测与响应]
  → Camera::update(player.pos(), level.bounds())  // IAPI-008 — 钳制视口

Playing::render(alpha)
  → [渲染视差背景]:
    for layer in level.parallax_layers() {
        layer.update_scroll(camera.offset());   // IAPI-010 Consumer
        draw_texture_tiled(layer, camera.offset());
    }
  → [渲染平台精灵]:
    for platform in level.platforms() { draw_sprite(platform); }
  → [渲染实体] → [渲染 HUD]
```

`Level` 在游戏启动时一次性构造（`Level::new()`），随后以不可变引用传递至所有消费方。

### 3. 关键设计决策与非显见约束

- **AABB 与 Vec2 定义位置**：置于 `src/level.rs`（本特性）而非独立 `src/geom.rs`，因为 Feature #2 是第一个需要这些几何原语的模块，且为所有下游 feature 的数据依赖源。若后续重构需要独立几何模块，移动成本低（类型为纯数据 struct，无复杂依赖）。
- **`Tile::Spike(AABB)` 提前定义**：虽然尖刺碰撞逻辑属于 Feature #5 (Hazards)，但其关卡位置数据在本特性中定义。`query_terrain` 返回 `Tile::Spike(AABB)` 告知调用方该位置存在危险物，碰撞判定由 Feature #5 的 Physics/Hazard 系统处理。此分离遵循数据/行为解耦原则。
- **视差纹理与渲染解耦**：`ParallaxLayer` 仅存储速度倍率与滚动偏移量，不持有纹理句柄。实际纹理绑定和绘制由 Playing 状态的渲染函数负责。这样设计使得 `ParallaxLayer` 可在无 Macroquad 上下文的单元测试中直接测试滚动计算逻辑。

### 4. 遗留/存量代码交互点

本特性为 greenfield 模块，不与既有代码发生修改性交互。仅需在 `lib.rs` 中添加两个模块声明。已完成的 Feature #1 (Engine Core) 中的 `GameLoop`、`StateMachine` trait 与本特性无直接耦合 — `Level` 实例由游戏状态创建并持有，通过 `StateMachine::update(dt)` 间接传递给各子系统。

env-guide.md §4 为 greenfield 状态，无强制内部库、禁用 API 或命名约定约束。代码风格跟随 Rust 社区惯例（`snake_case` 函数/方法、`CamelCase` 类型、`UpperCamelCase` 枚举变体）。

### 5. §4 Internal API Contract 集成

本特性是 IAPI-005 和 IAPI-008 的 **Provider**，同时是 IAPI-010 的 **Consumer**：

- **IAPI-005 Provider**：`Level::query_terrain(&self, aabb: &AABB) -> Vec<Tile>` 完全匹配 §4 定义的签名。Consumer 特性（F04 Camera 进行视口钳制时的地形判断、F05 Hazards 的尖刺碰撞检测）通过此接口获取地形数据。
- **IAPI-008 Provider**：`Level::bounds(&self) -> LevelBounds` 返回 `{ min_x, max_x, min_y, kill_y }`，Consumer 特性 F04 Camera 用于视口钳制，防止摄像机显示关卡外区域。
- **IAPI-010 Consumer**：`ParallaxLayer::update_scroll(&mut self, camera_offset: Vec2)` 接收来自 Camera 的世界偏移量。这是松耦合消费 — 调用方（Playing 状态渲染函数）获取 camera offset 后传入，`ParallaxLayer` 不 import Camera 模块。

### Boundary Conditions

| Parameter | Min | Max | Empty/Null | At boundary |
|-----------|-----|-----|------------|-------------|
| `query_terrain(aabb.w)` | 0.0（零宽查询） | 无上限（可查询整关） | w=0 时 AABB 退化为竖线；与平台边界恰好重合时返回该平台 tile | w=0 且 x 恰在平台左边界 → 返回该平台（含边界语义） |
| `query_terrain(aabb.h)` | 0.0（零高查询） | 无上限 | h=0 时 AABB 退化为横线；行为同上 | h=0 且 y 恰在平台顶部 → 返回该平台 |
| `query_terrain(aabb.x, aabb.y)` | 无下限（可为负） | 无上限（可超出关卡） | AABB 完全在关卡外 → 返回空 Vec | AABB 部分在关卡内、部分在外 → 仅返回与关卡内平台相交的 tile |
| `AABB::intersects` 边界接触 | — | — | — | `self.max_x == other.min_x` → `true`（含边界）；两个零尺寸 AABB 在相同坐标 → `true` |
| `ParallaxLayer::new(speed)` | `> 0.0` | 无严格上限 | — | speed=0.0 → 静态背景层；speed 为负 → 反向滚动（本设计不使用，但方法不拒绝） |
| `Level::bounds()` | 无参数 | — | — | 每次调用返回相同硬编码值；`kill_y` 大于 `max_y`（在可见区域下方） |

### Existing Code Reuse

N/A — searched keywords: [`Level`, `Platform`, `ParallaxLayer`, `Parallax`, `AABB`, `Vec2`, `Tile`, `LevelBounds`, `query_terrain`, `bounds`, `terrain`, `intersects`, `scroll`], no reusable match. 本特性为 greenfield 模块，代码库中尚无几何原语或关卡数据结构的既有实现。

## Test Inventory

| ID | Category | Traces To | Input / Setup | Expected | Kills Which Bug? |
|----|----------|-----------|---------------|----------|-----------------|
| T01 | FUNC/happy | FR-006 AC-1, §Interface Contract `query_terrain` | `level = Level::new()`; `aabb` = 玩家站在平台上的碰撞体位置 | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec，且至少有一项 | 地形查询未检测到重叠平台 → 玩家穿过平台坠落 |
| T02 | FUNC/happy | FR-006 AC-2, §Interface Contract `query_terrain` | `level = Level::new()`; `aabb` = 完全在平台区域外的空中位置 | `query_terrain(aabb)` 返回空 Vec | 地形查询在空白区域误报平台 → 玩家悬浮在空中 |
| T03 | FUNC/happy | FR-006 AC-3, §Interface Contract `query_terrain` | `level = Level::new()`; `aabb` = 玩家在平台正下方的碰撞体（头顶接触平台底部） | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec | 从下方查询时漏检平台 → 玩家可穿越平台底部 |
| T04 | FUNC/happy | FR-006 AC-4, §Interface Contract `query_terrain` | `level = Level::new()`; `aabb` = 玩家在平台侧面的碰撞体（水平相邻） | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec | 从侧面查询时漏检平台 → 玩家可穿墙 |
| T05 | FUNC/happy | §Interface Contract `bounds`, IAPI-008 | `level = Level::new()`; `bounds = level.bounds()` | `bounds.min_x == 0.0`; `bounds.max_x > 0.0`; `bounds.min_y == 0.0`; `bounds.kill_y >= bounds.max_y` | 边界值错误 → 摄像机钳制位置错误或 kill_y 在可见区域内 |
| T06 | FUNC/happy | §Interface Contract `ParallaxLayer` | `layer = ParallaxLayer::new(0.3)`; `layer.update_scroll(Vec2{x:100.0, y:50.0})` | `layer.scroll_offset.x == 30.0`; `layer.scroll_offset.y == 0.0` | 视差滚动倍率计算错误 → 背景与摄像机移动不同步 |
| T07 | FUNC/happy | §Interface Contract `Level::new` | `level = Level::new()` | `level.parallax_layers().len() == 3`；三层速度分别为 0.1, 0.3, 0.6 | 视差层数量或速度值错误 → 背景视觉效果异常 |
| T08 | FUNC/happy | §Interface Contract `AABB::intersects` | `a = AABB{x:0.0, y:0.0, w:10.0, h:10.0}`; `b = AABB{x:5.0, y:5.0, w:10.0, h:10.0}` | `a.intersects(&b) == true` | AABB 相交判定错误 → 碰撞检测全盘失效 |
| T09 | FUNC/error | §Interface Contract `query_terrain` postcondition | `level = Level::new()`; `aabb` = 完全在关卡边界外的位置 (如 x = -1000.0) | `query_terrain(aabb)` 返回空 Vec（不 panic、不返回错误结果） | 越界查询未正确处理 → panic 或返回错误 tile |
| T10 | BNDRY/edge | §Boundary Conditions `query_terrain` w/h | `aabb` 尺寸为 0×0，位于某平台 AABB 内部坐标 | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec | 零尺寸 AABB 被错误排除 → 点查询不适用于精确碰撞检测 |
| T11 | BNDRY/edge | §Boundary Conditions `query_terrain` 边界接触 | `aabb` 的右边界恰好等于某平台左边界 (max_x == platform.min_x) | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec（含边界相交） | 边界接触被错误排除 → 相邻平台间出现 1 像素穿模间隙 |
| T12 | BNDRY/edge | §Boundary Conditions `query_terrain` 边界接触(上) | `aabb` 的底边界恰好等于某平台顶边界 (max_y == platform.min_y) | `query_terrain(aabb)` 返回包含 `Tile::Platform(..)` 的 Vec | 玩家恰好踩在平台边缘时判定为未接触 → 玩家穿过平台 |
| T13 | BNDRY/edge | §Boundary Conditions `query_terrain` 部分越界 | `aabb` 一半在关卡内、一半在左边界外 | `query_terrain(aabb)` 仅返回关卡内与 aabb 相交的平台（无越界 panic） | 越界部分未处理 → 坐标溢出或 panic |
| T14 | BNDRY/edge | §Boundary Conditions `AABB::intersects` 零尺寸 | 两个宽高均为 0 的 AABB 在同一坐标 | `a.intersects(&b) == true` | 零尺寸 AABB 相交返回 false → 精确点碰撞检测失败 |
| T15 | BNDRY/edge | §Boundary Conditions `AABB::intersects` 不相交 | `a = AABB{x:0.0,y:0.0,w:10.0,h:10.0}`; `b = AABB{x:20.0,y:20.0,w:10.0,h:10.0}` (相距 10px) | `a.intersects(&b) == false` | 不相交的 AABB 误判为相交通 → 远距离误触发碰撞 |
| T16 | FUNC/happy | §Interface Contract `Level::new` 平台数量 | `level = Level::new()` | `level.platforms()` 返回非空切片，每个平台的 AABB 满足 `w > 0.0 && h > 0.0` | 关卡无平台或平台尺寸无效 → 玩家直接坠落 kill_y |
| T17 | BNDRY/edge | §Boundary Conditions `query_terrain` 全覆盖 | AABB 覆盖整个关卡 (`x=min_x, y=min_y, w=max_x-min_x, h=max_y-min_y`) | `query_terrain(aabb)` 返回所有平台的 tile（数量 = `platforms.len()`） | 大范围查询遗漏部分平台 → 碰撞检测不完整 |

**负向测试占比**：FUNC/error (T09) + BNDRY/* (T10-T15, T17) = 8 行。总计 17 行。8/17 = 47.1% ≥ 40%。

**ATS 类别对齐**：ATS 对 FR-006 要求的必须类别为 FUNC, BNDRY。Test Inventory 包含 FUNC/happy (8 行)、FUNC/error (1 行)、BNDRY/edge (8 行)，全部覆盖。

**INTG**: N/A — pure function, no external I/O。本特性为内存数据结构 + 纯查询方法。视差背景的实际纹理渲染在 Playing 状态的集成测试 (INT-004) 中验证，不属于本特性的单元测试范围。

## Verification Checklist
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Interface Contract 的 postconditions
  - AC-1 (着陆) → `query_terrain` postcondition (T01)
  - AC-2 (走出边缘) → `query_terrain` postcondition (T02)
  - AC-3 (底部碰撞) → `query_terrain` postcondition (T03)
  - AC-4 (侧面阻挡) → `query_terrain` postcondition (T04)
- [x] 所有 SRS 验收准则（来自 srs_trace）已追溯到 Test Inventory 行
  - AC-1 → T01; AC-2 → T02; AC-3 → T03; AC-4 → T04
- [x] Boundary Conditions 表覆盖所有非平凡参数
  - `query_terrain` 的 aabb 参数 (w/h/x/y) 均已覆盖
  - `AABB::intersects` 的边界接触行为已覆盖
  - `ParallaxLayer::new` 的 speed 参数已覆盖
- [x] Interface Contract Raises 列覆盖所有预期错误条件
  - 本特性所有方法均不抛出错误（纯查询/构造，返回空 Vec 处理无匹配情况）
- [x] Test Inventory 负向占比 >= 40% → 47.1%
- [x] ui:true 特性的 Visual Rendering Contract 完整 → N/A (ui: false)
- [x] 每个 Visual Rendering Contract 元素至少对应 1 行 UI/render Test Inventory → N/A
- [x] Existing Code Reuse 章节已填充 → "N/A — searched keywords, no reusable match"
- [x] Implementation Summary 为 3-5 段具体散文（含文件路径 + 类名），非 pseudocode → 5 段
- [x] UML 图触发判据检查完成：
  - `classDiagram`: 不触发 — 无跨模块 ≥2 类协作（本特性为内部组合）
  - `sequenceDiagram`: 不触发 — 调用链见 §Implementation Summary 散文
  - `stateDiagram-v2`: 不触发 — 所有方法无状态依赖
  - `flowchart TD`: 不触发 — `query_terrain` 为单循环单条件，无 ≥3 分支
- [x] 非类图装饰检查 → N/A (无 UML 图)
- [x] 每个图元素的 Test Inventory 追溯 → N/A (无 UML 图)
- [x] 每个被跳过的章节都写明 "N/A — [reason]"
  - Visual Rendering Contract: N/A — ui: false
  - INTG: N/A — pure function, no external I/O
  - UML 图: N/A — 不满足触发判据
- [x] §2.N 中所有函数/方法都至少有一行 Test Inventory：
  - `Level::query_terrain` → T01-T04, T09-T13, T17
  - `Level::bounds` → T05
  - `Level::new` → T07, T16
  - `AABB::intersects` → T08, T14, T15
  - `ParallaxLayer::new` / `update_scroll` → T06
  - 覆盖率 5/5 (100%)

## Clarification Addendum

> 无需澄清 — 全部规格明确。

| # | Category | Original Ambiguity | Resolution | Authority |
|---|----------|--------------------|------------|-----------|
| 1 | NFR-GAP | 关卡硬编码平台的具体位置、数量、尺寸未在 SRS 或 Design 中指定 | 定义一份合理的示范关卡布局：包含地面层平台、2-3 个不同高度的浮动平台、左右边界内足够空间供玩家移动。具体坐标值在实现时确定，以可玩性为指导原则。 | assumed |
| 2 | ASM-003 | 视差背景纹理资源未在 assets 模块中定义 | 视差层暂时使用纯色矩形替代纹理，直到 Feature ST 阶段集成实际精灵资源。`ParallaxLayer` 结构体预留纹理标识字段供后续扩展。 | assumed |
| 3 | NFR-GAP | `kill_y` 的具体数值未指定 | 设为关卡可见区域底部下方 200 像素处，即 `max_y + 200.0`。此值确保玩家在可见区域内正常游戏，仅在坠落出视口后触发死亡。 | assumed |
