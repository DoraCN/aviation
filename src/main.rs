use bevy::prelude::*;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
const PATH_POINT_DISTANCE: f32 = 2.0; // 路径点记录的最小间距
const MAX_PATH_POINTS: usize = 4096; // 轨迹点上限，超出后丢弃最早的点

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DORA小飞机模拟器".into(),
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

type ClearButtonInteractions<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<ClearPathButton>),
>;

/// 根据旋转/位移输入计算本帧的位移增量。
fn movement_delta(rotation: Quat, movement_factor: f32, speed: f32, delta_secs: f32) -> Vec3 {
    let direction = rotation * Vec3::Y;
    direction * (movement_factor * speed * delta_secs)
}

/// 将位置裁剪到世界边界内，`margin` 为每个方向的内缩量（例如飞机外接半径）。
fn clamp_to_bounds(translation: Vec3, bounds: Vec2, margin: Vec2) -> Vec3 {
    let extents = ((bounds / 2.0) - margin).max(Vec2::ZERO);
    let extents = Vec3::from((extents, 0.0));
    translation.min(extents).max(-extents)
}

/// 判断当前坐标是否需要作为新的轨迹点记录。
fn should_record_point(last_point: Option<Vec2>, current: Vec2, min_distance: f32) -> bool {
    last_point.is_none_or(|last| last.distance(current) >= min_distance)
}

/// 追加轨迹点，并在超过上限时丢弃最早的点。
fn push_path_point(points: &mut Vec<Vec2>, point: Vec2, max_points: usize) {
    points.push(point);
    if max_points > 0 && points.len() > max_points {
        let overflow = points.len() - max_points;
        points.drain(0..overflow);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("images/ship_C.png");

    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("控制小飞机"),
        TextFont {
                    font: asset_server.load("fonts/AlimamaDongFangDaKai-Regular.ttf"),
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
            font: asset_server.load("fonts/AlimamaDongFangDaKai-Regular.ttf"),
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
    images: Res<Assets<Image>>,
    query: Single<(&Player, &Sprite, &mut Transform)>,
) {
    let (ship, sprite, mut transform) = query.into_inner();

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

    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());

    let rotation = transform.rotation;
    transform.translation += movement_delta(
        rotation,
        movement_factor,
        ship.movement_speed,
        time.delta_secs(),
    );

    let margin = ship_bounding_radius(sprite, &images);
    transform.translation = clamp_to_bounds(transform.translation, BOUNDS, margin);
}

/// 计算飞机在任意旋转下都不出界的保守内缩量（外接圆半径）。
fn ship_bounding_radius(sprite: &Sprite, images: &Assets<Image>) -> Vec2 {
    let size = sprite
        .custom_size
        .or_else(|| images.get(&sprite.image).map(|image| image.size_f32()))
        .unwrap_or(Vec2::ZERO);
    Vec2::splat(size.length() / 2.0)
}

fn update_path_system(mut query: Query<(&mut AviationPath, &Transform), With<Player>>) {
    let Some((mut path, transform)) = query.iter_mut().next() else {
        return;
    };

    let current_position = transform.translation.truncate();

    if should_record_point(
        path.points.last().copied(),
        current_position,
        PATH_POINT_DISTANCE,
    ) {
        push_path_point(&mut path.points, current_position, MAX_PATH_POINTS);
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
    mut interactions: ClearButtonInteractions,
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

    if clear_requested && let Some(mut path) = path_query.iter_mut().next() {
        path.points.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_delta_moves_along_facing_direction() {
        let delta = movement_delta(Quat::IDENTITY, 1.0, 300.0, 0.5);
        assert!((delta - Vec3::new(0.0, 150.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn movement_delta_reverses_with_negative_factor() {
        let delta = movement_delta(Quat::IDENTITY, -1.0, 300.0, 0.5);
        assert!((delta - Vec3::new(0.0, -150.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn movement_delta_follows_rotation() {
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let delta = movement_delta(rotation, 1.0, 100.0, 1.0);
        assert!((delta - Vec3::new(-100.0, 0.0, 0.0)).length() < 1e-3);
    }

    #[test]
    fn clamp_keeps_position_inside_bounds() {
        let clamped = clamp_to_bounds(Vec3::new(10.0, 20.0, 0.0), BOUNDS, Vec2::ZERO);
        assert_eq!(clamped, Vec3::new(10.0, 20.0, 0.0));
    }

    #[test]
    fn clamp_limits_position_to_extents() {
        let clamped = clamp_to_bounds(Vec3::new(9999.0, -9999.0, 0.0), BOUNDS, Vec2::ZERO);
        assert_eq!(clamped, Vec3::new(600.0, -320.0, 0.0));
    }

    #[test]
    fn clamp_accounts_for_margin() {
        let clamped = clamp_to_bounds(
            Vec3::new(9999.0, 9999.0, 0.0),
            BOUNDS,
            Vec2::new(50.0, 20.0),
        );
        assert_eq!(clamped, Vec3::new(550.0, 300.0, 0.0));
    }

    #[test]
    fn clamp_margin_larger_than_bounds_collapses_to_origin() {
        let clamped = clamp_to_bounds(
            Vec3::new(100.0, 100.0, 0.0),
            BOUNDS,
            Vec2::new(9999.0, 9999.0),
        );
        assert_eq!(clamped, Vec3::ZERO);
    }

    #[test]
    fn record_first_point_always() {
        assert!(should_record_point(
            None,
            Vec2::new(1.0, 1.0),
            PATH_POINT_DISTANCE
        ));
    }

    #[test]
    fn skip_point_within_min_distance() {
        let last = Some(Vec2::ZERO);
        assert!(!should_record_point(
            last,
            Vec2::new(1.0, 0.0),
            PATH_POINT_DISTANCE
        ));
    }

    #[test]
    fn record_point_beyond_min_distance() {
        let last = Some(Vec2::ZERO);
        assert!(should_record_point(
            last,
            Vec2::new(3.0, 0.0),
            PATH_POINT_DISTANCE
        ));
    }

    #[test]
    fn push_point_appends_normally() {
        let mut points = vec![Vec2::ZERO];
        push_path_point(&mut points, Vec2::new(1.0, 1.0), 8);
        assert_eq!(points, vec![Vec2::ZERO, Vec2::new(1.0, 1.0)]);
    }

    #[test]
    fn push_point_drops_oldest_when_over_capacity() {
        let mut points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ];
        push_path_point(&mut points, Vec2::new(3.0, 0.0), 3);
        assert_eq!(points.len(), 3);
        assert_eq!(points.first().copied(), Some(Vec2::new(1.0, 0.0)));
        assert_eq!(points.last().copied(), Some(Vec2::new(3.0, 0.0)));
    }
}
