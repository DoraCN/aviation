# Aviation Simulator

Minimal Bevy project that renders a controllable aircraft for experimenting with keyboard input, motion constraints, and simple trail visualization.

## Requirements
- Rust toolchain with edition 2024 support (`rustup update stable`).
- A GPU/driver that can run Bevy 0.16 (Vulkan/Metal/OpenGL backends work).

## Getting Started
```bash
git clone <repo-url>
cd aviation
cargo run
```
The simulator opens a window titled “DORA小飞机模拟器”, with a “控制小飞机” label in the top-left corner. Assets load from `assets/images` and `assets/fonts`, so keep that directory structure when adding media.

## Controls
- `W` / `↑` accelerate forward.
- `S` / `↓` reverse thrust.
- `A` / `←` rotate counter-clockwise.
- `D` / `→` rotate clockwise.

Movement is clamped to a 1200×640 world. Rotation speed is 360° per second, and forward speed defaults to 300 units per second. Adjust these values in `src/main.rs` inside the `Player` component.

## Flight Path Trail
The aircraft records waypoints while moving and renders them as a white line strip using Bevy gizmos. The trail keeps up to 4096 points, dropping the oldest once the cap is reached. Click the “清除轨迹” button in the top-right corner (20 px from top/right) to reset the trail. Button states include hover and press feedback via color shifts.

## Development Workflow
- `cargo fmt` — format code to Rust style.
- `cargo clippy -- -D warnings` — lint and treat warnings as errors.
- `cargo test` — run unit/integration tests (add them alongside new features).
- `cargo run` — launch the interactive simulation.

Project logic lives entirely in `src/main.rs`. A `Player` component drives keyboard input, and `AviationPath` stores the recorded positions. Systems are scheduled using Bevy’s `FixedUpdate` for deterministic movement and `Update` for rendering/interaction.

## Contributing
Follow Conventional Commits (`feat:`, `chore:`, etc.) when pushing changes. Pull requests should describe gameplay impact, list validation commands, and include screenshots or clips when visuals change.
