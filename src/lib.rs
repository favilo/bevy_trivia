#![feature(impl_trait_in_assoc_type, associated_type_defaults)]
#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod loading;
mod menu;
mod player;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{
    // FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_quill::QuillPlugin;
use bevy_quill_obsidian::ObsidianUiPlugin;

// This example game uses States to separate logic
// See https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            // These are third party plugins that we need to use
            .add_plugins((DefaultPickingPlugins, QuillPlugin, ObsidianUiPlugin))
            // These are our own plugins
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
            ))
            .add_systems(OnEnter(GameState::Menu), setup_camera);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                //FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
