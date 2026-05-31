# 验收测试策略: Mario 2D Platformer Demo

**SRS 参考**: docs/plans/2026-05-31-mario-platformer-srs.md
**设计文档参考**: docs/plans/2026-05-31-mario-platformer-design.md
**UCD 参考**: docs/plans/2026-05-31-mario-platformer-ucd.md
**日期**: 2026-05-31
**状态**: Approved
**模板版本**: 1.0

## 1. 测试范围与策略概览

本 ATS 覆盖 Mario 2D Platformer Demo 的全部主动需求：16 项功能需求 (FR)、3 项非功能需求 (NFR)、0 项接口需求 (IFR)。FR-004 (Double Jump)、FR-005 (Wall Jump)、FR-007 (Moving Platforms)、FR-012 (Pipe Teleporters) 已延后，记录于 [deferred backlog](2026-05-31-mario-platformer-deferred.md)。

游戏为 Rust + Macroquad 独立桌面应用，纯本地运行——无网络、无认证、无外部数据源。测试以 Rust 内置 `#[test]` 框架为核心，逻辑状态/碰撞/物理全部可自动化；像素渲染精准度和分辨率切换需要手工截图验证。

## 2. 需求 → 验收场景映射

### 2.1 功能需求 (FR)

| Req ID | 需求摘要 | 验收场景 | 必须类别 | 优先级 | 自动化可行性 | 备注 |
|--------|---------|---------|---------|--------|-------------|------|
| FR-001 | 水平移动 | 加速至最大速度 (0.3s内) / 释放后摩擦减速 (0.2s至静止) / 最大速度→反向最大速度 / 同时按双键去抖处理 / 空中操控 (60%) | FUNC,BNDRY | Critical | Auto | 边界值：零速→最大速、最大速→零速、最大速→反向最大速 |
| FR-002 | 跳跃 | 短按 (<100ms) 约40%最大高度 / 长按达最大高度 / 禁止空中二段跳 / 天花板碰撞垂直速度归零 / 平台边缘走出重力激活下落 | FUNC,BNDRY | Critical | Auto | 边界值：跳跃时长上限=最大跳跃时长、天花板高度恰等于玩家碰撞体顶部 |
| FR-003 | 冲刺 | 1.5× 最大水平速度 / 冲刺跳跃水平距离+50% / 冲刺不影响摩擦力（释放方向键仍正常减速） | FUNC,BNDRY | High | Auto | 边界值：冲刺+跳跃叠加验证、冲刺中释放方向键→减速 |
| FR-006 | 固定平台 | 下落着陆 / 走出边缘坠落 / 下方不可穿越（头顶碰撞） / 侧面阻挡 | FUNC,BNDRY | Critical | Auto | 边界值：平台边缘刚好踩到、平台间最小间隙 |
| FR-008 | 金币收集 | 碰撞收集计数+1 / 同一金币不重复收集 / 重生后金币复位重新出现 | FUNC,BNDRY | High | Auto | 边界值：同时触碰多个金币、金币位于平台边缘 |
| FR-009 | 危险物 (尖刺/深渊) | 尖刺接触死亡 / 跳跃安全越过尖刺 / Y > kill_y 坠落即死 / 无敌状态豁免不受伤 | FUNC,BNDRY | Critical | Auto | 边界值：刚好擦过尖刺边缘、Y 坐标恰好等于 kill_y |
| FR-010 | 可踩踏敌人 | 巡逻往返A↔B / 踩踏消灭+玩家反弹 / 侧面接触玩家死亡 / 下方接触玩家死亡 / 到达路点精确反向（≤1帧内方向翻转） | FUNC,BNDRY | High | Auto | 边界值：刚好踩到敌人顶部碰撞体边界、路点坐标精确到达判定 |
| FR-011 | 问号方块 | 头顶撞击激活+产出奖励 / 已使用方块不产出 / 侧面/上方接触不激活 / 超级蘑菇→变大可承伤一次 / 火焰花→火球消灭路径敌人 / 概率分布验证 (N≥500次: 金币70%±5%、蘑菇15%±3%、花15%±3%) | FUNC,BNDRY | High | Auto | 概率分布需要统计验证：循环500+次问号方块激活，检验各奖励频次是否在容差范围内 |
| FR-013 | 摄像机跟随 | 水平 8%/帧追赶 / 静止无振荡 / 左边界钳制（不显示关卡外） / 右边界钳制 / 垂直死区60%+5%/帧追赶 | FUNC,BNDRY | Critical | Auto | 边界值：视口刚好到达关卡边界、玩家刚好在死区线上 |
| FR-014a | 死亡触发 | 生命-1 / 输入锁定1.5s (不可操控) / 生命>0→重生流程 / 生命=0→Game Over流程 | FUNC,BNDRY | Critical | Auto | 边界值：生命恰好在死亡后=0、死亡动画精确1.5s |
| FR-014b | 重生+无敌 | 有检查点→检查点重生 / 无敌2s闪烁(4Hz) / 2s后碰撞恢复+输入恢复 / 无检查点→关卡起点重生 | FUNC,BNDRY | Critical | Auto | 边界值：无敌刚好在2.0s结束、重生位置=检查点坐标 |
| FR-014c | Game Over | Game Over文字叠加层显示 / 空格键→完全重置(3命+0金币+关卡起点) / 不自动重启（等待玩家确认）/ "Press Space to Restart" 闪烁提示 | FUNC,BNDRY,UI | Critical | Auto (逻辑) + Manual: visual-judgment (画面) | 画面渲染验证需截图对比 |
| FR-015 | 胜利条件 (旗杆) | 旗杆触发滑下动画 / Victory画面+总金币数+"Press Space to Play Again" / 空格→完全重置 / 死亡优先于胜利（死亡中不可触发旗杆） | FUNC,BNDRY,UI | Critical | Auto (逻辑) + Manual: visual-judgment (画面) | 画面渲染验证需截图对比 |
| FR-016 | HUD | 初始状态：(3%,3%) 锚定+金币=0+生命=3 / 金币变化当前帧更新 / 生命变化当前帧更新 / 重置后恢复初始值 | FUNC,BNDRY,UI | Critical | Auto (坐标计算) + Manual: visual-judgment (像素) | 锚点偏差≤±2%视口可以坐标断言；像素渲染需截图 |
| FR-017 | 分辨率与显示 | 按ESC呼出选项菜单 / 方向键选择分辨率→回车确认→即时应用 / 全屏开关切换 / 再按ESC关闭菜单返回游戏 | FUNC,BNDRY,UI | High | Manual: visual-judgment | 显示模式切换无法在 headless 中验证；逻辑状态 (selected_index等) 可自动测试 |
| FR-018 | 60fps游戏循环 | 1秒内恰好60次模拟步进 (固定 dt=1/60s) / 渲染帧率波动不影响模拟速度 / 单帧最多追赶5步，超出丢弃 / 累加器初始化为0 | FUNC,BNDRY,PERF | Critical | Auto | 帧时间度量 + 累加器值断言；螺旋死亡防护验证 (注入模拟过载) |

### 2.2 非功能需求 (NFR)

| Req ID | 需求摘要 | 验收场景 | 必须类别 | 优先级 | 自动化可行性 | 备注 |
|--------|---------|---------|---------|--------|-------------|------|
| NFR-001 | 60fps 帧率 | P99帧时间≤16.67ms / 任意连续60s窗口帧率≥58fps / 3种分辨率+全屏下帧时间一致性 | PERF | Critical | Auto | 内置 FPS 计数器输出日志；帧时间以 `std::time::Instant` 度量 |
| NFR-002 | 多分辨率支持 | HUD锚定(3%,3%)偏差≤±2%视口@720p/1080p/1440p / 玩家可见区域占视口宽度45-50% / 全屏模式HUD位置不变 | PERF,UI | High | Manual: visual-judgment | 每种分辨率手工截图 + 像素测量 |
| NFR-003 | 像素艺术渲染 | 精灵8×放大后像素边界为纯色正方形 / 无过渡色/模糊边缘（最近邻缩放生效） / 每精灵≤16色 | UI | High | Manual: visual-judgment | 截图8×放大检查，需人眼判定；16色限制可自动检查调色板 |

### 2.3 约束与假设 (CON / ASM)

| Req ID | 需求摘要 | 验收场景 | 必须类别 | 优先级 | 自动化可行性 | 备注 |
|--------|---------|---------|---------|--------|-------------|------|
| CON-003 | Windows .exe 运行 | 在 Windows 10/11 上双击 .exe 启动成功 / 无 DLL 缺失错误 / 游戏窗口正常渲染 | FUNC | Critical | Manual: physical-device | 需实体 Windows 机器验证；SRS §10 Platform smoke test |
| CON-004 | 离线运行 | 断开网络连接后游戏正常启动和运行 / 所有功能可用（金币、敌人、方块、旗杆） / 无卡顿或错误提示 | FUNC | Critical | Auto | 在无网络环境下运行自动化测试套件；SRS §10 Air-gap test |
| ASM-002 | QWERTY 键盘 | 使用标准 QWERTY 键盘测试全部操作（WASD/方向键/空格/Shift/ESC）/ 若键位不可用，备选方案为 A/D+Space+Shift 仍可完成游戏 | FUNC,BNDRY | Low | Manual: physical-device | 违规测试：使用 AZERTY 键盘确认 A/D/Space 仍可操作 (备选键位) |
| ASM-005 | 选项菜单配置 | 通过选项菜单切换分辨率并确认生效 / 若菜单不可用，备选方案为启动参数设置分辨率 | FUNC | Low | Manual: visual-judgment | 违规测试：直接编辑配置确认回退路径可用 |

> CON-001 (Rust edition 2024) 与 CON-002 (Macroquad) 由 CI/build pipeline 验证；ASM-001/003/004 为范围假设，违规影响在 SRS §8 记录，本 ATS 不重复测试。

### 2.4 接口需求 (IFR)

[Not applicable — SRS §6 明确无 IFR 需求]

### 2.5 覆盖统计

| 类别 | 需求覆盖数 |
|------|----------|
| FUNC | 18 (16 FR + 2 CON/ASM) |
| BNDRY | 16 (全部 FR) |
| PERF | 3 (FR-018 + NFR-001 + NFR-002) |
| UI | 6 (FR-014c + FR-015 + FR-016 + FR-017 + NFR-002 + NFR-003) |
| Manual (fully) | 4 (FR-017 + NFR-002 + NFR-003 + ASM-005) |
| Manual (mixed) | 2 (FR-014c + FR-015: 逻辑 Auto + 画面 Manual) |
| **主动需求合计** | **20 (16 FR + 3 NFR + 0 IFR + 2 CON + 2 ASM)** |

## 3. 测试类别策略

类别语义、缩写、强制条件见本文档末尾「类别定义（参考）」表。每需求的必需类别已在 §2 映射表 `必须类别` 列声明。

| 类别 | 执行约束 |
|------|---------|
| **FUNC** | 每个 FR ≥ 1 happy-path + 1 error-path 场景；总计 ≥ 3 场景/FR。使用 `#[test]` 纯函数测试模式：构造输入状态 → 调用被测函数 → 断言输出值 |
| **BNDRY** | 每个含数值上限/范围/阈值的 FR 必须显式列出边界场景（速度上限=最大速度、跳跃时长=上限、生命数=0、无敌=2.0s、碰撞刚好在边缘）。所有 `f32` 比较使用 `approx` 容差（收敛用 epsilon=0.5px，物理用 epsilon=0.01） |
| **PERF** | FR-018 帧时间测试：运行固定 N=360 步模拟，断言总耗时 < N × (1/60)s × 1.05 系数。NFR-001：内置 FPS 计数器输出 min/max/avg/p99 帧时间日志。NFR-002：手工截图测量 |
| **UI** | HUD 锚定偏差用坐标计算断言 (≤ ±2% 视口)。画面渲染（GameOver/Victory/分辨率/全屏）归为 Manual: visual-judgment，交给 ST 阶段手工截图验证 |
| **SEC** | [Not applicable — 独立桌面游戏，无用户认证、外部输入、网络通信] |

## 4. NFR 测试方法矩阵

| NFR ID | 测试方法 | 工具 | 通过标准 | 负载参数 | 关联 Feature |
|--------|---------|------|---------|---------|-------------|
| NFR-001 | 帧时间度量 | `std::time::Instant` + 内置 FPS 计数器 | P99 帧时间 ≤ 16.67ms；任意连续 60s 窗口帧率 ≥ 58fps | 720p/1080p/1440p 窗口模式；全屏模式；Playing 状态满实体 (全部敌人/金币/方块/尖刺激活) | F01 Engine Core |
| NFR-002 | 手工截图验证 | OS 截图工具 | HUD 锚定 (3%, 3%) 偏差 ≤ ±2% 视口宽高；玩家可见区域占视口宽度 45-50% | 三种分辨率 (720p/1080p/1440p) + 全屏 | F01, F04, F09, F10 |
| NFR-003 | 手工 8× 放大检查 | 截图工具 + 图像查看器 8× 放大 | 像素边界为纯色正方形，无过渡色/模糊边缘；精灵 ≤ 16 色 | 三种分辨率下所有精灵类型 | F02 (Parallax + 精灵渲染) |

## 5. 跨 Feature 集成场景

| 场景 ID | 场景描述 | 涉及 Features | 数据流路径 | 验证要点 | ST 阶段覆盖 |
|-------------|-------------|--------------|---------------------|---------------------|--------|
| INT-001 | 玩家踩踏敌人 → 消灭+反弹 | F03, F07, F02 | IAPI-002 → IAPI-004 → CollisionEvent::EnemyStomp | 敌人从实体数组移除；玩家 vy 变为正值 (反弹)；玩家继续存活 | Feature ST |
| INT-002 | 玩家触碰尖刺 → 死亡触发 | F03, F05, F06 | IAPI-002 → IAPI-004 → CollisionEvent::HazardContact → StateMachine → Death | 生命-1；死亡动画开始；输入锁定 1.5s | Feature ST |
| INT-003 | 玩家坠入深渊 → 即死 | F03, F05, F06 | IAPI-002 → IAPI-004 → CollisionEvent::PitFall → StateMachine → Death | Y > kill_y 触发；不播放受伤动画而是直接坠落消失 | Feature ST |
| INT-004 | 玩家右移 → 摄像机跟随 → 视差滚动 | F03, F04, F02 | IAPI-007 → Camera::update() → IAPI-010 → Parallax::render() | 水平偏移每帧收敛 8%；3层背景分别以 0.1×/0.3×/0.6× 滚动；视口不超出关卡边界 | Feature ST |
| INT-005 | 玩家收集金币 → HUD 即时更新 | F03, F08, F09 | IAPI-004 → CollisionEvent::CoinCollect → Player.coins+1 → IAPI-009 → HUD::render() | HUD 金币计数在同一帧内更新；锚定位置 (3%, 3%) 不变 | Feature ST |
| INT-006 | 玩家死亡 → HUD 生命更新 | F03, F06, F09 | Death → LifeState.lives-1 → IAPI-009 → HUD | 生命数当前帧 -1；重生后保持正确值 | Feature ST |
| INT-007 | 选项切换分辨率 → 窗口即时调整 | F10, F01 | IAPI-011 (apply_display) → GameLoop 重建渲染目标 | 窗口尺寸匹配所选分辨率；480×270 虚拟画布保持；最近邻缩放生效 | System ST |
| INT-008 | 玩家走出平台边缘 → 坠落 | F03, F02 | IAPI-002 → IAPI-005 (query_terrain) → tiles 为空 → Player.on_ground=false → 重力 | 玩家 vy 开始向下增加；HUD 锚定位置不随坠落变化 | Feature ST |
| INT-009 | 完整通关: 起跑→吃币→顶方块→变大→踩敌→触旗→胜利 | F01-F09 | 全部 11 个 Contract ID 完整路径 | 状态流转 Playing→Victory 不中断；所有收集物计数正确；力量状态正确应用；动画完播；Victory 显示总金币数 | System ST |
| INT-010 | 顶方块→蘑菇→变大→碰危险→退化 | F03, F08, F05, F06 | IAPI-002 → IAPI-004 → QuestionBlockHit → PowerUp::SuperMushroom → Player::apply_powerup(Super) → HazardContact → take_damage() → PlayerState::Small | 蘑菇弹跳接触玩家→碰撞体 16×32；碰尖刺→恢复 16×16 而不死亡；无敌 2s 闪烁 | Feature ST |

---

## 附录: ATS 审核报告

### ATS Compliance Review Report

**Review Date**: 2026-05-31
**Documents Reviewed**:
- ATS Draft (Mario 2D Platformer Demo)
- SRS: docs/plans/2026-05-31-mario-platformer-srs.md (Approved)
- Design: docs/plans/2026-05-31-mario-platformer-design.md (Approved)
- UCD: docs/plans/2026-05-31-mario-platformer-ucd.md (Approved)

#### Summary

| Dimension | Verdict | Defects |
|-----------|---------|---------|
| R1: Coverage Completeness | FAIL | 2 Minor |
| R2: Category Diversity | PASS | 0 |
| R3.1: Path Coverage | FAIL | 1 Major |
| R3.2: Boundary & Edge Cases | FAIL | 1 Major |
| R3.3: State & Transition | FAIL | 1 Minor |
| R3.4: Error Handling | PASS | 0 |
| R3.5: Implicit Requirements | FAIL | 1 Major + 1 Minor |
| R4: Verifiability | PASS | 0 |
| R5: NFR Testability | PASS | 0 |
| R6: Cross-Feature Integration | PASS | 0 |
| R8.1: Scenario Cross-Ref (ATS vs SRS) | FAIL | 1 Major |
| R8.2: Pass Criteria Consistency | PASS | 0 |
| R8.3: Test Method Feasibility (ATS vs Design) | PASS | 0 |

**Total: 2 Major + 4 Minor = 6 defects. 0 Critical. 0 Cross-Reference Conflicts.**

**Final Verdict after fixes: PASS** — all 6 defects resolved in the published ATS document.

---

#### Defects Found (Draft Stage)

**M1 — R3.1/R3.2/R8.1: FR-011 Probability Distribution Scenario Missing**
- SRS §4 FR-011 AC-1 mandates verification of loot table distribution (Coin 70%, Super Mushroom 15%, Fire Flower 15%).
- **Fix applied**: Added explicit acceptance scenario "概率分布验证 (N≥500次: 金币70%±5%、蘑菇15%±3%、花15%±3%)" to FR-011 row in §2.1.

**M2 — R3.5: Constraint Enforcement Scenarios Missing**
- SRS §10 lists verification methods for all CON-xxx constraints.
- **Fix applied**: Added §2.3 Constraint & Assumption table with CON-003 (Windows .exe) and CON-004 (offline operation) scenarios. CON-001/002 delegated to CI/build pipeline.

**m1 — R1: Coverage Statistics UI Count Mismatch**
- ATS §2.4 stated "UI: 5" but mapping tables yield 6 requirements with UI category.
- **Fix applied**: Updated §2.5 to "UI: 6".

**m2 — R3.5: Assumption Violation Scenarios Missing**
- SRS §8 defines ASM-001 through ASM-005 with Impact if Invalid columns.
- **Fix applied**: Added ASM-002 (QWERTY keyboard) and ASM-005 (option menu config) violation scenarios to §2.3.

**m3 — R3.3: FR-010 Enemy Patrol State Transitions Only in Boundary Notes**
- SRS FR-010 AC-1 describes waypoint-reach → direction-reverse as a discrete state transition.
- **Fix applied**: Promoted to explicit acceptance scenario "到达路点精确反向（≤1帧内方向翻转）" in FR-010 row.

**m4 — R1: Missing Deferred FR Annotation**
- FR numbering gaps (FR-004/005/007/012) not explained.
- **Fix applied**: Added note in §1 referencing the deferred backlog document for FR-004, FR-005, FR-007, FR-012.

#### Cross-Reference Conflicts

None found. All threshold values, protocol definitions, feature references, and contract IDs are consistent between ATS and source documents (SRS, Design, UCD).

---

## 类别定义（参考）

| Category | Abbrev | 说明 | 何时必需 |
|----------|--------|------|---------|
| `functional` | FUNC | 正常路径与错误路径验证 | 始终 —— 每个 FR |
| `boundary` | BNDRY | 边界情况、上限、空值/最大值/零值 | 始终 —— 每个 FR |
| `security` | SEC | 注入、授权、数据校验 | FR 涉及用户输入、认证或外部数据时 |
| `performance` | PERF | 响应时间、吞吐、资源占用 | 包含性能指标的 NFR-xxx |
| `ui` | UI | 视觉渲染与交互验证 | feature 涉及 HUD/菜单/画面叠加层时 |
