use aviation::{
    BOUNDS, DirectionState, Player, apply_control, clear_path_button_system, draw_path_system,
    load_embedded_assets, setup, ship_bounding_radius, update_path_system,
};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DORA小飞机模拟器".into(),
                    resolution: bevy::window::WindowResolution::new(
                        BOUNDS.x as u32,
                        BOUNDS.y as u32,
                    )
                    .with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, (load_embedded_assets, setup).chain())
        .add_systems(FixedUpdate, (player_movement_system, update_path_system))
        .add_systems(Update, (draw_path_system, clear_path_button_system))
        .run();
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    images: Res<Assets<Image>>,
    query: Single<(&Player, &Sprite, &mut Transform)>,
) {
    let (ship, sprite, mut transform) = query.into_inner();

    let dir = DirectionState {
        up: keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW),
        down: keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS),
        left: keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA),
        right: keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD),
        fire: false,
        retry: false,
        exit: false,
    };

    let margin = ship_bounding_radius(sprite, &images);
    apply_control(
        &mut transform,
        ship,
        dir.rotation_factor(),
        dir.movement_factor(),
        time.delta_secs(),
        margin,
    );
}
