use std::{collections::HashMap, str::FromStr};

use fraction::Fraction;
use rea_rs::{PluginContext, Reaper, Timer};
use rea_score::{
    dom::midi_parse::{
        notations_to_first_and_last_selected,
        notations_to_first_selected,
    },
    notation::NotationType,
};
use reaper_imgui::{
    Context, ImGui, KeyBinding, KeyCode, KeyModifier,
};

use crate::error_box;

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
            if ctx.got_key_binding(
                &KeyBinding::new([], KeyCode::Escape),
                false,
            ) {
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
        KeyBinding::new(
            [KeyModifier::Ctrl, KeyModifier::Shift],
            KeyCode::D,
        ),
        Box::new(apply_dynamics),
    );
    kb.insert(
        KeyBinding::new([KeyModifier::Ctrl], KeyCode::T),
        Box::new(make_tuplet),
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
        Ok(i) => i
            .get("dynamic")
            .expect("should be value here")
            .to_string(),
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
            return error_box("Error!", format!("{}",err));
        }
    }
}

fn make_tuplet() {
    let rpr = Reaper::get();
    let rate_str = match rpr.get_user_inputs(
        "Type tuplet rate in form of '3/2' for regular triplet",
        vec!["rate"],
        None,
    ) {
        Ok(i) => {
            i.get("rate").expect("should be value here").to_string()
        }
        Err(_) => return,
    };
    if rate_str.is_empty() {
        return;
    }
    let rate = match Fraction::from_str(rate_str.as_str()) {
        Ok(rate) => rate,
        Err(err) => {
            return error_box(
                "Wrong rate string",
                format!(
                    "please, type the rate string in form of '3/2'\
                    \n original error: {}",
                    err
                ),
            );
        }
    };
    match notations_to_first_and_last_selected(vec![NotationType::Chord(
        rea_score::notation::chord_notations::ChordNotations::TupletRate(
            rate,
        ),
    )],vec![NotationType::Chord(
        rea_score::notation::chord_notations::ChordNotations::TupletEnd)]) {
        Ok(()) => (),
        Err(err) => {
            return error_box("Error!", format!("{}",err));
        }
    }
}
