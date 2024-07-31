use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::AudioSource;

use crate::{menu::serde::Menu, GameState};

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Menu>::new(&["menu.ron"]))
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Menu)
                    .load_collection::<AudioAssets>()
                    .load_collection::<TextureAssets>()
                    .load_collection::<MenuAssets>(),
            );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

pub const BEVY: &str = "textures/bevy.png";
pub const GITHUB: &str = "textures/github.png";

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    // #[asset(path = "textures/bevy.png")]
    // pub bevy: Handle<Image>,
    // #[asset(path = "textures/github.png")]
    // pub github: Handle<Image>,
    #[asset(path = "textures", collection(typed, mapped))]
    pub images: HashMap<String, Handle<Image>>,
}

#[derive(AssetCollection, Resource)]
pub struct MenuAssets {
    #[asset(path = "menus/main.menu.ron")]
    pub main: Handle<Menu>,

    #[asset(path = "menus/credits.menu.ron")]
    pub credits: Handle<Menu>,

    #[asset(path = "menus/settings.menu.ron")]
    pub settings: Handle<Menu>,

    #[asset(path = "menus/play.menu.ron")]
    pub play: Handle<Menu>,
}
