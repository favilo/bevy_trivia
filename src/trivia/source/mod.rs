use std::ops::Index;

use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub enum TriviaSource {
    String(StringSource),
    OpenTdb(OpenTdbCategories),
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
            Self::OpenTdb(_) => Box::new(std::iter::empty()),
        }
    }
}

static EMPTY_STRING: String = String::new();

impl Index<usize> for TriviaSource {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::String(source) => &source.0[index],
            // TODO: OpenTDB
            Self::OpenTdb(_) => &EMPTY_STRING,
        }
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq, Default)]
pub struct StringSource(Vec<String>);

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct OpenTdbCategories;
