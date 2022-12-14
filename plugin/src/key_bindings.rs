use std::collections::HashMap;

use rea_rs::{PluginContext, Reaper, Timer};
use rea_score::{
    dom::midi_parse::notations_to_first_selected, notation::NotationType,
};
use reaper_imgui::{Context, ImGui, KeyBinding, KeyCode, KeyModifier};

pub struct KeyBindings {
    _im_gui: ImGui,
    ctx: Context,
    key_bindings: HashMap<KeyBinding, Box<dyn Fn()>>,
}
impl KeyBindings {
    pub fn init(context: PluginContext) -> () {
        let rpr = Reaper::get_mut();
        let _im_gui = ImGui::load(context);
        let ctx = _im_gui.create_context("ReaScore KeyBindings");
        let obj = Self {
            _im_gui,
            ctx,
            key_bindings: make_key_bindings(),
        };
        rpr.register_timer(Box::new(obj))
    }
}
impl Timer for KeyBindings {
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ctx.capture_keyboard(true);
        let mut stop = false;
        if !self.ctx.window("ReaScore keybindings").open(|ctx| {
            if ctx
                .got_key_binding(&KeyBinding::new([], KeyCode::Escape), false)
            {
                println!("got escape: stopping");
                stop = true;
            }
            if let Some(ch) = ctx.got_input() {
                println!("got input: {ch}, stopping");
                stop = true;
            }
            for (kb, func) in self.key_bindings.iter() {
                if ctx.got_key_binding(kb, false) {
                    func();
                    stop = true;
                }
            }
        }) || stop
        {
            self.stop();
        }

        Ok(())
    }

    fn id_string(&self) -> String {
        "ReaScore KeyBindings".to_string()
    }
}

fn make_key_bindings() -> HashMap<KeyBinding, Box<dyn Fn()>> {
    let mut kb: HashMap<KeyBinding, Box<dyn Fn()>> = HashMap::new();
    kb.insert(
        KeyBinding::new([KeyModifier::Ctrl, KeyModifier::Shift], KeyCode::D),
        Box::new(apply_dynamics),
    );

    kb
}

fn apply_dynamics() {
    let rpr = Reaper::get();
    let dyn_str = match rpr.get_user_inputs(
        "Type dynamics definition",
        vec!["dynamic"],
        None,
    ) {
        Ok(i) => i.get("dynamic").expect("should be value here").to_string(),
        Err(_) => return,
    };
    if dyn_str.is_empty() {
        return;
    }
    match notations_to_first_selected(vec![NotationType::Chord(
        rea_score::notation::chord_notations::ChordNotations::Dynamics(
            dyn_str,
        ),
    )]) {
        Ok(()) => (),
        Err(err) => {
            rpr.show_message_box(
                "Error!",
                err.to_string(),
                rea_rs::MessageBoxType::Ok,
            )
            .unwrap_or(rea_rs::MessageBoxValue::Ok);
        }
    }
}
