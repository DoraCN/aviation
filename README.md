# Aviation Simulator

A minimal aircraft project built on [Bevy](https://bevyengine.org/) 0.19, exploring keyboard control, motion bounds, and trail rendering. It integrates the [dora-rs](https://dora-rs.ai/) dataflow framework to drive the aircraft from a separate controller ("controller → aircraft").

## Three Binaries

| Binary | Needs `dora` feature | Description |
|--------|:---:|------|
| `aviation` | No | Standalone keyboard version: control the aircraft directly with the keyboard |
| `aviation-dora` | Yes | Aircraft dora node: consumes `cmd` control commands to drive the aircraft, publishes `pose` |
| `aviation-controller` | Yes | Controller dora node: GUI directional buttons + keyboard, outputs `cmd` commands |

All shared logic (components, pure functions, common systems, embedded assets) lives in `src/lib.rs`. The font and aircraft image are **compiled into the binary** via `include_bytes!`, so they no longer depend on an `assets/` directory relative to the working directory — they render correctly under any working directory, including `dora run`.

## Requirements
- Rust toolchain with edition 2024 support (`rustup update stable`, Rust ≥ 1.88).
- A GPU/driver that can run Bevy 0.19 (Vulkan/Metal/OpenGL backends work).
- The [`dora` CLI](https://dora-rs.ai/) (1.0.0-rc series) for the dora versions.

## Getting Started (Standalone Keyboard Version)
```bash
git clone <repo-url>
cd aviation
cargo run
```
The window opens with the title "DORA小飞机模拟器" and a "控制小飞机" label in the top-left corner.

### Controls
- `W` / `↑`: accelerate forward
- `S` / `↓`: reverse thrust
- `A` / `←`: rotate counter-clockwise
- `D` / `→`: rotate clockwise

Movement is clamped to a 1200×640 world (the window matches this size). Default rotation speed is 360°/s and forward speed is 300 units/s; adjust them in the `Player` component in `src/lib.rs`.

## Flight Path Trail
The aircraft records waypoints while moving and renders them as a white line strip using Bevy gizmos. The trail keeps up to 4096 points, dropping the oldest once the cap is reached. Click the “清除轨迹” button in the top-right corner to reset the trail; the button gives hover/press color feedback.

## Running via dora

### 1. Build (release)
```bash
cargo build --release --features dora --bins
```
Artifacts land in `target/release/`: `aviation`, `aviation-dora`, `aviation-controller`.

### 2. Dataflow config (`dataflow.yml`)
```yaml
nodes:
  - id: controller
    path: target/release/aviation-controller
    inputs:
      tick: dora/timer/millis/16   # ~60Hz, drives periodic sends
    outputs: [cmd]

  - id: aviation
    path: target/release/aviation-dora
    inputs:
      cmd: controller/cmd
    outputs: [pose]
```

### 3. Launch
```bash
dora run dataflow.yml
```
Two windows open: the controller (directional buttons `↑ ← → ↓`, which also listens to the keyboard) and the aircraft. Operate the controller to drive the aircraft.

### Data Format
| Direction | id | Arrow type | Payload |
|-----------|-----|-----------|---------|
| controller → aviation | `cmd` | `Float32Array[2]` | `[rotation_factor, movement_factor]`, range [-1, 1] |
| aviation → downstream | `pose` | `Float32Array[3]` | `[x, y, angle]` |

The controller and aircraft bridge the blocking dora event loop and the Bevy main loop via a background thread and `crossbeam-channel`. The controller stops sending when no direction is active, so the aircraft stops as well.

## Architecture
- The `Player` component holds movement/rotation speeds, `AviationPath` stores waypoints, and `DirectionState` captures four-way input and maps it to control factors.
- `apply_control` is the shared motion core (rotate + advance + clamp to bounds), reused by both the keyboard and dora versions.
- Motion runs in Bevy's `FixedUpdate` (60Hz) for stability; rendering/interaction runs in `Update`.
- The font is subset with `pyftsubset` to keep only the glyphs used by the UI (`控制小飞机清除轨迹↑↓←→`), about 3KB.

## Development Workflow
```bash
cargo fmt                                        # format
cargo clippy --all-targets --features dora -- -D warnings   # lint (incl. dora versions)
cargo test                                       # unit tests
cargo run                                        # launch the standalone keyboard version
```

### Rebuilding the font subset after changing UI text
If you add or change UI text, regenerate the font subset (missing glyphs render as boxes):
```bash
pyftsubset source-font.ttf \
  --text="控制小飞机清除轨迹↑↓←→" \
  --output-file=assets/fonts/aviation.ttf \
  --no-hinting --desubroutinize --layout-features='' --name-IDs='' --notdef-outline
```

## Contributing
Follow Conventional Commits (`feat:`, `chore:`, etc.) when pushing changes. Pull requests should describe gameplay impact, list validation commands, and include screenshots or clips when visuals change.
