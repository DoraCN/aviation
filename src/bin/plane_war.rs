use aviation::plane_war::GamePlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "飞机大战".into(),
                        resolution: bevy::window::WindowResolution::new(480, 800)
                            .with_scale_factor_override(1.0),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,naga=warn".into(),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
