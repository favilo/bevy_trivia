use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_trivia::GamePlugin;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceAndFallback {
                    path: "assets".into(),
                },
            },
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
            GamePlugin,
        ))
        .run();
}
