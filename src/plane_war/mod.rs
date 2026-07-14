use bevy::prelude::*;

mod assets;
mod components;
mod resources;
mod systems;

use assets::*;
use components::*;
use resources::*;
use systems::*;

pub use resources::{InputMode, PlayerIntent};
#[cfg(feature = "dora")]
pub use resources::DoraBridge;

/// 游戏全局状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    GameOver,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Score>()
            .init_resource::<HighScore>()
            .init_resource::<FireCooldown>()
            .init_resource::<PlayerIntent>()
            .init_resource::<SpawnTimers>()
            .init_resource::<Difficulty>()
            .init_resource::<ExplosionTimer>()
            .init_resource::<InputMode>()
            .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.10)))
            // Loading → Playing
            .add_systems(OnEnter(GameState::Loading), load_game_assets)
            .add_systems(
                Update,
                check_assets_ready.run_if(in_state(GameState::Loading)),
            )
            // Playing — enter
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    reset_and_spawn_fighter,
                    spawn_background,
                    start_bg_music,
                    spawn_hud,
                )
                    .chain(),
            )
            // Playing — update
            .add_systems(
                FixedUpdate,
                (
                    player_input,
                    move_player,
                    player_fire,
                    spawn_enemies,
                    spawn_bombs,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (move_entities, bullet_enemy_collision, player_collision)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (despawn_offscreen, advance_difficulty, scroll_background)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                check_explosion_done.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup_game)
            // GameOver ⇄ Playing
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(
                Update,
                game_over_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), despawn_menu_ui);

        #[cfg(feature = "dora")]
        app.add_systems(
            FixedUpdate,
            dora_input_system
                .before(player_input)
                .run_if(in_state(GameState::Playing)),
        );
        #[cfg(feature = "dora")]
        app.add_systems(Update, dora_stop_system);
    }
}

// ══════════════════════════════════════════
//  Loading → Playing
// ══════════════════════════════════════════

fn load_game_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut audio: ResMut<Assets<bevy::audio::AudioSource>>,
) {
    let fighter = decode_image(FIGHTER_BYTES, &mut images);
    let fighter_bomb = decode_image(FIGHTER_BOMB_BYTES, &mut images);
    let bomb_img = decode_image(BOMB_BYTES, &mut images);

    let enemies_64: Vec<Handle<Image>> = ENEMY_64_BYTES
        .iter()
        .map(|(_, bytes)| decode_image(bytes, &mut images))
        .collect();
    let enemies_100: Vec<Handle<Image>> = ENEMY_100_BYTES
        .iter()
        .map(|(_, bytes)| decode_image(bytes, &mut images))
        .collect();

    let bg_music = audio.add(audio_source(BG_MUSIC_BYTES));
    let bomb_sfx = audio.add(audio_source(BOMB_SFX_BYTES));

    commands.insert_resource(GameAssets {
        fighter,
        fighter_bomb,
        bomb_img,
        enemies_64,
        enemies_100,
        bg_music,
        bomb_sfx,
    });
}

fn check_assets_ready(mut commands: Commands, mut next: ResMut<NextState<GameState>>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: WORLD_W,
                height: WORLD_H,
            },
            viewport_origin: Vec2::new(0.0, 0.0),
            ..OrthographicProjection::default_2d()
        }),
    ));
    next.set(GameState::Playing);
}

// ══════════════════════════════════════════
//  Playing OnEnter: 重置 + 生成
// ══════════════════════════════════════════

fn reset_and_spawn_fighter(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut score: ResMut<Score>,
    mut intent: ResMut<PlayerIntent>,
    mut timers: ResMut<SpawnTimers>,
    mut difficulty: ResMut<Difficulty>,
    mut explosion_timer: ResMut<ExplosionTimer>,
) {
    *score = Score(0);
    *intent = PlayerIntent::default();
    *difficulty = Difficulty::default();
    *explosion_timer = ExplosionTimer::default();
    // 敌人延迟 3 秒，炸弹延迟 5 秒
    *timers = SpawnTimers {
        enemy: Timer::from_seconds(3.0, TimerMode::Repeating),
        bomb: Timer::from_seconds(5.0, TimerMode::Repeating),
        difficulty: Timer::from_seconds(10.0, TimerMode::Repeating),
    };

    commands.spawn((
        Sprite::from_image(assets.fighter.clone()),
        Transform::from_xyz(WORLD_W / 2.0, 100.0, 1.0),
        Fighter,
    ));
}

// ══════════════════════════════════════════
//  GameOver → Playing
// ══════════════════════════════════════════

fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    score: Res<Score>,
    mut high: ResMut<HighScore>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if score.0 > high.0 {
            high.0 = score.0;
        }
        next.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        if score.0 > high.0 {
            high.0 = score.0;
        }
        exit.write(AppExit::Success);
    }
}

#[cfg(feature = "dora")]
fn dora_stop_system(bridge: Res<DoraBridge>, mut exit: MessageWriter<AppExit>) {
    if bridge.stop.load(std::sync::atomic::Ordering::SeqCst) {
        exit.write(AppExit::Success);
    }
}
