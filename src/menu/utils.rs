use bevy::log::warn;
use bevy_quill::{Callback, Cx};

pub fn open_link(cx: &mut Cx, link: &'static str) -> Callback {
    cx.create_callback(|| {
        if let Err(error) = webbrowser::open(link) {
            warn!("Failed to open link {error:?}");
        }
    })
}
