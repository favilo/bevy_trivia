use crate::actions::GameAction;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: textures.bevy.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert(Player);
}

fn move_player(
    time: Res<Time>,
    actions: Res<ActionState<GameAction>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if !actions.pressed(&GameAction::Move) {
        return;
    }
    let speed = 150.;
    let axis_pair = actions
        .clamped_axis_pair(&GameAction::Move)
        .expect("Move was pressed; impossible to get here otherwise");
    let movement = Vec3::new(
        axis_pair.x() * speed * time.delta_seconds(),
        axis_pair.y() * speed * time.delta_seconds(),
        0.,
    );

    for mut player_transform in &mut player_query {
        player_transform.translation += movement;
    }
}
