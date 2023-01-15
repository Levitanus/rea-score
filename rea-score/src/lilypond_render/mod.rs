use musical_note::Key;
use rea_rs::{ExtState, Reaper, TimeSignature};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderSettings {
    pub key: Key,
}
impl RenderSettings {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
    fn default() -> Self {
        Self {
            key: Key::from_str("c", musical_note::Scale::Major)
                .expect("Should be valid key"),
        }
    }
}

pub trait RendersToLilypond {
    fn render_lilypond(&self) -> String;
    fn global_render_settings() -> RenderSettings {
        if !Reaper::is_available() {
            return RenderSettings::default();
        }
        let rpr = Reaper::get();
        let pr = rpr.current_project();
        let settings = ExtState::new(
            "ReaScore",
            "render settings",
            RenderSettings::default(),
            true,
            &pr,
        );
        settings
            .get()
            .expect("Render Settings should be initialized anyway")
    }
}

impl RendersToLilypond for TimeSignature {
    fn render_lilypond(&self) -> String {
        let (num, denom) = (self.numerator, self.denominator);
        format!(r"\time {num}/{denom}")
    }
}

fn pixels_to_mm(size: (u32, u32), dpi: u32) -> (u32, u32) {
    let dpm = match dpi {
        x if x > 50 => dpi * 100 / 254,
        _ => 10,
    };
    (size.0 * 10 / dpm, size.1 * 10 / dpm)
}

pub fn preview_string(
    string: impl Into<String>,
    path: impl Into<PathBuf>,
    size: impl Into<Option<(u32, u32)>>,
    dpi: impl Into<Option<u32>>,
) -> Result<(), std::io::Error> {
    // line-width=120\mm
    // page-breaking = #ly:one-line-breaking
    // \include "lilypond-book-preamble.ly"
    let string = string.into();
    let size = size.into().unwrap_or((300, 100));
    let dpi = dpi.into().unwrap_or(101);
    let (w_mm, h_mm) = pixels_to_mm(size, dpi);
    let string = format!(
        r###"\version "2.24"
        \paper{{
            indent=0\mm
            oddFooterMarkup=##f
            oddHeaderMarkup=##f
            bookTitleMarkup = ##f
            scoreTitleMarkup = ##f
            #(set-paper-size '(cons (* {w_mm} mm) (* {h_mm} mm)))
        }}
        {{{string}}}
        "###
    );
    let path = path.into();
    // let input_path = path.with_extension("ly");
    // let mut input_file = File::create(input_path.clone())?;
    // input_file.write_all(string.as_bytes())?;
    let output_path = path.with_extension("");
    let mut lily = {
        Command::new("lilypond")
            .stdin(Stdio::piped())
            .arg(format!(
                "--output={}",
                output_path
                    .to_str()
                    .expect("can not make output file path")
            ))
            // .arg("-dbackend=svg")
            // .arg("-danti-alias-factor=8")
            .arg("--png")
            .arg("-dbackend=eps")
            .arg("-dno-gs-load-fonts")
            .arg("-dinclude-eps-fonts")
            .arg("-ddelete-intermediate-files")
            .arg(format!("-dresolution={dpi}"))
            .arg("-dgui=#t")
            // .arg("-dcrop=#t")
            // .arg("-dpreview")
            .arg("-s")
            .arg("-")
            // .arg(input_path.to_str().expect("can not get path"))
            .spawn()?
    };
    let mut stdin = lily.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(string.as_bytes())
            .expect("Failed to write to stdin");
    });

    let output =
        lily.wait_with_output().expect("Failed to read stdout");
    let output = String::from_utf8_lossy(&output.stdout);
    if !output.is_empty() {
        eprint!("{output}");
    }
    Ok(())
}

pub fn preview_file(path: PathBuf) {
    open::that(path.with_extension("png"))
        .expect("Can not open path");
}
