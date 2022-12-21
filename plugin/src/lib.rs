use rea_rs::{
    ActionKind, MidiMessage, NotationMessage, PluginContext, Reaper,
};
use reaper_macros::reaper_extension_plugin;
use std::error::Error;

pub fn print_midi() {
    let pr = Reaper::get().current_project();
    let it = pr.get_selected_item(0).unwrap();
    let take = it.active_take();
    take.iter_midi(None)
        .unwrap()
        .map(|event| {
            let msg = event.message();
            match NotationMessage::from_raw(msg.get_raw()) {
                None => (),
                Some(msg) => {
                    println!("notation: {}", msg);
                }
            }
        })
        .count();
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    Reaper::load(context);
    let rpr = Reaper::get_mut();
    let _id = rpr.register_action(
        "test_ext_print_midi",
        "TestExtension: print_midi",
        |_| Ok(print_midi()),
        ActionKind::NotToggleable,
    );
    Ok(())
}
