// 05-quality-toolkit.rs — 性能与质量验证工具
//
// 演示场景：帧率性能测量、多分辨率显示校验、精灵调色板合规性、像素艺术渲染配置。
// 覆盖特性：F11 60fps 帧率（NFR-001）, F12 多分辨率显示（NFR-002）,
//           F13 像素艺术渲染（NFR-003）
//
// 前置条件：
//   - cargo run --example 05-quality-toolkit
//   - 此示例为纯计算逻辑，可在无 GPU 环境下运行
//
// 本示例展示项目内置的质量保证工具：帧率统计、分辨率自检、精灵颜色数校验。

use mario_platformer::engine::SUPPORTED_RESOLUTIONS;
use mario_platformer::metrics::FrameMetrics;
use mario_platformer::verification::ResolutionVerifier;
use mario_platformer::systems::camera::CameraConfig;
use mario_platformer::systems::hud::HudRenderer;
use mario_platformer::palette::{self, SpritePalette};

fn main() {
    println!("=== Mario Platformer — 质量工具包 ===\n");

    // ── 1. 帧率性能测量（NFR-001） ────────────────────────────
    println!("── 1. 帧率性能测量（NFR-001: 60fps） ──");

    let mut metrics = FrameMetrics::new();
    println!("   [初始化] frame_count={}  min={:.3}ms  max={:.3}ms",
        metrics.frame_count,
        metrics.min_frame_time * 1000.0,
        metrics.max_frame_time * 1000.0);

    // 模拟 180 帧（3 秒 @ 60fps）的帧时间采样
    // 大部分帧在 ~16.67ms，少数帧因负载波动略高
    let simulated_times: Vec<f32> = (0..180).map(|i| {
        if i % 60 == 0 {
            // 每 60 帧一次短暂尖峰（GC/IO 模拟）
            0.020
        } else if i % 10 == 0 {
            0.017
        } else {
            // 正常帧：约 1/60 ≈ 0.01667s
            1.0 / 60.0
        }
    }).collect();

    for ft in &simulated_times {
        metrics.sample(*ft);
    }

    // 获取聚合统计
    let stats = metrics.stats();
    println!("\n   [180 帧统计]");
    println!("     最小帧时间: {:.3} ms", stats.min * 1000.0);
    println!("     最大帧时间: {:.3} ms", stats.max * 1000.0);
    println!("     平均帧时间: {:.3} ms", stats.avg * 1000.0);
    println!("     P99 帧时间: {:.3} ms  (NFR-001 要求 ≤ 16.67ms)", stats.p99 * 1000.0);
    println!("     滑动窗口 FPS: {:.1}", stats.window_fps);
    println!("     总帧数: {}", stats.frame_count);

    // 检查 P99 是否满足 60fps 要求
    let meets_60fps = stats.p99 <= 1.0 / 60.0 + 0.001; // 加小容差
    println!("\n   [判定] P99 帧时间 {}ms {} NFR-001 要求 (≤ 16.67ms)",
        format!("{:.3}", stats.p99 * 1000.0),
        if meets_60fps { "[通过]" } else { "[未通过]" });

    // 打印 FPS 日志（模拟引擎自动输出）
    println!("\n   [FPS 日志输出示例]");
    metrics.log_report();

    // ── 2. 多分辨率显示校验（NFR-002） ────────────────────────
    println!("\n── 2. 多分辨率显示校验（NFR-002） ──");

    println!("   支持的分辨率:");
    for &(w, h) in &SUPPORTED_RESOLUTIONS {
        println!("     - {} x {} ({}p)", w, h, h);
    }

    // 运行全部分辨率的验证套件
    let camera_config = CameraConfig::default();
    let reports = ResolutionVerifier::verify_all_resolutions(&camera_config);

    println!("\n   [全分辨率验证报告]");
    for report in &reports {
        let passed_mark = if report.passed { "[通过]" } else { "[未通过]" };
        println!("\n   {}  {}:", report.label, passed_mark);
        println!("     HUD 锚点: 预期 ({:.1}, {:.1})  偏差 ({:.4}, {:.4})  {}",
            report.hud.expected.0, report.hud.expected.1,
            report.hud.deviation.dx_pct, report.hud.deviation.dy_pct,
            if report.hud.passed { "[通过]" } else { "[未通过]" });
        println!("     可见区域比: {:.4}  (要求 45%–50%)  {}",
            report.visible_area.ratio,
            if report.visible_area.passed { "[通过]" } else { "[未通过]" });
    }

    // HUD 锚定一致性检查
    println!("\n   [HUD 锚定一致性]");
    for &(w, h) in &SUPPORTED_RESOLUTIONS {
        let (hx, hy) = HudRenderer::compute_anchor(w as f32, h as f32);
        let (vx, vy) = ResolutionVerifier::expected_hud_anchor(w as f32, h as f32);
        let match_x = (hx - vx).abs() < 0.01;
        let match_y = (hy - vy).abs() < 0.01;
        println!("     {}x{}: HUD=({:.1},{:.1})  Verifier=({:.1},{:.1})  {}",
            w, h, hx, hy, vx, vy,
            if match_x && match_y { "[一致]" } else { "[不一致!]" });
    }

    // 可见区域比计算演示
    println!("\n   [可见区域比计算]");
    // 玩家在世界坐标 x=200, 摄像机偏移 x=20, 虚拟画布宽度 480
    let ratio = ResolutionVerifier::player_visible_ratio(200.0, 20.0, 480.0);
    println!("     player_x=200  cam_offset=20  viewport_w=480");
    println!("     player_screen_x = {}  (200 - 20)", 200.0 - 20.0);
    println!("     可见区域比 = (480 - 180) / 480 = {:.3}  (62.5% ahead)", ratio);
    println!("     设计目标: 1.0 - player_target_x_pct = 1.0 - 0.375 = 0.625");
    println!("     匹配: {}", (ratio - 0.625).abs() < 0.001);

    // ── 3. 精灵调色板校验（NFR-003） ──────────────────────────
    println!("\n── 3. 精灵调色板校验（NFR-003: 像素艺术） ──");

    // 精灵颜色数检查
    println!("   [所有精灵调色板校验]");
    let sprite_reports = SpritePalette::verify_all();

    if sprite_reports.is_empty() {
        println!("     (无嵌入精灵)");
    } else {
        for report in &sprite_reports {
            let passed_mark = if report.passed { "[通过]" } else { "[未通过]" };
            println!("     {}: {} 色  {}",
                report.name, report.color_count, passed_mark);
            if let Some(ref err) = report.error {
                println!("       错误: {}", err);
            }
        }
    }

    // 计数验证示例：使用程序化生成的测试数据
    println!("\n   [程序化调色板验证]");
    // 创建一个 1×1 白色像素的 PNG 来验证 count_colors 函数
    let test_png = make_1x1_white_png();
    match SpritePalette::count_colors(&test_png) {
        Ok(count) => println!("     1×1 白色 PNG: {} 色 (预期 1)", count),
        Err(e) => println!("     解码失败: {:?}", e),
    }

    // ── 4. 像素艺术渲染配置 ──────────────────────────────────
    println!("\n── 4. 像素艺术渲染配置 ──");

    // 应用最近邻过滤
    palette::apply_pixel_art_filter();
    let filter_applied = palette::FILTER_APPLIED.load(std::sync::atomic::Ordering::Acquire);
    println!("   最近邻过滤器: {}",
        if filter_applied { "已启用 [Nearest]" } else { "未启用" });

    // 坐标舍入演示（像素对齐）
    println!("\n   [精灵坐标舍入（最近邻像素对齐）]");
    let test_coords = [
        macroquad::math::Vec2::new(10.3, 5.7),
        macroquad::math::Vec2::new(2.5, 3.5),   // 银行家舍入：2.5→2.0, 3.5→4.0
        macroquad::math::Vec2::new(-3.7, -3.2),
        macroquad::math::Vec2::new(100.0, 200.0), // 已经是整数
    ];

    for (i, pos) in test_coords.iter().enumerate() {
        let rounded = palette::round_sprite_pos(*pos);
        println!("     #{}: ({:.1}, {:.1}) → ({:.1}, {:.1})",
            i, pos.x, pos.y, rounded.x, rounded.y);
    }

    // ── 5. 工具使用小结 ──────────────────────────────────────
    println!("\n── 5. 工具使用小结 ──");
    println!("   FrameMetrics  : 集成到 GameLoop::tick() 中，每帧自动采样");
    println!("   ResolutionVerifier: 独立调用，在任何分辨率下验证 HUD 锚定和可见区域");
    println!("   SpritePalette : 编译期嵌入 PNG 并检查颜色数（无需 GPU）");
    println!("   palette::apply_pixel_art_filter(): 在 Macroquad 窗口初始化后调用一次");
    println!("   palette::round_sprite_pos(): 每次绘制精灵前舍入坐标以保证硬像素边缘");

    println!("\n=== 质量工具包演示完成 ===");
}

/// 构建一个最小的 1×1 白色 RGBA PNG，用于演示 palette 验证。
/// 在真实项目中，PNG 数据通过 include_bytes! 在编译期嵌入。
fn make_1x1_white_png() -> Vec<u8> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;

    let pixels = [255u8, 255, 255, 255]; // 1 RGBA 白色像素
    let mut buf = Vec::new();
    PngEncoder::new(&mut buf)
        .write_image(&pixels, 1, 1, image::ColorType::Rgba8)
        .expect("编码 PNG 失败");
    buf
}
