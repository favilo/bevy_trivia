use bevy::{a11y::Focus, prelude::*};
use bevy_mod_picking::prelude::{Listener, On};
use bevy_mod_stylebuilder::{StyleBuilder, StyleBuilderFont, StyleBuilderLayout};
use bevy_quill::*;
use bevy_quill_obsidian::{
    colors,
    controls::{
        Button as QuillButton, ButtonVariant, Icon, MenuButton, MenuItem as QuillMenuItem,
        MenuPopup, Slider as QuillSlider, Spacer,
    },
    focus::{AutoFocus, DefaultKeyListener, KeyPressEvent, TabGroup},
    size::Size,
    typography,
};
use serde::{Deserialize, Serialize};

use crate::{
    loading::TextureAssets, lobby::HostLobby, trivia::source::TriviaSource, ShowInspectorUi,
};

use super::{
    menu_button_style, menu_labeled_style, menu_row_style, menu_style, menu_text_input_style,
    utils::{is_false, open_link},
    widgets::{
        multi_dropdown::MultiDropdown as QuillMultiDropdown,
        text_input::{TextInput as QuillTextInput, TextInputType},
        UseComponentOrDefault,
    },
    MenuStack, WhichMenu,
};

/// A menu for the game
#[derive(Asset, Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Menu {
    /// The title of the menu
    title: String,

    /// Whether this is the main menu, or not
    #[serde(skip_serializing_if = "is_false", default)]
    main_menu: bool,

    /// The contents of the menu
    children: Vec<MenuItem>,
}

impl ViewTemplate for Menu {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let id = cx.create_entity();
        cx.world_mut().get_resource_mut::<Focus>().unwrap().0 = Some(id);
        Element::<NodeBundle>::for_entity(id)
            .named(&self.title)
            .style(menu_style)
            .insert_dyn(move |_| (TabGroup::default(), AutoFocus), ())
            .insert_if(cfg!(debug_assertions), || DefaultKeyListener)
            .insert_if(cfg!(debug_assertions), move || {
                On::<KeyPressEvent>::run(
                    move |event: Listener<KeyPressEvent>, mut show: ResMut<ShowInspectorUi>| {
                        if event.key_code == KeyCode::F12 {
                            show.0 = !show.0;
                        };
                    },
                )
            })
            .children((
                Element::<NodeBundle>::new()
                    .style((typography::text_strong, move |ss: &mut StyleBuilder| {
                        ss.min_height(Val::Px(70.0))
                            .font_size(48.0)
                            .color(colors::PRIMARY);
                    }))
                    .children(self.title.clone()),
                self.children
                    .iter()
                    .map(|item| item.clone().into_view_child())
                    .collect::<Vec<_>>(),
                Cond::new(
                    self.main_menu,
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuItem::Button(Button {
                        label: "Quit".to_string(),
                        action: MenuAction::Quit,
                    }),
                    #[cfg(target_arch = "wasm32")]
                    MenuItem::Button(Button {
                        label: "Quit".to_string(),
                        action: MenuAction::Reload,
                    }),
                    MenuItem::Button(Button {
                        label: "Back".to_string(),
                        action: MenuAction::Back,
                    }),
                ),
            ))
    }
}

/// An item to render in the menu
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub enum MenuItem {
    /// A label to display
    Label(Label),

    /// A spacer to fill the space
    Spacer,

    /// A sub menu to switch to
    SubMenu(SubMenu),

    /// A button to click
    Button(Button),

    /// A link to open in a browser
    Link(Link),

    /// A text input to enter text
    TextInput(TextInput),

    /// A slider to select a value
    Slider(Slider),

    /// A row of several items
    Row(Row),

    /// A dropdown to select an option
    Dropdown(Dropdown),

    /// A multi-dropdown to select multiple options
    MultiDropdown(MultiDropdown),
}

impl ViewTemplate for MenuItem {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        let item: Self = self.to_owned();
        match item {
            MenuItem::Label(label) => label.into_view_child(),
            MenuItem::Spacer => Spacer.into_view_child(),
            MenuItem::SubMenu(sub_menu) => sub_menu.into_view_child(),
            MenuItem::Button(button) => button.into_view_child(),
            MenuItem::Link(link) => link.into_view_child(),
            MenuItem::TextInput(text_input) => text_input.into_view_child(),
            MenuItem::Slider(slider) => slider.into_view_child(),
            MenuItem::Row(row) => row.into_view_child(),
            MenuItem::Dropdown(dropdown) => dropdown.into_view_child(),
            MenuItem::MultiDropdown(multi_dropdown) => multi_dropdown.into_view_child(),
        }
    }
}

/// The action to perform when a button is clicked
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TypePath)]
pub enum MenuAction {
    /// Quit the game
    Quit,

    /// Go back to the previous menu in the stack
    Back,

    #[cfg(target_arch = "wasm32")]
    /// Reload the game, web only
    Reload,

    /// Open the Lobby from the [`WhichMenu::HostGame`] Menu
    HostLobby,
}

/// A button to click
/// Currently only supports the `Quit` and `Back` actions
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Button {
    /// The label to display
    label: String,

    /// The action to perform when the button is clicked
    action: MenuAction,
}

impl ViewTemplate for Button {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let label = self.label.clone();
        let action = self.action;

        QuillButton::new()
            .on_click(cx.create_callback(
                move |mut commands: Commands,
                      mut next_state: ResMut<NextState<WhichMenu>>,
                      mut menu_stack: ResMut<MenuStack>,
                      mut app_exit: EventWriter<AppExit>,
                      start_host_lobby: Res<HostLobby>| {
                    debug!("Menu Stack: {:?}", menu_stack);
                    match action {
                        MenuAction::Quit => {
                            app_exit.send(AppExit::Success);
                        }
                        MenuAction::Back => {
                            if let Some(menu) = menu_stack.pop() {
                                next_state.set(menu);
                            } else {
                                next_state.set(WhichMenu::Main);
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        MenuAction::Reload => {
                            let location = gloo_utils::window().location();
                            location.reload().unwrap();
                        }
                        MenuAction::HostLobby => {
                            commands.run_system(**start_host_lobby);
                        }
                    }
                },
            ))
            .style(menu_button_style)
            .size(Size::Xl)
            .children(label)
    }
}

/// A label to display
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Label {
    /// The label to display
    label: String,

    /// The height of the label
    #[serde(skip_serializing_if = "Option::is_none", default)]
    height: Option<f32>,

    /// The font size of the label
    #[serde(skip_serializing_if = "Option::is_none", default)]
    font_size: Option<f32>,

    /// The color of the label
    #[serde(skip_serializing_if = "Option::is_none", default)]
    color: Option<Color>,
}

impl ViewTemplate for Label {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        let label = self.label.clone();
        let height = self.height;
        let font_size = self.font_size;
        let color = self.color;

        Element::<NodeBundle>::new()
            .style((typography::text_default, move |ss: &mut StyleBuilder| {
                if let Some(height) = height {
                    ss.min_height(Val::Px(height));
                }
                if let Some(font_size) = font_size {
                    ss.font_size(font_size);
                }
                if let Some(color) = color {
                    ss.color(color);
                }
            }))
            .children(label)
    }
}

/// A link to open in a browser
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Link {
    /// The label to display
    label: String,

    /// The url to open when the link is clicked
    url: String,

    /// The icon to display next to the label
    #[serde(skip_serializing_if = "Option::is_none", default)]
    icon: Option<String>,
}

impl ViewTemplate for Link {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let url = self.url.clone();
        let label = self.label.clone();
        let icon = self.icon.clone();

        info!("TextureAssets: {:?}", cx.use_resource::<TextureAssets>());
        QuillButton::new()
            .on_click(open_link(cx, url))
            .style(menu_button_style)
            .size(Size::Xl)
            .children((
                icon.map(|asset| {
                    let asset = asset.clone();
                    Icon::new(cx.use_resource::<TextureAssets>().images[&asset].clone())
                }),
                label,
            ))
    }
}

/// The variant of the button: Primary or Default
#[derive(Serialize, Deserialize, TypePath, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MenuButtonVariant {
    /// The default button style
    #[default]
    Default,

    /// The primary button style
    Primary,
}

impl MenuButtonVariant {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl From<MenuButtonVariant> for ButtonVariant {
    fn from(variant: MenuButtonVariant) -> Self {
        match variant {
            MenuButtonVariant::Default => ButtonVariant::Default,
            MenuButtonVariant::Primary => ButtonVariant::Primary,
        }
    }
}

/// A button that switches to a different menu when clicked
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct SubMenu {
    /// The label to display on the button
    label: String,

    /// The menu to switch to
    menu: WhichMenu,

    /// The variant of the button: Primary or Default
    #[serde(skip_serializing_if = "MenuButtonVariant::is_default", default)]
    variant: MenuButtonVariant,
}

impl ViewTemplate for SubMenu {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let menu = self.menu;

        QuillButton::new()
            .on_click(cx.create_callback(
                move |mut next_state: ResMut<NextState<WhichMenu>>,
                      current_state: Res<State<WhichMenu>>,
                      mut menu_stack: ResMut<MenuStack>| {
                    menu_stack.push(current_state.to_owned());
                    next_state.set(menu);
                    debug!("Menu Stack: {:?}", menu_stack);
                },
            ))
            .style(menu_button_style)
            .size(Size::Xl)
            .children(self.label.clone())
            .variant(self.variant.into())
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct TextInput {
    /// The label to display
    label: String,

    /// The default value of the text input
    #[serde(skip_serializing_if = "String::is_empty", default)]
    default_value: String,

    /// The maximum length of the text input
    #[serde(skip_serializing_if = "Option::is_none", default)]
    max_length: Option<usize>,

    /// The name of the text input, for fetching the value from components
    name: String,

    /// The type of the text input
    #[serde(
        skip_serializing_if = "TextInputType::is_default",
        default,
        rename(deserialize = "type", serialize = "type")
    )]
    type_: TextInputType,
}

impl ViewTemplate for TextInput {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        let label = self.label.clone();
        let default_value = self.default_value.clone();

        Element::<NodeBundle>::new()
            .style((menu_labeled_style, typography::text_strong))
            .children((
                label,
                QuillTextInput::new()
                    .named(&self.name)
                    .default_value(default_value)
                    .max_length(self.max_length)
                    .style(menu_text_input_style)
                    .size(Size::Xl)
                    .type_(self.type_),
            ))
    }
}

/// A slider to select a value
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Slider {
    /// The label to display
    label: String,

    /// The default value of the slider
    value: usize,

    /// The minimum value of the slider
    min: usize,

    /// The maximum value of the slider
    max: usize,

    /// The name of the slider, for fetching the value from components
    name: String,
}

#[derive(Component, Debug, Default, Clone, Reflect, Deref, DerefMut, PartialEq, Eq)]
pub struct SliderValue(usize);

impl ViewTemplate for Slider {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let id = cx.create_entity();
        let label = self.label.clone();
        let value = cx
            .use_component_or::<SliderValue>(id, SliderValue(self.value))
            .clone();
        let value = cx.create_mutable(*value);

        Element::<NodeBundle>::for_entity(id)
            .style((menu_labeled_style, typography::text_strong))
            .insert_dyn(Name::new, self.name.clone())
            .insert_dyn(SliderValue, value.get(cx))
            .children((
                label,
                QuillSlider::new()
                    .value(value.get(cx) as f32)
                    .precision(0)
                    .step(1.0)
                    .range(self.min as f32..=self.max as f32)
                    .style(menu_text_input_style)
                    .on_change(cx.create_callback(move |v: In<f32>, world: &mut World| {
                        info!("Slider value changed to {:?}", *v);
                        let mut ent = world.entity_mut(id);
                        if !ent.contains::<SliderValue>() {
                            ent.insert(SliderValue(v.round() as usize));
                        }
                        value.set(world, v.round() as usize);
                    })),
            ))
    }
}

/// A row of several items
#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Row(Vec<MenuItem>);

impl ViewTemplate for Row {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        Element::<NodeBundle>::new().style(menu_row_style).children(
            self.0
                .iter()
                .map(|item| item.clone().into_view_child())
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Dropdown {
    label: String,
    source: TriviaSource,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    selected: Option<usize>,
}

impl ViewTemplate for Dropdown {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        let label = self.label.clone();
        let source = self.source.clone();

        Element::<NodeBundle>::new()
            .style((menu_labeled_style, typography::text_strong))
            .children(
                MenuButton::new().children(label).popup(
                    MenuPopup::new().children(
                        source
                            .iter()
                            .map(|s| QuillMenuItem::new().label(s).into_view_child())
                            .collect::<Vec<_>>(),
                    ),
                ),
            )
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct MultiDropdown {
    /// The label to display
    label: String,
    /// The options to choose from
    options: TriviaSource,
    /// The selected options
    selected: Vec<usize>,
    /// The name of the dropdown, for fetching the value from components
    name: String,
}

impl ViewTemplate for MultiDropdown {
    type View = impl View;

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        let label = self.label.clone();
        let options = self.options.clone();
        let selected = &self.selected;

        QuillMultiDropdown::new()
            .label(label)
            .source(options)
            .selected(selected)
            .named(&self.name)
            .into_view_child()
    }
}
