# 飞机大战（Plane War）独立小游戏 · 设计方案

> 目标：在本仓库内新增一个**经典竖版飞机大战小游戏**，作为 **dora 数据流的一个节点**，由现有控制器 `aviation-controller` 驱动操控。玩法尽量简化：敌人用一组**图标 PNG**（emoji 风格），另有**炸弹 💣 障碍（不可击落，只能躲避）**，碰到即爆炸。**竖屏、固定窗口**。技术栈沿用 Bevy 0.19 + Rust 2024。

---

## 0. 定位与课程边界（重要）

- 本课程默认交付的、开箱即用的部分是**「用控制器操控小飞机」**：即 `aviation-controller → aviation-dora` 这条数据流，提供完整可运行代码与默认 `dataflow.yml`。
- **飞机大战是进阶内容**：我们**提供完整、可运行的飞机大战代码**（`plane-war` 节点），但**默认不接入**课程的 `dataflow.yml`。
- 学员若想玩飞机大战，需要**自己动手**：
  1. 以 `--features dora` 编译出 `plane-war`；
  2. 修改 / 另写一个 dataflow 文件，把 `aviation-controller` 的 `cmd` 接到 `plane-war` 节点。
- 一句话：**同一个控制器，既能开小飞机，也能玩飞机大战**——区别只在 dataflow 怎么连线。

---

## 1. 背景调研：微信《经典飞机大战》

- 2013 年 8 月微信 5.0 随版本推出的 HTML5 小游戏（俗称「打飞机」），启动微信需先通关一次，凭 4 亿用户量迅速走红。
- 玩法是**传统竖版飞行射击（STG）**：控制屏幕下方战机，躲避并击落自上而下的敌机，**触碰敌机即结束**，累计积分并有好友排行榜。
- 2014 年 1 月升级为独立 App《全民飞机大战》（多机型、僚机、主动攻击等）。

本方案只借鉴其**极简爽快的核心玩法**，并按用户要求做了简化与 dora 化改造。

---

## 2. 控制方式：完全由 dora 驱动

飞机大战节点**不自行监听键盘做操作**，而是通过 dora 输入 `cmd` 接收 `aviation-controller` 的指令（与小飞机同一套控制器、同一套协议）。

### 2.1 控制协议扩展（向后兼容）
现有协议 `cmd = Float32Array[2] = [rotation_factor, movement_factor]`，**扩展为 3 个分量**：

| 分量 | 含义 | 计算 | 取值 |
|------|------|------|------|
| 0 | 水平/旋转轴 | `left - right` | -1 / 0 / 1 |
| 1 | 垂直/推进轴 | `up - down` | -1 / 0 / 1 |
| 2 | **动作键（开火）** | `fire ? 1 : 0` | 0 / 1 |

**兼容性**：`aviation-dora`（小飞机）的 `parse_command` 只读前两个分量、忽略第三个，**无需改动**即可继续工作。飞机大战读全部三个分量。

### 2.2 各消费者对同一 `cmd` 的解释
| 节点 | 分量 0 | 分量 1 | 分量 2 |
|------|--------|--------|--------|
| `aviation-dora`（小飞机） | 旋转（左正=逆时针） | 前后推进 | 忽略 |
| `plane-war`（飞机大战） | 水平平移（`move_x = -分量0`，left→左移） | 垂直平移（`move_y = 分量1`，up→上移） | 开火（1=射击） |

### 2.3 `aviation-controller` 需新增「开火」键
- 界面新增一个按钮（如「🔫 / 开火」），并绑定键盘键（建议 `空格` 或 `J`）。
- 控制器状态从四方向扩展为 `DirectionState + fire: bool`。
- 发送时机：**任意方向或开火激活时**发送 `cmd`；全部松开则停止发送（沿用现有「松手即停」策略）。
- 该按键对小飞机无副作用（小飞机忽略分量 2），因此单一控制器可同时服务两种玩法。

---

## 3. 玩法规格（简化版）

### 3.0 屏幕与窗口
- **竖屏**、**固定尺寸、不可调节**（`resizable: false`）。
- 基准分辨率 **480×800**（逻辑像素，`scale_factor_override(1.0)`），世界坐标与之对齐；如需更高可整体等比放大（如 540×960）。
- 战机活动范围限制在窗口内。

### 3.1 我方战机
- 贴图 `assets/images/fighter.png`，位于屏幕下方区域。
- 移动：由 `cmd` 的分量 0/1 驱动的**平移**，限制在窗口范围内（无旋转）。
- 开火：`cmd` 分量 2 为 1 时**发射子弹**（按住持续发射，带射速冷却）。
- 生命：碰到**任何敌人或炸弹**即爆炸 → 游戏结束（一条命，还原经典手感）。

### 3.2 敌人（图标 PNG，可被击落，两档分值）
- 从屏幕上方随机位置生成，向下匀速移动，飞出底部即回收。
- 素材来自两个目录，**按尺寸区分分值**：

| 档位 | 目录 | 尺寸 | 分值 | 素材（随机抽取） |
|------|------|:----:|:----:|------|
| 小 | `assets/images/64/` | 64×64 | **1 分** | alien / ufo / caterpillar / clown-face / exploding-head / face-with-steam-from-nose / gift / goal / partying-face / pile-of-poo / see-no-evil-monkey / thinking-face / cards（13 种） |
| 大 | `assets/images/100/` | 100×100 | **2 分** | chatbot / dollar-bag / robotic / star-struck / trust（5 种） |

- 生成时从对应目录随机挑一张贴图；两档按一定概率混合出现。
- **1 击即毁**（不做血量分级）：被子弹命中 → 播放爆炸 → 加对应分（1 或 2）、消失。
- 碰撞半径可依据 64 / 100 尺寸分别设定。

### 3.3 炸弹障碍（`bomb.png`，不可击落，只能躲避）
- 贴图 `assets/images/bomb.png`，从上方下落。
- **子弹对其无效**（穿过 / 不掉血，不可摧毁）。
- 我方战机碰到炸弹 → 立即爆炸 → 游戏结束。
- 作用：制造必须**主动躲避**的威胁，提升操作压力。

### 3.4 子弹
- 我方子弹向上直线飞行，用简单形状（小圆点 / 短竖条纯色精灵）即可。
- 命中敌人 → 敌人爆炸并计分；命中炸弹 → 无效。

### 3.5 计分与存档
- 击落敌人累加分数（小 +1 / 大 +2），HUD 实时显示。
- 本地保存历史最高分（用户配置目录存档）。
- 结束页：本局得分、历史最高、重开提示。

### 3.6 难度
- 随时间提升敌人 / 炸弹的生成频率与下落速度上限（分档封顶）。

---

## 4. 美术资源与加载

游戏所需素材均为**现成 PNG**（用户已提供），无需运行时依赖工作目录：

| 用途 | 路径 | 说明 |
|------|------|------|
| 我方战机 | `assets/images/fighter.png` | 玩家 |
| 炸弹障碍 | `assets/images/bomb.png` | 不可摧毁 |
| 小敌人（1 分） | `assets/images/64/*.png` | 64×64，13 种 |
| 大敌人（2 分） | `assets/images/100/*.png` | 100×100，5 种 |
| 爆炸 | 复用敌人被击/战机死亡表现 | 可用缩放淡出或简单闪帧（无专门序列图时先用程序化表现） |

### 加载方式：编译期嵌入
- 与本仓库现有做法一致，用 `include_bytes!` 把上述 PNG **编译进二进制**，运行时用 `Image::from_buffer` 解码为 `Handle<Image>`，**摆脱工作目录依赖**（`dora run` 下也能正常加载）。
- 在 `assets.rs` 内维护两个数组：`ENEMIES_64: &[(&str, &[u8])]`、`ENEMIES_100: &[(&str, &[u8])]`，逐个 `include_bytes!` 列出；`fighter` / `bomb` 单独嵌入。启动时全部解码并存入 `GameAssets` 资源（含分档的 `Vec<Handle<Image>>`）。

> ⚠️ 版权：图标为 emoji 风格素材，请确认其许可（如来自 Noto Color Emoji / OpenMoji 等开源集，遵守 OFL/CC 许可）。

---

## 5. 音频

### 5.1 背景音乐
- 素材：`assets/media/bg.mp3`（~500KB）。
- 方式：`include_bytes!` 嵌入、`AudioSource` 构造、`AudioPlayer` + `PlaybackSettings::LOOP` 循环播放。
- 游戏进入 Playing 状态时开始播放，退出时停止。

### 5.2 爆炸音效
- 素材：`assets/media/bomb.mp3`（~47KB）。
- 用途：我方战机被敌人或炸弹碰撞时播放（一次性，不循环）。
- 方式：同样 `include_bytes!` 嵌入，触发爆炸时通过 `AudioPlayer` 播放（`PlaybackMode::Once`）。

### 5.3 文件组织
```
assets/
  media/
    bg.mp3            # 背景音乐（~500KB）
    bomb.mp3          # 爆炸音效（~47KB）
```

### 5.4 后续音效
射击、拾取道具等音效可在需要时按相同模式追加。

---

## 6. 技术架构（Bevy 0.19 + dora）

### 5.1 程序形态
- 新增二进制 `plane-war`，`required-features = ["dora"]`（作为 dora 节点接收 `cmd`）。
- 后台线程 + `crossbeam-channel` 桥接 dora 事件循环与 Bevy 主循环（复用 `aviation-dora` 的模式）。
- 可选：无 dora 输入时提供本地键盘 fallback，仅用于开发调试（默认关闭，正式以 dora 驱动为准）。

```
src/
  bin/
    plane_war.rs          # 入口：装配 App、dora 桥接、状态机
  plane_war/
    mod.rs                # GamePlugin 聚合
    dora_bridge.rs        # dora 线程 + cmd 接收（PlaneCommand{move_x,move_y,fire}）
    components.rs         # Player/Enemy/Bomb/Bullet/Explosion/Velocity/Collider
    resources.rs          # Score/HighScore/Difficulty/SpawnTimers/GameAssets
    states.rs             # GameState: Loading/MainMenu/Playing/GameOver
    assets.rs             # 嵌入 PNG（fighter/bomb/64/100）与解码
    systems/
      input.rs            # 读取 dora cmd -> 更新 PlayerIntent 资源
      player.rs           # 平移、开火冷却
      enemy.rs            # 敌人生成/移动/回收（两档分值）
      bomb.rs             # 炸弹生成/移动（不可摧毁）
      bullet.rs           # 子弹移动/命中（对 bomb 无效）
      collision.rs        # 圆形碰撞：子弹×敌人、战机×(敌人|炸弹)
      explosion.rs        # 爆炸表现
      hud.rs              # 计分/状态 UI
      spawn.rs            # 难度驱动刷怪
docs/
  plane-war-design.md
```

窗口：竖屏固定，`Window { resizable: false, resolution: 480×800 (scale_factor_override 1.0), .. }`。

### 5.2 状态机
```
GameState: Loading -> MainMenu -> Playing -> GameOver ( -> Playing 重开 )
```
系统用 `run_if(in_state(Playing))` 门控；`OnEnter/OnExit` 建场景与清理（标记组件批量 despawn）。

### 5.3 dora 桥接与输入
- `dora_bridge.rs`：后台线程 `DoraNode::init_from_env`，监听 `cmd` 输入，解析为 `PlaneCommand{ move_x, move_y, fire }` 通过 channel 送入 Bevy。
- `input.rs`：`try_recv` 取最新指令写入 `PlayerIntent` 资源；无指令则保持静止、不开火。
- 停止：收到 `Event::Stop` → 置标志 → Bevy 发 `AppExit`（复用现有模式）。

### 5.4 组件 / 资源（简化）
- `Player { fire_cooldown: Timer }`
- `Enemy { score: u32 }`（1 击毁，无血量字段；score = 1 或 2）
- `Bomb`（无血量、不可摧毁标记）
- `Bullet`、`Velocity(Vec2)`、`Collider { radius }`
- `Explosion { timer }`、`Despawnable`
- 资源：`PlayerIntent{move_x,move_y,fire}`、`Score`、`HighScore`、`Difficulty`、`SpawnTimers`、`GameAssets{ fighter, bomb, enemies_64: Vec<Handle<Image>>, enemies_100: Vec<Handle<Image>> }`

### 5.5 碰撞（圆形）
- 子弹 × 敌人 → 敌人爆炸 + 计分（子弹销毁）。
- 子弹 × 炸弹 💣 → **无效**（不处理，子弹继续或穿过）。
- 战机 × (敌人 | 炸弹) → 战机爆炸 → `GameOver`。

---

## 7. dataflow 连线

### 6.1 课程默认（开箱即用，控制小飞机）
```yaml
nodes:
  - id: controller
    path: target/release/aviation-controller
    inputs: { tick: dora/timer/millis/16 }
    outputs: [cmd]
  - id: aviation
    path: target/release/aviation-dora
    inputs: { cmd: controller/cmd }
    outputs: [pose]
```

### 6.2 进阶（学员自行改造，玩飞机大战）
```yaml
nodes:
  - id: controller
    path: target/release/aviation-controller
    inputs: { tick: dora/timer/millis/16 }
    outputs: [cmd]
  - id: plane-war
    path: target/release/plane-war
    inputs: { cmd: controller/cmd }
```
> 同一个 `controller`，仅把 `cmd` 接到 `plane-war` 即可。四方向控制战机平移，新增的开火键触发射击。

---

## 8. 里程碑（建议实现顺序）
1. **M1**：`plane-war` bin + 状态机 + 卷轴背景 + dora 桥接 + 战机平移（emoji-PNG）。
2. **M2**：自动/按键开火、子弹、敌人 emoji 生成与移动、命中销毁与计分。
3. **M3**：炸弹 💣 障碍（不可摧毁、碰撞致死）、爆炸动画、HUD。
4. **M4**：GameOver / 最高分存档 / 主菜单 / 难度曲线。
5. **M5**：`aviation-controller` 增加开火键并扩展 `cmd` 第三分量；联调整条数据流。
6. **M6**：打磨（音效、抖动、难度平衡）。

---

## 9. 测试策略
- 纯逻辑抽函数 + 单测：圆形碰撞 `circles_overlap`、子弹对敌人 / 炸弹的差异化处理、`cmd` 解析（含第三分量）、难度参数推进、平移边界裁剪。
- 系统层不做集成测试，靠纯函数覆盖 + 手动联调。
- 验证：`cargo fmt`、`cargo clippy --all-targets --features dora -- -D warnings`、`cargo test`、以及通过 dora 联调运行。

---

## 10. 依赖与影响
- 复用现有 `dora` feature（`dora-node-api` / `crossbeam-channel` / `eyre`）与嵌入资源、字体子集方案。
- `aviation-controller`、`aviation-dora` 因 `cmd` 向后兼容扩展，**小飞机侧无需改动**。
- 新增 `plane-war` 二进制；美术资源为 `assets/images/` 下已提供的 PNG（`fighter.png`、`bomb.png`、`64/`、`100/`），编译期嵌入。

---

## 11. 可选扩展
- 道具（双弹 / 清屏炸弹）、敌人分级与血量、Boss、敌人主动攻击、音效与排行榜。
