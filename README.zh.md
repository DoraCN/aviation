# 航空模拟器

基于 Bevy 的最小化飞行器项目，通过键盘控制、运动边界与轨迹渲染来探索交互逻辑。

## 环境要求
- 支持 Rust 2024 edition 的工具链（建议执行 `rustup update stable`）。
- 能运行 Bevy 0.16 的 GPU / 驱动（Vulkan、Metal、OpenGL 均可）。

## 快速开始
```bash
git clone <repo-url>
cd aviation
cargo run
```
程序会打开标题为“DORA小飞机模拟器”的窗口。资源文件位于 `assets/images` 与 `assets/fonts`，添加新素材时请保持该目录结构。

## 操作说明
- `W` / `↑`：前进加速
- `S` / `↓`：反向推力
- `A` / `←`：逆时针旋转
- `D` / `→`：顺时针旋转

飞行范围限制在 1200×640。默认旋转速度为每秒 360°，前进速度为每秒 300 单位，可在 `src/main.rs` 的 `Player` 组件中调整。

## 飞行轨迹
飞机行进时会记录路径点，并使用 Bevy gizmos 绘制白色折线。单击右上角距边缘 20 像素的“清除线条”按钮可重置轨迹，按钮在悬停和按下时会通过颜色反馈状态。

## 开发流程
- `cargo fmt` —— 按 Rust 规范格式化代码
- `cargo clippy -- -D warnings` —— 运行 Lint，并将警告视为错误
- `cargo test` —— 运行单元 / 集成测试（新增功能请补充测试）
- `cargo run` —— 启动交互式模拟器

全部业务逻辑集中在 `src/main.rs`。`Player` 组件处理键盘输入，`AviationPath` 存储路径点。系统通过 Bevy 的 `FixedUpdate` 保证运动稳定，通过 `Update` 处理渲染与交互。

## 贡献指南
提交时遵循 Conventional Commits（例如 `feat:`、`chore:` 等）。更多协作规范（素材组织、测试要求等）见 `AGENTS.md`。提交 PR 时请描述玩法影响、列出验证命令，并在视觉变更时附加截图或短视频。
