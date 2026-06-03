# Mario 2D Platformer Demo — Examples

Usage examples for external developers and AI Code Agents.

## Prerequisites

- Rust edition 2024 工具链（`rustup` 已安装 `stable` channel）
- 项目依赖：`macroquad = "0.4"`, `image = "0.24"`
- 在项目根目录下执行所有命令

## How to Run

```bash
# 列出所有示例
cargo run --example

# 编译检查（不运行）
cargo check --examples

# 运行指定示例
cargo run --example 01-quick-start
cargo run --example 02-player-physics
cargo run --example 03-game-lifecycle
cargo run --example 04-entity-interaction
cargo run --example 05-quality-toolkit
```

## Examples

| # | Scenario | File | How to run | Covers |
|---|----------|------|------------|--------|
| 1 | Quick Start — 游戏世界初始化 | [01-quick-start.rs](01-quick-start.rs) | `cargo run --example 01-quick-start` | F01 Engine, F02 Level, F03 Player, F06 PlayingState |
| 2 | Player Physics — 玩家物理调优 | [02-player-physics.rs](02-player-physics.rs) | `cargo run --example 02-player-physics` | F03 Player (移动/跳跃/冲刺/受击) |
| 3 | Game Lifecycle — 状态机生命周期 | [03-game-lifecycle.rs](03-game-lifecycle.rs) | `cargo run --example 03-game-lifecycle` | F06 Life/Death/Win, F05 Hazards |
| 4 | Entity Interaction — 实体交互系统 | [04-entity-interaction.rs](04-entity-interaction.rs) | `cargo run --example 04-entity-interaction` | F07 Enemy, F08 Collectibles, F05 Hazards |
| 5 | Quality Toolkit — 质量验证工具 | [05-quality-toolkit.rs](05-quality-toolkit.rs) | `cargo run --example 05-quality-toolkit` | F11 60fps, F12 Multi-Res, F13 Pixel Art |

## Scenario Descriptions

### 1. Quick Start（快速入门）
从零构建游戏世界：创建关卡几何体、查询地形图块、初始化玩家和摄像机、创建 PlayingState、运行固定时间步长模拟循环并观察状态变化。**推荐新开发者首选此示例**。

### 2. Player Physics（玩家物理）
深入 Player 实体的可配置物理参数：自定义 `PlayerConfig` 以调整加速度、最大速度、摩擦力、跳跃初速度和冲刺倍率；模拟不同输入模式下的水平移动、可变高度跳跃（轻按 vs 长按）、冲刺速度以及能力状态机（Small ↔ Super ↔ Fire）转换。

### 3. Game Lifecycle（游戏生命周期）
完整的状态机流转演示：危险检测（尖刺接触判定）、死亡动画 → 检查点复活（lives > 0）或游戏结束（lives = 0）、旗杆触发 → 胜利画面、游戏重置。展示 DeadState、GameOverState、VictoryState 的创建与转换。

### 4. Entity Interaction（实体交互）
展示所有交互实体系统：敌人巡逻与踩踏/侧面碰撞判定、金币收集与重生复位、问号方块激活与战利品表（Coin 70% / SuperMushroom 15% / FireFlower 15%）、能力道具生成与收集、火球飞行与敌人碰撞。

### 5. Quality Toolkit（质量工具包）
展示内置的性能与品质验证工具集：FrameMetrics 帧时间统计（min/max/avg/p99/滑动窗口 FPS）、ResolutionVerifier 多分辨率 HUD 锚定与可见区域校验、SpritePalette 精灵调色板颜色数合规性检查、像素艺术渲染配置（最近邻过滤、坐标舍入）。
