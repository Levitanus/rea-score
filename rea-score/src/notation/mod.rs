use rea_rs::midi::Notation as MNotation;
use rea_rs::NotationMessage;

use self::chord_notations::ChordNotations;
use self::note_notations::NoteNotations;

pub mod chord_notations;
pub mod note_notations;

/// Get reascore notations, if any.
fn notations_from_midi(msg: NotationMessage) -> Option<Vec<NotationType>> {
    match msg.notation() {
        MNotation::Note(ch, nt, tk) => {
            let tokens = reascore_tokens(tk)?;
            let notes = tokens.iter().filter_map(|tk| {
                Some(NotationType::Note(ch, nt, tk.parse().ok()?))
            });
            let chords = tokens.iter().filter_map(|tk| {
                Some(NotationType::Chord(ch, nt, tk.parse().ok()?))
            });
            Some(notes.chain(chords).collect())
        }
        MNotation::Track(_) => todo!(),
        MNotation::Unknown(_) => todo!(),
    }
}

/// Get reascore tokens, if any.
fn reascore_tokens(tokens: Vec<String>) -> Option<Vec<String>> {
    let v: Vec<String> = tokens
        .into_iter()
        .filter(|tk| tk.starts_with("#ReaScore"))
        .collect();
    match v.len() {
        0 => None,
        1 => {
            let mut tokens = v[0].split(":");
            tokens.next()?;
            Some(
                String::from(tokens.next()?)
                    .split("|")
                    .map(|v| String::from(v))
                    .collect(),
            )
        }
        _ => {
            eprintln!("More than one ReaScore token: {:?}", v);
            None
        }
    }
}

/// Handles parsing and representation of raw notation
#[derive(Debug)]
pub enum NotationType {
    /// channel, note
    Note(u8, u8, NoteNotations),
    /// channel, note: note still presents, as chord
    /// events will be deduplicated by mapping to events.
    Chord(u8, u8, ChordNotations),
    Event,
}
