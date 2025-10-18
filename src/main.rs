use bevy::prelude::*;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
const PATH_POINT_DISTANCE: f32 = 2.0; // 路径点记录的最小间距

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DORA小飞机模拟器".into(),
                // resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (player_movement_system, update_path_system))
        .add_systems(Update, (draw_path_system, clear_path_button_system))
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

/// 用于记录小飞机的移动路径点
#[derive(Component, Default)]
struct AviationPath {
    points: Vec<Vec2>,
}

#[derive(Component)]
struct ClearPathButton;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("images/ship_C.png");

    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("控制小飞机"),
        TextFont {
            font: asset_server.load("fonts/Alibaba_PuHuiTi_2.0_55_Regular_55_Regular.ttf"),
            font_size: 26.0,
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
        Sprite::from_image(ship_handle),
        Player {
            movement_speed: 300.0,
            rotation_speed: f32::to_radians(360.0),
        },
        AviationPath::default(),
    ));

    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 0.45, 0.8)),
            BorderRadius::all(Val::Px(20.0)),
            ClearPathButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("清除轨迹"),
                TextFont {
                    font: asset_server.load("fonts/Alibaba_PuHuiTi_2.0_55_Regular_55_Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor::WHITE,
            ));
        });
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

fn update_path_system(mut query: Query<(&mut AviationPath, &Transform), With<Player>>) {
    let Some((mut path, transform)) = query.iter_mut().next() else {
        return;
    };

    let current_position = transform.translation.truncate();

    let should_record = path.points.last().map_or(true, |last_point| {
        last_point.distance(current_position) >= PATH_POINT_DISTANCE
    });

    if should_record {
        path.points.push(current_position);
    }
}

fn draw_path_system(path_query: Query<&AviationPath, With<Player>>, mut gizmos: Gizmos) {
    let Some(path) = path_query.iter().next() else {
        return;
    };

    if path.points.len() < 2 {
        return;
    }

    let points = path
        .points
        .iter()
        .map(|point| Vec3::new(point.x, point.y, 0.0))
        .collect::<Vec<_>>();

    gizmos.linestrip(points, Color::WHITE);
}

fn clear_path_button_system(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ClearPathButton>),
    >,
    mut path_query: Query<&mut AviationPath, With<Player>>,
) {
    let normal = Color::srgb(0.0, 0.45, 0.8);
    let hovered = Color::srgb(0.0, 0.55, 0.95);
    let pressed = Color::srgb(0.0, 0.35, 0.65);

    let mut clear_requested = false;

    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = pressed.into();
                clear_requested = true;
            }
            Interaction::Hovered => *color = hovered.into(),
            Interaction::None => *color = normal.into(),
        }
    }

    if clear_requested {
        if let Some(mut path) = path_query.iter_mut().next() {
            path.points.clear();
        }
    }
}
