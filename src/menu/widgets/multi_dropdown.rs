use bevy::{
    ecs::{change_detection::DetectChanges, system::SystemState},
    prelude::*,
    utils::HashSet,
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleBuilderFont, StyleBuilderLayout, StyleHandle};
use bevy_quill::*;
use bevy_quill_obsidian::{
    controls::{Checkbox, MenuButton, MenuPopup},
    focus::AutoFocus,
    size::Size,
    typography, RoundedCorners,
};

use crate::{
    menu::{menu_text_input_style, widgets::UseComponentOrDefault},
    trivia::source::TriviaSource,
};

#[derive(Component, Debug, Default, Clone, Deref, DerefMut, PartialEq, Eq)]
pub struct MultiDropdownSelected(HashSet<usize>);

#[derive(Default, Clone, PartialEq)]
pub struct MultiDropdown {
    pub label: String,
    pub source: TriviaSource,
    pub selected: HashSet<usize>,

    pub size: Size,
    pub disabled: bool,
    pub style: StyleHandle,
    pub tab_index: i32,
    pub corners: RoundedCorners,
    pub auto_focus: bool,

    pub name: String,
}

impl MultiDropdown {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn source(mut self, source: TriviaSource) -> Self {
        self.source = source;
        self
    }

    pub fn selected(mut self, selected: &[usize]) -> Self {
        self.selected = HashSet::from_iter(selected.iter().copied());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    pub fn corners(mut self, corners: RoundedCorners) -> Self {
        self.corners = corners;
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn named(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

impl ViewTemplate for MultiDropdown {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let id = cx.create_entity();

        let label = self.label.clone();
        let source = self.source.clone();
        let selected = self.selected.clone();
        let size = self.size;
        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let corners = self.corners;
        let name = self.name.clone();

        let source = cx.use_component_or::<TriviaSource>(id, source).clone();
        // When the source is done fetching, update the selected values
        cx.create_effect_ext(
            move |world: &mut World, (source,)| {
                if source.get_selected().is_none() {
                    return;
                }
                world
                    .entity_mut(id)
                    .insert(MultiDropdownSelected(source.get_selected().unwrap()));
            },
            (source.clone(),),
            EffectOptions {
                run_immediately: true,
            },
        );
        let selected = cx
            .use_component_or::<MultiDropdownSelected>(
                id,
                MultiDropdownSelected(
                    source
                        .get_selected()
                        .unwrap_or(HashSet::from_iter(selected.iter().copied())),
                ),
            )
            .clone();

        Element::<NodeBundle>::for_entity(id)
            .insert_dyn(Name::new, name.clone())
            .style((
                style_multi_dropdown,
                typography::text_default,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height()).font_size(size.font_size());
                    ss.padding(0);
                },
                self.style.clone(),
            ))
            .insert_if(self.auto_focus, || AutoFocus)
            .children(
                MenuButton::new()
                    .style(menu_text_input_style)
                    .children(label)
                    .disabled(disabled)
                    .tab_index(tab_index)
                    .corners(corners)
                    .popup(
                        MenuPopup::new().children(
                            source
                                .iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let is_selected = selected.contains(&i);
                                    Checkbox::new()
                                        .checked(is_selected)
                                        .label(s.clone())
                                        .style(|sb: &mut StyleBuilder| {
                                            sb.align_self(AlignSelf::Start)
                                                .width(Val::Percent(100.0));
                                        })
                                        .on_change(cx.create_callback(
                                            move |value: In<bool>, world: &mut World| {
                                                let entity = &mut world.entity_mut(id);
                                                let mut selected = entity
                                                    .get_mut::<MultiDropdownSelected>()
                                                    .expect(
                                                        "MultiDropdownValues set by `create()`",
                                                    );
                                                if value.0 {
                                                    selected.insert(i);
                                                } else {
                                                    selected.remove(&i);
                                                }
                                                let selected = (**selected).clone();
                                                let mut source =
                                                    entity.get_mut::<TriviaSource>().expect(
                                                        "MultiDropdownValues set by `create()`",
                                                    );
                                                source.set_selected(selected);
                                            },
                                        ))
                                        .into_view_child()
                                })
                                .collect::<Vec<_>>(),
                        ),
                    ),
            )
    }
}

fn style_multi_dropdown(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .width(Val::Percent(75.0))
        .align_items(AlignItems::Center)
        .column_gap(10);
}
