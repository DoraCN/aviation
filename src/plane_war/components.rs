use bevy::prelude::*;

// ── 组件 ──

/// 我方战机
#[derive(Component)]
pub struct Fighter;

/// 敌人（1 击即毁）
#[derive(Component)]
pub struct Enemy(pub u32); // 分值：1 或 2

/// 炸弹障碍（不可击落）
#[derive(Component)]
pub struct Bomb;

/// 我方子弹
#[derive(Component)]
pub struct Bullet;

/// 运动速度
#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// 圆形碰撞体
#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
}

/// 出界时自动回收的标记
#[derive(Component)]
pub struct OffscreenDespawn;

/// 主菜单 / 结算界面 UI 标记
#[derive(Component)]
pub struct MenuUi;

/// 卷轴背景标记
#[derive(Component)]
pub struct ScrollingBg;

/// 飞机爆炸动画标记
#[derive(Component)]
pub struct FighterBomb;
