use bevy::prelude::*;
use bevy_quill::*;
use bevy_quill_obsidian::{
    controls::{Button, Icon, Spacer},
    size::Size,
};

use crate::loading::TextureAssets;

use super::{menu_button_style, menu_row_style, menu_style, utils::open_link, WhichMenu};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CreditsMenu;

impl ViewTemplate for CreditsMenu {
    type View = impl View;

    fn create(&self, cx: &mut bevy_quill::Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .named("CreditsMenu")
            .style(menu_style)
            .children((
                Spacer,
                Element::<NodeBundle>::new()
                    .style(menu_row_style)
                    .children((
                        Button::new()
                            .on_click(open_link(
                                cx,
                                "https://github.com/NiklasEi/bevy_game_template",
                            ))
                            .size(Size::Xl)
                            .children((
                                Icon::new(cx.use_resource::<TextureAssets>().github.clone()),
                                " Made with Bevy",
                            )),
                        Spacer,
                        Button::new()
                            .on_click(open_link(cx, "https://bevyengine.org"))
                            .size(Size::Xl)
                            .children((
                                Icon::new(cx.use_resource::<TextureAssets>().bevy.clone()),
                                " Open source",
                            )),
                    )),
                Button::new()
                    .on_click(
                        cx.create_callback(|mut next_state: ResMut<NextState<WhichMenu>>| {
                            next_state.set(WhichMenu::Main);
                        }),
                    )
                    .style(menu_button_style)
                    .size(Size::Xl)
                    .children("Back"),
            ))
    }
}
