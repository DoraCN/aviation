#![cfg(feature = "dora")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use aviation::{
    BOUNDS, Player, apply_control, clear_path_button_system, draw_path_system,
    load_embedded_assets, setup, ship_bounding_radius, update_path_system,
};
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, Event, IntoArrow, MetadataParameters};

/// dora 下发的控制指令，取值约定 [-1, 1]。
#[derive(Debug, Clone, Copy)]
struct ControlCommand {
    rotation_factor: f32,
    movement_factor: f32,
}

/// 回传给 dora 下游的飞机位姿。
#[derive(Debug, Clone, Copy)]
struct PoseOutput {
    x: f32,
    y: f32,
    angle: f32,
}

/// 连接 dora 后台线程与 Bevy 主循环的桥接资源。
#[derive(Resource)]
struct DoraBridge {
    commands: Receiver<ControlCommand>,
    poses: Sender<PoseOutput>,
    stop: Arc<AtomicBool>,
}

const INPUT_CMD: &str = "cmd";
const OUTPUT_POSE: &str = "pose";

fn main() {
    let bridge = spawn_dora_thread();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DORA小飞机模拟器 (dora)".into(),
                        resolution: bevy::window::WindowResolution::new(
                            BOUNDS.x as u32,
                            BOUNDS.y as u32,
                        )
                        .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter: format!("{},zenoh=off", bevy::log::DEFAULT_FILTER),
                    ..default()
                }),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(bridge)
        .add_systems(Startup, (load_embedded_assets, setup).chain())
        .add_systems(
            FixedUpdate,
            (dora_input_system, update_path_system, dora_output_system).chain(),
        )
        .add_systems(
            Update,
            (draw_path_system, clear_path_button_system, dora_stop_system),
        )
        .run();
}

/// 启动后台线程运行 dora 节点，并返回与之通信的桥接资源。
fn spawn_dora_thread() -> DoraBridge {
    let (cmd_tx, cmd_rx) = unbounded::<ControlCommand>();
    let (pose_tx, pose_rx) = unbounded::<PoseOutput>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();

    thread::spawn(move || {
        if let Err(err) = run_dora_node(cmd_tx, pose_rx, &thread_stop) {
            eprintln!("dora 节点退出: {err}");
        }
        thread_stop.store(true, Ordering::SeqCst);
    });

    DoraBridge {
        commands: cmd_rx,
        poses: pose_tx,
        stop,
    }
}

/// dora 事件循环：解析 `cmd` 输入转发给 Bevy，并把最新位姿通过 `pose` 输出。
fn run_dora_node(
    cmd_tx: Sender<ControlCommand>,
    pose_rx: Receiver<PoseOutput>,
    stop: &AtomicBool,
) -> eyre::Result<()> {
    let (mut node, mut events) = DoraNode::init_from_env()?;
    let pose_id = DataId::from(OUTPUT_POSE.to_owned());

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } if id.as_str() == INPUT_CMD => {
                if let Some(command) = parse_command(&data) {
                    let _ = cmd_tx.send(command);
                }

                if let Some(pose) = latest(&pose_rx) {
                    node.send_output(
                        pose_id.clone(),
                        MetadataParameters::default(),
                        vec![pose.x, pose.y, pose.angle].into_arrow(),
                    )?;
                }
            }
            Event::Stop(_) => break,
            _ => {}
        }
    }

    stop.store(true, Ordering::SeqCst);
    Ok(())
}

/// 从 Arrow 数据解析控制指令，要求至少包含两个浮点值 `[rotation, movement]`。
fn parse_command(data: &dora_node_api::ArrowData) -> Option<ControlCommand> {
    let values = Vec::<f32>::try_from(data).ok()?;
    if values.len() < 2 {
        return None;
    }
    Some(ControlCommand {
        rotation_factor: values[0].clamp(-1.0, 1.0),
        movement_factor: values[1].clamp(-1.0, 1.0),
    })
}

/// 排空通道，仅保留最后一个元素。
fn latest<T>(rx: &Receiver<T>) -> Option<T> {
    let mut value = None;
    while let Ok(next) = rx.try_recv() {
        value = Some(next);
    }
    value
}

/// 取 dora 最新指令并应用到飞机。
fn dora_input_system(
    time: Res<Time>,
    bridge: Res<DoraBridge>,
    images: Res<Assets<Image>>,
    query: Single<(&Player, &Sprite, &mut Transform)>,
) {
    let Some(command) = latest(&bridge.commands) else {
        return;
    };

    let (ship, sprite, mut transform) = query.into_inner();
    let margin = ship_bounding_radius(sprite, &images);
    apply_control(
        &mut transform,
        ship,
        command.rotation_factor,
        command.movement_factor,
        time.delta_secs(),
        margin,
    );
}

/// 把飞机当前位姿发送到 dora 线程用于回传。
fn dora_output_system(bridge: Res<DoraBridge>, query: Single<&Transform, With<Player>>) {
    let transform = query.into_inner();
    let pose = PoseOutput {
        x: transform.translation.x,
        y: transform.translation.y,
        angle: transform.rotation.to_euler(EulerRot::ZYX).0,
    };
    let _ = bridge.poses.send(pose);
}

/// 当 dora 节点结束时退出 Bevy 应用。
fn dora_stop_system(bridge: Res<DoraBridge>, mut exit: MessageWriter<AppExit>) {
    if bridge.stop.load(Ordering::SeqCst) {
        exit.write(AppExit::Success);
    }
}
