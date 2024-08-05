#![feature(impl_trait_in_assoc_type, associated_type_defaults)]
#![allow(clippy::type_complexity)]

mod actions;
mod audio;
pub mod loading;
mod lobby;
pub mod menu;
mod player;
mod trivia;

use crate::{
    actions::ActionsPlugin, audio::InternalAudioPlugin, loading::LoadingPlugin, lobby::LobbyPlugin,
    menu::MenuPlugin, player::PlayerPlugin,
};

#[cfg(debug_assertions)]
use bevy::diagnostic::{
    // FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::{app::App, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_picking::prelude::{Listener, On};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_quill::{Element, QuillPlugin, View};
use bevy_quill_obsidian::focus::{DefaultKeyListener, KeyPressEvent};
use bevy_quill_obsidian::{colors, ObsidianUiPlugin};
use lobby::{GameName, NumQuestions, QuestionDifficulty, QuestionTypes};

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

    // The lobby is waiting for players to join
    Lobby,
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
                LobbyPlugin,
            ))
            .insert_resource(ClearColor(colors::BACKGROUND.into()))
            .insert_resource(ShowInspectorUi(false))
            .add_systems(Startup, setup_camera);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                //FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
                EguiPlugin,
                bevy_inspector_egui::DefaultInspectorConfigPlugin,
            ))
            .add_systems(Update, inspector_ui)
            .add_systems(Startup, setup_inspector);
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_inspector(mut commands: Commands) {
    commands.spawn(
        Element::<NodeBundle>::new()
            .insert_dyn(
                |_| {
                    (
                        DefaultKeyListener,
                        On::<KeyPressEvent>::run(
                            move |event: Listener<KeyPressEvent>,
                                  mut show: ResMut<ShowInspectorUi>| {
                                info!("KeyPressEvent: {:?}", event.key_code);
                                if event.key_code == KeyCode::Backquote {
                                    show.0 = !show.0;
                                }
                            },
                        ),
                    )
                },
                (),
            )
            .to_root(),
    );
}

macro_rules! ui_for_resource {
    ($resource:ty, $world:expr, $ui:expr) => {
        $ui.collapsing(stringify!($resource), |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_resource::<$resource>($world, ui)
        });
    };
}

#[derive(Resource)]
struct ShowInspectorUi(bool);

fn inspector_ui(world: &mut World) {
    let show_inspector_ui = world.resource::<ShowInspectorUi>().0;
    if !show_inspector_ui {
        return;
    }

    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            // bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Resources")
                .default_open(true)
                .show(ui, |ui| {
                    ui_for_resource!(GameName, world, ui);
                    ui_for_resource!(NumQuestions, world, ui);
                    ui_for_resource!(QuestionDifficulty, world, ui);
                    ui_for_resource!(QuestionTypes, world, ui);
                });

            // ui.heading("Entities");
            // bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}
