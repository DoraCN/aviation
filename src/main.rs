use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            (DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "小飞机模拟器 - Dora".into(),
                    resolution: (800., 600.).into(),
                    ..default()
                }),
                ..default()
            })),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        // .add_systems(FixedUpdate, ())
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

fn setup(mut commands: Commands, assert_server: Res<AssetServer>) {
    let ship_handel = assert_server.load("images/ship_C.png");

    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("控制小飞机"),
        TextFont {
            font: assert_server.load("fonts/Alibaba_PuHuiTi_2.0_55_Regular_55_Regular.ttf"),
            font_size: 33.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    commands.spawn((
        Sprite::from_image(ship_handel),
        Player {
            movement_speed: 300.0,
            rotation_speed: f32::to_radians(360.0),
        },
    ));
}
