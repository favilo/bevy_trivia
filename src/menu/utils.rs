use bevy::log::warn;
use bevy_quill::{Callback, Cx};

pub fn open_link(cx: &mut Cx, link: impl AsRef<str> + Send + Sync + 'static) -> Callback {
    cx.create_callback(move || {
        if let Err(error) = webbrowser::open(link.as_ref()) {
            warn!("Failed to open link {error:?}");
        }
    })
}

pub fn is_false(value: &bool) -> bool {
    !*value
}
