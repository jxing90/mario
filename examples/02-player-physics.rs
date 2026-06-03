// 02-player-physics.rs — 玩家物理调优与状态机
//
// 演示场景：自定义 PlayerConfig，模拟移动/跳跃/冲刺/受击等不同输入组合下的玩家行为。
// 覆盖特性：F03 Player Controller（FR-001 水平移动，FR-002 跳跃，FR-003 冲刺）
//
// 前置条件：
//   - cargo run --example 02-player-physics
//
// 本示例展示如何调节物理参数以打造不同手感，以及玩家的能力状态机如何运作。

use mario_platformer::entities::player::{Player, PlayerConfig, PlayerState};
use mario_platformer::input::InputState;
use mario_platformer::level::{Level, Vec2};

/// 返回一个模拟"按住右箭头"的输入快照。
fn input_hold_right() -> InputState {
    InputState {
        right: true,
        left: false,
        jump: false,
        jump_just: false,
        sprint: false,
        esc_just: false,
        confirm: false,
    }
}

/// 返回一个模拟"按住右箭头 + 空格（跳跃）"的输入快照。
fn input_right_and_jump() -> InputState {
    InputState {
        right: true,
        left: false,
        jump: true,
        jump_just: false,
        sprint: false,
        esc_just: false,
        confirm: false,
    }
}

/// 返回一个模拟"刚按下空格"的输入快照 —— 触发起跳。
fn input_jump_just_pressed() -> InputState {
    InputState {
        right: false,
        left: false,
        jump: true,
        jump_just: true,
        sprint: false,
        esc_just: false,
        confirm: false,
    }
}

/// 返回一个模拟"按住右箭头 + Shift（冲刺）"的输入快照。
fn input_sprint_right() -> InputState {
    InputState {
        right: true,
        left: false,
        jump: false,
        jump_just: false,
        sprint: true,
        esc_just: false,
        confirm: false,
    }
}

/// 零输入（闲置状态）。
fn input_none() -> InputState {
    InputState::default()
}

fn main() {
    println!("=== Mario Platformer — 玩家物理演示 ===\n");

    // ── 1. 默认配置参数 ───────────────────────────────────────
    let config = PlayerConfig::default();
    println!("[PlayerConfig 默认值]");
    println!("  加速度: {:.0} px/s²", config.acceleration);
    println!("  最大速度: {:.0} px/s", config.max_speed);
    println!("  摩擦力: {:.0} px/s²", config.friction);
    println!("  跳跃初速度: {:.0} px/s (负=向上)", config.jump_initial_velocity);
    println!("  最大跳跃时长: {:.2}s", config.max_jump_duration);
    println!("  冲刺倍率: {:.1}x", config.sprint_multiplier);
    println!("  空中操控系数: {:.1}", config.air_control_factor);
    println!("  重力: {:.0} px/s²", config.gravity);

    // ── 2. 自定义配置示例 ─────────────────────────────────────
    // 你可以在创建 Player 时覆盖任意参数来打造专属手感。
    let _custom_config = PlayerConfig {
        max_speed: 300.0,      // 更快！
        jump_initial_velocity: -500.0, // 跳得更高！
        ..PlayerConfig::default()
    };
    println!("\n[自定义配置] max_speed=300, jump_velocity=-500");

    let level = Level::new();
    let dt = 1.0 / 60.0;

    // ── 3. 水平移动演示 ──────────────────────────────────────
    println!("\n── 水平移动：从静止加速到最大速度 ──");
    let mut player = Player::new(config);
    player.pos = Vec2 { x: 100.0, y: 584.0 };
    player.on_ground = true;

    for i in 0..60 {
        let input = input_hold_right();
        let terrain = level.query_terrain(&player.collider());
        player.update(dt, &input, &terrain);
        if i % 10 == 0 || i == 59 {
            println!("  帧 {:>3}: x={:>8.1}  vx={:>8.1}  on_ground={}",
                i, player.pos.x, player.vel.x, player.on_ground);
        }
    }
    println!("  经过 1 秒加速后，玩家速度已达到 ~{:.0} px/s", player.vel.x);

    // ── 4. 摩擦减速演示 ──────────────────────────────────────
    println!("\n── 摩擦减速：松开方向键后滑行停止 ──");
    for i in 0..30 {
        let input = input_none();
        let terrain = level.query_terrain(&player.collider());
        player.update(dt, &input, &terrain);
        println!("  帧 {:>3}: vx={:>8.1}", i, player.vel.x);
        if player.vel.x.abs() < 1.0 {
            println!("  第 {} 帧已基本停止", i);
            break;
        }
    }

    // ── 5. 跳跃演示（轻按 vs 长按） ──────────────────────────
    println!("\n── 可变高度跳跃 ──");

    // 5a: 轻按跳跃（仅触发起跳，不持续按住）
    let mut player_jump = Player::new(config);
    player_jump.pos = Vec2 { x: 100.0, y: 584.0 };
    player_jump.on_ground = true;
    let start_y = player_jump.pos.y;

    // 第一帧：按下跳跃键
    let input_press = input_jump_just_pressed();
    let terrain = level.query_terrain(&player_jump.collider());
    player_jump.update(dt, &input_press, &terrain);

    // 后续帧：松开跳跃键（轻按效果）
    for _i in 1..40 {
        let input = input_none(); // 模拟松开空格键
        let terrain = level.query_terrain(&player_jump.collider());
        player_jump.update(dt, &input, &terrain);
    }
    println!("  轻按跳跃：起点 y={:.0}  最高点 y={:.0}  上升高度={:.0} px",
        start_y, player_jump.pos.y, start_y - player_jump.pos.y);

    // 5b: 长按跳跃（持续按住空格键）
    let mut player_hold = Player::new(config);
    player_hold.pos = Vec2 { x: 100.0, y: 584.0 };
    player_hold.on_ground = true;
    let start_y2 = player_hold.pos.y;

    let input_press2 = input_jump_just_pressed();
    let terrain2 = level.query_terrain(&player_hold.collider());
    player_hold.update(dt, &input_press2, &terrain2);

    // 持续按住空格键
    for _i in 1..40 {
        let input = input_right_and_jump(); // jump=true 模拟长按
        let terrain2 = level.query_terrain(&player_hold.collider());
        player_hold.update(dt, &input, &terrain2);
    }
    println!("  长按跳跃：起点 y={:.0}  最高点 y={:.0}  上升高度={:.0} px",
        start_y2, player_hold.pos.y, start_y2 - player_hold.pos.y);
    println!("  长按比轻按多跳高了约 {:.0} px",
        (start_y2 - player_hold.pos.y) - (start_y - player_jump.pos.y));

    // ── 6. 冲刺演示 ──────────────────────────────────────────
    println!("\n── 冲刺（1.5× 速度） ──");
    let mut player_sprint = Player::new(config);
    player_sprint.pos = Vec2 { x: 100.0, y: 584.0 };
    player_sprint.on_ground = true;

    for _i in 0..30 {
        let input = input_sprint_right();
        let terrain = level.query_terrain(&player_sprint.collider());
        player_sprint.update(dt, &input, &terrain);
    }
    println!("  冲刺 0.5 秒后速度: {:.0} px/s (基础最大值 {} x 1.5 = {:.0})",
        player_sprint.vel.x, config.max_speed, config.max_speed * config.sprint_multiplier);

    // ── 7. 能力状态机演示（Small ↔ Super ↔ Fire） ────────────
    println!("\n── 玩家能力状态机 ──");
    let mut player_power = Player::new(config);
    println!("  初始状态: {:?}", player_power.state);

    // 吃到超级蘑菇
    player_power.apply_powerup(PlayerState::Super);
    println!("  使用蘑菇后: {:?}  碰撞体高度: {:.0}",
        player_power.state, player_power.collider().h);

    // 升级到火焰花
    player_power.apply_powerup(PlayerState::Fire);
    println!("  使用火焰花后: {:?}", player_power.state);

    // 受伤（Super → Small 降级，不死）
    let died = player_power.take_damage();
    assert!(!died, "Super 形态受伤应降级为 Small，不会直接死亡");
    println!("  受伤后: {:?}  碰撞体高度: {:.0}  死亡: {}",
        player_power.state, player_power.collider().h, died);

    // 受伤（Small → 死亡）
    let died = player_power.take_damage();
    assert!(died, "Small 形态受伤应直接死亡");
    println!("  再次受伤（Small）: 死亡 = {}", died);

    println!("\n=== 玩家物理演示完成 ===");
}
