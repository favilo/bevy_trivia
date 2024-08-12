use bevy::{
    ecs::system::{SystemId, SystemParam},
    prelude::*,
};

use crate::{
    menu::{
        serde::SliderValue,
        widgets::{
            multi_dropdown::MultiDropdownSelected,
            text_input::{TextInputValue, TextInputValueInvalid},
        },
    },
    trivia::source::TriviaSource,
    GameState,
};

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        let system_id = app.register_system(start_host_lobby);
        app.insert_resource(HostLobby(system_id))
            .init_resource::<QuestionDifficulty>()
            .init_resource::<NumQuestions>()
            .init_resource::<QuestionTypes>()
            .init_resource::<GameName>()
            .register_type::<QuestionDifficulty>()
            .register_type::<NumQuestions>()
            .register_type::<QuestionTypes>()
            .register_type::<GameName>();
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct HostLobby(pub SystemId);

#[derive(Default, Deref, DerefMut, Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameName(String);

#[derive(Default, Deref, DerefMut, Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct NumQuestions(usize);

// TODO: Make this a set of Enums
#[derive(Default, Deref, DerefMut, Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct QuestionTypes(Vec<String>);

// TODO: Make this a set of Enums
#[derive(Default, Deref, DerefMut, Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct QuestionDifficulty(Vec<String>);

#[derive(SystemParam)]
struct HostLobbyParams<'w, 's> {
    texts: Query<'w, 's, (&'static TextInputValue, &'static Name)>,
    dropdowns: Query<
        'w,
        's,
        (
            &'static TriviaSource,
            &'static MultiDropdownSelected,
            &'static Name,
        ),
    >,
    sliders: Query<'w, 's, (&'static SliderValue, &'static Name)>,
    game_name: ResMut<'w, GameName>,
    num_questions: ResMut<'w, NumQuestions>,
    question_types: ResMut<'w, QuestionTypes>,
    question_difficulty: ResMut<'w, QuestionDifficulty>,
}

impl<'w, 's> HostLobbyParams<'w, 's> {
    fn fetch_form_values(&mut self) {
        *self.game_name = GameName(
            (*self
                .texts
                .iter()
                .find_map(|(v, n)| (n.as_str() == "game_name").then_some(v.clone()))
                .unwrap())
            .clone(),
        );
        // TODO: Safely parse the values
        *self.num_questions = NumQuestions(
            *self
                .sliders
                .iter()
                .find_map(|(v, n)| (n.as_str() == "num_questions").then_some(v.clone()))
                .unwrap(),
        );
        *self.question_types = QuestionTypes({
            let (source, selected) = self
                .dropdowns
                .iter()
                .find_map(|(s, v, n)| (n.as_str() == "question_types").then_some((s, v)))
                .unwrap();
            selected.iter().map(|i| source[*i].clone()).collect()
        });
        *self.question_difficulty = QuestionDifficulty({
            let (source, selected) = self
                .dropdowns
                .iter()
                .find_map(|(s, v, n)| (n.as_str() == "difficulty").then_some((s, v)))
                .unwrap();
            selected.iter().map(|i| source[*i].clone()).collect()
        });
    }
}

fn start_host_lobby(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut params: HostLobbyParams,
    invalid_text_inputs: Query<&TextInputValueInvalid>,
) {
    if invalid_text_inputs.iter().len() > 0 {
        return;
    }

    params.fetch_form_values();
    next_game_state.set(GameState::Playing);
}
