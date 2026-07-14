use bevy::prelude::*;
use rand::Rng;

use super::GameState;
use super::components::*;
use super::resources::*;

const FIGHTER_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 500.0;
const FIGHTER_RADIUS: f32 = 25.0;
const BULLET_RADIUS: f32 = 5.0;
const ENEMY_64_RADIUS: f32 = 28.0;
const ENEMY_100_RADIUS: f32 = 44.0;
const BOMB_RADIUS: f32 = 30.0;

// ══════════════════════════════════════════
//  玩家输入
// ══════════════════════════════════════════

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<PlayerIntent>,
    input_mode: Res<InputMode>,
) {
    if *input_mode == InputMode::Dora {
        return;
    }
    let mut mx = 0.0f32;
    let mut my = 0.0f32;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        mx -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        mx += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        my += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        my -= 1.0;
    }

    intent.move_x = mx;
    intent.move_y = my;
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyJ) {
        intent.fire = true;
    }
}

/// dora 版输入：从 DoraBridge channel 读取最新 PlayerIntent。
#[cfg(feature = "dora")]
pub fn dora_input_system(bridge: Res<DoraBridge>, mut intent: ResMut<PlayerIntent>) {
    let mut latest = None;
    while let Ok(cmd) = bridge.commands.try_recv() {
        latest = Some(cmd);
    }
    *intent = latest.unwrap_or_default();
}

// ══════════════════════════════════════════
//  移动
// ══════════════════════════════════════════

pub fn move_player(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    mut query: Query<&mut Transform, With<Fighter>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    transform.translation.x += intent.move_x * FIGHTER_SPEED * dt;
    transform.translation.y += intent.move_y * FIGHTER_SPEED * dt;

    transform.translation.x = transform.translation.x.clamp(30.0, WORLD_W - 30.0);
    transform.translation.y = transform.translation.y.clamp(40.0, WORLD_H - 40.0);
}

pub fn move_entities(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;
    }
}

// ══════════════════════════════════════════
//  开火
// ══════════════════════════════════════════

pub fn player_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut cd: ResMut<FireCooldown>,
    mut intent: ResMut<PlayerIntent>,
    fighter_query: Query<&Transform, With<Fighter>>,
) {
    cd.0.tick(time.delta());
    if !cd.0.just_finished() || !intent.fire {
        return;
    }
    let Ok(t) = fighter_query.single() else {
        return;
    };

    intent.fire = false;
    cd.0.reset();
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.9, 0.2),
            custom_size: Some(Vec2::new(6.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(t.translation.x, t.translation.y + 30.0, 0.0),
        Velocity(Vec2::new(0.0, BULLET_SPEED)),
        CircleCollider {
            radius: BULLET_RADIUS,
        },
        Bullet,
        OffscreenDespawn,
    ));
}

// ══════════════════════════════════════════
//  敌人 & 炸弹生成
// ══════════════════════════════════════════

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: ResMut<SpawnTimers>,
    difficulty: Res<Difficulty>,
    assets: Res<GameAssets>,
) {
    timers.enemy.tick(time.delta());
    if !timers.enemy.just_finished() {
        return;
    }

    let mut rng = rand::rng();
    let is_big = rng.random::<f32>() < difficulty.big_enemy_chance;
    let (handle, score, radius) = if is_big && !assets.enemies_100.is_empty() {
        let idx = rng.random_range(0..assets.enemies_100.len());
        (assets.enemies_100[idx].clone(), 2, ENEMY_100_RADIUS)
    } else {
        let idx = rng.random_range(0..assets.enemies_64.len());
        (assets.enemies_64[idx].clone(), 1, ENEMY_64_RADIUS)
    };

    let x = rng.random_range(40.0..(WORLD_W - 40.0));
    let speed = rng.random_range(difficulty.enemy_speed_range.0..difficulty.enemy_speed_range.1);

    commands.spawn((
        Sprite::from_image(handle),
        Transform::from_xyz(x, WORLD_H + 30.0, 0.0),
        Velocity(Vec2::new(0.0, -speed)),
        CircleCollider { radius },
        Enemy(score),
        OffscreenDespawn,
    ));
}

pub fn spawn_bombs(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: ResMut<SpawnTimers>,
    difficulty: Res<Difficulty>,
    assets: Res<GameAssets>,
) {
    timers.bomb.tick(time.delta());
    if !timers.bomb.just_finished() {
        return;
    }

    let mut rng = rand::rng();
    let x = rng.random_range(60.0..(WORLD_W - 60.0));
    let speed = rng.random_range(difficulty.bomb_speed_range.0..difficulty.bomb_speed_range.1);

    commands.spawn((
        Sprite::from_image(assets.bomb_img.clone()),
        Transform::from_xyz(x, WORLD_H + 30.0, 0.0),
        Velocity(Vec2::new(0.0, -speed)),
        CircleCollider {
            radius: BOMB_RADIUS,
        },
        Bomb,
        OffscreenDespawn,
    ));
}

// ══════════════════════════════════════════
//  碰撞
// ══════════════════════════════════════════

fn circles_overlap(a_pos: Vec2, a_r: f32, b_pos: Vec2, b_r: f32) -> bool {
    a_pos.distance(b_pos) < a_r + b_r
}

pub fn bullet_enemy_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &CircleCollider), With<Bullet>>,
    enemies: Query<(Entity, &Transform, &CircleCollider, &Enemy)>,
    mut score: ResMut<Score>,
) {
    for (b_e, b_t, b_c) in &bullets {
        for (e_e, e_t, e_c, enemy) in &enemies {
            if circles_overlap(
                b_t.translation.truncate(),
                b_c.radius,
                e_t.translation.truncate(),
                e_c.radius,
            ) {
                commands.entity(b_e).despawn();
                commands.entity(e_e).despawn();
                score.0 += enemy.0;
                break; // 一发子弹只命中一个敌人
            }
        }
    }
}

type EnemyCollidersQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static CircleCollider), (Without<Bomb>, Without<Bullet>)>;

pub fn player_collision(
    mut commands: Commands,
    fighter_query: Query<(Entity, &Transform), With<Fighter>>,
    enemies: EnemyCollidersQuery,
    bombs: Query<(&Transform, &CircleCollider), With<Bomb>>,
    mut explosion_timer: ResMut<ExplosionTimer>,
    assets: Res<GameAssets>,
) {
    let Ok((f_e, f_t)) = fighter_query.single() else {
        return;
    };
    let f_pos = f_t.translation.truncate();

    let hit = enemies.iter().any(|(e_t, e_c)| {
        circles_overlap(
            f_pos,
            FIGHTER_RADIUS,
            e_t.translation.truncate(),
            e_c.radius,
        )
    }) || bombs.iter().any(|(b_t, b_c)| {
        circles_overlap(
            f_pos,
            FIGHTER_RADIUS,
            b_t.translation.truncate(),
            b_c.radius,
        )
    });

    if hit && explosion_timer.0.is_none() {
        play_bomb_sfx(&mut commands, &assets);
        commands.entity(f_e).despawn();
        commands.spawn((
            Sprite {
                image: assets.fighter_bomb.clone(),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_translation(f_t.translation),
            FighterBomb,
        ));
        explosion_timer.0 = Some(Timer::from_seconds(0.5, TimerMode::Once));
    }
}

pub fn check_explosion_done(
    time: Res<Time>,
    mut explosion_timer: ResMut<ExplosionTimer>,
    mut next: ResMut<NextState<GameState>>,
    mut bomb_query: Query<Entity, With<FighterBomb>>,
    mut commands: Commands,
) {
    if let Some(ref mut timer) = explosion_timer.0 {
        timer.tick(time.delta());
        if timer.just_finished() {
            explosion_timer.0 = None;
            for e in &mut bomb_query {
                commands.entity(e).despawn();
            }
            next.set(GameState::GameOver);
        }
    }
}

fn play_bomb_sfx(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        bevy::audio::AudioPlayer::<bevy::audio::AudioSource>::new(assets.bomb_sfx.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

// ══════════════════════════════════════════
//  回收出界
// ══════════════════════════════════════════

pub fn despawn_offscreen(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<OffscreenDespawn>>,
) {
    for (entity, transform) in &query {
        let p = transform.translation;
        if p.y < -60.0 || p.x < -60.0 || p.x > WORLD_W + 60.0 || p.y > WORLD_H + 60.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ══════════════════════════════════════════
//  难度提升
// ══════════════════════════════════════════

pub fn advance_difficulty(
    time: Res<Time>,
    mut timers: ResMut<SpawnTimers>,
    mut difficulty: ResMut<Difficulty>,
) {
    timers.difficulty.tick(time.delta());
    if !timers.difficulty.just_finished() {
        return;
    }

    difficulty.level += 1;
    let lv = difficulty.level as f32;

    difficulty.enemy_interval = (1.2 - lv * 0.08).max(0.3);
    difficulty.enemy_speed_range = (100.0 + lv * 15.0, 180.0 + lv * 20.0);
    difficulty.bomb_interval = (4.0 - lv * 0.3).max(1.5);
    difficulty.bomb_speed_range = (120.0 + lv * 27.0, 200.0 + lv * 22.0);
    difficulty.big_enemy_chance = (0.2 + lv * 0.05).min(0.55);

    timers.enemy = Timer::from_seconds(difficulty.enemy_interval, TimerMode::Repeating);
}

// ══════════════════════════════════════════
//  卷轴背景
// ══════════════════════════════════════════

pub fn spawn_background(mut commands: Commands) {
    for row in 0..3 {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.04, 0.04, 0.10),
                custom_size: Some(Vec2::new(WORLD_W, WORLD_H + 2.0)),
                ..default()
            },
            Transform::from_xyz(0.0, row as f32 * WORLD_H, -10.0),
            ScrollingBg,
            Velocity(Vec2::new(0.0, -40.0)),
        ));
    }
    // 装饰星星
    let mut rng = rand::rng();
    for _ in 0..60 {
        let x = rng.random_range(0.0..WORLD_W);
        let y = rng.random_range(0.0..WORLD_H);
        let size = rng.random_range(1.0..3.0);
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.6),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, -5.0),
            ScrollingBg,
            Velocity(Vec2::new(0.0, -rng.random_range(20.0..60.0))),
            OffscreenDespawn,
        ));
    }
}

pub fn scroll_background(
    mut query: Query<(&mut Transform, &Velocity), With<ScrollingBg>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut transform, vel) in &mut query {
        transform.translation.y += vel.y * dt;
        if transform.translation.y < -WORLD_H {
            transform.translation.y += WORLD_H * 3.0;
        }
    }
}

// ══════════════════════════════════════════
//  HUD
// ══════════════════════════════════════════

pub fn spawn_hud(mut commands: Commands, high: Res<HighScore>) {
    commands.spawn((
        Text::new(format!("Score: 0  Best: {}", high.0)),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor::WHITE,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Auto,
            right: Val::Auto,
            ..default()
        },
    ));
}

pub fn update_hud(score: Res<Score>, high: Res<HighScore>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        *text = Text::new(format!("Score: {}  Best: {}", score.0, high.0));
    }
}

// ══════════════════════════════════════════
//  背景音乐
// ══════════════════════════════════════════

pub fn start_bg_music(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        bevy::audio::AudioPlayer::<bevy::audio::AudioSource>::new(assets.bg_music.clone()),
        PlaybackSettings::LOOP,
    ));
}

// ══════════════════════════════════════════
//  GameOver
// ══════════════════════════════════════════

pub fn spawn_game_over(mut commands: Commands, score: Res<Score>, high: Res<HighScore>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            MenuUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor::WHITE,
                MenuUi,
            ));

            parent.spawn((
                Text::new(format!("Score: {}  Best: {}", score.0, high.0)),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor::WHITE,
                MenuUi,
            ));

            parent.spawn((
                Node {
                    height: Val::Px(20.0),
                    ..default()
                },
                MenuUi,
            ));

            parent.spawn((
                Text::new("[R] Retry"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor::WHITE,
                MenuUi,
            ));

            parent.spawn((
                Text::new("[E] Exit"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor::WHITE,
                MenuUi,
            ));
        });
}

pub fn despawn_menu_ui(mut commands: Commands, query: Query<Entity, With<MenuUi>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

type GameEntitiesQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<Fighter>,
        With<Enemy>,
        With<Bullet>,
        With<Bomb>,
        With<ScrollingBg>,
        With<OffscreenDespawn>,
    )>,
>;

pub fn cleanup_game(mut commands: Commands, query: GameEntitiesQuery) {
    for e in &query {
        commands.entity(e).despawn();
    }
}
