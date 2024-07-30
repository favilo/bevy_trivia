use bevy::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_quill::*;
use bevy_quill_obsidian::{
    colors,
    controls::{Button, ButtonVariant, Spacer},
    size::Size,
    typography,
};

use super::{menu_button_style, menu_style, WhichMenu};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MainMenu;

impl ViewTemplate for MainMenu {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .named("MainMenu")
            .style(menu_style)
            .children((
                Element::<NodeBundle>::new()
                    .style((typography::text_default, move |ss: &mut StyleBuilder| {
                        ss.min_height(Val::Px(150.0))
                            .font_size(48.0)
                            .color(colors::PRIMARY);
                    }))
                    .children("Bevy Trivia"),
                Button::new()
                    .variant(ButtonVariant::Primary)
                    .on_click(
                        cx.create_callback(|mut next_state: ResMut<NextState<WhichMenu>>| {
                            next_state.set(WhichMenu::Play);
                        }),
                    )
                    .size(Size::Xl)
                    .style(menu_button_style)
                    .children("Play"),
                Button::new()
                    .on_click(
                        cx.create_callback(|mut next_state: ResMut<NextState<WhichMenu>>| {
                            next_state.set(WhichMenu::Settings);
                        }),
                    )
                    .size(Size::Xl)
                    .style(menu_button_style)
                    .children("Settings"),
                Spacer,
                Button::new()
                    .on_click(
                        cx.create_callback(|mut next_state: ResMut<NextState<WhichMenu>>| {
                            next_state.set(WhichMenu::Credits);
                        }),
                    )
                    .size(Size::Xl)
                    .style(menu_button_style)
                    .children("Credits"),
            ))
    }
}
