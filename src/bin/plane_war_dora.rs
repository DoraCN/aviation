#![cfg(feature = "dora")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use aviation::plane_war::{DoraBridge, GamePlugin, InputMode, PlayerIntent};
use bevy::prelude::*;
use crossbeam_channel::unbounded;
use dora_node_api::{DoraNode, Event};

const INPUT_CMD: &str = "cmd";

fn main() {
    let (cmd_tx, cmd_rx) = unbounded::<PlayerIntent>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();

    thread::spawn(move || {
        if let Err(err) = run_dora_node(cmd_tx, &thread_stop) {
            eprintln!("plane-war dora node exit: {err}");
        }
        thread_stop.store(true, Ordering::SeqCst);
    });

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Plane War (dora)".into(),
                        resolution: bevy::window::WindowResolution::new(480, 800)
                            .with_scale_factor_override(1.0),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter: format!("{},zenoh=off", bevy::log::DEFAULT_FILTER),
                    ..default()
                }),
        )
        .insert_resource(InputMode::Dora)
        .insert_resource(DoraBridge {
            commands: cmd_rx,
            stop,
        })
        .add_plugins(GamePlugin)
        .run();
}

fn run_dora_node(
    cmd_tx: crossbeam_channel::Sender<PlayerIntent>,
    stop: &AtomicBool,
) -> eyre::Result<()> {
    let (_node, mut events) = DoraNode::init_from_env()?;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { ref id, data, .. } if id.as_str() == INPUT_CMD => {
                let intent = parse_cmd(&data);
                let _ = cmd_tx.send(intent);
            }
            Event::Stop(_) => break,
            _ => {}
        }
    }

    stop.store(true, Ordering::SeqCst);
    Ok(())
}

fn parse_cmd(data: &dora_node_api::ArrowData) -> PlayerIntent {
    let values = Vec::<f32>::try_from(data).unwrap_or_default();
    let h = *values.first().unwrap_or(&0.0); // rotation_factor (left-right)
    let v = values.get(1).copied().unwrap_or(0.0); // movement_factor (up-down)
    let fire = values.get(2).copied().unwrap_or(0.0) > 0.5;
    PlayerIntent {
        move_x: -h, // left (positive rotation) → move left
        move_y: v,  // up (positive movement) → move up
        fire,
    }
}
