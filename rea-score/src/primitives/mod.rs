//! Elements, from which DOM constructed.
//!
//! At first, one TimeMap is created from reaper time-line (or else).
//! Then Voice is created, based on the TimeMap.
//! Then it is filled by events.
//! Then multiple voices organized in DOM.
//! then rendered to *.ly source file and compiled by LilyPong.

pub mod container;
pub mod event;
pub mod fraction_tools;
pub mod length;
pub mod measure;
pub mod pitch;
pub mod position;
pub mod time_map;

pub use event::{Chord, EventInfo, EventType, Note};
pub use fraction_tools::{limit_denominator, normalize_fraction};
pub use length::Length;
pub use measure::Measure;
pub use pitch::{
    midi_to_note, Accidental, Key, NoteName, Octave, Pitch, ResolvedPitch,
    Scale,
};
pub use position::{AbsolutePosition, RelativeDistance, RelativePosition};
pub use time_map::{MeasureInfo, TimeMap, TimeMapMeasures};

static LIMIT_DENOMINATOR: u64 = 128;

#[cfg(test)]
mod tests {

    #[test]
    fn test_measure() {}
}
