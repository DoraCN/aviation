#![cfg(feature = "dora")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use aviation::plane_war::{
    DoraBridge, GamePlugin, GameState, HighScore, InputMode, PlayerIntent, Score,
};
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

    let cmd_rx_clone = cmd_rx.clone();

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
        .add_systems(
            Update,
            dora_game_over_input(cmd_rx_clone).run_if(in_state(GameState::GameOver)),
        )
        .run();
}

#[expect(clippy::type_complexity)]
fn dora_game_over_input(
    cmd_rx: crossbeam_channel::Receiver<PlayerIntent>,
) -> impl FnMut(ResMut<NextState<GameState>>, Res<Score>, ResMut<HighScore>, MessageWriter<AppExit>)
{
    move |mut next, score, mut high, mut exit| {
        let mut latest: Option<PlayerIntent> = None;
        while let Ok(cmd) = cmd_rx.try_recv() {
            latest = Some(cmd);
        }
        let Some(cmd) = latest else {
            return;
        };
        if cmd.retry {
            if score.0 > high.0 {
                high.0 = score.0;
            }
            next.set(GameState::Playing);
        }
        if cmd.exit {
            if score.0 > high.0 {
                high.0 = score.0;
            }
            exit.write(AppExit::Success);
        }
    }
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
    let h = *values.first().unwrap_or(&0.0);
    let v = values.get(1).copied().unwrap_or(0.0);
    let fire = values.get(2).copied().unwrap_or(0.0) > 0.5;
    let retry = values.get(3).copied().unwrap_or(0.0) > 0.5;
    let exit = values.get(4).copied().unwrap_or(0.0) > 0.5;
    PlayerIntent {
        move_x: -h,
        move_y: v,
        fire,
        retry,
        exit,
    }
}
