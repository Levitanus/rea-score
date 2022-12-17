// use reaper_high as rh;
// use reaper_low::PluginContext;
// use reaper_macros::reaper_extension_plugin;
// use reaper_medium as rm;
// use reaper_medium::ReaperSession;
// use std::error::Error;

pub mod dom;
pub mod primitives;
// pub mod rpr_connect;

// #[reaper_extension_plugin]
// fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
//     let session = ReaperSession::load(context);
//     session
//         .reaper()
//         .show_console_msg("Hello world from reaper-rs medium-level API!");
//     Ok(())
// }
