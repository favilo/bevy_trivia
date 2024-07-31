use ::serde::{Deserialize, Serialize};
use bevy::ecs::system::RunSystemOnce;
use bevy::{prelude::*, ui};
use bevy_mod_stylebuilder::{StyleBuilder, StyleBuilderLayout};
use bevy_quill::{IntoViewChild, View, ViewChild};
use leafwing_input_manager::action_state::ActionState;
use serde::Menu;

use credits::CreditsMenu;

use crate::actions::GameAction;
use crate::loading::{MenuAssets, TextureAssets};
use crate::GameState;

pub mod serde;
pub mod utils;

use main_menu::MainMenu;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuStack>()
            .add_sub_state::<WhichMenu>()
            .add_systems(
                StateTransition,
                last_transition::<WhichMenu>
                    .pipe(menu_transition)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(Update, move_focus.run_if(in_state(GameState::Menu)));
    }
}

#[derive(Default, Deref, DerefMut, Clone, Debug, Resource)]
pub struct MenuStack(Vec<WhichMenu>);

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, SubStates, Default, TypePath, Serialize, Deserialize,
)]
#[source(GameState = GameState::Menu)]
pub enum WhichMenu {
    #[default]
    Main,
    Play,
    Settings,
    Credits,
}

impl WhichMenu {
    pub fn to_view(&self, assets: &MenuAssets, menus: &Assets<Menu>) -> ViewChild {
        menus
            .get(match self {
                Self::Main => &assets.main,
                Self::Play => &assets.play,
                Self::Settings => &assets.settings,
                Self::Credits => &assets.credits,
            })
            .expect("main menu")
            .clone()
            .into_view_child()
    }
}

#[derive(Component)]
struct MenuMarker;

fn menu_style(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .position(ui::PositionType::Absolute)
        .padding(50)
        .left(0)
        .right(0)
        .bottom(0)
        .top(0)
        .row_gap(10)
        .align_items(AlignItems::Center);
}

fn menu_row_style(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .width(Val::Percent(75.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .column_gap(10);
}

fn menu_button_style(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .width(Val::Percent(75.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .column_gap(10);
}

fn setup_menu(
    mut commands: Commands,
    current_state: Res<State<WhichMenu>>,
    assets: Res<MenuAssets>,
    menus: Res<Assets<Menu>>,
) {
    commands.spawn((current_state.to_view(&assets, &menus).to_root(), MenuMarker));
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<MenuMarker>>) {
    info!("cleanup_menu");
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}

/// This system is responsible for transitioning between the different menu states
/// It is only active during the State `GameState::Menu`
fn menu_transition(transition: In<Option<StateTransitionEvent<WhichMenu>>>, world: &mut World) {
    let Some(transition) = transition.0 else {
        return;
    };

    if transition.exited == transition.entered {
        return;
    }

    let _ = world.run_system_once(cleanup_menu);
    let _ = world.run_system_once(setup_menu);
}

fn move_focus(actions: Res<ActionState<GameAction>>) {
    if actions.just_pressed(&GameAction::Move) {}
}
