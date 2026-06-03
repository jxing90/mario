// 01-quick-start.rs — Mario 2D Platformer Demo Quick Start
//
// 演示场景：初始化游戏世界、创建玩家和关卡、运行游戏循环、检查状态。
// 覆盖特性：F01 Engine Core, F02 Level & Background, F03 Player Controller, F06 PlayingState
//
// 前置条件：
//   - Rust edition 2024 工具链已安装
//   - 项目根目录下执行：cargo run --example 01-quick-start
//
// 本示例展示外部开发者如何用最少步骤创建一个可运行的游戏世界。

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::level::{Level, AABB, Vec2};
use mario_platformer::states::{LifeState, PlayingState};
use mario_platformer::systems::camera::{Camera, CameraConfig};

fn main() {
    println!("=== Mario Platformer — Quick Start ===\n");

    // ── 第一步：创建关卡几何体 ─────────────────────────────────
    // Level::new() 自动构建包含 3 个平台、3 个尖刺、3 层视差背景的硬编码关卡。
    let level = Level::new();
    let bounds = level.bounds();

    println!("[关卡] 边界:");
    println!("  左边界 min_x = {:.0}", bounds.min_x);
    println!("  右边界 max_x = {:.0}", bounds.max_x);
    println!("  死亡平面 kill_y = {:.0}", bounds.kill_y);
    println!("  平台数量: {}", level.platforms().len());
    println!("  视差背景层数: {}", level.parallax_layers().len());

    // ── 第二步：查询地形 ───────────────────────────────────────
    // 用 AABB 查询某个区域与哪些地形图块重叠。
    // 这在对某个区域进行碰撞检测时很有用（例如：角色所在区域的地形）。
    let query_box = AABB { x: 0.0, y: 580.0, w: 500.0, h: 40.0 };
    let terrain = level.query_terrain(&query_box);
    println!("\n[地形查询] 与查询框重叠的图块数: {}", terrain.len());
    for tile in &terrain {
        match tile {
            mario_platformer::level::Tile::Platform(aabb) => {
                println!("  - 平台 @ ({:.0}, {:.0}) 尺寸 ({:.0} x {:.0})",
                    aabb.x, aabb.y, aabb.w, aabb.h);
            }
            mario_platformer::level::Tile::Spike(aabb) => {
                println!("  - 尖刺 @ ({:.0}, {:.0}) 尺寸 ({:.0} x {:.0})",
                    aabb.x, aabb.y, aabb.w, aabb.h);
            }
            mario_platformer::level::Tile::Empty => {}
        }
    }

    // ── 第三步：创建玩家 ──────────────────────────────────────
    // Player::new() 使用默认的 PlayerConfig（模拟原版马力欧手感）。
    let mut player = Player::new(PlayerConfig::default());
    // 将玩家放置在关卡起点附近的地面上（y=584 贴近地面平台 y=600）。
    player.pos = Vec2 { x: 100.0, y: 584.0 };
    player.on_ground = true;

    println!("\n[玩家] 初始状态:");
    println!("  位置: ({:.0}, {:.0})", player.pos.x, player.pos.y);
    println!("  金币: {}  生命: {}", player.coins, player.lives);
    println!("  形态: {:?}", player.state);
    println!("  碰撞体: ({:.0} x {:.0})", player.collider().w, player.collider().h);

    // ── 第四步：设置摄像机 ────────────────────────────────────
    let camera = Camera::new(CameraConfig::default());
    let (vw, vh) = camera.viewport();
    println!("\n[摄像机] 虚拟画布: {:.0} x {:.0}", vw, vh);

    // ── 第五步：创建游玩状态（PlayingState） ──────────────────
    // PlayingState 将玩家、关卡、摄像机、敌人、检查点、旗杆等连接在一起。
    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);

    println!("\n[游玩状态] 初始化完成:");
    println!("  敌人数: {}", state.enemies.len());
    println!("  检查点数: {}", state.checkpoints.len());
    println!("  旗杆位置: ({:.0}, {:.0})", state.flagpole.pos.x, state.flagpole.pos.y);

    // ── 第六步：运行模拟几帧 ──────────────────────────────────
    // 以固定时间步长 dt=1/60s 运行 120 帧（模拟 2 秒）。
    // 注意：由于没有键盘输入，玩家只会在重力作用下落到平台上。
    const DT: f32 = 1.0 / 60.0;
    println!("\n[模拟] 以固定 dt={:.4}s 运行 120 帧（2 秒）...", DT);
    for i in 0..120 {
        state.update(DT);
        // 每 30 帧报告一次状态
        if (i + 1) % 30 == 0 {
            let stats = state.player.stats();
            println!(
                "  第 {:>3} 帧 — 玩家位置: ({:>7.1}, {:>7.1})  速度: ({:>7.1}, {:>7.1})  金币: {}  生命: {}",
                i + 1,
                state.player.pos.x, state.player.pos.y,
                state.player.vel.x, state.player.vel.y,
                stats.coins, stats.lives);
        }
    }

    // ── 第七步：最终状态检查 ──────────────────────────────────
    let final_stats = state.player.stats();
    let camera_offset = state.camera.offset();
    println!("\n[最终状态]");
    println!("  玩家位置: ({:.0}, {:.0})", state.player.pos.x, state.player.pos.y);
    println!("  玩家着地: {}", state.player.on_ground);
    println!("  金币: {}  生命: {}", final_stats.coins, final_stats.lives);
    println!("  摄像机偏移: ({:.0}, {:.0})", camera_offset.x, camera_offset.y);

    // ── 第八步：检查旗杆是否触发 ─────────────────────────────
    if state.check_flagpole() {
        println!("  [胜利] 玩家已到达旗杆！");
    } else {
        println!("  [进行中] 玩家尚未到达旗杆。");
    }

    println!("\n=== 快速入门演示完成 ===");
    println!("开发者可在此基础上添加键盘输入、渲染逻辑等内容。");
}
