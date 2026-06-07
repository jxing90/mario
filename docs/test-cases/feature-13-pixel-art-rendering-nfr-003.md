# 测试用例集: Pixel Art Rendering (NFR-003)

**Feature ID**: 13
**关联需求**: NFR-003 (Pixel Art Rendering)
**日期**: 2026-06-04
**测试标准**: ISO/IEC/IEEE 29119-3
**模板版本**: 1.0

## 摘要

| 类别 | 用例数 |
|------|--------|
| functional | 11 |
| boundary | 6 |
| ui | 1 |
| security | 0 |
| performance | 0 |
| **合计** | **18** |

> Specification resolutions applied from Feature Design Clarification Addendum: 无需澄清 —— 全部规格明确。
> ATS category enforcement: NFR-003 requires UI category (ATS §2.2). UI case ST-UI-013-002 satisfies this constraint.
> Notes: AC-1 (8x 放大像素边界检查) 需要人工视觉判断，标记为手动测试 ST-UI-013-002。

---

### 用例编号

ST-FUNC-013-001

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证 `count_colors` 对 1×1 纯白色 PNG 返回正确的唯一颜色数量 (1)

### 前置条件

- Rust 工具链已安装（env-guide.md §2）
- 项目已通过 `cargo build` 构建成功
- 测试通过程序化生成的 1×1 RGBA 白色 PNG 执行

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 1×1 纯白色 RGBA PNG 字节数组（(255,255,255,255)） | PNG 有效，1 种唯一颜色 |
| 2 | 调用 `SpritePalette::count_colors(&png_bytes)` | 函数不 panic |
| 3 | 断言返回值为 `Ok(1)` | 唯一颜色数量 = 1；Alpha 通道被计入颜色四元组 |

### 验证点

- Alpha 通道计入颜色计数（不可忽略）
- 最小合法 PNG（1×1）能正确解码
- 返回值类型为 `Result<u32, PaletteError>`

### 后置检查

- 无（纯函数，无副作用）

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t1_1x1_single_color_count`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-002

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证 `count_colors` 对 4×4 含 16 种不同 RGBA 颜色 PNG 返回正确色数 (16)

### 前置条件

- Rust 工具链已安装
- 测试通过程序化生成的 4×4 16 色 RGBA PNG 执行

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 4×4 含 16 种不同 RGBA 颜色的 PNG 字节数组 | PNG 有效，16 种唯一 RGBA 颜色 |
| 2 | 调用 `SpritePalette::count_colors(&png_bytes)` | 函数不 panic |
| 3 | 断言返回值为 `Ok(16)` | 唯一颜色数量 = 16，无颜色重复计数或遗漏 |

### 验证点

- HashSet 键包含全部 RGBA 四分量
- PNG 行对齐字节不影响颜色计数
- 不同 Alpha 值的相同 RGB 算作不同颜色

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t2_16_color_count`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-003

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证 `check_sprite` 对恰好 16 色精灵返回 `passed: true`

### 前置条件

- Rust 工具链已安装
- 测试使用 16 色 RGBA PNG

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 16 色 RGBA PNG | PNG 有效 |
| 2 | 调用 `SpritePalette::check_sprite("test", &png_bytes)` | 函数不 panic |
| 3 | 断言 `report.name == "test"` | 精灵名称被正确保留 |
| 4 | 断言 `report.color_count == 16` | 颜色计数正确 |
| 5 | 断言 `report.passed == true` | 恰好 16 色通过检查（≤ 16 含上限） |
| 6 | 断言 `report.error.is_none()` | 无错误信息 |

### 验证点

- 边界判断使用 `<=` 而非 `<`（off-by-one 防护）
- `SpriteReport` 所有字段正确填充
- 通过/失败判定逻辑正确

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t3_check_sprite_16_colors_pass`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-005

### 关联需求

NFR-003 AC-1（最近邻插值缩放）

### 测试目标

验证 `apply_pixel_art_filter()` 设置 `FILTER_APPLIED` 原子标志为 true

### 前置条件

- Rust 工具链已安装
- `FILTER_APPLIED` 原子标志初始化为 false

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 重置 `FILTER_APPLIED` 为 false（确保干净状态） | `FILTER_APPLIED.load(Acquire) == false` |
| 2 | 调用 `apply_pixel_art_filter()` | 函数执行，无 panic |
| 3 | 以 `Ordering::Acquire` 读取 `FILTER_APPLIED` | 值为 `true` |
| 4 | 再次读取 `FILTER_APPLIED` | 值保持 `true`（无翻转） |

### 验证点

- 过滤器应用后标志位正确设置
- 标志位的 store/load 内存顺序正确（Release/Acquire）
- 幂等性：多次调用不会翻转标志

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t5_apply_filter_sets_flag`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-006

### 关联需求

NFR-003 AC-1（最近邻插值缩放）

### 测试目标

验证 `round_sprite_pos` 将浮点坐标取整到最近整数（标准用例）

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `round_sprite_pos(Vec2 { x: 10.3, y: 5.7 })` | 函数不 panic |
| 2 | 断言返回值 `x == 10.0` | 10.3 取整到 10.0 |
| 3 | 断言返回值 `y == 6.0` | 5.7 取整到 6.0 |

### 验证点

- 使用 `round()` 而非 `floor()` 或 `trunc()`
- 正值取整方向正确
- Y 轴符号处理正确

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t6_round_sprite_pos_standard_case`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-007

### 关联需求

NFR-003（像素艺术渲染质量属性）

### 测试目标

验证 `count_colors` 对空字节切片返回 `Err(PaletteError::InvalidPng)` 且不 panic

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `SpritePalette::count_colors(&[])` | 函数不 panic |
| 2 | 断言返回值为 `Err(PaletteError::InvalidPng)` | 空输入被正确拒绝 |
| 3 | 断言错误信息 `msg` 非空 | 错误信息包含解码器诊断 |

### 验证点

- 空输入不触发 panic
- 错误类型为 `InvalidPng`
- 错误信息非空（携带解码器详情）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t7_count_colors_empty_input_error`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-008

### 关联需求

NFR-003（像素艺术渲染质量属性）

### 测试目标

验证 `count_colors` 对损坏/截断的 PNG 返回 `Err(PaletteError::InvalidPng)`

### 前置条件

- Rust 工具链已安装
- 测试数据：有效 PNG 签名 + IHDR 后截断的字节序列

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造有效 PNG 签名后截断的损坏字节序列 | 数据包含 PNG 魔数但无完整 IHDR/IDAT/IEND |
| 2 | 调用 `SpritePalette::count_colors(&corrupted)` | 函数不 panic |
| 3 | 断言返回值为 `Err(PaletteError::InvalidPng)` | 截断数据被检测为无效 PNG |

### 验证点

- 截断 PNG 不触发 panic
- 静默返回错误色数（防御）
- 解码器错误被正确捕获并传播

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t8_count_colors_corrupted_png_error`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-009

### 关联需求

NFR-003（像素艺术渲染质量属性）

### 测试目标

验证 `check_sprite` 将 `count_colors` 的错误正确传播到 `SpriteReport.error` 字段

### 前置条件

- Rust 工具链已安装
- 测试数据：非 PNG 格式的无效字节序列

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造非 PNG 格式字节序列 (`b"definitely not a png..."`) | 数据明确不是 PNG |
| 2 | 调用 `SpritePalette::check_sprite("bad", &bad_bytes)` | 函数不 panic |
| 3 | 断言 `report.name == "bad"` | 名称被保留 |
| 4 | 断言 `report.passed == false` | 无效输入不通过 |
| 5 | 断言 `report.error.is_some()` | 错误信息被传播 |
| 6 | 断言 `report.error.unwrap()` 非空字符串 | 错误信息有实质内容 |

### 验证点

- `count_colors` 错误没有被吞掉
- `passed == false` 且不是 true（假阴性防护）
- 错误信息非空

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t9_check_sprite_error_propagation`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-010

### 关联需求

NFR-003 AC-1（最近邻插值缩放）

### 测试目标

验证 `round_sprite_pos` 对 NaN 输入不 panic 且传播 NaN

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `round_sprite_pos(Vec2 { x: f32::NAN, y: 5.0 })` | 函数不 panic |
| 2 | 断言 `result.x.is_nan() == true` | NaN 被传播，非静默转换为 0.0 |
| 3 | 断言 `result.y == 5.0` | 正常值不受影响 |

### 验证点

- NaN 输入不触发 panic
- NaN 不被静默转换为 0.0
- 正常值处理不受 NaN 分量影响

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t10_round_sprite_pos_nan_no_panic`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-011

### 关联需求

NFR-003 AC-1（最近邻插值缩放）、F02（Level & Background）

### 测试目标

验证关卡 tile 渲染坐标经 `round_sprite_pos` 处理后为整数值（模拟摄像机变换后取整）

### 前置条件

- Rust 工具链已安装
- 模拟场景：平台 tile 世界坐标 (256.7, 180.3)，摄像机偏移 (20.5, 0.0)

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 计算屏幕坐标：`screen_x = 256.7 - 20.5 = 236.2`、`screen_y = 180.3` | 屏幕坐标含小数 |
| 2 | 调用 `round_sprite_pos(Vec2(236.2, 180.3))` | 函数不 panic |
| 3 | 断言 `rounded == Vec2(236.0, 180.0)` | 坐标取整到最近整数 |
| 4 | 断言 `rounded.x.fract() == 0.0` 且 `rounded.y.fract() == 0.0` | 结果无小数部分 |

### 验证点

- 摄像机变换后坐标被正确取整
- 无子像素坐标残留（`fract() == 0.0`）
- 取整方向不是 floor/ceil/trunc 而是 round

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette_real_test::t18_level_tile_coordinates_integer_after_rounding`
- **Test Type**: Real

---

### 用例编号

ST-FUNC-013-012

### 关联需求

NFR-003 AC-1（最近邻插值缩放）、F02（Parallax 背景层）

### 测试目标

验证视差背景层坐标（含分数滚动偏移）经 `round_sprite_pos` 处理后为整数值

### 前置条件

- Rust 工具链已安装
- 模拟场景：视差因子 0.5×，摄像机偏移 33.7 → 视差偏移 16.85

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 计算视差后屏幕坐标：`screen_x = 100.0 - 33.7 * 0.5 = 83.15` | 屏幕坐标含小数 |
| 2 | 调用 `round_sprite_pos(Vec2(83.15, 50.0))` | 函数不 panic |
| 3 | 断言 `rounded == Vec2(83.0, 50.0)` | 坐标取整正确 |
| 4 | 断言 `rounded.x.fract() == 0.0` | 视差层坐标无子像素偏移 |

### 验证点

- Parallax 分数偏移不引入子像素坐标
- 取整在摄像机变换和视差计算之后执行
- 所有背景层坐标均为整数

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: functional
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette_real_test::t18b_parallax_layer_coordinates_rounded`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-001

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证恰好 16 色精灵通过调色板检查（上限边界值）

### 前置条件

- Rust 工具链已安装
- 测试使用恰好 16 种不同 RGBA 颜色的 PNG

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 16 色 RGBA PNG | PNG 有效，恰好 16 种唯一颜色 |
| 2 | 调用 `SpritePalette::check_sprite("border_16", &png_bytes)` | 函数不 panic |
| 3 | 断言 `report.passed == true` | 恰好 16 色通过（≤ 16 用 `>=` 判定） |
| 4 | 断言 `report.color_count == 16` | 计数为 16 |

### 验证点

- 上限边界值（exactly 16）通过检查
- off-by-one 防护：判定使用 `>=` 而非 `>`
- 色数 = 16 时 `passed` 为 true

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t11_boundary_exactly_16_colors_pass`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-002

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证 17 色精灵被调色板检查拒绝（超出上限 1）

### 前置条件

- Rust 工具链已安装
- 测试使用恰好 17 种不同 RGBA 颜色的 PNG

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 17 色 RGBA PNG | PNG 有效，恰好 17 种唯一颜色 |
| 2 | 调用 `SpritePalette::check_sprite("over_17", &png_bytes)` | 函数不 panic |
| 3 | 断言 `report.passed == false` | 17 色 > 16 色上限，应被拒绝 |
| 4 | 断言 `report.color_count == 17` | 计数为 17（非 15 或其他错误值） |

### 验证点

- 超出上限 1 被正确拒绝
- 循环边界设置不为 15
- `color_count` 反映真实色数而非截断值

### 后置检查

- 无

### 元数据

- **优先级**: High
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t12_boundary_17_colors_fail`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-003

### 关联需求

NFR-003 AC-2（所有精灵 ≤ 16 色调色板）

### 测试目标

验证 1×1 最小合法 PNG 图像能正确解码和计数

### 前置条件

- Rust 工具链已安装
- 测试使用 1×1 RGBA PNG（单像素白色）

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 构造 1×1 RGBA PNG | PNG 有效，1×1 像素 |
| 2 | 调用 `SpritePalette::count_colors(&png_bytes)` | 返回 `Ok(1)` |
| 3 | 调用 `SpritePalette::check_sprite("min_1x1", &png_bytes)` | 返回 `SpriteReport { passed: true, color_count: 1 }` |

### 验证点

- 最小尺寸图像不触发除零或边界错误
- 宽度/高度比较使用 `<=` 而非 `<` 检查零尺寸
- `check_sprite` 对 1 色精灵返回 `passed: true`

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t13_boundary_minimum_1x1_image`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-004

### 关联需求

NFR-003 AC-1（最近邻插值缩放）

### 测试目标

验证 `.5` 坐标的银行家舍入行为（ties to even）

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `round_sprite_pos(Vec2 { x: 2.5, y: 3.5 })` | 函数不 panic |
| 2 | 断言 `result.x == 2.0` | 2.5 → 2.0（ties to even，2 是偶数） |
| 3 | 断言 `result.y == 4.0` | 3.5 → 4.0（ties to even，4 是偶数） |

### 验证点

- 银行家舍入而非总是向上取整
- 2.5 → 2.0 和 3.5 → 4.0 均正确
- ties to even 规则一致应用

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t14_boundary_bankers_rounding`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-005

### 关联需求

NFR-003 AC-1（最近邻插值缩放）

### 测试目标

验证负坐标的取整行为

### 前置条件

- Rust 工具链已安装

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 调用 `round_sprite_pos(Vec2 { x: -3.7, y: -3.2 })` | 函数不 panic |
| 2 | 断言 `result.x == -4.0` | -3.7 取整到 -4.0（远离零） |
| 3 | 断言 `result.y == -3.0` | -3.2 取整到 -3.0（向零） |

### 验证点

- 负坐标取整方向正确（不是 `trunc()` 朝向零）
- -3.7 → -4.0（远离零方向）
- -3.2 → -3.0（向零方向）

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette::tests::test_t15_boundary_negative_rounding`
- **Test Type**: Real

---

### 用例编号

ST-BNDRY-013-006

### 关联需求

NFR-003 AC-1（最近邻插值缩放）、F02（Level & Background）

### 测试目标

验证一批不同位置的关卡 tile 经摄像机变换和取整后均产生整数坐标

### 前置条件

- Rust 工具链已安装
- 模拟场景：5 个平台 tile 在不同世界位置，摄像机 x=150.25

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 对每个 tile 世界坐标计算屏幕坐标并调用 `round_sprite_pos` | 所有调用不 panic |
| 2 | 对每个结果断言 `x.fract() == 0.0` 且 `y.fract() == 0.0` | 所有 tile 均为整数像素坐标 |
| 3 | 对每个结果断言 `x` 和 `y` 等于 `round(expected)` | 取整方向正确 |

### 验证点

- 批量 tile 坐标全部为整数（无遗漏）
- 不同 tile 位置的取整行为一致
- 摄像机偏移不影响取整正确性

### 后置检查

- 无

### 元数据

- **优先级**: Medium
- **类别**: boundary
- **已自动化**: Yes
- **手动测试原因**: N/A
- **测试引用**: `palette_real_test::t18c_tile_batch_integer_coordinates`
- **Test Type**: Real

---

### 用例编号

ST-UI-013-002

### 关联需求

NFR-003 AC-1（像素边界 8× 放大检查）

### 测试目标

通过 8× 放大截图检查，验证精灵渲染使用最近邻插值，像素边界呈现纯色正方形，无过渡色或模糊边缘

### 前置条件

- 游戏编译并运行（`cargo build --release` → `target/release/mario-platformer.exe`）
- 精灵在游戏窗口中渲染（依赖 F02 Level & Background 提供关卡场景）
- 截屏工具可用（Windows Snipping Tool 或 PrintScreen）
- 图像查看器支持 8× 放大（如 Windows Photos、GIMP、Paint.NET）
- 全三种分辨率（720p/1080p/1440p）下执行

### 测试步骤

| Step | 操作 | 预期结果 |
| ---- | ---- | -------- |
| 1 | 以 720p 窗口模式启动游戏，进入 Playing 状态 | 游戏画面显示精灵（coin、heart）和关卡 tile |
| 2 | 截取游戏画面，用图像查看器以 8× 放大检查精灵边缘 | 精灵像素边界呈现纯色正方形，无过渡色/模糊边缘 |
| 3 | 对 1080p 分辨率重复步骤 2 | 像素边界保持纯色正方形，最近邻缩放生效 |
| 4 | 对 1440p 分辨率重复步骤 2 | 像素边界保持纯色正方形，最近邻缩放生效 |
| 5 | 在三种分辨率下检查关卡 tile 边缘 | tile 像素边界为纯色正方形，无模糊 |

### 验证点

- 精灵 8× 放大后像素边界为纯色正方形（无过渡色）
- 最近邻插值在全三种分辨率下生效
- 关卡 tile 同样使用最近邻缩放
- 无线性/双线性插值导致的模糊边缘

### 后置检查

- 游戏可正常退出
- 截图存档至指定路径

### 元数据

- **优先级**: High
- **类别**: ui
- **已自动化**: No
- **手动测试原因**: visual-judgment
- **测试引用**: N/A (manual)
- **Test Type**: Real

---

## 可追溯矩阵

| 用例 ID | 关联需求 | verification_step | 自动化测试 | Test Type | 结果 |
|---------|----------|-------------------|-----------|---------|------|
| ST-FUNC-013-001 | NFR-003 | AC-2 (精灵 ≤ 16 色) | `palette::tests::test_t1_1x1_single_color_count` | Real | PASS |
| ST-FUNC-013-002 | NFR-003 | AC-2 (精灵 ≤ 16 色) | `palette::tests::test_t2_16_color_count` | Real | PASS |
| ST-FUNC-013-003 | NFR-003 | AC-2 (精灵 ≤ 16 色) | `palette::tests::test_t3_check_sprite_16_colors_pass` | Real | PASS |
| ST-FUNC-013-005 | NFR-003 | AC-1 (最近邻缩放) | `palette::tests::test_t5_apply_filter_sets_flag` | Real | PASS |
| ST-FUNC-013-006 | NFR-003 | AC-1 (最近邻缩放) | `palette::tests::test_t6_round_sprite_pos_standard_case` | Real | PASS |
| ST-FUNC-013-007 | NFR-003 | AC-2 (错误处理) | `palette::tests::test_t7_count_colors_empty_input_error` | Real | PASS |
| ST-FUNC-013-008 | NFR-003 | AC-2 (错误处理) | `palette::tests::test_t8_count_colors_corrupted_png_error` | Real | PASS |
| ST-FUNC-013-009 | NFR-003 | AC-2 (错误传播) | `palette::tests::test_t9_check_sprite_error_propagation` | Real | PASS |
| ST-FUNC-013-010 | NFR-003 | AC-1 (NaN 处理) | `palette::tests::test_t10_round_sprite_pos_nan_no_panic` | Real | PASS |
| ST-FUNC-013-011 | NFR-003, F02 | AC-1 (tile 坐标取整) | `palette_real_test::t18_level_tile_coordinates_integer_after_rounding` | Real | PASS |
| ST-FUNC-013-012 | NFR-003, F02 | AC-1 (视差坐标取整) | `palette_real_test::t18b_parallax_layer_coordinates_rounded` | Real | PASS |
| ST-BNDRY-013-001 | NFR-003 | AC-2 (16 色上限边界) | `palette::tests::test_t11_boundary_exactly_16_colors_pass` | Real | PASS |
| ST-BNDRY-013-002 | NFR-003 | AC-2 (17 色超出边界) | `palette::tests::test_t12_boundary_17_colors_fail` | Real | PASS |
| ST-BNDRY-013-003 | NFR-003 | AC-2 (1×1 最小图像) | `palette::tests::test_t13_boundary_minimum_1x1_image` | Real | PASS |
| ST-BNDRY-013-004 | NFR-003 | AC-1 (银行家舍入) | `palette::tests::test_t14_boundary_bankers_rounding` | Real | PASS |
| ST-BNDRY-013-005 | NFR-003 | AC-1 (负坐标取整) | `palette::tests::test_t15_boundary_negative_rounding` | Real | PASS |
| ST-BNDRY-013-006 | NFR-003, F02 | AC-1 (批量 tile 坐标) | `palette_real_test::t18c_tile_batch_integer_coordinates` | Real | PASS |
| ST-UI-013-002 | NFR-003 | AC-1 (8× 放大检查) | N/A (manual) | Real | PENDING-MANUAL |

> 结果 valid values: `PENDING`, `PASS`, `FAIL`, `MANUAL-PASS`, `MANUAL-FAIL`, `BLOCKED`, `PENDING-MANUAL`

## Real Test Case Execution Summary

| Metric | Count |
|--------|-------|
| Total Real Test Cases | 18 |
| Passed | 17 |
| Failed | 0 |
| Pending | 1 |

> Real test cases = test cases with Test Type `Real` (executed against a real running environment, not Mock).
> Any Real test case FAIL blocks the feature from being marked `"passing"` — must be fixed and re-executed.
> 17/17 automated cases PASS; 1 manual case (ST-UI-013-002) awaits human review.

## Manual Test Case Summary

| Metric | Count |
|--------|-------|
| Total Manual Test Cases | 1 |
| Manual Passed (MANUAL-PASS) | 0 |
| Manual Failed (MANUAL-FAIL) | 0 |
| Blocked | 0 |
| Pending (PENDING-MANUAL) | 1 |

> Manual test cases = test cases with `已自动化: No`. Results collected via human review gate after automated execution.
> Any MANUAL-FAIL blocks the feature from being marked `"passing"` — same as automated FAIL.
> ST-UI-013-002 requires 8× magnification pixel boundary visual inspection across 720p/1080p/1440p.
