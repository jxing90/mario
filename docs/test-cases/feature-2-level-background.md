# 测试用例集: Level & Background

**Feature ID**: 2
**关联需求**: FR-006
**日期**: 2026-06-01
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 10 |
| boundary | 7 |
| ui | 0 |
| security | 0 |
| performance | 0 |
| **合计** | **17** |

---

### 用例编号

ST-FUNC-002-001

### 关联需求

FR-006（Fixed Platforms）— AC-1: 玩家从上方落在平台顶部表面

### 测试目标

验证 `query_terrain` 在玩家碰撞体底部与平台顶部重叠时正确返回该平台的 `Tile::Platform`。

### 前置条件

- Rust 工具链已安装，`cargo test` 可用
- `Level::new()` 构造成功，包含至少 1 个平台

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level`，含 3 个平台、3 层视差背景 |
| 2 | 取第一个平台的 AABB（地面平台：x=0.0, y=600.0, w=2000.0, h=40.0） | 平台 AABB 有效（w>0, h>0） |
| 3 | 构造玩家 AABB（16x16），位于平台上方，底部（max_y）恰好接触平台顶部（platform.y） | 玩家 AABB.y + h == platform.y |
| 4 | `level.query_terrain(&player_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- `query_terrain` 对玩家站在平台上的位置返回至少 1 个 `Tile::Platform`
- AABB 相交判定使用含边界语义（边界接触算作重叠）
- 结果不包含 `Tile::Spike`（除非玩家 AABB 与尖刺重叠）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t01_query_terrain_player_standing_on_platform`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-002

### 关联需求

FR-006（Fixed Platforms）— AC-2: 玩家走出平台边缘坠落

### 测试目标

验证 `query_terrain` 在玩家 AABB 完全不在任何平台区域时返回空 `Vec`。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 构造玩家 AABB（16x16），位置在高空中（x=100.0, y=-500.0），远在任何平台之上 | 玩家 AABB 与所有平台无重叠 |
| 3 | `level.query_terrain(&air_aabb)` | 返回空 `Vec`（`result.is_empty() == true`） |

### 验证点

- 空中查询不返回任何平台 tile
- 不会误报平台存在（否则玩家悬浮在空中）
- 空结果不触发 panic

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t02_query_terrain_player_in_air_returns_empty`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-003

### 关联需求

FR-006（Fixed Platforms）— AC-3: 玩家从下方头顶碰撞平台底部

### 测试目标

验证 `query_terrain` 在玩家头部（碰撞体顶部）接触平台底部时正确检测到平台。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台的 AABB | 平台 AABB 有效 |
| 3 | 构造玩家 AABB（16x16），位于平台正下方，玩家顶部（min_y）恰好接触平台底部（platform.y + platform.h） | 玩家 y == platform.y + platform.h |
| 4 | `level.query_terrain(&player_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- 碰撞方向中立：`query_terrain` 不区分方向，上/下/侧面接触均返回平台
- 从下方查询同样检测到平台（用于阻止玩家穿透天花板）
- 碰撞方向判断（上/下/侧）属于 Physics 模块职责，不属于 `query_terrain`

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t03_query_terrain_player_head_hits_platform_bottom`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-004

### 关联需求

FR-006（Fixed Platforms）— AC-4: 玩家侧面碰撞平台

### 测试目标

验证 `query_terrain` 在玩家水平方向紧贴平台侧边时正确检测到平台。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台的 AABB | 平台 AABB 有效 |
| 3 | 构造玩家 AABB（16x16），位于平台左侧，玩家右边界（max_x）恰好接触平台左边界（platform.x） | 玩家 x+w == platform.x |
| 4 | `level.query_terrain(&player_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- 侧面接触同样检测到平台（用于阻止玩家穿墙）
- 含边界语义确保紧贴时判定为相交（`max_x == min_x` 算作重叠）
- 调用方可依据碰撞方向决定水平阻挡 or 垂直支撑

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t04_query_terrain_player_side_collides_with_wall`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-005

### 关联需求

FR-006（Fixed Platforms）— IAPI-008: LevelBounds 查询

### 测试目标

验证 `Level::bounds()` 返回的关卡边界满足基本几何约束，且为纯函数（每次调用返回相同值）。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | `bounds = level.bounds()` | 返回 `LevelBounds` |
| 3 | 断言 `bounds.min_x <= bounds.max_x` | 左边界 ≤ 右边界 |
| 4 | 断言 `bounds.max_x - bounds.min_x > 0.0` | 关卡宽度 > 0 |
| 5 | 断言 `bounds.kill_y >= max_y`（可见区域底部 ≤ 死亡平面） | kill_y 在可见区域之下 |
| 6 | 再次调用 `bounds2 = level.bounds()` | `bounds2` 的 `min_x`, `max_x` 与首次调用一致（浮点容差内） |

### 验证点

- 边界值符合几何约束：min_x ≤ max_x，width > 0
- kill_y 位于可见区域下方（玩家在屏幕上不会误触发死亡）
- `bounds()` 是纯函数，多次调用返回一致值

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t05_level_bounds_returns_valid_level_bounds`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-006

### 关联需求

FR-006（Fixed Platforms）— IAPI-010 Consumer: ParallaxLayer 滚动偏移计算

### 测试目标

验证 `ParallaxLayer::update_scroll(camera_offset)` 正确计算水平滚动偏移（offset.x = camera.x * speed），垂直偏移恒为 0。

### 前置条件

- Rust 工具链已安装
- `ParallaxLayer::new(0.3)` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `let mut layer = ParallaxLayer::new(0.3)` | 创建 layer，speed=0.3，scroll_offset=(0,0) |
| 2 | `layer.update_scroll(Vec2 { x: 100.0, y: 50.0 })` | 更新滚动偏移 |
| 3 | 断言 `layer.scroll_offset.x ≈ 30.0`（100.0 × 0.3） | 水平偏移 = camera.x × speed |
| 4 | 断言 `layer.scroll_offset.y ≈ 0.0` | 垂直偏移恒为 0（水平视差滚动） |

### 验证点

- 滚动倍率乘法正确：`camera_offset.x * speed`
- 垂直偏移不受 camera Y 影响（始终为 0）
- 浮点计算精度在容差内（1e-5）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t06_parallax_layer_scroll_offset_calculation`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-007

### 关联需求

FR-006（Fixed Platforms）— Level::new 后置条件: 3 层视差背景

### 测试目标

验证 `Level::new()` 创建恰好 3 层视差背景，速度倍率分别为 0.1（远）、0.3（中）、0.6（近）。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | `layers = level.parallax_layers()` | 返回 `&[ParallaxLayer]` |
| 3 | 断言 `layers.len() == 3` | 恰好 3 层 |
| 4 | 断言 `layers[0].speed ≈ 0.1`（远层） | 速度倍率 0.1 |
| 5 | 断言 `layers[1].speed ≈ 0.3`（中层） | 速度倍率 0.3 |
| 6 | 断言 `layers[2].speed ≈ 0.6`（近层） | 速度倍率 0.6 |

### 验证点

- 视差层数量 = 3
- 从远到近速度递增（0.1 → 0.3 → 0.6），符合透视投影直觉
- 速度值精确（浮点容差内）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t07_level_creates_three_parallax_layers_with_correct_speeds`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-008

### 关联需求

FR-006（Fixed Platforms）— AABB::intersects 重叠检测

### 测试目标

验证 `AABB::intersects` 正确检测两个有明确重叠区域的 AABB，且满足交换律。

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `a = AABB { x:0, y:0, w:10, h:10 }` | AABB 占据区域 [0,10] × [0,10] |
| 2 | 构造 `b = AABB { x:5, y:5, w:10, h:10 }` | AABB 占据区域 [5,15] × [5,15] |
| 3 | 断言 `a.intersects(&b) == true` | 重叠区域 [5,10] × [5,10] |
| 4 | 断言 `b.intersects(&a) == true` | 验证交换律：b.intersects(a) == a.intersects(b) |

### 验证点

- x 轴与 y 轴均重叠 → intersects 返回 true
- 交换律成立
- 基础碰撞检测原语正确性（所有下游功能依赖此方法）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t08_aabb_intersects_detects_overlapping_boxes`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-009

### 关联需求

FR-006（Fixed Platforms）— 越界查询: 不 panic，返回空

### 测试目标

验证 `query_terrain` 对远在关卡外的 AABB 坐标不 panic，正确返回空 `Vec`。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 构造 AABB 在关卡左上方极远处（x=-10000, y=-10000） | 完全在关卡外 |
| 3 | `level.query_terrain(&far_out_aabb)` | 返回空 `Vec`，不 panic |
| 4 | 构造 AABB 在关卡右下方极远处（x=50000, y=50000） | 完全在关卡外 |
| 5 | `level.query_terrain(&far_right_aabb)` | 返回空 `Vec`，不 panic |

### 验证点

- 负坐标查询不 panic（无索引溢出）
- 远超关卡范围的坐标查询不 panic
- 始终返回空 `Vec` 而非错误结果
- 符合接口契约 Raises 列："所有方法均不抛出错误"

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t09_query_terrain_far_outside_level_returns_empty`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-002-010

### 关联需求

FR-006（Fixed Platforms）— Level::new 后置条件: 平台尺寸有效性

### 测试目标

验证 `Level::new()` 创建的所有平台均具有正的有效尺寸（w > 0, h > 0）且坐标值有限。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | `platforms = level.platforms()` | 返回非空切片 |
| 3 | 断言 `platforms.len() >= 1` | 至少 1 个平台 |
| 4 | 对每个平台 i：断言 `platform.aabb.w > 0.0` | 平台宽度 > 0 |
| 5 | 对每个平台 i：断言 `platform.aabb.h > 0.0` | 平台高度 > 0 |
| 6 | 对每个平台 i：断言所有 `x, y, w, h` 均为有限浮点数 | 无 NaN 或 Inf |
| 7 | 断言 `platforms.len() == 3` | 硬编码 3 个平台 |

### 验证点

- 所有平台尺寸为正且非零（否则碰撞检测无意义）
- 坐标和尺寸值均为有限浮点数（无 NaN/Inf）
- 平台数量 ≥ 1（否则玩家无处站立，直接坠落）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t16_level_platforms_have_valid_dimensions`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-001

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 零尺寸 AABB 查询

### 测试目标

验证 `query_terrain` 对 w=0, h=0 的点查询 AABB（位于平台内部）正确返回该平台。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台的 AABB，计算其中心点坐标 | (cx, cy) 在平台内部 |
| 3 | 构造点查询 AABB（w=0, h=0）位于平台中心 | AABB 退化为点 |
| 4 | `level.query_terrain(&point_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- 零尺寸 AABB 不被排除（点查询支持）
- 含边界语义使点恰好落在平台边缘时也判定为内部
- 用于精确碰撞检测（点与平台 AABB 的位置关系）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t10_zero_size_aabb_inside_platform_returns_platform`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-002

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 右边界恰好接触平台左边界

### 测试目标

验证含边界语义：查询 AABB 的右边界（max_x）恰等于平台左边界（min_x）时判定为相交。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台 AABB | 获取 platform.min_x |
| 3 | 构造 AABB（16x16），其 max_x == platform.min_x（恰好接触但无正面积重叠） | 边界紧贴 |
| 4 | `level.query_terrain(&player_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- 含边界判定（`>=` 而非 `>`）：边界接触算作相交
- 避免相邻平台间因排他语义产生 1 像素穿模间隙
- 使用 `<=` 和 `>=` 运算符（非严格不等式）

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t11_boundary_touch_right_edge_equals_platform_left_edge`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-003

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 底部边界恰好接触平台顶部边界

### 测试目标

验证含边界语义：玩家 AABB 的底边（max_y）恰等于平台顶边（min_y）时判定为相交。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台 AABB | 获取 platform.min_y（平台顶部） |
| 3 | 构造玩家 AABB（16x16），玩家 bottom（max_y）== platform.min_y | 恰好站在平台表面 |
| 4 | `level.query_terrain(&player_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec |

### 验证点

- 玩家恰好踩在平台表面时判定为接触
- 避免排他语义导致玩家穿过平台 1 像素
- 与 T11 类似，验证垂直方向含边界语义

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t12_boundary_touch_bottom_edge_equals_platform_top_edge`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-004

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 部分越界 AABB

### 测试目标

验证 `query_terrain` 对部分在关卡内、部分在关卡外的 AABB 正确返回关卡内相交的平台（不 panic、不遗漏）。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功，至少 1 个平台的 x >= 0

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 取第一个平台的 AABB（x=0.0） | 平台在关卡最左边界 |
| 3 | 构造 AABB 起点 x=-500（关卡外），但宽度足够延伸至与平台重叠 | 部分在关卡外、部分与平台重叠 |
| 4 | `level.query_terrain(&half_out_aabb)` | 返回包含 `Tile::Platform(_)` 的 Vec，不 panic |

### 验证点

- 越界坐标不导致 panic（无索引溢出、无负数索引）
- 仅在关卡外部分被忽略，关卡内重叠正确检测
- 不因为部分越界而返回空或跳过有效重叠

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t13_partial_out_of_bounds_aabb_returns_only_intersecting_platforms`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-005

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 两个零尺寸 AABB 在同一点

### 测试目标

验证两个 w=0, h=0 的 AABB 在同一坐标时 `intersects` 返回 `true`。

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `a = AABB { x:5, y:5, w:0, h:0 }` | 点 AABB at (5, 5) |
| 2 | 构造 `b = AABB { x:5, y:5, w:0, h:0 }` | 同一点 |
| 3 | 断言 `a.intersects(&b) == true` | 点重合 → 相交 |
| 4 | 断言 `b.intersects(&a) == true` | 验证交换律 |

### 验证点

- 零尺寸 AABB 不受"空 AABB" 早期返回（early return）影响
- 点-点碰撞检测正确
- 含边界语义统一应用（包括零尺寸极限情况）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t14_two_zero_size_aabbs_at_same_coordinate_intersect`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-006

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 不相交 AABB

### 测试目标

验证完全分离的 AABB 返回 `false`（无相交），包括水平分离、垂直分离、对角分离。

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 `a = AABB { x:0, y:0, w:10, h:10 }` | 区域 [0,10] × [0,10] |
| 2 | 构造对角分离 `b = AABB { x:20, y:20, w:10, h:10 }` | 区域 [20,30] × [20,30]，各轴间隔 10 |
| 3 | 断言 `a.intersects(&b) == false` | 对角分离：不相交 |
| 4 | 断言 `b.intersects(&a) == false` | 交换律 |
| 5 | 构造水平分离 `d = AABB { x:15, y:0, w:10, h:10 }` | 同一 Y，X 轴间隔 5 |
| 6 | 断言 `a.intersects(&d) == false` | 水平分离：不相交 |
| 7 | 构造垂直分离 `f = AABB { x:0, y:15, w:10, h:10 }` | 同一 X，Y 轴间隔 5 |
| 8 | 断言 `a.intersects(&f) == false` | 垂直分离：不相交 |

### 验证点

- 不相交判定正确（对角、水平、垂直三种方向）
- 不产生假阳性（false positive）碰撞
- 使用 `&&`（AND）而非 `||`（OR）组合各轴判定

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t15_non_overlapping_aabbs_do_not_intersect`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-002-007

### 关联需求

FR-006（Fixed Platforms）— 边界条件: 全覆盖 AABB 查询

### 测试目标

验证覆盖整个关卡的 AABB 查询返回所有平台，不遗漏任何一个。

### 前置条件

- Rust 工具链已安装
- `Level::new()` 构造成功

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | `Level::new()` 构造关卡实例 | 返回 `Level` |
| 2 | 获取 `bounds = level.bounds()` 和 `platform_count = level.platforms().len()` | 记录总平台数 |
| 3 | 构造覆盖整个关卡的 AABB（从 min_x 到 max_x，从 min_y 到 kill_y） | 覆盖全部平台区域 |
| 4 | `result = level.query_terrain(&full_aabb)` | 返回 Vec |
| 5 | 统计结果中的 `Tile::Platform` 数量 | `found_count == platform_count` |

### 验证点

- 全覆盖查询不遗漏任何平台
- 不存在提前终止（early termination）导致遗漏后续平台
- 迭代完整遍历所有平台，过滤条件正确应用

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **测试引用**: `tests/level_background_test.rs::t17_full_level_aabb_returns_all_platforms`
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-002-001 | FR-006 AC-1 | verification_steps[0] | t01_query_terrain_player_standing_on_platform | Real | PASS |
| ST-FUNC-002-002 | FR-006 AC-2 | verification_steps[1] | t02_query_terrain_player_in_air_returns_empty | Real | PASS |
| ST-FUNC-002-003 | FR-006 AC-3 | verification_steps[2] | t03_query_terrain_player_head_hits_platform_bottom | Real | PASS |
| ST-FUNC-002-004 | FR-006 AC-4 | verification_steps[3] | t04_query_terrain_player_side_collides_with_wall | Real | PASS |
| ST-FUNC-002-005 | FR-006 IAPI-008 | — | t05_level_bounds_returns_valid_level_bounds | Real | PASS |
| ST-FUNC-002-006 | FR-006 IAPI-010 | — | t06_parallax_layer_scroll_offset_calculation | Real | PASS |
| ST-FUNC-002-007 | FR-006 postcondition | — | t07_level_creates_three_parallax_layers_with_correct_speeds | Real | PASS |
| ST-FUNC-002-008 | FR-006 AC-1~4 | — | t08_aabb_intersects_detects_overlapping_boxes | Real | PASS |
| ST-FUNC-002-009 | FR-006 postcondition | — | t09_query_terrain_far_outside_level_returns_empty | Real | PASS |
| ST-FUNC-002-010 | FR-006 postcondition | — | t16_level_platforms_have_valid_dimensions | Real | PASS |
| ST-BNDRY-002-001 | FR-006 AC-1 | — | t10_zero_size_aabb_inside_platform_returns_platform | Real | PASS |
| ST-BNDRY-002-002 | FR-006 AC-4 | — | t11_boundary_touch_right_edge_equals_platform_left_edge | Real | PASS |
| ST-BNDRY-002-003 | FR-006 AC-1 | — | t12_boundary_touch_bottom_edge_equals_platform_top_edge | Real | PASS |
| ST-BNDRY-002-004 | FR-006 postcondition | — | t13_partial_out_of_bounds_aabb_returns_only_intersecting_platforms | Real | PASS |
| ST-BNDRY-002-005 | FR-006 AC-1 | — | t14_two_zero_size_aabbs_at_same_coordinate_intersect | Real | PASS |
| ST-BNDRY-002-006 | FR-006 AC-1~4 | — | t15_non_overlapping_aabbs_do_not_intersect | Real | PASS |
| ST-BNDRY-002-007 | FR-006 AC-1~4 | — | t17_full_level_aabb_returns_all_platforms | Real | PASS |

> srs_trace 覆盖：
> - FR-006 AC-1 (着陆): ST-FUNC-002-001, ST-BNDRY-002-001, ST-BNDRY-002-003, ST-BNDRY-002-005, ST-FUNC-002-008, ST-BNDRY-002-006, ST-BNDRY-002-007
> - FR-006 AC-2 (走出边缘): ST-FUNC-002-002, ST-FUNC-002-008, ST-BNDRY-002-006, ST-BNDRY-002-007
> - FR-006 AC-3 (底部碰撞): ST-FUNC-002-003, ST-FUNC-002-008, ST-BNDRY-002-006, ST-BNDRY-002-007
> - FR-006 AC-4 (侧面阻挡): ST-FUNC-002-004, ST-BNDRY-002-002, ST-FUNC-002-008, ST-BNDRY-002-006, ST-BNDRY-002-007
> - IAPI-008 (bounds): ST-FUNC-002-005
> - IAPI-010 (parallax): ST-FUNC-002-006
> - Level::new postcondition: ST-FUNC-002-007, ST-FUNC-002-009, ST-FUNC-002-010, ST-BNDRY-002-004
>
> 4 条 AC 全覆盖。每个 AC 有 ≥1 条 FUNC 用例直接映射。BNDRY 用例从边界语义角度补充覆盖。ATS 要求类别 FUNC + BNDRY 均已满足。

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 17 |
| Passed | 17 |
| Failed | 0 |
| Pending | 0 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> All 17 cases PASS via `cargo test` execution (2026-06-01). No failures, no manual test cases.
