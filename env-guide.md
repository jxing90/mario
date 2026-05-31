---
version: 1.1
approved_by: jxing
approved_date: 2026-06-01
approved_sections: ["§3", "§4"]
---

# env-guide.md —— 环境契约（单一事实源）

> **用户可编辑。** Claude 在以下场景读取本文件：服务启停、构建/测试命令、存量代码库约束。
> 本文件是下游流水线（Worker / TDD / Quality / Feature-ST）的**单一事实源**，任何对 §3 或 §4 的修改都必须经过人工审批（更新本文件头的 `approved_by` / `approved_date` / `approved_sections`）。

## 目录
- §1 服务生命周期
- §2 环境配置
- §3 构建与执行命令
- §4 存量代码库约束
- §5 测试环境依赖
- §6 人工审批记录

---

## §1 服务生命周期

> 启停、重启协议、PID/端口约定。

**No server processes — this is a native desktop application.** The game binary (`mario-platformer.exe`) is the sole runtime process. There are no services, daemons, or servers to manage. Environment activation (Rust toolchain) is covered in §2.

### Services 清单
| Service Name | Port | Start Command | Stop Command | Verify URL |
|---|---|---|---|---|
| _(none)_ | _(none)_ | _(none)_ | _(none)_ | _(none)_ |

### 启动全部服务（输出捕获）
_Not applicable — no server processes._

### 验证服务在运行
_Not applicable — no server processes._

### 停止全部服务（PID 优先，端口 fallback）
_Not applicable — no server processes._

### 验证服务已停止
_Not applicable — no server processes._

### 重启协议（Restart Protocol，4 步）
_Not applicable — no server processes. The game application restarts by re-launching the binary._

---

## §2 环境配置

> 环境变量清单、.env.example 关联、必需 configs。

### 环境激活命令

本项目依赖 Rust 工具链。确认 `rustup` 和 `cargo` 在 PATH 中：

```powershell
# Windows (PowerShell) — 验证 Rust 已安装
rustup --version
cargo --version
rustc --version
```

```bash
# Windows (Git Bash) — 验证 Rust 已安装
rustup --version
cargo --version
rustc --version
```

若未安装 Rust，从 https://rustup.rs 安装，接受默认配置（包括 MSVC 工具链）。

设置默认工具链为 stable（edition 2024 要求）：

```powershell
# 安装/更新至最新 stable
rustup update stable
rustup default stable
```

Macroquad 在 Windows 上依赖 MSVC 构建工具。验证 `link.exe` 可用：

```powershell
where link.exe   # 应在 Visual Studio Build Tools 或 MSVC 路径中
```

若无 MSVC 构建工具，通过 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) 安装，勾选 "Desktop development with C++" 工作负载。

### 必需环境变量

参见 `.env.example`（由 `long-task-init-features` 生成）。本项目为离线桌面游戏，无 `env`-type 配置项；若 `required_configs[]` 为空则无需额外环境变量。

### Config 加载

参见 `scripts/check_configs.py`（由 `long-task-init-features` 生成）。本项目无可配置外部服务；配置项（如游戏物理参数、关卡布局）硬编码于 Rust 常量中。

---

## §3 构建与执行命令

> **下游流水线消费区**。TDD Red/Green、Quality Gate、Feature-ST 通过读取本段获取命令。
> 所有命令使用 **静默执行协议**（输出重定向到 `/tmp/*.log`，仅在失败时提取）。

### 构建命令

```bash
# Git Bash on Windows
cargo build --release > /tmp/build-$$.log 2>&1; echo $? > /tmp/build-$$.exit
# 成功：读 /tmp/build-$$.log 最后 30 行:  tail -30 /tmp/build-$$.log
# 失败：读 /tmp/build-$$.log 最后 100 行:  tail -100 /tmp/build-$$.log | grep -iE 'error|warning'
```

```powershell
# Windows PowerShell
$log = "$env:TEMP\build-$PID.log"; $exit = "$env:TEMP\build-$PID.exit"
cargo build --release *> $log; $LASTEXITCODE | Out-File -FilePath $exit
# 成功：Get-Content $log -Tail 30
# 失败：Get-Content $log -Tail 100 | Select-String -Pattern 'error|warning' -CaseSensitive:$false
```

**构建产物**：`target/release/mario-platformer.exe`

### 单元测试命令

```bash
# Git Bash on Windows
cargo test > /tmp/ut-$$.log 2>&1; echo $? > /tmp/ut-$$.exit
# 成功：无需读日志
# 失败：tail -100 /tmp/ut-$$.log | grep -iE 'FAIL|ERROR|test result:'
```

```powershell
# Windows PowerShell
$log = "$env:TEMP\ut-$PID.log"; $exit = "$env:TEMP\ut-$PID.exit"
cargo test *> $log; $LASTEXITCODE | Out-File -FilePath $exit
# 成功：无需读日志
# 失败：Get-Content $log -Tail 100 | Select-String -Pattern 'FAIL|ERROR|test result:' -CaseSensitive:$false
```

### 覆盖率命令

覆盖率使用 `cargo-tarpaulin`。首次使用需安装：

```bash
cargo install cargo-tarpaulin
```

```bash
# Git Bash on Windows — 运行覆盖率并捕获输出
cargo tarpaulin --out Xml --output-dir target/tarpaulin > /tmp/cov-$$.log 2>&1; echo $? > /tmp/cov-$$.exit
# 提取 line/branch 覆盖率百分比:  grep -oP '(\d+\.\d+)% coverage' /tmp/cov-$$.log
```

```powershell
# Windows PowerShell
$log = "$env:TEMP\cov-$PID.log"; $exit = "$env:TEMP\cov-$PID.exit"
cargo tarpaulin --out Xml --output-dir target/tarpaulin *> $log; $LASTEXITCODE | Out-File -FilePath $exit
```

**覆盖率门禁**（来自 `feature-list.json`）：
- 行覆盖率 ≥ 80%
- 分支覆盖率 ≥ 70%

覆盖率 XML 报告写入 `target/tarpaulin/tarpaulin-report.xml`。

### 静态分析命令

```bash
# Git Bash on Windows
cargo clippy -- -D warnings > /tmp/static-$$.log 2>&1; echo $? > /tmp/static-$$.exit
# 失败：grep -E '(error|warning)\[' /tmp/static-$$.log
```

```powershell
# Windows PowerShell
$log = "$env:TEMP\static-$PID.log"; $exit = "$env:TEMP\static-$PID.exit"
cargo clippy -- -D warnings *> $log; $LASTEXITCODE | Out-File -FilePath $exit
# 失败：Get-Content $log | Select-String -Pattern '(error|warning)\[' -CaseSensitive:$false
```

`clippy` 随 Rust 工具链附带，无需单独安装。`-D warnings` 将 clippy 警告升级为错误，确保 CI 级严格性。

### Re-check 协议
- 任何命令失败 → 修复后**仅重跑失败的测试/步骤**（by name），不整轮重跑
- 运行单个测试：`cargo test <test_name>`
- 运行单个模块测试：`cargo test <module_name>::`
- 临时文件清理（Bash）：`trap 'rm -f /tmp/*-$$.log /tmp/*-$$.exit' EXIT`
- 临时文件清理（PowerShell）：`Remove-Item "$env:TEMP\build-$PID.log", "$env:TEMP\build-$PID.exit" -ErrorAction SilentlyContinue`

### 工具/环境故障 Fallback
命令本身异常退出（如 `cargo: command not found` / `link.exe not found` / 依赖下载超时）：
1. 诊断根因（Rust 工具链未装 / MSVC 构建工具缺失 / 网络不可用）
2. 视情况跑 `init.ps1` 或按 §2 验证 Rust 工具链
3. 重试一次仍失败 → SubAgent 返回 `status: blocked`，evidence 前缀 `[ENV-ERROR]` 附故障摘要；**绝不跳过**测试继续推进

### 工具版本锁定

| 工具 | 最低版本 | 说明 |
|------|---------|------|
| Rust (rustc/rustup) | ≥ 1.85.0 | Edition 2024 需 Rust 1.85+；`rustup update stable` 安装最新 |
| cargo | (随 Rust 附带) | 构建系统与包管理器 |
| cargo-tarpaulin | ≥ 0.30.0 | 代码覆盖率工具；`cargo install cargo-tarpaulin` |
| cargo-clippy | (随 Rust 附带) | 静态分析 linter |
| MSVC Build Tools | Visual Studio 2022 | Macroquad 在 Windows 上的 C 依赖编译所需 |

---

## §4 存量代码库约束

> **下游流水线消费区**（单一事实源）。Feature Design、TDD、Worker 的新代码必须遵守以下约束。
> **数据源**：`docs/rules/*.md`（由 `codebase-scanner` 扫描填充）。init 阶段直接从 `docs/rules/` 提取关键约束投影到此处；设计文档**不再**镜像这些约束。
> 本段变更必须经人工审批（见 §6）。

### §4.1 强制内部库
| 场景 | 必须使用 | 禁止重新实现 |
|---|---|---|
| _(empty — greenfield project)_ | | |

### §4.2 禁用 API
| API / 模式 | 禁用理由 | 替代方案 |
|---|---|---|
| _(empty — greenfield project)_ | | |

### §4.3 代码风格基线
- 命名约定：_(待填充 — greenfield project)_
- 文件布局：_(待填充 — greenfield project)_
- 错误处理模式：_(待填充 — greenfield project)_

### §4.4 构建系统约定
- 构建产物目录：`target/`（Cargo 默认）
- 忽略清单参考：`.gitignore`（Cargo 生成）
- 依赖锁文件：`Cargo.lock`
- 项目清单：`Cargo.toml`（edition = "2024"）

---

## §5 测试环境依赖

> 数据库、消息队列、第三方服务的本地替身配置。

### 数据库
_None — this project has no database dependency. All game state is ephemeral in-memory Rust structs._

### 消息队列
_None — this project has no message queue dependency._

### 第三方服务
_None — this project is an offline desktop application with zero network dependencies (CON-004). No external API calls, no web services._

### UI 功能测试
本项目包含图形用户界面（2D 游戏渲染），但为**原生桌面应用**（Macroquad + wgpu/OpenGL 渲染），不属于浏览器或 WebView 类 UI。Chrome DevTools MCP 不适用于本项目的 UI 测试场景。

UI 验证策略：
- **单元测试**（`cargo test`）：覆盖核心游戏逻辑（物理计算、碰撞检测、状态机转换、HUD 数值更新），不依赖实际窗口/渲染上下文
- **手动验收**：视觉质量验收（分辨率切换、像素艺术最近邻缩放、HUD 锚定位置）通过人工截屏对比执行，对应 FR-017 / NFR-002 / NFR-003

### 替身配置
_None — no mock servers, WireMock, MockServer, or testcontainers are needed. All game entities interact in-process within the same Rust binary._

---

## §6 人工审批记录

> **任何对 §3 或 §4 的修改必须经过人工审批**。Worker Step 0 读取本段 frontmatter 决定是否阻断启动。

### 审批流程
1. 开发/AI 修改 §3 或 §4
2. 用户审阅 diff
3. 用户更新本文件头 YAML frontmatter：
   ```yaml
   ---
   version: <bump>
   approved_by: <user-handle>
   approved_date: <YYYY-MM-DD>
   approved_sections: ["§3", "§4"]  # 或全部
   ---
   ```
4. `python scripts/validate_env_guide.py env-guide.md --strict` 通过 → Worker 可启动

### 首次生成豁免
由 `long-task-init` 首次生成时，`approved_by: null` 表示豁免状态；下次修改 §3/§4 时必须审批。

### 历史记录
| 日期 | 版本 | 审批人 | 变更摘要 |
|---|---|---|---|
| 2026-05-31 | 1.0 | (null) | 初始生成 — Mario 2D Platformer Demo 环境契约。Rust edition 2024 + Macroquad 0.4 技术栈；Windows 原生桌面应用；无服务/数据库/外部依赖；greenfield 项目 §4 为占位状态 |

---

*by long task skill*
