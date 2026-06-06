// 04-entity-interaction.rs — 实体交互系统
//
// 演示场景：敌人巡逻与踩踏判定、金币收集、问号方块激活与战利品表、
//           能力道具（蘑菇/火焰花）收集、火球与敌人碰撞。
// 覆盖特性：F07 Patrol Enemy（FR-010 踩踏敌人）,
//           F08 Collectibles & Blocks（FR-008 金币, FR-011 问号方块）,
//           F05 Hazards（FR-009 危险检测）
//
// 前置条件：
//   - cargo run --example 04-entity-interaction
//
// 本示例展示所有交互实体的创建、模拟更新与碰撞判定管线。

use mario_platformer::entities::coin::Coin;
use mario_platformer::entities::enemy::{Enemy, EnemyConfig};
use mario_platformer::entities::player::{Player, PlayerConfig, PlayerState};
use mario_platformer::entities::question_block::QuestionBlock;
use mario_platformer::entities::power_up::{PowerUp, PowerUpKind};
use mario_platformer::entities::fireball::Fireball;
use mario_platformer::entities::loot_table::LootTable;
use mario_platformer::level::{Level, Vec2};
use mario_platformer::systems::physics::{Physics, CollisionEvent};

fn main() {
    println!("=== Mario Platformer — 实体交互演示 ===\n");

    let dt = 1.0 / 60.0;
    let _level = Level::new();

    // ── 1. 敌人巡逻系统 ───────────────────────────────────────
    println!("── 1. 敌人巡逻系统 ──");

    let enemy_config = EnemyConfig::default();
    println!("   EnemyConfig: speed={:.0} px/s  bounce_velocity={:.0} px/s",
        enemy_config.speed, enemy_config.bounce_velocity);

    // 创建一只在地面上巡逻的敌人
    let mut enemy = Enemy::new(
        Vec2 { x: 400.0, y: 584.0 },                 // 初始位置（地面）
        Vec2 { x: 350.0, y: 584.0 },                 // 路点 A（左）
        Vec2 { x: 550.0, y: 584.0 },                 // 路点 B（右）
        enemy_config,
    );
    println!("\n   敌人初始位置: ({:.0}, {:.0})  存活: {}",
        enemy.pos.x, enemy.pos.y, enemy.alive);

    // 模拟巡逻几个周期
    for sec in 1..=5 {
        for _ in 0..60 {
            enemy.update(dt, &[]);
        }
        println!("   {:.0}s 后: 位置=({:.0}, {:.0})  速度=({:.0}, {:.0})",
            sec as f32, enemy.pos.x, enemy.pos.y, enemy.vel.x, enemy.vel.y);
    }

    // ── 2. 踩踏判定（敌人碰撞检测） ───────────────────────────
    println!("\n── 2. 踩踏 vs 侧面碰撞判定 ──");

    // 2a: 踩踏判定 —— 玩家从上方掉落，速度向下
    let mut player_stomp = Player::new(PlayerConfig::default());
    player_stomp.pos = Vec2 {
        x: enemy.pos.x,
        y: enemy.pos.y - 20.0, // 在敌人上方 20px
    };
    player_stomp.vel.y = 200.0; // 向下速度 > 0 = 踩踏条件

    let enemies = vec![
        Enemy::new(
            Vec2 { x: 400.0, y: 584.0 },
            Vec2 { x: 350.0, y: 584.0 },
            Vec2 { x: 550.0, y: 584.0 },
            EnemyConfig::default(),
        ),
    ];
    let stomp_events = Physics::enemy_check(&player_stomp, &enemies, dt);
    for event in &stomp_events {
        match event {
            CollisionEvent::EnemyStomp(i) => {
                println!("   [踩踏成功] 玩家踩到了敌人 #{} —— 敌人被消灭，玩家弹起", i);
            }
            CollisionEvent::EnemyContact(i) => {
                println!("   [侧面碰撞] 玩家撞到了敌人 #{} —— 玩家死亡！", i);
            }
            _ => {}
        }
    }

    // 2b: 侧面碰撞判定 —— 玩家在敌人侧面
    let mut player_side = Player::new(PlayerConfig::default());
    player_side.pos = Vec2 {
        x: enemy.pos.x + 20.0, // 在敌人右侧
        y: enemy.pos.y,
    };
    player_side.vel.y = 0.0; // 无垂直速度

    let side_events = Physics::enemy_check(&player_side, &enemies, dt);
    for event in &side_events {
        match event {
            CollisionEvent::EnemyStomp(_) => println!("   [踩踏] 不应发生"),
            CollisionEvent::EnemyContact(i) => {
                println!("   [侧面接触] 玩家从侧面接触到敌人 #{} —— 触发死亡", i);
            }
            _ => {}
        }
    }

    // ── 3. 金币收集 ───────────────────────────────────────────
    println!("\n── 3. 金币收集系统 ──");

    let mut coins = vec![
        Coin::new(Vec2 { x: 200.0, y: 500.0 }),
        Coin::new(Vec2 { x: 250.0, y: 500.0 }),
        Coin::new(Vec2 { x: 300.0, y: 450.0 }),
    ];
    println!("   创建了 {} 枚金币", coins.len());

    let mut player_coin = Player::new(PlayerConfig::default());
    player_coin.pos = Vec2 { x: 200.0, y: 500.0 };

    let coin_events = Physics::coin_check(&player_coin, &coins);
    println!("   玩家在金币位置处 —— 收集事件数: {}", coin_events.len());
    for event in &coin_events {
        if let CollisionEvent::CoinCollect(i) = event {
            println!("     - 收集了金币 #{} @ ({:.0}, {:.0})",
                i, coins[*i].pos.x, coins[*i].pos.y);
            coins[*i].collected = true;
            player_coin.coins += 1;
        }
    }
    println!("   玩家金币计数: {}  (预期 1)", player_coin.coins);

    // 再次检测同一位置 —— 已收集金币不会再次触发
    let recheck = Physics::coin_check(&player_coin, &coins);
    println!("   再次检测已收集金币 —— 事件数: {} (预期 0)", recheck.len());

    // 重置金币（模拟死亡后重生）
    for coin in &mut coins {
        coin.reset();
    }
    println!("   死亡重生后 —— 金币已重置");

    // ── 4. 问号方块与战利品表 ─────────────────────────────────
    println!("\n── 4. 问号方块 & 战利品表 ──");

    let mut qblock = QuestionBlock::new(Vec2 { x: 500.0, y: 300.0 });
    println!("   问号方块位置: ({:.0}, {:.0})  used={}",
        qblock.pos.x, qblock.pos.y, qblock.used);

    // 战利品表概率分布
    println!("\n   [战利品表] Coin=70%  SuperMushroom=15%  FireFlower=15%");

    // 使用不同随机值演示所有可能结果
    let test_values = [0.00, 0.50, 0.70, 0.80, 0.85, 1.00];
    for val in &test_values {
        let reward = LootTable::roll_with_value(*val);
        println!("     roll({:.2}) → {:?}", val, reward);
    }

    // 激活方块（使用固定值 0.5 → Coin）
    let reward = qblock.activate();
    println!("\n   激活方块（roll=0.5）: 产出 {:?}  used={}", reward, qblock.used);

    // ── 5. 能力道具生成 ───────────────────────────────────────
    println!("\n── 5. 能力道具系统 ──");

    // 超级蘑菇：从方块上方生成，带弹跳初始速度
    let mut mushroom = PowerUp::new(
        PowerUpKind::SuperMushroom,
        Vec2 { x: 500.0, y: 240.0 }, // 在方块上方
    );
    println!("   超级蘑菇: pos=({:.0}, {:.0})  vel=({:.0}, {:.0})",
        mushroom.pos.x, mushroom.pos.y, mushroom.vel.x, mushroom.vel.y);

    // 火焰花：静止在生成位置
    let flower = PowerUp::new(
        PowerUpKind::FireFlower,
        Vec2 { x: 500.0, y: 240.0 },
    );
    println!("   火焰花:   pos=({:.0}, {:.0})  vel=({:.0}, {:.0})",
        flower.pos.x, flower.pos.y, flower.vel.x, flower.vel.y);

    // 模拟蘑菇弹跳几帧（无地形时仅受重力下落）
    for _i in 0..30 {
        mushroom.update(dt, &[]);
    }
    println!("\n   蘑菇 0.5s 后: pos=({:.0}, {:.0})  vel=({:.0}, {:.0})",
        mushroom.pos.x, mushroom.pos.y, mushroom.vel.x, mushroom.vel.y);

    // ── 6. 能力道具收集 ───────────────────────────────────────
    println!("\n── 6. 能力道具收集 ──");

    let mut player_powerup = Player::new(PlayerConfig::default());
    player_powerup.pos = Vec2 { x: 500.0, y: 500.0 };

    let power_ups = vec![
        PowerUp::new(PowerUpKind::SuperMushroom, Vec2 { x: 500.0, y: 500.0 }),
    ];

    let pu_events = Physics::powerup_check(&player_powerup, &power_ups);
    for event in &pu_events {
        if let CollisionEvent::PowerUpCollect(i) = event {
            println!("   收集了 {:?} #{}!", power_ups[*i].kind, i);
            player_powerup.apply_powerup(PlayerState::Super);
        }
    }
    println!("   玩家状态: {:?}  碰撞体高度: {:.0}",
        player_powerup.state, player_powerup.collider().h);

    // ── 7. 火球系统 ───────────────────────────────────────────
    println!("\n── 7. 火球系统 ──");

    // 创建一枚向右飞行的火球
    let mut fireball = Fireball::new(
        Vec2 { x: 600.0, y: 500.0 },
        1, // facing = 1 (向右)
    );
    println!("   创建火球: pos=({:.0}, {:.0})  vel=({:.0}, {:.0})  alive={}",
        fireball.pos.x, fireball.pos.y, fireball.vel.x, fireball.vel.y, fireball.alive);

    // 火球飞行并最终超时
    for sec in 0..3 {
        for _ in 0..60 {
            fireball.update(dt);
        }
        println!("   {:.0}s 后: alive={}  timer={:.2}",
            sec as f32, fireball.alive, fireball.timer);
    }

    // ── 8. 火球与敌人碰撞 ────────────────────────────────────
    println!("\n── 8. 火球击中敌人 ──");

    // 重新创建火球和敌人，让它们重叠
    let fireballs = vec![
        Fireball::new(Vec2 { x: 450.0, y: 584.0 }, 1),
    ];
    let enemy_targets = vec![
        Enemy::new(
            Vec2 { x: 450.0, y: 584.0 },
            Vec2 { x: 400.0, y: 584.0 },
            Vec2 { x: 500.0, y: 584.0 },
            EnemyConfig::default(),
        ),
    ];

    let fb_events = Physics::fireball_enemy_check(&fireballs, &enemy_targets);
    for event in &fb_events {
        if let CollisionEvent::FireballHitEnemy(fi, ei) = event {
            println!("   火球 #{} 击中了敌人 #{}！敌人被消灭。", fi, ei);
        }
    }

    // ── 9. 关卡实体概览 ──────────────────────────────────────
    println!("\n── 9. 关卡内实体概览 ──");

    let state = mario_platformer::states::PlayingState::new(
        Player::new(PlayerConfig::default()),
        mario_platformer::states::LifeState::new(),
    );
    println!("   平台数: {}", state.level.platforms().len());
    println!("   敌方数: {}", state.enemies.len());
    println!("   检查点数: {}", state.checkpoints.len());
    println!("   旗杆: ({:.0}, {:.0})  phase={:?}",
        state.flagpole.pos.x, state.flagpole.pos.y, state.flagpole.phase);
    // 地形查询也会返回尖刺
    let full_terrain = state.level.query_terrain(
        &mario_platformer::level::AABB { x: 0.0, y: 0.0, w: 2000.0, h: 1000.0 }
    );
    let spike_count = full_terrain.iter()
        .filter(|t| matches!(t, mario_platformer::level::Tile::Spike(_)))
        .count();
    println!("   尖刺数: {}", spike_count);

    println!("\n=== 实体交互演示完成 ===");
}
