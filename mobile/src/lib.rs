use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_trivia::GamePlugin;

#[cfg(target_os = "android")]
use keyboard::AndroidKeyboardPlugin;

#[cfg(target_os = "android")]
mod keyboard;

#[bevy_main]
fn main() {
    let mut app = App::new();
    app.add_plugins((
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
    ));
    #[cfg(target_os = "android")]
    {
        app.add_plugins(AndroidKeyboardPlugin);
    }
    app.run();
}
