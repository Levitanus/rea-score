use std::{
    error::Error,
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
    thread,
};

use rea_rs::{ExtState, PluginContext, Position, Reaper, Timer};
use rea_score::{
    lilypond_render::{preview_string, RendersToLilypond},
    primitives::RelativePosition,
};
use reaper_imgui::{
    Context, ContextFlags, Dock, ImGui, ImageHandle, SetWidth, Size,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    code: String,
    dpi: u32,
    dock: Dock,
}

pub struct PreviewWindow {
    ctx: Context,
    _imgui: ImGui,
    state: ExtState<'static, State, Reaper>,
    size: Size,
    image: ImageHandle,
    reload: Option<Receiver<bool>>,
    render: bool,
    next_dock: Dock,
    hash: String,
}
impl PreviewWindow {
    pub fn init(context: PluginContext, temp_path: impl Into<PathBuf>) {
        let imgui = ImGui::load(context);
        let mut ctx = imgui
            .create_context("ReaScore preview")
            .with_flags(ContextFlags::DockingEnable);
        let temp_path = temp_path.into().join("rea_score_preview.png");
        let image = ctx.image_handle(temp_path);
        let state = State {
            code: String::from("c'"),
            dpi: 80,
            dock: Dock::Reaper(3),
        };
        let state = ExtState::new(
            "ReaScore",
            "preview window",
            state,
            true,
            Reaper::get(),
        );
        let next_dock =
            state.get().expect("can not load Preview Window state").dock;
        Reaper::get_mut().register_timer(Box::new(Self {
            ctx,
            _imgui: imgui,
            state,
            size: Default::default(),
            image,
            reload: None,
            render: true,
            next_dock,
            hash: Default::default(),
        }));
    }
    fn state(&self) -> State {
        self.state.get().expect("Can not load Window state!")
    }

    fn render_input(&mut self) {
        let (send, recieve) = channel();
        self.reload = Some(recieve);
        let state = self.state();
        let path = self.image.path.clone();
        let text = state.code.clone();
        let size = self.size;
        let dpi = state.dpi;
        thread::spawn(move || {
            preview_string(text, path, (size.width, size.height), dpi)
                .expect("Can not preview string");
            send.send(true)
        });
    }
    fn check_item(&mut self) {
        let pr = Reaper::get().current_project();
        let track = match pr.get_selected_track(0) {
            Some(tr) => tr,
            None => return,
        };
        let hash = match track.midi_hash(false) {
            Some(hash) => hash,
            None => return,
        };
        if hash == self.hash {
            return;
        }
        self.hash = hash;
        let start_pos: Position = RelativePosition::new(1, 0.0.into()).into();
        let end_pos = Position::new(pr.length());
        let code =
            rea_score::dom::parse_track_in_bounds(track, start_pos, end_pos);
        // println!("----CODE AS DOM: ---\n{:#?}", code);
        let code = match code {
            Ok(c) => c,
            Err(err) => {
                Reaper::get()
                    .show_message_box(
                        "Error while rendering preview!",
                        format!(
                            "Error occurred, while preview rendered:\n{err}"
                        ),
                        rea_rs::MessageBoxType::Ok,
                    )
                    .expect("Error while displaying error");
                return;
            }
        };
        let mut state =
            self.state.get().expect("can not load Preview Window state");
        let code = code.render_lilypond();
        println!("Lily code:\n{code}");
        state.code = code;
        self.state.set(state);
        self.render = true;
    }
}
impl Timer for PreviewWindow {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_item();
        let mut state = self.state();
        // println!("code before window:\n{}", state.code);
        if !self
            .ctx
            .window("preview")
            .dock(&self.next_dock)
            .open(|ctx| {
                let mut size = ctx.window_viewport().work_size();
                if size.height > 40 {
                    size.height -= 40;
                }
                if self.size != size {
                    self.size = size;
                    self.render = true;
                }
                ctx.int_input("dpi", state.dpi as i32)
                    .set_width(80)
                    .changed(|dpi| {
                        state.dpi = dpi as u32;
                        self.render = true;
                    });
                ctx.sameline(120, None);
                self.next_dock = ctx
                    .dock_widget("dock", &mut state.dock)
                    .set_width(80)
                    .next_dock();
                let reload = match &self.reload {
                    None => false,
                    Some(v) => match v.try_recv() {
                        Ok(_) => true,
                        Err(_) => false,
                    },
                };
                ctx.image(self.image.clone())
                    .expect("handle not from context")
                    .force_reload(reload)
                    .show();
                self.state.set(state.clone());
            })
        {
            self.stop()
        };

        if self.render {
            self.render_input();
            self.render = false;
        }

        Ok(())
    }

    fn id_string(&self) -> String {
        "ReaScore preview".to_string()
    }
}
