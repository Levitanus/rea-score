use rea_rs::{PluginContext, Reaper};
use rea_rs_macros::reaper_extension_plugin;

use std::error::Error;
mod preview_window;

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
    println!("loaded! action result: {:?}", _id);
    Ok(())
}
