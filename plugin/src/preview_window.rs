use std::{
    error::Error,
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
    thread,
    time::Instant,
};

use rea_rs::{
    ExtState, Measure, PluginContext, Position, Reaper, Timer,
};
use rea_score::lilypond_render::{
    preview_string, RendersToLilypond,
};
use reaper_imgui::{
    Context, ContextFlags, Dock, ImGui, ImageHandle, SetWidth, Size,
};
use serde::{Deserialize, Serialize};

use crate::error_box;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    code: String,
    dpi: u32,
    dock: Dock,
    preview_bars_amount: u32,
    render_lag: u32,
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
    cursor_pos: Position,
    last_render: Instant,
    ready_for_render: bool,
}
impl PreviewWindow {
    pub fn init(
        context: PluginContext,
        temp_path: impl Into<PathBuf>,
    ) {
        let imgui = ImGui::load(context);
        let mut ctx = imgui
            .create_context("ReaScore preview")
            .with_flags(ContextFlags::DockingEnable);
        let temp_path =
            temp_path.into().join("rea_score_preview.png");
        let image = ctx.image_handle(temp_path);
        let state = State {
            code: String::from("c'"),
            dpi: 80,
            dock: Dock::Reaper(3),
            preview_bars_amount: 4,
            render_lag: 500,
        };
        let state = ExtState::new(
            "ReaScore",
            "preview window",
            state,
            true,
            Reaper::get(),
        );
        let next_dock = state
            .get()
            .expect("can not load Preview Window state")
            .dock;
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
            cursor_pos: Default::default(),
            last_render: Instant::now(),
            ready_for_render: false,
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
            preview_string(
                text,
                path,
                (size.width, size.height),
                dpi,
            )
            .expect("Can not preview string");
            send.send(true)
        });
    }
    fn check_item(&mut self) {
        let now = Instant::now();
        let rpr = Reaper::get();
        let pr = rpr.current_project();
        let cursor_pos = pr.get_cursor_position();
        let track = match pr.get_selected_track(0) {
            Some(tr) => tr,
            None => return,
        };
        let hash = match track.midi_hash(false) {
            Some(hash) => hash,
            None => return,
        };
        if hash == self.hash && cursor_pos == self.cursor_pos {
            return;
        }
        if !self.ready_for_render {
            self.ready_for_render = true;
            self.last_render = now;
            return;
        }
        let render_lag = self.state().render_lag;
        if now.duration_since(self.last_render).as_millis()
            < render_lag as u128
        {
            return;
        }
        self.ready_for_render = false;
        self.last_render = now;
        self.hash = hash;
        self.cursor_pos = cursor_pos;
        let (start_pos, end_pos) =
            match self.preview_bounds(rpr, &pr) {
                Ok(value) => value,
                Err(value) => return value,
            };
        let code = rea_score::dom::parse_track_in_bounds(
            track, start_pos, end_pos,
        );
        let code = match code {
            Ok(c) => c,
            Err(err) => {
                return error_box(
                    "Error while rendering preview!",
                    err.to_string(),
                )
            }
        };
        let mut state = self
            .state
            .get()
            .expect("can not load Preview Window state");
        let code = code.render_lilypond();
        println!("Lily code:\n{code}");
        state.code = code;
        self.state.set(state);
        self.render = true;
    }

    fn preview_bounds(
        &mut self,
        rpr: &Reaper,
        pr: &rea_rs::Project,
    ) -> Result<(Position, Position), ()> {
        let cursor_pos: Position = match rpr.active_midi_editor() {
            Some(e) => {
                match e.item(pr).active_take().iter_midi(None) {
                    Ok(mid) => match mid
                        .filter_note_on()
                        .filter(|ev| ev.selected())
                        .next()
                    {
                        Some(ev) => Position::from_ppq(
                            ev.ppq_position(),
                            &e.item(pr).active_take(),
                        ),
                        None => pr.get_cursor_position(),
                    },
                    Err(err) => {
                        return Err(error_box(
                            "Error while rendering preview!",
                            err.to_string(),
                        ))
                    }
                }
            }
            None => pr.get_cursor_position(),
        };
        let bars = self.state().preview_bars_amount;
        let mut start_idx =
            Measure::from_position(cursor_pos, pr).index;
        if start_idx > bars / 2 {
            start_idx -= bars / 2;
        }
        if start_idx < 1 {
            start_idx = 1;
        }
        let end_idx = start_idx + bars - 1;
        let start_pos = Measure::from_index(start_idx, pr).start;
        let mut end_pos = Measure::from_index(end_idx, pr).end;
        println!(
            "start measure: {:#?}\nend measure: {:#?}",
            Measure::from_index(start_idx, pr),
            Measure::from_index(end_idx, pr)
        );
        if Position::new(pr.length()) < end_pos {
            end_pos = Position::new(pr.length());
        }
        Ok((start_pos, end_pos))
    }
}
impl Timer for PreviewWindow {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_item();
        let mut state = self.state();
        // println!("code before window:\n{}", state.code);
        if !self.ctx.window("preview").dock(&self.next_dock).open(
            |ctx| {
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

                ctx.sameline(200, None);

                ctx.int_input(
                    "preview bars",
                    state.preview_bars_amount as i32,
                )
                .set_width(80)
                .changed(|bars| {
                    state.preview_bars_amount = bars as u32;
                    self.render = true;
                });

                ctx.sameline(400, None);
                ctx.int_input(
                    "render lag ms",
                    state.render_lag as i32,
                )
                .set_width(80)
                .changed(|lag| state.render_lag = lag as u32);

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
            },
        ) {
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
