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
    menu_button_style, menu_row_style, menu_style, utils::open_link, MenuStack, WhichMenu,
};

#[derive(Asset, Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Menu {
    title: String,
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
            ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TypePath)]
pub enum MenuItem {
    Label(Label),
    Spacer,
    SubMenu(SubMenu),
    Button(Button),
    Link(Link),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TypePath)]
pub enum MenuAction {
    Quit,
    Back,
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Button {
    label: String,
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

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Label {
    label: String,
    height: Option<f32>,
    font_size: Option<f32>,
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

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct Link {
    label: String,
    url: String,
    icon: Option<String>,
}

impl ViewTemplate for Link {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let url = self.url.clone();
        let label = self.label.clone();
        let icon = self.icon.clone();

        QuillButton::new()
            .on_click(open_link(cx, Box::leak(Box::new(url))))
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

#[derive(Serialize, Deserialize, TypePath, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MenuButtonVariant {
    #[default]
    Default,
    Primary,
}

impl From<MenuButtonVariant> for ButtonVariant {
    fn from(variant: MenuButtonVariant) -> Self {
        match variant {
            MenuButtonVariant::Default => ButtonVariant::Default,
            MenuButtonVariant::Primary => ButtonVariant::Primary,
        }
    }
}

#[derive(Serialize, Deserialize, TypePath, Clone, Debug, PartialEq)]
pub struct SubMenu {
    label: String,
    menu: WhichMenu,
    varaint: Option<MenuButtonVariant>,
}

impl ViewTemplate for SubMenu {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        let menu = self.menu;

        let mut button = QuillButton::new()
            .on_click(
                cx.create_callback(move |mut next_state: ResMut<NextState<WhichMenu>>| {
                    next_state.set(menu);
                }),
            )
            .style(menu_button_style)
            .size(Size::Xl)
            .children(self.label.clone());

        if let Some(variant) = self.varaint {
            button = button.variant(variant.into());
        }

        button
    }
}

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
