use std::ops::Index;

use bevy::{
    prelude::*,
    utils::{hashbrown::hash_map::DefaultHashBuilder, HashSet},
};
use bevy_http_client::{
    prelude::{HttpTypedRequestTrait, TypedRequest, TypedResponse},
    HttpClient,
};
use serde::Deserialize;

use crate::{
    trivia::source::{DoneFetching, Fetching, TriviaSource},
    utils::BiHashMap,
};

pub struct OpenTdbPlugin;

impl Plugin for OpenTdbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (OpenTdbSource::update_system, OpenTdbSource::finalize_system),
        );
        app.register_request_type::<OpenTdbResponse>();
    }
}

#[derive(Deserialize, TypePath, Debug, Clone, PartialEq)]
/// Use OpenTDB as the source of trivia questions
#[serde(from = "SourceType")]
pub enum OpenTdbSource {
    /// The Indicates we need to fetch the categories
    Categories(Categories),
}

impl OpenTdbSource {
    pub fn iter(&self) -> Box<dyn Iterator<Item = String>> {
        match self {
            Self::Categories(categories) => categories.iter(),
        }
    }

    pub fn get_selected(&self) -> Option<HashSet<usize>> {
        match self {
            Self::Categories(categories) => categories.get_selected(),
        }
    }

    pub fn set_selected(&mut self, selected: HashSet<usize>) {
        match self {
            Self::Categories(categories) => categories.set_selected(selected),
        }
    }

    pub fn update(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        events: &mut EventWriter<TypedRequest<OpenTdbResponse>>,
    ) {
        match self {
            Self::Categories(ref mut categories) => categories.update(commands, entity, events),
        }
    }

    pub fn finalize(&mut self, commands: &mut Commands, entity: Entity, response: OpenTdbResponse) {
        match self {
            Self::Categories(ref mut categories) => categories.finalize(commands, entity, response),
        }
    }

    pub fn update_system(
        mut sources: Query<(Entity, &mut TriviaSource), (Without<Fetching>, Without<DoneFetching>)>,
        mut commands: Commands,
        mut events: EventWriter<TypedRequest<OpenTdbResponse>>,
    ) {
        for (entity, mut source) in sources.iter_mut() {
            let TriviaSource::OpenTdb(ref mut source) = &mut *source else {
                continue;
            };
            info!("Updating OpenTDB source: {:#?}", source);
            source.update(&mut commands, entity, &mut events);
        }
    }

    pub fn finalize_system(
        mut sources: Query<(Entity, &mut TriviaSource), (With<Fetching>, Without<DoneFetching>)>,
        mut commands: Commands,
        mut events: ResMut<Events<TypedResponse<OpenTdbResponse>>>,
    ) {
        for event in events.drain() {
            for (entity, mut source) in sources.iter_mut() {
                let event = event.clone();
                let TriviaSource::OpenTdb(ref mut source) = &mut *source else {
                    continue;
                };
                info!("Finalizing OpenTDB source: {:#?}", source);
                (*source).finalize(&mut commands, entity, event);
            }
        }
    }
}

impl Index<usize> for OpenTdbSource {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Categories(source) => &source[index],
        }
    }
}

#[derive(Deserialize, TypePath, Debug, Clone, PartialEq)]
pub enum SourceType {
    Categories,
}

impl From<SourceType> for OpenTdbSource {
    fn from(source: SourceType) -> Self {
        match source {
            SourceType::Categories => Self::Categories(Categories::default()),
        }
    }
}

#[derive(TypePath, Debug, PartialEq, Default, Clone)]
pub enum Categories {
    #[default]
    Todo,

    /// Indicates we are waiting for the HTTP request to return
    InProgress,

    /// Indicates we have received the HTTP response
    Done {
        map: BiHashMap<String, usize>,
        names: Vec<String>,
        selected: HashSet<usize>,
    },
}

impl Categories {
    fn iter(&self) -> Box<dyn Iterator<Item = String>> {
        match self {
            Self::Todo => Box::new(std::iter::empty()),
            Self::InProgress => Box::new(std::iter::empty()),
            Self::Done { names, .. } => Box::new(names.clone().into_iter()),
        }
    }

    fn get_selected(&self) -> Option<HashSet<usize>> {
        match self {
            Self::Todo => None,
            Self::InProgress => None,
            Self::Done { selected, .. } => Some(selected.clone()),
        }
    }

    fn set_selected(&mut self, new: HashSet<usize>) {
        match self {
            Self::Todo => {}
            Self::InProgress => {}
            Self::Done { selected, .. } => *selected = new,
        }
    }

    fn update(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        events: &mut EventWriter<TypedRequest<OpenTdbResponse>>,
    ) {
        *self = Self::InProgress;
        let request = HttpClient::new()
            .get("https://opentdb.com/api_category.php")
            .entity(entity)
            .with_type::<OpenTdbResponse>();
        events.send(request);
        // Mark the entity as being fetched
        commands.entity(entity).insert(Fetching);
    }

    fn finalize(&mut self, commands: &mut Commands, entity: Entity, response: OpenTdbResponse) {
        #[allow(irrefutable_let_patterns)]
        let OpenTdbResponse::Categories(response) = response
        else {
            return;
        };

        let build_hasher_default = DefaultHashBuilder::default();
        let mut map = BiHashMap::with_capacity_and_hashers(
            response.trivia_categories.len(),
            build_hasher_default.clone(),
            build_hasher_default.clone(),
        );
        let mut names = Vec::with_capacity(response.trivia_categories.len());
        for category in response.trivia_categories {
            map.insert(category.name.clone(), category.id);
            names.push(category.name);
        }
        names.sort();
        *self = Self::Done {
            selected: (0..map.len()).collect(),
            names,
            map,
        };
        info!("Finished fetching categories: {:#?}", self);
        commands.entity(entity).insert(DoneFetching);
    }
}

impl Index<usize> for Categories {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Todo => panic!("Cannot index into empty Categories"),
            Self::InProgress => panic!("Cannot index into empty Categories"),
            Self::Done { names, .. } => &names[index],
        }
    }
}

#[derive(Deserialize, TypePath, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OpenTdbResponse {
    Categories(CategoriesResponse),
}

#[derive(Deserialize, TypePath, Debug, Clone, PartialEq)]
pub struct CategoriesResponse {
    pub trivia_categories: Vec<Category>,
}

#[derive(Deserialize, TypePath, Debug, Clone, PartialEq)]
pub struct Category {
    pub id: usize,
    pub name: String,
}
