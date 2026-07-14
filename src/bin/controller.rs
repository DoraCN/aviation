#![cfg(feature = "dora")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use aviation::{DirectionState, load_embedded_font};
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, Event, IntoArrow, MetadataParameters};

const INPUT_TICK: &str = "tick";
const OUTPUT_CMD: &str = "cmd";

const BTN_SIZE: Val = Val::Px(64.0);
const BTN_COLOR: Color = Color::srgb(0.2, 0.5, 0.8);
const BTN_ACTIVE: Color = Color::srgb(0.2, 0.7, 1.0);

#[derive(Component, Clone, Copy)]
enum DirButton {
    Up,
    Down,
    Left,
    Right,
    Fire,
}

#[derive(Resource)]
struct DoraChannel {
    direction_tx: Sender<LatestDirection>,
    stop: Arc<AtomicBool>,
}

fn main() {
    let (dir_tx, dir_rx) = unbounded::<LatestDirection>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();

    thread::spawn(move || {
        if let Err(err) = run_dora_node(dir_rx, &thread_stop) {
            eprintln!("dora controller node exit: {err}");
        }
        thread_stop.store(true, Ordering::SeqCst);
    });

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DORA Plane Controller".into(),
                        resolution: bevy::window::WindowResolution::new(320, 380)
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
        .insert_resource(DoraChannel {
            direction_tx: dir_tx,
            stop,
        })
        .insert_resource(LatestDirection::default())
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                input_direction_system,
                button_visual_system,
                send_direction_system,
            )
                .chain(),
        )
        .add_systems(Update, dora_stop_system)
        .run();
}

fn run_dora_node(dir_rx: Receiver<LatestDirection>, stop: &AtomicBool) -> eyre::Result<()> {
    let (mut node, mut events) = DoraNode::init_from_env()?;
    let cmd_id = DataId::from(OUTPUT_CMD.to_owned());
    let mut latest: Option<LatestDirection> = None;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { ref id, .. } if id.as_str() == INPUT_TICK => {
                while let Ok(next) = dir_rx.try_recv() {
                    latest = Some(next);
                }

                let pending = if let Some(ref mut ld) = latest {
                    let has_fire = ld.fire_pending > 0;
                    if has_fire {
                        ld.fire_pending -= 1;
                    }
                    let mut state = ld.dir;
                    state.fire = has_fire;
                    state
                } else {
                    DirectionState::default()
                };

                if pending.is_any_pressed() {
                    node.send_output(
                        cmd_id.clone(),
                        MetadataParameters::default(),
                        vec![
                            pending.rotation_factor(),
                            pending.movement_factor(),
                            pending.fire_factor(),
                            pending.retry_factor(),
                            pending.exit_factor(),
                        ]
                        .into_arrow(),
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

fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = load_embedded_font(&mut fonts);

    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|parent| {
            // ↑
            parent
                .spawn(button_bundle(DirButton::Up))
                .with_children(|p| {
                    p.spawn(label_text("↑", &font));
                });

            // ← →
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(button_bundle(DirButton::Left))
                        .with_children(|p| {
                            p.spawn(label_text("←", &font));
                        });
                    row.spawn(button_bundle(DirButton::Right))
                        .with_children(|p| {
                            p.spawn(label_text("→", &font));
                        });
                });

            // ↓
            parent
                .spawn(button_bundle(DirButton::Down))
                .with_children(|p| {
                    p.spawn(label_text("↓", &font));
                });

            // FIRE
            parent.spawn(Node {
                height: Val::Px(8.0),
                ..default()
            });
            parent
                .spawn(button_bundle(DirButton::Fire))
                .with_children(|p| {
                    p.spawn((
                        Text::new("FIRE"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor::WHITE,
                    ));
                });
        });
}

fn button_bundle(kind: DirButton) -> (Button, Node, BackgroundColor, DirButton) {
    (
        Button,
        Node {
            width: BTN_SIZE,
            height: BTN_SIZE,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BTN_COLOR),
        kind,
    )
}

fn label_text(label: &str, font: &Handle<Font>) -> (Text, TextFont, TextColor) {
    (
        Text::new(label.to_owned()),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor::WHITE,
    )
}

#[derive(Resource, Default)]
struct LatestDirection {
    dir: DirectionState,
    fire_pending: u32,
}

fn input_direction_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    buttons: Query<(&Interaction, &DirButton)>,
    mut latest: ResMut<LatestDirection>,
) {
    let mut dir = DirectionState {
        up: keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW),
        down: keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS),
        left: keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA),
        right: keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD),
        retry: keyboard_input.pressed(KeyCode::KeyR),
        exit: keyboard_input.pressed(KeyCode::KeyE),
        ..Default::default()
    };

    for (interaction, &kind) in &buttons {
        if matches!(interaction, Interaction::Pressed) {
            match kind {
                DirButton::Up => dir.up = true,
                DirButton::Down => dir.down = true,
                DirButton::Left => dir.left = true,
                DirButton::Right => dir.right = true,
                DirButton::Fire => {
                    dir.fire = true;
                    latest.fire_pending = latest.fire_pending.saturating_add(1);
                }
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyJ) {
        latest.fire_pending = latest.fire_pending.saturating_add(1);
    }

    latest.dir = dir;
}

fn button_visual_system(
    latest: Res<LatestDirection>,
    mut colors: Query<(&DirButton, &mut BackgroundColor)>,
) {
    for (&kind, mut color) in &mut colors {
        let active = match kind {
            DirButton::Up => latest.dir.up,
            DirButton::Down => latest.dir.down,
            DirButton::Left => latest.dir.left,
            DirButton::Right => latest.dir.right,
            DirButton::Fire => latest.dir.fire,
        };
        *color = if active {
            BTN_ACTIVE.into()
        } else {
            BTN_COLOR.into()
        };
    }
}

fn send_direction_system(channel: Res<DoraChannel>, latest: Res<LatestDirection>) {
    let _ = channel.direction_tx.send(LatestDirection {
        dir: latest.dir,
        fire_pending: latest.fire_pending,
    });
}

fn dora_stop_system(channel: Res<DoraChannel>, mut exit: MessageWriter<AppExit>) {
    if channel.stop.load(Ordering::SeqCst) {
        exit.write(AppExit::Success);
    }
}
