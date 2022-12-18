use std::error::Error;

use rea_rs::{ActionKind, PluginContext, Reaper};
use reaper_macros::reaper_extension_plugin;

mod dom;
mod midi;
mod notation;
mod primitives;

// pub mod rpr_connect;

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    Reaper::load(context);
    let rpr = Reaper::get_mut();
    let _id = rpr.register_action(
        "rea_score_print_midi",
        "ReaScore: print_midi",
        |_| Ok(midi::print_midi()),
        ActionKind::NotToggleable,
    );

    Ok(())
}
