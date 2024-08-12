use std::ops::Index;

use bevy::{prelude::*, reflect::TypePath, utils::HashSet};
use open_tdb::{OpenTdbPlugin, OpenTdbSource};
use serde::{Deserialize, Serialize};
use trivia_api::TriviaApiSource;

pub mod open_tdb;
pub mod trivia_api;

pub struct SourcePlugin;

impl Plugin for SourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpenTdbPlugin);
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fetching;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DoneFetching;

#[derive(Deserialize, TypePath, Clone, Debug, PartialEq, Component)]
pub enum TriviaSource {
    String(StringSource),
    // TODO: make this configurable
    OpenTdb(OpenTdbSource),
    TriviaApi(TriviaApiSource),
}

impl Default for TriviaSource {
    fn default() -> Self {
        Self::String(StringSource::default())
    }
}

impl TriviaSource {
    pub fn iter(&self) -> Box<dyn Iterator<Item = String>> {
        match self {
            Self::String(source) => Box::new(source.clone().0.into_iter()),
            Self::OpenTdb(opentdb) => Box::new(opentdb.iter()),
            Self::TriviaApi(_) => Box::new(std::iter::empty()),
        }
    }

    pub fn get_selected(&self) -> Option<HashSet<usize>> {
        match self {
            Self::String(_) => None,
            Self::OpenTdb(opentdb) => opentdb.get_selected(),
            Self::TriviaApi(_) => None,
        }
    }

    pub fn set_selected(&mut self, selected: HashSet<usize>) {
        match self {
            Self::String(_) => {}
            Self::OpenTdb(opentdb) => opentdb.set_selected(selected),
            Self::TriviaApi(_) => {}
        }
    }
}

static EMPTY_STRING: String = String::new();

impl Index<usize> for TriviaSource {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::String(source) => &source.0[index],
            Self::OpenTdb(source) => &source[index],
            Self::TriviaApi(_) => &EMPTY_STRING,
        }
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq, Default)]
pub struct StringSource(Vec<String>);

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub enum SourceEvent {
    Categories,
}
