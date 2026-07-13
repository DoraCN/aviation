pub mod plane_war;
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType};
use bevy::prelude::*;

pub const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
pub const PATH_POINT_DISTANCE: f32 = 2.0; // 路径点记录的最小间距
pub const MAX_PATH_POINTS: usize = 4096; // 轨迹点上限，超出后丢弃最早的点

const BUTTON_NORMAL: Color = Color::srgb(0.0, 0.45, 0.8);
const BUTTON_HOVERED: Color = Color::srgb(0.0, 0.55, 0.95);
const BUTTON_PRESSED: Color = Color::srgb(0.0, 0.35, 0.65);

// 编译期嵌入的资源，避免运行时依赖工作目录下的 assets 目录。
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/aviation.ttf");
const SHIP_IMAGE_BYTES: &[u8] = include_bytes!("../assets/images/ship_C.png");

#[derive(Component)]
pub struct Player {
    pub movement_speed: f32,
    pub rotation_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            movement_speed: 300.0,
            rotation_speed: f32::to_radians(360.0),
        }
    }
}

/// 用于记录小飞机的移动路径点
#[derive(Component, Default)]
pub struct AviationPath {
    pub points: Vec<Vec2>,
}

#[derive(Component)]
pub struct ClearPathButton;

type ClearButtonInteractions<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<ClearPathButton>),
>;

/// 根据旋转/位移输入计算本帧的位移增量。
pub fn movement_delta(rotation: Quat, movement_factor: f32, speed: f32, delta_secs: f32) -> Vec3 {
    let direction = rotation * Vec3::Y;
    direction * (movement_factor * speed * delta_secs)
}

/// 将位置裁剪到世界边界内，`margin` 为每个方向的内缩量（例如飞机外接半径）。
pub fn clamp_to_bounds(translation: Vec3, bounds: Vec2, margin: Vec2) -> Vec3 {
    let extents = ((bounds / 2.0) - margin).max(Vec2::ZERO);
    let extents = Vec3::from((extents, 0.0));
    translation.min(extents).max(-extents)
}

/// 判断当前坐标是否需要作为新的轨迹点记录。
pub fn should_record_point(last_point: Option<Vec2>, current: Vec2, min_distance: f32) -> bool {
    last_point.is_none_or(|last| last.distance(current) >= min_distance)
}

/// 追加轨迹点，并在超过上限时丢弃最早的点。
pub fn push_path_point(points: &mut Vec<Vec2>, point: Vec2, max_points: usize) {
    points.push(point);
    if max_points > 0 && points.len() > max_points {
        let overflow = points.len() - max_points;
        points.drain(0..overflow);
    }
}

/// 四个方向的状态标记。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectionState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl DirectionState {
    /// 方向 → 控制系数（与键盘版完全一致），取值 [-1, 1]。
    pub fn rotation_factor(self) -> f32 {
        (self.left as u8 as f32) - (self.right as u8 as f32)
    }

    pub fn movement_factor(self) -> f32 {
        (self.up as u8 as f32) - (self.down as u8 as f32)
    }

    /// 是否有任意方向被激活。
    pub fn is_any_pressed(self) -> bool {
        self.up || self.down || self.left || self.right
    }
}

/// 将旋转/位移控制量应用到飞机变换：旋转、前进并裁剪到边界内。
///
/// 键盘版与 dora 版共用此逻辑，`rotation_factor` / `movement_factor` 取值约定为 [-1, 1]。
pub fn apply_control(
    transform: &mut Transform,
    ship: &Player,
    rotation_factor: f32,
    movement_factor: f32,
    delta_secs: f32,
    margin: Vec2,
) {
    transform.rotate_z(rotation_factor * ship.rotation_speed * delta_secs);

    let rotation = transform.rotation;
    transform.translation +=
        movement_delta(rotation, movement_factor, ship.movement_speed, delta_secs);
    transform.translation = clamp_to_bounds(transform.translation, BOUNDS, margin);
}

/// 计算飞机在任意旋转下都不出界的保守内缩量（外接圆半径）。
pub fn ship_bounding_radius(sprite: &Sprite, images: &Assets<Image>) -> Vec2 {
    let size = sprite
        .custom_size
        .or_else(|| images.get(&sprite.image).map(|image| image.size_f32()))
        .unwrap_or(Vec2::ZERO);
    Vec2::splat(size.length() / 2.0)
}

/// 编译期嵌入资源的运行时句柄。
#[derive(Resource, Clone)]
pub struct EmbeddedAssets {
    pub font: Handle<Font>,
    pub ship: Handle<Image>,
}

/// 将嵌入的字体解码并注册为 Bevy 资源，返回可复用的句柄。
pub fn load_embedded_font(fonts: &mut Assets<Font>) -> Handle<Font> {
    fonts.add(Font::from_bytes(FONT_BYTES.to_vec()))
}

/// 将嵌入的 PNG 精灵解码并注册为 Bevy 资源，返回可复用的句柄。
pub fn load_embedded_ship(images: &mut Assets<Image>) -> Handle<Image> {
    let image = Image::from_buffer(
        SHIP_IMAGE_BYTES,
        ImageType::Format(ImageFormat::Png),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .expect("failed to decode embedded ship_C.png");
    images.add(image)
}

/// Startup 系统：注册嵌入资源并存入 [`EmbeddedAssets`]，需在 `setup` 之前运行。
pub fn load_embedded_assets(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(EmbeddedAssets {
        font: load_embedded_font(&mut fonts),
        ship: load_embedded_ship(&mut images),
    });
}

/// 生成通用场景：相机、标题标签、飞机、清除轨迹按钮。
pub fn setup(mut commands: Commands, assets: Res<EmbeddedAssets>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("控制小飞机"),
        TextFont {
            font: assets.font.clone().into(),
            font_size: FontSize::Px(26.0),
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
        Sprite::from_image(assets.ship.clone()),
        Player::default(),
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
                border_radius: BorderRadius::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(BUTTON_NORMAL),
            ClearPathButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("清除轨迹"),
                TextFont {
                    font: assets.font.clone().into(),
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
        });
}

pub fn update_path_system(mut query: Query<(&mut AviationPath, &Transform), With<Player>>) {
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

pub fn draw_path_system(path_query: Query<&AviationPath, With<Player>>, mut gizmos: Gizmos) {
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

pub fn clear_path_button_system(
    mut interactions: ClearButtonInteractions,
    mut path_query: Query<&mut AviationPath, With<Player>>,
) {
    let mut clear_requested = false;

    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                clear_requested = true;
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON_NORMAL.into(),
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
    fn apply_control_rotates_and_moves_within_bounds() {
        let ship = Player::default();
        let mut transform = Transform::default();
        apply_control(&mut transform, &ship, 0.0, 1.0, 0.1, Vec2::ZERO);
        assert!(transform.translation.y > 0.0);
        assert!(transform.translation.y <= BOUNDS.y / 2.0);
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

    mod direction_state {
        use super::DirectionState;

        #[test]
        fn neutral_all_zero() {
            let d = DirectionState::default();
            assert!(!d.is_any_pressed());
            assert_eq!(d.rotation_factor(), 0.0);
            assert_eq!(d.movement_factor(), 0.0);
        }

        #[test]
        fn forward_movement_factor_is_positive() {
            let d = DirectionState {
                up: true,
                ..Default::default()
            };
            assert_eq!(d.movement_factor(), 1.0);
            assert_eq!(d.rotation_factor(), 0.0);
        }

        #[test]
        fn backward_movement_factor_is_negative() {
            let d = DirectionState {
                down: true,
                ..Default::default()
            };
            assert_eq!(d.movement_factor(), -1.0);
        }

        #[test]
        fn left_rotation_factor_is_positive() {
            let d = DirectionState {
                left: true,
                ..Default::default()
            };
            assert_eq!(d.rotation_factor(), 1.0);
        }

        #[test]
        fn right_rotation_factor_is_negative() {
            let d = DirectionState {
                right: true,
                ..Default::default()
            };
            assert_eq!(d.rotation_factor(), -1.0);
        }

        #[test]
        fn up_and_down_cancel() {
            let d = DirectionState {
                up: true,
                down: true,
                ..Default::default()
            };
            assert_eq!(d.movement_factor(), 0.0);
        }

        #[test]
        fn left_and_right_cancel() {
            let d = DirectionState {
                left: true,
                right: true,
                ..Default::default()
            };
            assert_eq!(d.rotation_factor(), 0.0);
        }

        #[test]
        fn is_any_pressed() {
            assert!(
                DirectionState {
                    up: true,
                    ..Default::default()
                }
                .is_any_pressed()
            );
            assert!(!DirectionState::default().is_any_pressed());
        }
    }
}
