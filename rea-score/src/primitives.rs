use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Sub},
};

use fraction::Fraction;
use reaper_medium::PositionInQuarterNotes;

use crate::reaper;

pub use reaper::TimeSignature;

pub struct TimeMap {
    measures: HashMap<u32, Measure>,
}
impl TimeMap {
    pub fn pos_relative_from_absolute(self, absolute: AbsolutePosition) -> Option<RelativePosition> {
        let mut counted_abs = AbsolutePosition::from(0.0);
        for (idx, measure) in self.measures {
            counted_abs += measure.length;
            if counted_abs > absolute {
                return Some(RelativePosition {
                    measure_index: idx,
                    measure_position: MeasurePosition::from((absolute - counted_abs).position),
                });
            }
        }
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct Measure {
    index: u32,
    time_signature: TimeSignature,
    events: HashMap<MeasurePosition, EventInfo>,
    length: Length,
}
impl Measure {
    pub fn new(index: u32, time_signature: TimeSignature) -> Self {
        let length = Length {
            fraction: Fraction::new(time_signature.numerator, time_signature.denominator),
        };
        let mut events = HashMap::new();
        events.insert(
            MeasurePosition::from(0.0),
            EventInfo {
                position: RelativePosition::new(index, MeasurePosition::from(0.0)),
                length: length.clone(),
                event: EventType::Rest,
            },
        );
        Self {
            index,
            time_signature,
            events,
            length,
        }
    }
}

pub type MeasurePosition = Fraction;
// type MeasureLength = Fraction;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct AbsolutePosition {
    position: Fraction,
}
impl AbsolutePosition {
    pub fn new(position: Fraction) -> Self {
        Self { position }
    }
}
impl Add for AbsolutePosition {
    fn add(self, rhs: Self) -> Self {
        Self {
            position: self.position + rhs.position,
        }
    }

    type Output = Self;
}
impl Sub for AbsolutePosition {
    fn sub(self, rhs: Self) -> Self {
        Self {
            position: self.position - rhs.position,
        }
    }

    type Output = Self;
}
impl Add<Length> for AbsolutePosition {
    fn add(self, rhs: Length) -> Self::Output {
        Self {
            position: self.position + rhs.fraction,
        }
    }

    type Output = Self;
}
impl AddAssign<Length> for AbsolutePosition {
    fn add_assign(&mut self, rhs: Length) {
        self.position += rhs.fraction
    }
}
impl From<reaper::Position> for AbsolutePosition {
    fn from(position: reaper::Position) -> Self {
        Self {
            position: MeasurePosition::from(position.quarters_from_project_start.get() / 4.0),
        }
    }
}
impl Into<reaper::Position> for AbsolutePosition {
    fn into(self) -> reaper::Position {
        let ps = self.position;
        let absolute: f64 = (*ps.numer().unwrap() as f64) / (*ps.denom().unwrap() as f64);
        reaper::Position::from_beats(PositionInQuarterNotes::new(absolute * 4.0))
    }
}
impl From<f64> for AbsolutePosition {
    fn from(value: f64) -> Self {
        Self {
            position: Fraction::from(value),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RelativePosition {
    measure_index: u32,
    measure_position: MeasurePosition,
}
impl RelativePosition {
    pub fn new(measure_index: u32, measure_position: MeasurePosition) -> Self {
        Self {
            measure_index,
            measure_position,
        }
    }
    pub fn from_absolute(time_map: TimeMap, absolute: AbsolutePosition) -> Option<Self> {
        time_map.pos_relative_from_absolute(absolute)
    }
}
impl From<reaper::Position> for RelativePosition {
    fn from(value: reaper::Position) -> Self {
        Self {
            measure_index: value.bar as u32,
            measure_position: MeasurePosition::from(value.quarters_from_bar_start.get() / 4.0),
        }
    }
}
impl From<AbsolutePosition> for RelativePosition {
    fn from(position: AbsolutePosition) -> Self {
        let rpr_pos: reaper::Position = position.into();
        Self::from(rpr_pos)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Length {
    fraction: Fraction,
}
#[derive(Debug, PartialEq, PartialOrd)]
pub struct EventInfo {
    position: RelativePosition,
    length: Length,
    event: EventType,
    // event: Box<dyn Event>
}
#[derive(Debug, PartialEq, PartialOrd)]
pub enum EventType {
    Rest,
    Note(Note),
    Chord,
}
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Rest {}
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Note {
    pitch: Pitch,
}
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Pitch {
    midi: u8,
    note: NoteName,
    accidental: Accidental,
    octave: i8,
}
#[derive(Debug, PartialEq, PartialOrd)]
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Accidental {
    Natural,
    Sharp,
    DoubleSharp,
    Flat,
    DoubleFlat,
}

#[cfg(test)]
mod tests {
    use reaper_medium::PositionInQuarterNotes;

    use crate::{primitives::MeasurePosition, reaper as rpr};

    use super::RelativePosition;

    #[test]
    fn position() {
        let a = RelativePosition::from(rpr::Position {
            bar: 2,
            quarters_from_bar_start: PositionInQuarterNotes::new(1.0),
            quarters_from_bar_end: PositionInQuarterNotes::new(3.0),
            quarters_from_project_start: PositionInQuarterNotes::new(9.0),
        });
        assert_eq!(
            a,
            RelativePosition {
                measure_index: 2,
                measure_position: MeasurePosition::new(1u64, 4u16)
            }
        );
    }
}
