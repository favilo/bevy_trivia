use bevy::{
    a11y::Focus,
    app::{App, Plugin, Update},
    log::info,
    prelude::{DetectChanges, Query, Res},
};
use bevy_trivia::menu::widgets::text_input::TextInputValue;
use jni::objects::JObject;

pub struct AndroidKeyboardPlugin;

impl Plugin for AndroidKeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, show_soft_keyboard_from_focus);
    }
}

fn show_soft_keyboard_from_focus(focus: Res<Focus>, text_value: Query<&TextInputValue>) {
    if !focus.is_changed() {
        return;
    }

    let Some(entity) = focus.0 else {
        show_soft_keyboard(false);
        return;
    };

    if text_value.get(entity).is_ok() {
        show_soft_keyboard(true);
    } else {
        show_soft_keyboard(false);
    }
}

fn show_soft_keyboard(show: bool) -> bool {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();

    let class_ctxt = env.find_class("android/content/Context").unwrap();

    let ime = env
        .get_static_field(class_ctxt, "INPUT_METHOD_SERVICE", "Ljava/lang/String;")
        .unwrap();
    let obj = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };
    let ime_manager = env
        .call_method(
            &obj,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[ime.borrow()],
        )
        .unwrap()
        .l()
        .unwrap();

    let jni_window = env
        .call_method(&obj, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
        .unwrap();
    let view = env
        .call_method(&jni_window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
        .unwrap();

    if show {
        let result = env
            .call_method(
                &ime_manager,
                "showSoftInput",
                "(Landroid/view/View;I)Z",
                &[(&view).into(), 0i32.into()],
            )
            .unwrap()
            .z()
            .unwrap();
        info!("showSoftInput result: {:?}", result);
        result
    } else {
        let window_token = env
            .call_method(view, "getWindowToken", "()Landroid/os/IBinder;", &[])
            .unwrap()
            .l()
            .unwrap();

        let result = env
            .call_method(
                &ime_manager,
                "hideSoftInputFromWindow",
                "(Landroid/os/IBinder;I)Z",
                &[(&window_token).into(), 0i32.into()],
            )
            .unwrap()
            .z()
            .unwrap();
        info!("hideSoftInputFromWindow result: {:?}", result);
        result
    }
}
