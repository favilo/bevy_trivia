use bevy::{prelude::*, reflect::Reflect};
use leafwing_input_manager::{
    action_state::ActionState,
    axislike::{DualAxis, VirtualDPad},
    input_map::InputMap,
    plugin::InputManagerPlugin,
    Actionlike,
};

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GameAction>::default())
            .init_resource::<ActionState<GameAction>>()
            .insert_resource(GameAction::input_map());
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Actionlike)]
pub enum GameAction {
    Move,
    Select,
}

impl GameAction {
    fn input_map() -> InputMap<Self> {
        let mut map = InputMap::default();
        map.insert(Self::Move, VirtualDPad::wasd())
            .insert(Self::Move, VirtualDPad::arrow_keys())
            .insert(Self::Move, DualAxis::right_stick())
            .insert(Self::Select, KeyCode::Space)
            .insert(Self::Select, KeyCode::Enter);
        map
    }
}

