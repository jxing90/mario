// 03-game-lifecycle.rs — 游戏状态机生命周期
//
// 演示场景：游玩 → 死亡 → 重生 → 游戏结束 → 到达旗杆胜利
// 覆盖特性：F06 Life, Death & Win（FR-014a 死亡触发, FR-014b 重生+无敌,
//           FR-014c 游戏结束, FR-015 胜利条件）, F05 Hazards（FR-009 危险检测）
//
// 前置条件：
//   - cargo run --example 03-game-lifecycle
//
// 本示例展示一局完整游戏中的状态流转：触发危险 → 死亡动画 → 复活或游戏结束
// → 到达旗杆 → 胜利画面。

use mario_platformer::entities::player::{Player, PlayerConfig};
use mario_platformer::entities::flagpole::FlagpolePhase;
use mario_platformer::level::Vec2;
use mario_platformer::state::StateMachine;
use mario_platformer::states::{LifeState, PlayingState, DeadState, VictoryState, GameState};
use mario_platformer::systems::physics::Physics;

fn main() {
    println!("=== Mario Platformer — 游戏生命周期演示 ===\n");

    let dt = 1.0 / 60.0;

    // ── 第一部分：初始化 ──────────────────────────────────────
    println!("── 1. 游戏开始 ──");
    let mut player = Player::new(PlayerConfig::default());
    player.pos = Vec2 { x: 100.0, y: 584.0 };
    player.on_ground = true;

    let life_state = LifeState::new();
    let mut state = PlayingState::new(player, life_state);
    println!("   初始生命: {}  初始金币: {}",
        state.player.lives, state.player.coins);
    println!("   旗杆状态: {:?}  (x={:.0}, y={:.0})",
        state.flagpole.phase, state.flagpole.pos.x, state.flagpole.pos.y);

    // ── 第二部分：尖刺危险检测 ───────────────────────────────
    println!("\n── 2. 危险检测（尖刺接触） ──");

    // 扮演状态内部在每帧调用 Physics::hazard_check。
    // 这里演示直接调用 Physics API 检查危险。
    let test_player = Player::new(PlayerConfig::default());
    let terrain = state.level.query_terrain(&test_player.collider());
    let kill_y = state.level.bounds().kill_y;

    let hazard_events = Physics::hazard_check(&test_player, &terrain, kill_y);
    println!("   玩家在 ({:.0}, {:.0}) 处 —— 危险事件数: {}",
        test_player.pos.x, test_player.pos.y, hazard_events.len());

    // 模拟玩家在尖刺位置（300, 596）
    let mut spike_player = Player::new(PlayerConfig::default());
    spike_player.pos = Vec2 { x: 300.0, y: 596.0 };
    let spike_terrain = state.level.query_terrain(&spike_player.collider());
    let spike_events = Physics::hazard_check(&spike_player, &spike_terrain, kill_y);
    println!("   玩家在尖刺位置 ({:.0}, {:.0}) —— 危险事件数: {}",
        spike_player.pos.x, spike_player.pos.y, spike_events.len());
    for event in &spike_events {
        println!("     - 事件: {:?}", event);
    }

    // ── 第三部分：死亡 → 复活流程 ─────────────────────────────
    println!("\n── 3. 死亡 → 复活流程 ──");

    // 模拟玩家在敌人侧面碰撞后死亡
    let player_pos = Vec2 { x: 300.0, y: 500.0 };
    let dead = DeadState::new(
        2,          // lives: 死亡后剩余 2 条命（从 3 减 1）
        10,         // coins: 保留金币数
        1,          // current_level
        None,       // checkpoint: 未激活检查点（从关卡起点重生）
        player_pos, // 死亡位置
        1280.0,     // screen_w
        720.0,      // screen_h
    );
    println!("   DeadState 创建: lives={} coins={} death_timer={:.2}",
        dead.lives, dead.coins, dead.death_timer);

    // 用 GameState 包装 DeadState
    let mut game_state = GameState::Dead(dead);

    // 模拟死亡动画完成（运行足够帧数让计时器归零）
    for _ in 0..100 {
        game_state.update(dt);
    }

    // 死亡动画结束后应自动转为 Playing（lives > 0）或 GameOver（lives == 0）
    match &game_state {
        GameState::Playing(playing) => {
            println!("   [重生] 死亡动画完成 —— 已重生");
            println!("     位置: ({:.0}, {:.0})  生命: {}  金币: {}  无敌时间: {:.2}s",
                playing.player.pos.x, playing.player.pos.y,
                playing.player.lives, playing.player.coins,
                playing.invuln_timer);
        }
        GameState::GameOver(govr) => {
            println!("   [游戏结束] lives=0 —— 已显示 Game Over 画面");
            println!("     最终金币: {}", govr.coins);
        }
        _ => println!("   [意外] 状态: {:?}", std::mem::discriminant(&game_state)),
    }

    // ── 第四部分：游戏结束流程（3 次死亡后） ──────────────────
    println!("\n── 4. 游戏结束流程（生命归零） ──");

    let dead_govr = DeadState::new(
        0,    // lives: 死亡后生命为 0
        15,   // coins
        1,    // current_level
        None, // checkpoint
        Vec2 { x: 500.0, y: 500.0 },
        1280.0, 720.0,
    );
    let mut govr_state = GameState::Dead(dead_govr);

    for _ in 0..100 {
        govr_state.update(dt);
    }

    match &govr_state {
        GameState::GameOver(govr) => {
            println!("   [Game Over] 画面已显示");
            println!("     最终金币数: {}", govr.coins);
            println!("     提示: 'Press Space to Restart'");
            // 空格键重置游戏
            govr_state.full_reset();
        }
        GameState::Playing(_) => {
            println!("   [重生] lives > 0，不应该出现此情况");
        }
        _ => {}
    }

    // 重置后检查状态
    match &govr_state {
        GameState::Playing(playing) => {
            println!("   [重置完成] 新游戏开始");
            println!("     生命: {}  金币: {}  位置: ({:.0}, {:.0})",
                playing.player.lives, playing.player.coins,
                playing.player.pos.x, playing.player.pos.y);
        }
        _ => println!("   重置后状态异常"),
    }

    // ── 第五部分：到达旗杆 → 胜利 ─────────────────────────────
    println!("\n── 5. 到达旗杆 · 胜利流程 ──");

    let mut player_win = Player::new(PlayerConfig::default());
    player_win.pos = Vec2 { x: 100.0, y: 584.0 };
    player_win.coins = 42; // 模拟收集了 42 枚金币
    player_win.on_ground = true;

    let life_state_win = LifeState::new();
    let mut playing = PlayingState::new(player_win, life_state_win);

    // 将玩家传送到旗杆位置
    playing.player.pos = Vec2 {
        x: playing.flagpole.pos.x,
        y: playing.flagpole.pos.y,
    };

    // 运行一帧触发旗杆检测
    playing.update(dt);

    // 查看旗杆状态
    println!("   旗杆碰撞检测结果: {}", playing.check_flagpole());
    println!("   旗杆状态: {:?}", playing.flagpole.phase);
    println!("   金币总数: {}", playing.player.coins);

    // 模拟旗杆动画完成 → VictoryState
    playing.flagpole.phase = FlagpolePhase::Done;
    if playing.flagpole.phase == FlagpolePhase::Done {
        let victory = VictoryState::new(playing.player.coins, playing.current_level);
        println!("\n   [Victory!] 胜利画面");
        println!("     关卡: 1-{}  收集金币数: {}", victory.level, victory.coins);
        println!("     提示: 'Press Space to Play Again'");
    }

    // ── 第六部分：检查点激活演示 ──────────────────────────────
    println!("\n── 6. 检查点系统 ──");

    // 检查关卡中预设的检查点
    for (i, cp) in state.checkpoints.iter().enumerate() {
        println!("   检查点 {}: ({:.0}, {:.0})  activated={}",
            i, cp.pos.x, cp.pos.y, cp.activated);
    }

    // 玩家走到检查点位置
    state.player.pos = Vec2 { x: 500.0, y: 400.0 };
    state.update(dt); // 运行一帧触发检查点检测
    println!("   玩家到达检查点后 — activated 状态已更新。");

    println!("\n=== 游戏生命周期演示完成 ===");
}
