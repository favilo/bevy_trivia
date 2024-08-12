use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct TriviaApiSource;
