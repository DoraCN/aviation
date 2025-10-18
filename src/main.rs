use bevy::prelude::*;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
const PATH_POINT_DISTANCE: f32 = 2.0; // 路径点记录的最小间距

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "小飞机模拟器 - Dora".into(),
                // resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (player_movement_system, update_path_system))
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

/// 用于记录小飞机的移动路径点
#[derive(Component)]
struct AviationPath(Vec<Vec2>);

/// 用于标记路径绘图实体的组件
#[derive(Component)]
struct PathEntity;

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

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.into_inner();

    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        movement_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        movement_factor -= 1.0;
    }

    // println!("{rotation_factor:?} {movement_factor:?}");

    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());

    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = movement_factor * ship.movement_speed * time.delta_secs();
    let translation_delta = movement_direction * movement_distance;

    transform.translation += translation_delta;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn update_path_system(query: Single<(&mut AviationPath, &mut Transform), With<Player>>) {
    let (mut path, transform) = query.into_inner();
    let current_position = transform.translation.truncate();

    if let Some(last_point) = path.0.last() {
        if last_point.distance(current_position) < PATH_POINT_DISTANCE {
            // println!("current_position: {current_position:?}");
            path.0.push(current_position);
        }
    }
}
