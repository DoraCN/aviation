use bevy::prelude::*;

// ── 资源 ──

/// 所有预加载的游戏素材句柄。
#[derive(Resource, Clone)]
pub struct GameAssets {
    pub fighter: Handle<Image>,
    pub fighter_bomb: Handle<Image>,
    pub bomb_img: Handle<Image>,
    pub enemies_64: Vec<Handle<Image>>,
    pub enemies_100: Vec<Handle<Image>>,
    pub bg_music: Handle<bevy::audio::AudioSource>,
    pub bomb_sfx: Handle<bevy::audio::AudioSource>,
}

/// 当前得分。
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// 历史最高分。
#[derive(Resource, Default)]
pub struct HighScore(pub u32);

/// 难度参数，随时间提升。
#[derive(Resource)]
pub struct Difficulty {
    pub level: u32,
    /// 敌人生成间隔（秒），随 level 递减
    pub enemy_interval: f32,
    /// 炸弹生成间隔（秒）
    pub bomb_interval: f32,
    /// 敌人下落速度范围
    pub enemy_speed_range: (f32, f32),
    /// 炸弹下落速度范围
    pub bomb_speed_range: (f32, f32),
    /// 生成敌人大档（100×100）的概率
    pub big_enemy_chance: f32,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            level: 0,
            enemy_interval: 1.2,
            bomb_interval: 4.0,
            enemy_speed_range: (100.0, 180.0),
            bomb_speed_range: (120.0, 200.0),
            big_enemy_chance: 0.2,
        }
    }
}

/// 各种刷怪计时器。
#[derive(Resource)]
pub struct SpawnTimers {
    pub enemy: Timer,
    pub bomb: Timer,
    pub difficulty: Timer,
}

impl Default for SpawnTimers {
    fn default() -> Self {
        Self {
            enemy: Timer::from_seconds(1.2, TimerMode::Repeating),
            bomb: Timer::from_seconds(4.0, TimerMode::Repeating),
            difficulty: Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }
}

/// 开火冷却。
#[derive(Resource)]
pub struct FireCooldown(pub Timer);

/// 爆炸延迟——看到爆炸动画后数秒才切到 GameOver。
#[derive(Resource, Default)]
pub struct ExplosionTimer(pub Option<Timer>);

impl Default for FireCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.18, TimerMode::Repeating))
    }
}

/// Player 意图（移动方向 + 开火）
#[derive(Resource, Default)]
pub struct PlayerIntent {
    pub move_x: f32,
    pub move_y: f32,
    pub fire: bool,
}

/// 窗口 / 世界尺寸。
pub const WORLD_W: f32 = 480.0;
pub const WORLD_H: f32 = 800.0;
