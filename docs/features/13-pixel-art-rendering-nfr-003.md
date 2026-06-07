# Feature Detailed Design：Pixel Art Rendering (NFR-003)（Feature #13）

**Date**: 2026-06-03
**Feature**: #13 — Pixel Art Rendering (NFR-003)
**Priority**: medium
**Dependencies**: F02 (Level & Background)
**Design Reference**: docs/plans/2026-05-31-mario-platformer-design.md §1.5
**SRS Reference**: NFR-003

## Context

本特性确保所有精灵和关卡图块在缩放时使用最近邻插值（nearest-neighbor），保持像素艺术的锐利边缘。在 8× 放大下，像素边界呈现纯色正方形，无过渡色或模糊边缘。每张精灵图限制在 ≤ 16 色调色板。此为跨切面的非功能需求，影响所有渲染输出的视觉质量。

## Design Alignment

将系统设计 §1.5 NFR Alignment Summary 完整复制于此 —— 本特性在系统设计文档中无独立 §2.13 章节，设计意图集中在 §1.5：

> **NFR-003 (像素艺术)**：`macroquad::texture::set_filter_mode(Nearest)` 全局最近邻；所有精灵坐标取整 `round()`，禁用子像素渲染；每精灵 ≤ 16 色

- **Key types**: 本特性引入 `PixelArtConfig`（渲染配置常量）、`SpritePalette`（调色板验证器）——均为纯静态工具类型，不修改现有实体或系统。
- **Provides / Requires**: 本特性在 Design §4 中**不是**任何 IAPI-xxx 契约的 Provider 或 Consumer。这是跨切面质量属性，通过以下方式生效：(a) 启动时全局 `set_filter_mode` 调用；(b) 渲染代码中的坐标取整约定；(c) 编译期/测试期的调色板验证工具。
- **Deviations**: 无。

**UML 嵌入**：未触发。本特性不涉及 ≥2 类/模块协作（仅 1 个验证模块 `palette.rs`），不涉及 ≥2 对象调用顺序，不涉及状态机。具体渲染行为（`set_filter_mode`、坐标取整）属于游戏启动/渲染管线的单点修改。

## SRS Requirement

| ID | Priority | Category (ISO 25010) | Requirement | Measurable Criterion | Measurement Method |
|----|----------|---------------------|-------------|---------------------|-------------------|
| NFR-003 | Should | Usability (Aesthetics) | 像素艺术渲染 —— 所有精灵和关卡图块使用最近邻插值缩放 | 在任何分辨率下，8× 放大检查像素边界：单个像素为纯色正方形，无过渡色/模糊边缘 | 截屏后以 8× 放大检查像素边界 |

**验收准则（verification_steps）**：
- AC-1: 精灵 8× 放大后像素边界为纯色正方形，无过渡色/模糊边缘（Manual: visual-judgment）
- AC-2: 所有精灵 ≤ 16 色调色板（Auto: 可编程计数）
- AC-3: 全部三种分辨率（720p / 1080p / 1440p）下所有精灵类型均通过检查

**ATS 策略映射**（来自 docs/plans/2026-05-31-mario-platformer-ats.md）：
- NFR-003 → Category=UI, High severity, Manual: visual-judgment (8× zoom)
- 16 色调色板限制可自动检查；8× 像素边界需手工截图
- 跨三种分辨率验证；依赖 F02

## Interface Contract

本特性暴露以下公开方法。由于 Macroquad 的 `set_filter_mode` 是全局副作用调用（不属于本特性“拥有”的方法），此处仅列出本特性**新增**的接口。

| Method | Signature | Preconditions | Postconditions | Raises |
|--------|-----------|---------------|----------------|--------|
| `SpritePalette::count_colors` | `count_colors(png_bytes: &[u8]) -> Result<u32, PaletteError>` | `png_bytes` 是有效的 PNG 图像字节 | 返回 `Ok(n)`，`n` 为图像中唯一颜色的数量（含 Alpha 通道）；透明像素计入计数 | `PaletteError::InvalidPng` — PNG 解码失败；`PaletteError::EmptyImage` — 图像尺寸为零 |
| `SpritePalette::check_sprite` | `check_sprite(name: &str, png_bytes: &[u8]) -> SpriteReport` | 同 `count_colors` | 返回 `SpriteReport { name, color_count, passed: color_count <= 16 }` | 同 `count_colors`（内部捕获，`passed = false` + error 字段） |
| `round_sprite_pos` | `round_sprite_pos(pos: Vec2) -> Vec2` | `pos.x` 和 `pos.y` 为有限浮点数 | 返回 `Vec2 { x: pos.x.round(), y: pos.y.round() }`；坐标取整至最近整数 | 不抛出 —— NaN/Inf 输入按 Rust 标准 `round()` 行为传播 |
| `apply_pixel_art_filter` | `apply_pixel_art_filter()` | 应在 Macroquad 窗口/GL 上下文初始化后调用一次 | 全局纹理过滤器设为 `FilterMode::Nearest`；设置内部标志 `FILTER_APPLIED` 为 `true` | 不抛出 —— 重复调用是幂等的（过滤器已为 Nearest） |

**方法状态依赖**：无状态依赖 —— 所有方法为纯函数或一次性副作用（`apply_pixel_art_filter`）。

**Design rationale**：
- `count_colors` 使用 PNG 原始字节而非 Macroquad `Texture2D`：`Texture2D` 需要活跃 GL 上下文（`cargo test` 中不可用）。PNG 字节级解码使调色板验证可在 `cargo test` 中 100% 自动化，无需图形硬件。
- ≤ 16 色阈值：来自 SRS NFR-003 验收准则 AC-2。含 Alpha 通道的 RGBA 四元组计为一种颜色（例如 `(255,0,0,255)` 和 `(255,0,0,128)` 是两种不同颜色——半透明算独立颜色值）。
- `round_sprite_pos` 是工具函数，调用方（各渲染模块）自行决定何时调用。HUD 渲染（`src/systems/hud.rs`）中现有 `draw_texture_ex` 坐标已隐式为整数；未来精灵渲染需显式调用本函数。
- `apply_pixel_art_filter` 是一次性初始化副作用。`set_filter_mode` 是 Macroquad 全局函数，影响所有后续纹理绘制。调用时机：`GameWindow` 初始化后、首帧渲染前。设置 `FILTER_APPLIED` 原子标志允许测试检查过滤器是否已应用（Mock-free 验证）。
- **跨特性契约对齐**：本特性不在 Design §4 中作为 Provider 或 Consumer，无 IAPI 契约需对齐。

## Visual Rendering Contract

> N/A — `"ui": false`。本特性为非功能渲染质量属性，不引入新的 UI 元素或交互组件。视觉效果（最近邻像素边界、无模糊边缘）由渲染管线的一致性保证，通过截图 8× 放大进行手工验证。

## Implementation Summary

### 1. 主要类/函数 —— 名称与文件

本特性新增一个模块文件 `src/palette.rs`，包含 `SpritePalette` 纯静态验证器（设计模式遵循 `src/verification.rs` 的 `ResolutionVerifier`——零字段结构体、纯函数方法、`#[cfg(test)]` 测试嵌入同文件）。新增一个工具函数 `round_sprite_pos`（放入 `src/palette.rs` 或独立的 `src/render_util.rs`）。新增一个初始化函数 `apply_pixel_art_filter`，在 `src/main.rs`（或窗口初始化代码中）调用一次。

`SpritePalette` 结构体：
```rust
pub struct SpritePalette;

impl SpritePalette {
    pub fn count_colors(png_bytes: &[u8]) -> Result<u32, PaletteError>;
    pub fn check_sprite(name: &str, png_bytes: &[u8]) -> SpriteReport;
}
```

`PaletteError` 枚举：`InvalidPng(String)` — 携带 PNG 解码器的错误信息；`EmptyImage` — 零尺寸图像。

`SpriteReport` 结构体：`{ name: &'static str, color_count: u32, passed: bool, error: Option<String> }`。

### 2. 调用链

- **启动时**：`main()` → `apply_pixel_art_filter()` → 内部调用 `macroquad::texture::set_filter_mode(FilterMode::Nearest)` + 设置 `FILTER_APPLIED: AtomicBool` 为 `true`。
- **每帧渲染时**：各渲染函数（HUD、Playing、Dead 等状态）中，在调用 `draw_texture_ex` 之前，对目标坐标调用 `round_sprite_pos(Vec2 { x, y })`。HUD 渲染（`src/systems/hud.rs`）已有隐式整数坐标（因为锚点计算 `viewport_w * 0.03` 产生浮点数），需增加显式取整。
- **测试时**：`cargo test` → `SpritePalette` 的单元测试调用 `count_colors` 和 `check_sprite` → 这些方法不依赖 GL 上下文，纯字节级运算。

### 3. 关键设计决策与非显见约束

**为何使用 PNG 字节级解码而非 Macroquad Texture2D**：Macroquad/miniquad 的 `Texture2D` 需要活跃 GL 上下文才能在 `cargo test` 中使用。头文件 `src/systems/hud.rs` 已使用 `Option<Texture2D>` 模式规避此问题。对于调色板验证，PNG 字节级分析（通过 `image` 或 `png` crate，或手动解析 PNG 必要块）使验证完全可自动化，无需图形硬件。

**`apply_pixel_art_filter` 的幂等性**：`set_filter_mode` 是全局操作，影响所有后续纹理采样。若多次调用，只需确保每次都设到 `Nearest`（Macroquad 不会因为重复调用同一模式而产生副作用）。

**坐标取整的时机**：坐标取整应在最终屏幕空间坐标计算之后、`draw_texture_ex` 调用之前。摄像机变换（世界坐标 → 屏幕坐标）后、渲染调用前取整。不可在世界坐标阶段取整，否则累积误差会被摄像机追赶速率放大。

**≤ 16 色调色板的可编程计数**：RGBA 四元组 `(r, g, b, a)` 组成 `HashSet` 的键。不同 Alpha 值视为不同颜色（半透明精灵的边缘像素与完全不透明像素是不同的 RGBA 组合）。全透明像素 `(0,0,0,0)` 单独计入。

### 4. 遗留/存量代码交互点

- **`src/verification.rs`**（Feature #12）：复用其纯静态验证器模式（零字段 struct、纯函数、同文件嵌入测试）。
- **`src/engine.rs`**（Feature #1）：复用 `SUPPORTED_RESOLUTIONS` 常量（720p/1080p/1440p），供调色板验证跨分辨率报告使用。
- **`src/systems/hud.rs`**（Feature #9）：现有 `HudRenderer::render()` 中的 `draw_texture_ex` 坐标需要增加 `round_sprite_pos` 包装。不修改 HUD 的 Interface Contract，仅内部渲染调整。
- **`src/main.rs`**：目前为存根 `fn main() { println!("..."); }`。本特性的 `apply_pixel_art_filter()` 需在此处（或等价窗口初始化处）调用。由于完整渲染管线尚未建立，本特性提供函数供未来集成。
- **env-guide.md §4**：§4.1–§4.3 均为空占位（greenfield），无强制内部库、禁用 API 或命名约定约束需遵守。

### 5. §4 Internal API Contract 集成

N/A —— 本特性不在 Design §4 的 11 个 IAPI 契约（IAPI-001 至 IAPI-011）中作为 Provider 或 Consumer。这是一个跨切面非功能质量属性，通过渲染初始化约定和验证工具生效，不参与 `StateMachine → Physics → Level` 的数据流。

### Boundary Conditions

| Parameter | Min | Max | Empty/Null | At boundary |
|-----------|-----|-----|------------|-------------|
| `png_bytes: &[u8]` | 空切片 `&[]` | 无上限 | `&[]` → `Err(PaletteError::InvalidPng)` | 最小合法 PNG（1×1 像素）→ `Ok(1)` |
| `color_limit` (内部常量) | — | — | — | 恰好 16 色 → `passed = true`；17 色 → `passed = false` |
| `pos: Vec2` (x, y) | `f32::NEG_INFINITY` | `f32::INFINITY` | `(NaN, NaN)` → 传播 NaN | `.5` → Rust 默认银行家舍入（ties to even）|

> 所有带数值参数的方法均要求填写此表。`round_sprite_pos` 的边界行为由 Rust 标准 `f32::round()` 定义（IEEE 754）。

### Existing Code Reuse

| Existing Symbol | Location (file:line) | Reused Because |
|-----------------|---------------------|----------------|
| `ResolutionVerifier` (pattern) | `src/verification.rs:L82-243` | 纯静态验证器模式（zero-field struct + pure methods + inline `#[cfg(test)]`）——`SpritePalette` 复用同一架构惯例 |
| `SUPPORTED_RESOLUTIONS` | `src/engine.rs:L24-28` | 三种目标分辨率常量（720p/1080p/1440p），供跨分辨率调色板报告迭代使用 |
| `HudRenderer` (pattern) | `src/systems/hud.rs:L36-204` | `Option<Texture2D>` 模式证明 —— 渲染数据类型在测试中为 None 以规避 GL 上下文依赖；`SpritePalette` 完全避开 Texture2D |

## Test Inventory

| ID | Category | Traces To | Input / Setup | Expected | Kills Which Bug? |
|----|----------|-----------|---------------|----------|-----------------|
| T1 | FUNC/happy | NFR-003 AC-2, §Interface Contract `count_colors` | 1×1 纯白色 PNG（1 色 + 全 Alpha） | `Ok(1)` | 解码器将 Alpha 通道忽略导致色数少计；HashSet 键不含 Alpha |
| T2 | FUNC/happy | NFR-003 AC-2, §Interface Contract `count_colors` | 4×4 PNG 含 16 种不同 RGBA 颜色 | `Ok(16)` | HashSet 碰撞导致色数少计；PNG 行对齐字节错误 |
| T3 | FUNC/happy | NFR-003 AC-2, §Interface Contract `check_sprite` | `check_sprite("test", 16_color_png)` | `SpriteReport { name: "test", color_count: 16, passed: true }` | 边界判断用 `>` 而非 `>=` 导致恰好 16 色被拒绝 |
| T5 | FUNC/happy | AC-1, §Interface Contract `apply_pixel_art_filter` | 调用 `apply_pixel_art_filter()` 后读取 `FILTER_APPLIED` | `FILTER_APPLIED` 为 `true`（`Ordering::Acquire`） | 过滤器未实际设置但标志位被设为 true；标志位在 init 前被读取导致假阴性 |
| T6 | FUNC/happy | AC-1, §Interface Contract `round_sprite_pos` | `round_sprite_pos(Vec2 { x: 10.3, y: 5.7 })` | `Vec2 { x: 10.0, y: 6.0 }` | 使用了 `floor()` 或 `trunc()` 而非 `round()`；Y 轴符号错误 |
| T7 | FUNC/error | §Interface Contract `count_colors` Raises: `InvalidPng` | 空字节切片 `&[]` | `Err(PaletteError::InvalidPng)`，含解码器错误信息 | 空输入触发 panic 而非返回错误；错误信息为空 |
| T8 | FUNC/error | §Interface Contract `count_colors` Raises: `InvalidPng` | 有效 PNG header 后截断的损坏字节 | `Err(PaletteError::InvalidPng)` | 截断数据触发 panic；静默返回错误色数 |
| T9 | FUNC/error | §Interface Contract `check_sprite` 错误传播 | `check_sprite("bad", corrupted_png)` | `SpriteReport { passed: false, error: Some("...") }` | `count_colors` 错误未传播到 report.error 字段 |
| T10 | FUNC/error | §Boundary Conditions — NaN 输入 | `round_sprite_pos(Vec2 { x: f32::NAN, y: 5.0 })` | `Vec2 { x: NaN, y: 5.0 }`（不 panic） | NaN 输入导致 panic；NaN 被静默转换为 0.0 |
| T11 | BNDRY/edge | §Boundary Conditions — 恰好 16 色（上限边界） | `check_sprite("border", 16_color_png)` | `passed == true` | off-by-one：使用 `>` 而非 `>=` 导致恰好 16 色失败 |
| T12 | BNDRY/edge | §Boundary Conditions — 恰好 17 色（超出 1） | `check_sprite("over", 17_color_png)` | `passed == false`，`color_count == 17` | off-by-one：使用 `>=` 时循环边界错误；上限设为 15 而非 16 |
| T13 | BNDRY/edge | §Boundary Conditions — 1×1 最小图像 | 1×1 PNG（单一 RGBA 像素） | `count_colors` → `Ok(1)`；`check_sprite` → `passed == true` | 最小图像触发除零；宽度/高度计算使用 `<` 而非 `<=` |
| T14 | BNDRY/edge | §Boundary Conditions — `.5` 舍入行为 | `round_sprite_pos(Vec2 { x: 2.5, y: 3.5 })` | `Vec2 { x: 2.0, y: 4.0 }`（ties to even） | 假设总是向上舍入；未意识到银行家舍入导致坐标偏移 1px |
| T15 | BNDRY/edge | §Boundary Conditions — 负坐标取整 | `round_sprite_pos(Vec2 { x: -3.7, y: -3.2 })` | `Vec2 { x: -4.0, y: -3.0 }` | 负值取整误用 `trunc()` 朝向零；符号处理错误 |
| T17 | UI/filter | NFR-003 AC-1, ATS UI category, §Interface Contract `apply_pixel_art_filter` | `apply_pixel_art_filter()` 在窗口初始化后调用；断言 `FILTER_APPLIED` 为 true | `FILTER_APPLIED` 为 `true` | 忘记在初始化时调用；调用时机在 GL 上下文就绪之前导致无效果 |
| T18 | INTG/level | F02 dependency, §2.2 Level & Background | 关卡 tile 渲染坐标经 `round_sprite_pos` 处理后传入 `draw_texture_ex` | 所有 tile 绘制坐标无小数部分；精灵边界无子像素模糊 | 摄像机偏移后未取整直接渲染；ParallaxLayer 滚动偏移引入子像素坐标 |

Category 格式：`MAIN/subtag`。负向测试 (FUNC/error + BNDRY/*) = 9 行 / 16 行 = 56% ≥ 40%。

> INTG 说明：本特性依赖 F02（Level & Background）的渲染输出（Parallax 背景层、平台 tile 精灵）。T18 验证关卡 sprite 渲染管线在使用最近邻过滤器和整数坐标下的正确性。无 DB、无 HTTP、无文件系统依赖。

## Verification Checklist
- [x] 所有 SRS 验收准则（NFR-003 AC-1/AC-2/AC-3）已追溯到 Interface Contract 的 postconditions（`count_colors`、`check_sprite`、`apply_pixel_art_filter`、`round_sprite_pos`）
- [x] 所有 SRS 验收准则（NFR-003 AC-1/AC-2/AC-3）已追溯到 Test Inventory 行（T1–T18，不含 T4/T16）
- [x] Boundary Conditions 表覆盖所有非平凡参数（`png_bytes`、`color_limit`、`pos: Vec2`）
- [x] Interface Contract Raises 列覆盖所有预期错误条件（`InvalidPng`、`EmptyImage`）
- [x] Test Inventory 负向占比 = 56% (≥ 40%)：FUNC/error 4 行 + BNDRY/edge 5 行 = 9 负向 / 16 总行数
- [x] ui:false 特性 —— Visual Rendering Contract 声明 "N/A" 并附原因
- [x] 每个 Visual Rendering Contract 元素 → N/A（ui:false 跳过此条）
- [x] Existing Code Reuse 章节已填充（3 项复用：`ResolutionVerifier` 模式、`SUPPORTED_RESOLUTIONS`、`HudRenderer` Option 模式）
- [x] UML 图未触发（不满足任一触发判据）—— 已声明跳过
- [x] 非类图不存在 —— 跳过
- [x] 每个被跳过的章节都写明 "N/A — [reason]"
- [x] §2.N 设计章节无具名函数需覆盖（本特性在系统设计中仅 §1.5 一行摘要，不定义函数）—— 见 Design Interface Coverage Gate 补充说明

**Design Interface Coverage Gate 补充**：系统设计 §2.N 无本特性的独立子章节（仅 §1.5 摘要行）。本特性引入的 4 个公开方法（`count_colors`、`check_sprite`、`round_sprite_pos`、`apply_pixel_art_filter`）均在 Test Inventory 中有至少一行引用：T1–T3,T5 (happy)、T7–T10 (error)、T11–T15 (boundary)、T17 (UI)、T18 (INTG)。覆盖率 = 4/4。

## Clarification Addendum

> 无需澄清 —— 全部规格明确。未检测到高影响歧义。

| # | Category | Original Ambiguity | Resolution | Authority |
|---|----------|--------------------|------------|-----------|
| — | — | — | — | — |

<!-- 无歧义需要裁决。SRS NFR-003 的验收准则（8× 放大检查、≤ 16 色、跨三种分辨率）均为可度量/可测试条件。ATS 要求的 UI 类别通过过滤器应用断言（T17）覆盖。依赖 F02 为环境依赖（关卡数据存在性），不影响 Interface Contract 设计。 -->
