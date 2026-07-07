# 航空模拟器

基于 [Bevy](https://bevyengine.org/) 0.19 的最小化飞行器项目，围绕键盘控制、运动边界、轨迹渲染展开，并集成 [dora-rs](https://dora-rs.ai/) 数据流框架，实现「控制器 → 小飞机」的分布式驱动。

## 三个二进制

| 二进制 | 需要 `dora` feature | 说明 |
|--------|:---:|------|
| `aviation` | 否 | 独立键盘版：直接用键盘控制小飞机 |
| `aviation-dora` | 是 | 小飞机 dora 节点：接收 `cmd` 控制指令驱动飞机，回传 `pose` 位姿 |
| `aviation-controller` | 是 | 控制器 dora 节点：GUI 四方向按钮 + 键盘，输出 `cmd` 指令 |

所有共享逻辑（组件、纯函数、通用系统、嵌入资源）集中在 `src/lib.rs`。字体与飞机图片通过 `include_bytes!` **编译进二进制**，运行时不依赖工作目录下的 `assets/`，因此在 `dora run` 等任意工作目录下都能正常显示。

## 环境要求
- 支持 Rust 2024 edition 的工具链（建议 `rustup update stable`，Rust ≥ 1.88）。
- 能运行 Bevy 0.19 的 GPU / 驱动（Vulkan、Metal、OpenGL 均可）。
- 使用 dora 版时需安装 [`dora` CLI](https://dora-rs.ai/)（1.0.0-rc 系列）。

## 快速开始（独立键盘版）
```bash
git clone <repo-url>
cd aviation
cargo run
```
程序会打开标题为“DORA小飞机模拟器”的窗口，左上角显示“控制小飞机”标签。

### 操作说明
- `W` / `↑`：前进加速
- `S` / `↓`：反向推力
- `A` / `←`：逆时针旋转
- `D` / `→`：顺时针旋转

飞行范围限制在 1200×640（窗口尺寸与之对齐）。默认旋转速度 360°/s、前进速度 300 单位/s，可在 `src/lib.rs` 的 `Player` 组件中调整。

## 飞行轨迹
飞机行进时会记录路径点，并使用 Bevy gizmos 绘制白色折线。轨迹最多保留 4096 个点，超出后丢弃最早的点。单击右上角的“清除轨迹”按钮可重置轨迹，按钮在悬停和按下时会通过颜色反馈状态。

## 通过 dora 运行

### 1. 编译（release）
```bash
cargo build --release --features dora --bins
```
产物位于 `target/release/`：`aviation`、`aviation-dora`、`aviation-controller`。

### 2. dataflow 配置（`dataflow.yml`）
```yaml
nodes:
  - id: controller
    path: target/release/aviation-controller
    inputs:
      tick: dora/timer/millis/16   # ~60Hz，驱动周期发送
    outputs: [cmd]

  - id: aviation
    path: target/release/aviation-dora
    inputs:
      cmd: controller/cmd
    outputs: [pose]
```

### 3. 启动
```bash
dora run dataflow.yml
```
两个窗口会打开：控制器（四方向按钮 `↑ ← → ↓`，同时监听键盘）和小飞机。操作控制器即可驱动小飞机运动。

### 数据通信格式
| 方向 | id | Arrow 类型 | 内容 |
|------|-----|-----------|------|
| controller → aviation | `cmd` | `Float32Array[2]` | `[rotation_factor, movement_factor]`，取值 [-1, 1] |
| aviation → 下游 | `pose` | `Float32Array[3]` | `[x, y, angle]` |

控制器与小飞机通过后台 dora 线程 + `crossbeam-channel` 桥接 Bevy 主循环；控制器松开所有方向时停止发送，小飞机随即停下。

## 架构说明
- `Player` 组件保存运动/旋转速度，`AviationPath` 存储路径点，`DirectionState` 表示四方向输入并映射为控制系数。
- `apply_control` 为共享的运动核心（旋转 + 前进 + 边界裁剪），键盘版与 dora 版复用。
- 运动逻辑跑在 Bevy 的 `FixedUpdate`（60Hz）保证稳定，渲染/交互在 `Update`。
- 字体经 `pyftsubset` 子集化，仅保留界面用到的字形（`控制小飞机清除轨迹↑↓←→`），体积约 3KB。

## 开发流程
```bash
cargo fmt                                        # 格式化
cargo clippy --all-targets --features dora -- -D warnings   # Lint（含 dora 版）
cargo test                                       # 单元测试
cargo run                                        # 启动独立键盘版
```

### 更新界面文字后重建字体子集
若新增/修改了界面文字，需重新生成字体子集（漏字会显示为方框）：
```bash
pyftsubset 原字体.ttf \
  --text="控制小飞机清除轨迹↑↓←→" \
  --output-file=assets/fonts/aviation.ttf \
  --no-hinting --desubroutinize --layout-features='' --name-IDs='' --notdef-outline
```

## 贡献指南
提交时遵循 Conventional Commits（例如 `feat:`、`chore:` 等）。提交 PR 时请描述玩法影响、列出验证命令，并在视觉变更时附加截图或短视频。
