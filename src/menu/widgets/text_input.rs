use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    ecs::system::SystemState,
    prelude::*,
    text::BreakLineOn,
    ui,
};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{ListenerInput, On},
};
use bevy_mod_stylebuilder::{
    StyleBuilder, StyleBuilderBackground, StyleBuilderFont, StyleBuilderLayout,
    StyleBuilderOutline, StyleHandle, StyleTuple,
};
use bevy_quill::{Callback, Cx, EffectOptions, Element, RunCallback, View, ViewTemplate};
use bevy_quill_obsidian::{
    colors,
    controls::{Disabled, IsDisabled},
    focus::{
        AutoFocus, FocusVisible, KeyCharEvent, KeyPressEvent, NavAction, TabIndex, TabNavigation,
    },
    hooks::{UseIsFocus, UseIsHover},
    size::Size,
    typography, RoundedCorners,
};
use serde::{Deserialize, Serialize};

use crate::loading::MenuAssets;

use super::UseComponentOrDefault;

#[derive(Component, Debug, Default, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
pub struct TextInputValue(String);

#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut, PartialEq, Eq, Reflect)]
pub struct TextInputCursorPos(usize);

#[derive(Component, Debug, Reflect)]
pub struct TextInputCursorTimer {
    pub timer: Timer,
    pub visible: bool,
    should_reset: bool,
}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct TextInputValueInvalid;

impl Default for TextInputCursorTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true,
            should_reset: false,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum TextInputType {
    #[default]
    Text,
    Integer,
    Float,
}

impl TextInputType {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Text)
    }

    pub fn is_valid(&self, v: &str) -> bool {
        match self {
            Self::Text => true,
            Self::Integer => v.parse::<i32>().is_ok(),
            Self::Float => v.parse::<f32>().is_ok(),
        }
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct Placeholder(String);

#[derive(Default, Clone, PartialEq)]
pub struct TextInput {
    pub default_value: String,
    pub size: Size,
    pub disabled: bool,
    pub style: StyleHandle,
    pub tab_index: i32,
    pub corners: RoundedCorners,
    pub auto_focus: bool,
    pub minimal: bool,
    pub on_submit: Option<Callback>,
    pub max_length: Option<usize>,
    pub name: String,
    pub type_: TextInputType,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_value(mut self, default_value: String) -> Self {
        self.default_value = default_value;
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

    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    pub fn on_submit(mut self, callback: Callback) -> Self {
        self.on_submit = Some(callback);
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

    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
        self
    }

    pub fn max_length(mut self, max_length: Option<usize>) -> Self {
        self.max_length = max_length;
        self
    }

    pub fn named(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn type_(mut self, type_: TextInputType) -> Self {
        self.type_ = type_;
        self
    }
}

fn style_text_input(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .column_gap(10);
}

fn style_text_input_bg(ss: &mut StyleBuilder) {
    ss.display(Display::Grid)
        .position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0);
}

fn fetch_value_and_cursor_pos<'c>(
    cx: &'c mut Cx,
    id: Entity,
    default: String,
) -> (&'c TextInputValue, usize) {
    let mut ent = cx.world_mut().entity_mut(id);
    if !ent.contains::<TextInputValue>() {
        ent.insert(TextInputValue(default));
    }
    if !ent.contains::<TextInputCursorPos>() {
        let value = ent.get::<TextInputValue>().unwrap();
        ent.insert(TextInputCursorPos(value.len()));
    }

    let cursor_pos = **cx.use_component::<TextInputCursorPos>(id).unwrap();
    let value = cx.use_component::<TextInputValue>(id).unwrap();
    (value, cursor_pos)
}

impl ViewTemplate for TextInput {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let id = cx.create_entity();

        let max_length = self.max_length;
        let default_value = max_length
            .map(|l| self.default_value.chars().take(l).collect())
            .unwrap_or(self.default_value.clone());

        let _invalid = cx.world_mut().init_component::<TextInputValueInvalid>();
        let _is_invalid: Option<TextInputValueInvalid> =
            cx.use_component::<TextInputValueInvalid>(id).cloned();
        let (value, cursor_pos) = fetch_value_and_cursor_pos(cx, id, default_value);

        let before = value.chars().take(cursor_pos).collect::<String>();
        let after = value
            .chars()
            .skip(cursor_pos)
            .chain(std::iter::once(' '))
            .collect::<String>();

        let my_value = value.clone();
        let is_valid = self.type_.is_valid(&my_value);
        cx.create_effect_ext(
            |world, (value,)| {
                if self.type_.is_valid(&value) {
                    world.entity_mut(id).remove::<TextInputValueInvalid>();
                } else {
                    world.entity_mut(id).insert(TextInputValueInvalid);
                }
            },
            (my_value,),
            EffectOptions {
                run_immediately: true,
            },
        );
        let timer = cx.use_component_or_default::<TextInputCursorTimer>(id);
        let cursor_visible = timer.visible;

        let hovering = cx.is_hovered(id);
        let focused = cx.is_focus_visible(id);
        let menu_assets = cx.use_resource::<MenuAssets>();
        let cursor_font = menu_assets.cursor_font.clone();

        let corners = self.corners;
        let minimal = self.minimal;

        let size = self.size;
        let on_submit = self.on_submit;

        Element::<NodeBundle>::for_entity(id)
            .style((
                typography::text_default,
                style_text_input,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height()).font_size(size.font_size());
                    if minimal {
                        ss.padding(0);
                    } else {
                        ss.padding((size.font_size() * 0.75, 0));
                    }
                },
                self.style.clone(),
            ))
            .insert_dyn(Name::new, self.name.clone())
            .insert_dyn(TabIndex, self.tab_index)
            .insert_if(self.disabled, || Disabled)
            .insert_if(self.auto_focus, || AutoFocus)
            .insert_dyn(
                move |_| {
                    (
                        AccessibilityNode::from(NodeBuilder::new(Role::TextInput)),
                        On::<Pointer<Click>>::run(move |world: &mut World| {
                            let mut focus = world.get_resource_mut::<Focus>().unwrap();
                            focus.0 = Some(id);
                            let mut focus_visible =
                                world.get_resource_mut::<FocusVisible>().unwrap();
                            focus_visible.0 = true;
                            if !world.is_disabled(id) {
                                let mut event = world
                                    .get_resource_mut::<ListenerInput<Pointer<Click>>>()
                                    .unwrap();
                                event.stop_propagation();
                            }
                        }),
                        // Not going to worry about drag events for now
                        On::<KeyPressEvent>::run(move |world: &mut World| {
                            if world.is_disabled(id) {
                                return;
                            }
                            let mut state = SystemState::<(
                                ResMut<ListenerInput<KeyPressEvent>>,
                                Query<(
                                    &mut TextInputValue,
                                    &mut TextInputCursorPos,
                                    &mut TextInputCursorTimer,
                                )>,
                                ResMut<Focus>,
                                ResMut<FocusVisible>,
                                TabNavigation,
                            )>::new(world);
                            let (mut event, mut qs, mut focus, mut focus_visible, nav) =
                                state.get_mut(world);
                            if event.target != id {
                                return;
                            }
                            let key_code = event.key_code;
                            let shift = event.shift;
                            let (mut value, mut cursor_pos, mut cursor_timer) = qs
                                .get_mut(id)
                                .expect("this entity should have provided components entity");
                            let mut run_callback = false;
                            match key_code {
                                KeyCode::Enter => {
                                    run_callback = true;
                                    event.stop_propagation();
                                }
                                code @ (KeyCode::ArrowLeft | KeyCode::ArrowRight) => {
                                    if code == KeyCode::ArrowLeft {
                                        if **cursor_pos > 0 {
                                            **cursor_pos -= 1;
                                        }
                                    } else if **cursor_pos < value.len() {
                                        **cursor_pos += 1;
                                    }
                                    cursor_timer.should_reset = true;
                                    event.stop_propagation();
                                }
                                KeyCode::Home => {
                                    **cursor_pos = 0;
                                    cursor_timer.should_reset = true;
                                    event.stop_propagation();
                                }
                                KeyCode::End => {
                                    **cursor_pos = value.len();
                                    cursor_timer.should_reset = true;
                                    event.stop_propagation();
                                }
                                code @ (KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::Tab) => {
                                    let next = nav.navigate(
                                        Some(id),
                                        if code == KeyCode::ArrowDown
                                            || (code == KeyCode::Tab && !shift)
                                        {
                                            NavAction::Next
                                        } else {
                                            NavAction::Previous
                                        },
                                    );
                                    if next.is_some() {
                                        focus.0 = next;
                                        focus_visible.0 = true;
                                    }
                                    event.stop_propagation();
                                }
                                KeyCode::Backspace => {
                                    if **cursor_pos > 0 {
                                        value.remove(**cursor_pos - 1);
                                        **cursor_pos -= 1;
                                        cursor_timer.should_reset = true;
                                    }
                                    event.stop_propagation();
                                }
                                KeyCode::Delete => {
                                    if **cursor_pos < (**value).len() {
                                        value.remove(**cursor_pos);
                                        cursor_timer.should_reset = true;
                                    }
                                    event.stop_propagation();
                                }
                                KeyCode::Space => {
                                    if max_length.map_or(true, |l| value.len() < l) {
                                        value.insert(**cursor_pos, ' ');
                                        **cursor_pos += 1;
                                        cursor_timer.should_reset = true;
                                    }
                                    event.stop_propagation();
                                }
                                _ => {}
                            }
                            if run_callback && on_submit.is_some() {
                                world.run_callback(on_submit.unwrap(), ());
                            }
                        }),
                        On::<KeyCharEvent>::run(move |world: &mut World| {
                            if world.is_disabled(id) {
                                return;
                            }
                            let mut event = world
                                .get_resource_mut::<ListenerInput<KeyCharEvent>>()
                                .unwrap();
                            if event.target != id {
                                return;
                            }
                            event.stop_propagation();
                            let key = event.key;
                            // Just a single line, ignore newline characters, especially because we
                            // should have already on_submit callback
                            if key == '\n' {
                                return;
                            }
                            let mut qs = world.query::<(
                                &mut TextInputValue,
                                &mut TextInputCursorPos,
                                &mut TextInputCursorTimer,
                            )>();
                            let (mut value, mut cursor_pos, mut cursor_timer) = qs
                                .get_mut(world, id)
                                .expect("this entity should have provided components entity");
                            if max_length.map_or(false, |l| value.len() >= l) {
                                return;
                            }

                            value.insert(**cursor_pos, key);
                            **cursor_pos += 1;
                            cursor_timer.should_reset = true;
                        }),
                    )
                },
                (),
            )
            .children((
                Element::<NodeBundle>::new()
                    .named("TextInput::Background")
                    .style(style_text_input_bg)
                    .insert_dyn(
                        move |size| corners.to_border_radius(size.border_radius()),
                        self.size,
                    )
                    .style_dyn(
                        |(minimal, disabled, hovering, is_valid), sb| {
                            let color = if !is_valid {
                                colors::DESTRUCTIVE
                            } else if minimal {
                                colors::TRANSPARENT
                            } else {
                                text_input_bg_color(disabled, hovering)
                            };
                            sb.background_color(color);
                        },
                        (minimal, self.disabled, hovering, is_valid),
                    )
                    .style_dyn(
                        move |focused, sb| {
                            match focused {
                                true => {
                                    sb.outline_color(colors::FOCUS)
                                        .outline_width(2)
                                        .outline_offset(2);
                                }
                                false => {
                                    sb.outline_color(Option::<Color>::None);
                                }
                            };
                        },
                        focused,
                    ),
                Element::<TextBundle>::new()
                    .named("TextInput::Text")
                    .style((typography::text_default,))
                    .insert_dyn(
                        move |(visible, focused, before, after)| {
                            let default_style = TextStyle {
                                color: colors::BACKGROUND.into(),
                                ..Default::default()
                            };
                            let sections = vec![
                                TextSection {
                                    value: before,
                                    style: default_style.clone(),
                                },
                                TextSection {
                                    value: "|".into(),
                                    style: TextStyle {
                                        font: cursor_font.clone(),
                                        color: if visible && focused {
                                            colors::BACKGROUND.into()
                                        } else {
                                            colors::TRANSPARENT.into()
                                        },
                                        ..default_style.clone()
                                    },
                                },
                                TextSection {
                                    value: after,
                                    style: default_style,
                                },
                            ];
                            Text {
                                sections,
                                justify: JustifyText::Left,
                                linebreak_behavior: BreakLineOn::NoWrap,
                            }
                        },
                        (cursor_visible, focused, before.clone(), after.clone()),
                    ),
            ))
    }
}

pub(crate) fn text_input_bg_color(is_disabled: bool, is_hovering: bool) -> Srgba {
    let base_color = colors::FOREGROUND;
    match (is_disabled, is_hovering) {
        (true, _) => base_color.with_alpha(0.2),
        (_, true) => base_color.lighter(0.02),
        (_, false) => base_color,
    }
}
