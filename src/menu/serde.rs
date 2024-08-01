use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use bevy_mod_stylebuilder::{StyleBuilder, StyleBuilderFont, StyleBuilderLayout};
use bevy_quill::*;
use bevy_quill_obsidian::{
    colors,
    controls::{Button as QuillButton, ButtonVariant, Icon, Spacer},
    size::Size,
    typography,
};
use serde::{Deserialize, Serialize};

use crate::loading::TextureAssets;

use super::{
    menu_button_style, menu_row_style, menu_style,
    utils::{is_false, open_link},
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

    fn create(&self, _cx: &mut bevy_quill::Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .named(&self.title)
            .style(menu_style)
            .children((
                Element::<NodeBundle>::new()
                    .style((typography::text_default, move |ss: &mut StyleBuilder| {
                        ss.min_height(Val::Px(150.0))
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
                    MenuItem::Button(Button {
                        label: "Quit".to_string(),
                        action: MenuAction::Quit,
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

    /// A row of several items
    Row(Row),
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
            MenuItem::Row(row) => row.into_view_child(),
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
                move |mut next_state: ResMut<NextState<WhichMenu>>,
                      mut menu_stack: ResMut<MenuStack>,
                      mut app_exit: EventWriter<AppExit>| {
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
