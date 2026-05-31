# Mario 2D Platformer Demo — Worker Session Guide

> **角色**: 本项目 Worker 会话（`long-task-work-design` / `long-task-work-tdd` / `long-task-work-st`）的工作流导航。
> **命令单一事实源**: `env-guide.md` §3。本指南仅导航流程，不内嵌具体构建 / 测试 / 覆盖率命令。

## Orient

每轮 Worker 会话启动时：

1. 调 `python scripts/phase_route.py --json` 读取 `next_skill`、`feature_id`、`starting_new`
2. 读 `feature-list.json` 根的 `current`、`constraints[]`、`assumptions[]`、`quality_gates`、`tech_stack`
3. 读 `docs/plans/*-srs.md` 中 `target_feature.srs_trace` 指向的 FR/NFR 节
4. 读 `docs/plans/*-design.md` §1（架构概览）+ 当前特性对应的 §2.N 子节
5. 读 `env-guide.md` §4（存量代码库约束）——greenfield 项目 §4 为占位状态
6. 若 `target_feature.ui == true` 且 `docs/plans/*-ucd.md` 存在：读对应 UI 组件节
7. 若 `docs/plans/*-ats.md` 存在：读 `target_feature.srs_trace` 的类别映射行
8. `git log --oneline -10`

## Bootstrap

按 `env-guide.md` §2 激活 Rust 工具链环境：

- 验证 `rustup`、`cargo`、`rustc` 在 PATH 中
- 验证 MSVC 构建工具可用（Macroquad 依赖）
- 若 `init.ps1` / `init.sh` 存在且环境未就绪：运行一次
- 本游戏为原生桌面应用，无服务进程。Worker 阶段无需启动任何服务

环境就绪后，以 `env-guide.md` §3 的构建命令验证项目可编译。

## Config Gate

运行 `python scripts/check_configs.py feature-list.json --feature <id>` 检查目标特性的必需配置。

本项目为离线桌面游戏（CON-004），无 `env`-type 配置项（无 API key、无外部服务 URL、无数据库连接）。若 `required_configs[]` 为空，Config Gate 直接通过。

若有 `required_configs` 缺失：通过 AskUserQuestion 收集 → 写入 `.env` → 重跑 `check_configs.py` 直至通过。

## TDD Red

`long-task-work-tdd` Step 3a 分发 `long-task-tdd-red` SubAgent。

**测试命令**: See `env-guide.md` §3 单元测试命令。

**关键规则（来自 ATS 类别约束）**:
- 每个 FR ≥ 1 happy-path + 1 error-path 场景；总计 ≥ 3 场景/FR
- 含 BNDRY 类别的 FR：必须显式覆盖边界值（速度上限、跳跃时长上限、生命=0、无敌=2.0s、碰撞刚好在边缘）
- `f32` 比较使用 `approx` 容差（收敛用 epsilon=0.5px，物理用 epsilon=0.01）
- 含 PERF 类别的 FR (FR-018 / NFR-001)：帧时间度量测试

**UI 特性额外规则**: 见下方 "UI Testing" 节。

## TDD Green

`long-task-work-tdd` Step 3b 分发 `long-task-tdd-green` SubAgent。

编写最小实现使全部测试通过。实现必须遵守 `env-guide.md` §4 存量代码库约束。
构建命令: See `env-guide.md` §3 构建命令。

## Coverage Gate

`long-task-work-tdd` Step 4 分发 `long-task-quality` SubAgent。

覆盖率阈值（来自 `feature-list.json`）：
- **行覆盖率 ≥ 80%**
- **分支覆盖率 ≥ 70%**

覆盖率命令: See `env-guide.md` §3 覆盖率命令。

低于阈值 → 回到 TDD Red 扩测，不进入 Refactor。

## TDD Refactor

`long-task-work-tdd` Step 3c 分发 `long-task-tdd-refactor` SubAgent。

在保持测试全绿的前提下清理代码。运行静态分析（`cargo clippy -- -D warnings`），修复全部违规。
静态分析命令: See `env-guide.md` §3 静态分析命令。

**重构后重新验证**: §4 契约对齐 + §6 实现摘要一致性 + §8 测试清单覆盖。

## Verification Enforcement

**绝不标记 `passing` 而未展示实跑通过的测试输出。** 每个 verification_step 必须逐条确认全绿。

若观察到 "should pass" 或 "probably works" 想法 → 停下，运行实际测试，读取完整输出。
See `env-guide.md` §3 测试命令。

**UI 正向渲染验证**: 手工截图对比 (FR-017 / NFR-002 / NFR-003)，在 Feature-ST 阶段执行。

## ST Test Cases

`long-task-work-st` Step 3 分发 `long-task-feature-st` SubAgent 产出 ISO/IEC/IEEE 29119 测试用例文档。

ST 用例覆盖 ATS §2 中各 FR 声明的全部必须类别（FUNC / BNDRY / PERF / UI）。含 UI 类别的需求 (FR-014c、FR-015、FR-016、FR-017) 需额外的 Manual: visual-judgment 用例。

**硬关卡**: ATS 必须类别无 ST 用例 → `blocked` → 回 Step 3 扩 ST 用例。

## Inline Compliance Check

`long-task-work-st` Step 4 在主 agent 直接执行（无 SubAgent）：

- **接口契约校验 (P2)**: grep 实现文件确认 Design §4 Interface Contract 每个 PUBLIC 方法签名匹配
- **Test Inventory ↔ 测试文件交叉 (T2)**: `grep` 每行测试函数名确认存在于测试文件
- **UCD 抽查 (U1)**: UI 特性 grep CSS/样式文件查不在 UCD 色板中的硬编码颜色
- **§4 存量约定全差异扫描**: `git diff HEAD~1 --name-only` → 对变更文件核查 §4.1/§4.2/§4.3
- **ATS 类别覆盖卫生**: `python scripts/check_ats_coverage.py docs/plans/*-ats.md --feature-list feature-list.json --feature <id>`

任意检查失败 → 就地修复重校。

## Persist

`long-task-work-st` Step 5 执行最终落盘：

1. **git commit** 实现 + 测试 + ST 测试用例文档
2. 抓取 SHA: `git rev-parse --short HEAD`
3. 更新 `feature-list.json`: `target_feature.status: failing → passing`，根 `current: {feature_id, phase} → null`
4. 更新 `RELEASE_NOTES.md` (Keep a Changelog 格式)
5. 更新 `task-progress.md` 追加会话条目含 risks
6. 校验: `python scripts/validate_features.py feature-list.json`
7. 再次 git commit 进度文件

## Config Management

本项目配置管理策略：

- **游戏参数**（物理常量、关卡布局）: 硬编码于 `src/` 目录 Rust 常量中，无外部配置文件
- **分辨率/全屏设置**: 通过游戏内 Options 菜单 (FR-017 / F10) 运行时切换，不持久化
- **环境变量**: 本项目为离线桌面游戏，无 API key 或外部服务 URL。若将来引入 `required_configs`，通过 `.env` 文件管理
- **新增/更新 config**: 
  1. 在 `feature-list.json` 的 `required_configs[]` 中声明新配置项（name / type / description / required_by / check_hint）
  2. `env` 类型：更新 `.env.example` 模板注释
  3. `file` 类型：更新 `.env.example` 说明文件生成方式
  4. 运行 `python scripts/check_configs.py feature-list.json` 验证声明完整性

## Real Test Convention

本项目以 **Rust 原生 `#[test]`** 为测试框架，区分"真实测试"与"mock/占位测试"：

**识别方法**:
- **真实测试（Real tests）**: 测试函数调用业务逻辑（实体方法、物理计算、状态机转换、摄像机算法、碰撞检测），构造输入 → 调用被测函数 → 断言输出。这些测试不依赖窗口/GPU/渲染上下文。
- **非真实测试（Mock-reliant / Placeholder tests）**: 需要窗口实例化 (`macroquad::Window::new`)、渲染上下文、GPU 纹理的测试。本项目中这些场景归入 Manual: visual-judgment，不编写 `#[test]` 函数。

**目录约定**:
- 单元测试：与源码同文件，`#[cfg(test)] mod tests { ... }`
- 集成测试：`tests/` 目录下 `*.rs` 文件

**仅运行真实测试**: See `env-guide.md` §3 单元测试命令。

**示例（真实测试 vs Mock-reliant）**:

```rust
// 真实测试 — 可自动化，不依赖 GPU/窗口
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_accelerates_to_max_speed() {
        let mut player = Player::new(Vec2::new(0.0, 0.0));
        let input = InputState { right: true, ..Default::default() };
        for _ in 0..20 { player.update(1.0/60.0, &input, &[]); }
        assert!((player.vel.x - player.config.max_speed).abs() < 0.01);
    }
}

// 非真实测试 — 需要窗口上下文，归入 Manual: visual-judgment
// fn test_hud_rendered_at_anchor_position() { ... }
```

## UI Testing

本项目包含 **3 个 UI 特性**（F06: GameOver/Victory overlay、F09: HUD、F10: Options menu）。由于游戏为 **Macroquad 原生桌面应用**（非浏览器/WebView），**Chrome DevTools MCP 不适用于本项目的 UI 测试**（See `env-guide.md` §5）。

**UI 验证策略**:

| 阶段 | 方法 | 工具 | 覆盖范围 |
|------|------|------|---------|
| TDD Red/Green | 单元测试覆盖核心逻辑 | `cargo test` | 状态机转换、HUD 数值更新、坐标计算、生命/金币计数 |
| Feature-ST | 手工截图对比（Manual: visual-judgment） | OS 截图工具 + 像素测量 | 分辨率切换、HUD 锚定 (3%,3%) 偏差 ≤ ±2%、GameOver/Victory 画面渲染、像素艺术最近邻缩放 |
| System-ST | 手工全流程走查 + 截图 | OS 截图工具 | 完整通关路径视觉验证、全分辨率 + 全屏模式 |

**UI 特性的 `[devtools]` verification_steps**:
对于 UI 特性，`verification_steps` 中以 `[devtools]` 前缀声明正面视觉存在断言。
由于本项目的 Chrome DevTools MCP 不可用约束，`[devtools]` 步骤在 Feature-ST 阶段转换为人工截图验收单，但语义结构保持一致：
- `EXPECT` → 截图必须包含的元素（文字/图标/锚点位置）
- `REJECT` → 截图不得出现的问题（模糊边缘/偏移/缺失元素）

## Critical Rules

1. **每会话一特性一阶段** —— 不做跨阶段或多特性并行
2. **Feature Design 不可绕过** —— 每特性先设计后 TDD
3. **TDD Red 必须失败** —— 测试必须先红后绿；禁止先写实现再补测试
4. **覆盖率为硬关卡** —— 低于阈值绝不通融；扩测或通过 `long-task-increment` 修订 srs_trace
5. **ST 不可绕过** —— AI 可修的内部修；人类介入的 blocked
6. **ATS 类别必覆盖** —— ST 用例必须覆盖 ATS §2 声明的所有必须类别
7. **status=passing 同步清空 current** —— 两者原子更新；passing 特性不得再被 current 锁住
8. **env-guide.md §3 为命令单一事实源** —— 不在此 Guide 内嵌命令；防双源漂移
9. **SRS/Design/UCD 模糊不得假设** —— 返 blocked 走 Clarification Addendum
10. **绝不提交损坏代码** —— 会话结束时测试必须全绿或未修改
11. **不更新 RELEASE_NOTES 不 Persist** —— 发布说明与代码同步

---

*by long task skill*
