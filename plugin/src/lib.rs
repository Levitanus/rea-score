use rea_rs::{
    keys::{FVirt, KeyBinding, VKeys},
    IntEnum, PluginContext, Reaper,
};
use rea_rs_macros::reaper_extension_plugin;

use std::error::Error;
mod key_bindings;
mod preview_window;

use key_bindings::KeyBindings;
use preview_window::PreviewWindow;

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    print!("rea_score extension... ");
    Reaper::init_global(context);
    let rpr = Reaper::get_mut();
    let temp_path = std::env::temp_dir();
    let _id = rpr.register_action(
        "rea-score preview",
        "ReaScore: preview window",
        move |_| Ok(PreviewWindow::init(context, temp_path.clone())),
        None,
    );
    println!("loaded preview! action result: {:?}", _id);

    let _id = rpr.register_action(
        "rea-score key_binding",
        "ReaScore: 2nd level key binding",
        move |_| Ok(KeyBindings::init(context)),
        KeyBinding::new(
            FVirt::FALT | FVirt::FCONTROL,
            VKeys::VK_A.int_value() as u16,
        ),
    );
    println!("loaded keybindings! action result: {:?}", _id);
    Ok(())
}

/// Show error box with OK button to user
fn error_box(title: impl Into<String>, msg: impl Into<String>) {
    Reaper::get()
        .show_message_box(
            title,
            format!("Error occurred, while preview rendered:\n{}", msg.into()),
            rea_rs::MessageBoxType::Ok,
        )
        .expect("Error while displaying error");
}
