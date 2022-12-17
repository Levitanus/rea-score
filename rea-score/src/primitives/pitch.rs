pub use musical_note::{
    midi_to_note, Accidental, Key, NoteName, Octave, Scale,
};

use musical_note::{Note, ResolvedNote};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ResolvedPitch {
    Name(String),
    Note(ResolvedNote),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Pitch {
    note: Note,
    note_name: Option<String>,
}
impl Pitch {
    /// note_name used to change note pitch to lilypond strings in MIDI Editor.
    pub fn from_midi(
        midi: u8,
        accidental: Option<Accidental>,
        note_name: Option<String>,
    ) -> Self {
        Self {
            note: Note::from_midi(midi, accidental),
            note_name,
        }
    }
    pub fn resolve(&self, key: Key) -> ResolvedPitch {
        match &self.note_name {
            Some(name) => ResolvedPitch::Name(name.to_string()),
            None => ResolvedPitch::Note(self.note.resolve(key)),
        }
    }
}
