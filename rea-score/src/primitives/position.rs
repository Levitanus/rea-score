//! Everything needed to manipulate positions of events.
//!
//! Mainly, there are two kinds of positions: absolute and relative.
//!
//! Relative represents bar number and distance from bar start.
//! Absolute represents distance from the start of project,
//! even if exported area starts not from the 1 measure.
//!
//! # Examples
//!
//! ```
//! use fraction::Fraction;
//! use rea_score::primitives::position::{
//!     AbsolutePosition, RelativePosition, Distance, RelativeDistance};
//! use rea_score::primitives::Length;
//!
//! let a = AbsolutePosition::from(0.0);
//! let b = AbsolutePosition::from(Fraction::new(4u64, 4u64));
//! let c = AbsolutePosition::from(Fraction::new(0u64, 4u64));
//! assert_eq!(a, c);
//! assert_ne!(a, b);
//! assert_eq!(b.get(), Fraction::from(1.0));
//! let distance = a.get_distance_as_length(&b, None);
//! assert_eq!(distance, Length::from(1.0));
//! let distance = b.get_distance_as_length(&a, None);
//! assert_eq!(distance, Length::from(1.0));
//! ```
//!
//! Relative calculations are possible with providing of `TimeMap`,
//! or with accessing of reaper API (which is hard to make from test code).
//!
//! ```no_run
//! use std::collections::HashMap;
//! use fraction::Fraction;
//! use rea_score::primitives::position::{
//!     AbsolutePosition, RelativePosition, Distance, RelativeDistance};
//! use rea_score::primitives::{Length, time_map::{TimeMap, MeasureInfo}};
//! use rea_rs::TimeSignature;
//!
//! let measures = HashMap::from([
//!     (1, MeasureInfo{
//!             index: 1,
//!             time_signature: TimeSignature::new(7, 8),
//!             length: Length::from(7.0/8.0),
//!         }
//!     ),
//!     (2, MeasureInfo{
//!             index: 2,
//!             time_signature: TimeSignature::new(5, 8),
//!             length: Length::from(5.0/8.0),
//!         }
//!     ),
//! ]);
//! let time_map = TimeMap::new(measures, 0.0.into());
//!
//! let a_relative = RelativePosition::new(1, 0.0.into());
//! let a_absolute = AbsolutePosition::from(0.0);
//! let b_relative = RelativePosition::new(2, Fraction::new(3u64, 8u64));
//! let b_absolute = AbsolutePosition::from(Fraction::new(7 + 3 as u64, 8u64));
//! let distance = a_relative.get_relative_distance(&b_relative, Some(&time_map));
//! assert_eq!(distance, RelativeDistance{
//!         measures: 1,
//!         before_first_barline: Length::from(Fraction::new(7u64, 8u64)),
//!         after_last_barline: Length::from(Fraction::new(3u64, 8u64)),
//!     }
//! );
//! let distance = a_relative.get_relative_distance(&b_absolute, Some(&time_map));
//! assert_eq!(distance, RelativeDistance{
//!         measures: 1,
//!         before_first_barline: Length::from(Fraction::new(7u64, 8u64)),
//!         after_last_barline: Length::from(Fraction::new(3u64, 8u64)),
//!     }
//! );
//! let distance = b_relative.get_relative_distance(&a_relative, Some(&time_map));
//! assert_eq!(distance, RelativeDistance{
//!         measures: -1,
//!         before_first_barline: Length::from(Fraction::new(7u64, 8u64)),
//!         after_last_barline: Length::from(Fraction::new(3u64, 8u64)),
//!     }
//! );
//! let distance = a_relative.get_distance_as_length(&b_absolute, Some(&time_map));
//! assert_eq!(distance, Length::from(Fraction::new(10u64, 8u64)));
//! ```

use std::ops::{Add, AddAssign, Sub};

use fraction::Fraction;
use rea_rs::{Measure, Reaper};

use super::{limit_denominator, time_map::TimeMap, Length, LIMIT_DENOMINATOR};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RelativeDistance {
    /// can be negative
    pub measures: i32,
    /// from the left event to the first barline
    pub before_first_barline: Length,
    /// from the last barline to the right event
    pub after_last_barline: Length,
}

/// Provides unified generic interface for calculating distance
/// between different types of positions.
pub trait Distance<T: GenericPosition>: GenericPosition {
    /// Calculates absolute distance between positions.
    ///
    /// If both positions are `AbsolutePosition` — no TimeMap or reaper
    /// calls needed.
    ///
    /// If TimeMap is provided — calculates using TimeMap. Otherwise —
    /// uses reaper API.
    fn get_distance_as_length(
        &self,
        other: &T,
        time_map: Option<&TimeMap>,
    ) -> Length {
        let mut a = self.get_absolute_position(time_map).get();
        let mut b = other.get_absolute_position(time_map).get();
        if a < b {
            (a, b) = (b, a);
        }
        Length::from(a - b)
    }

    /// Calculates relative distance between positions.
    ///
    /// If TimeMap is provided — calculates using TimeMap. Otherwise —
    /// uses reaper API.
    fn get_relative_distance(
        &self,
        other: &T,
        time_map: Option<&TimeMap>,
    ) -> RelativeDistance {
        let mut a = self.get_relative_position(time_map);
        let mut b = other.get_relative_position(time_map);
        let measures =
            b.get_measure_index() as i32 - a.get_measure_index() as i32;
        if a > b {
            (a, b) = (b, a);
        }

        RelativeDistance {
            measures,
            before_first_barline: a.get_distance_to_bar_end(time_map),
            after_last_barline: Length::from(b.get_position()),
        }
    }
}

/// Unifies interface of concerting positions from relative to absolute.
///
/// If no TimeMap provided — uses reaper API.
pub trait GenericPosition {
    fn get_absolute_position(
        &self,
        time_map: Option<&TimeMap>,
    ) -> AbsolutePosition;
    fn get_relative_position(
        &self,
        time_map: Option<&TimeMap>,
    ) -> RelativePosition;
}

/// Absolute position in whole notes.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct AbsolutePosition {
    position: Fraction,
}
impl AbsolutePosition {
    pub fn new(position: Fraction) -> Self {
        Self { position }
    }

    /// By default, quantizes up to 1/128
    pub fn get(&self) -> Fraction {
        limit_denominator(self.position, LIMIT_DENOMINATOR).unwrap()
    }
}
impl Distance<RelativePosition> for AbsolutePosition {}
impl Distance<AbsolutePosition> for AbsolutePosition {}
impl GenericPosition for AbsolutePosition {
    fn get_absolute_position(
        &self,
        _time_map: Option<&TimeMap>,
    ) -> AbsolutePosition {
        AbsolutePosition {
            position: self.position,
        }
    }
    fn get_relative_position(
        &self,
        time_map: Option<&TimeMap>,
    ) -> RelativePosition {
        if time_map.is_some() {
            let time_map = time_map.unwrap();
            let pos = time_map.pos_relative_from_absolute(self);
            if pos.is_some() {
                return pos.unwrap();
            }
        }
        let pos_f64: f64 = self.clone().into();
        let position = rea_rs::Position::from_quarters(
            pos_f64,
            &Reaper::get().current_project(),
        );
        RelativePosition::from(position)
    }
}
impl Add for AbsolutePosition {
    fn add(self, rhs: Self) -> Self {
        Self {
            position: self.get() + rhs.get(),
        }
    }

    type Output = Self;
}
impl Sub for AbsolutePosition {
    fn sub(self, rhs: Self) -> Self {
        Self {
            position: self.get() - rhs.get(),
        }
    }

    type Output = Self;
}
impl Add<Length> for AbsolutePosition {
    fn add(self, rhs: Length) -> Self::Output {
        Self {
            position: self.get() + rhs.get(),
        }
    }

    type Output = Self;
}
impl AddAssign<Length> for AbsolutePosition {
    fn add_assign(&mut self, rhs: Length) {
        self.position = self.get() + rhs.get()
    }
}
impl From<rea_rs::Position> for AbsolutePosition {
    fn from(position: rea_rs::Position) -> Self {
        Self {
            position: Fraction::from(
                position.as_quarters(&Reaper::get().current_project()) / 4.0,
            ),
        }
    }
}
impl From<Fraction> for AbsolutePosition {
    fn from(value: Fraction) -> Self {
        Self { position: value }
    }
}
impl Into<rea_rs::Position> for AbsolutePosition {
    fn into(self) -> rea_rs::Position {
        let absolute: f64 = self.into();
        rea_rs::Position::from_quarters(
            absolute * 4.0,
            &Reaper::get().current_project(),
        )
    }
}
impl From<f64> for AbsolutePosition {
    fn from(value: f64) -> Self {
        Self {
            position: Fraction::from(value),
        }
    }
}
impl Into<f64> for AbsolutePosition {
    fn into(self) -> f64 {
        let ps = self.get();
        (*ps.numer().unwrap() as f64) / (*ps.denom().unwrap() as f64)
    }
}

/// Represents relative position (like in score).
///
/// Can not be used by itself and totally depends on project timeline or
/// on TimeMap.
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Hash)]
pub struct RelativePosition {
    measure_index: u32,
    /// distance from the start of the measure.
    measure_position: Fraction,
}
impl RelativePosition {
    /// # Parameters:
    /// * measure index: measure number (1-based)
    /// * measure_position: distance from start of measure.
    pub fn new(measure_index: u32, measure_position: Fraction) -> Self {
        Self {
            measure_index,
            measure_position,
        }
    }
    pub fn from_absolute(
        time_map: TimeMap,
        absolute: &AbsolutePosition,
    ) -> Option<Self> {
        time_map.pos_relative_from_absolute(absolute)
    }
    /// position in measure.
    pub fn get_position(&self) -> Fraction {
        limit_denominator(self.measure_position, LIMIT_DENOMINATOR).unwrap()
    }
    pub fn set_position(&mut self, position: Fraction) -> &Self {
        self.measure_position = position;
        self
    }
    /// measure (1-based)
    pub fn get_measure_index(&self) -> u32 {
        self.measure_index
    }
    pub fn set_measure_index(&mut self, index: u32) -> &Self {
        self.measure_index = index;
        self
    }
    /// for example: with position of 3/8 in 4/4 measure returns Length of 5/8
    pub fn get_distance_to_bar_end(
        &self,
        time_map: Option<&TimeMap>,
    ) -> Length {
        let project = Reaper::get().current_project();
        match time_map {
            Some(time_map) => {
                let measure_info =
                    time_map.get_measure_info(self.measure_index);
                Length::from(measure_info.length.get() - self.get_position())
            }
            None => {
                let measure =
                    rea_rs::Measure::from_index(self.measure_index, &project);
                let measure_length = Length::from(
                    measure.end.as_quarters(&project)
                        - measure.start.as_quarters(&project),
                );
                Length::from(measure_length.get() - self.get_position())
            }
        }
    }
}
impl Distance<AbsolutePosition> for RelativePosition {}
impl Distance<RelativePosition> for RelativePosition {}
impl GenericPosition for RelativePosition {
    fn get_absolute_position(
        &self,
        time_map: Option<&TimeMap>,
    ) -> AbsolutePosition {
        let project = Reaper::get().current_project();
        match time_map {
            Some(time_map) => time_map.pos_absolute_from_relative(self),
            None => {
                let measure =
                    rea_rs::Measure::from_index(self.measure_index, &project);
                AbsolutePosition::from(
                    Fraction::from(measure.start.as_quarters(&project) / 4.0)
                        + self.get_position(),
                )
            }
        }
    }
    fn get_relative_position(
        &self,
        _time_map: Option<&TimeMap>,
    ) -> RelativePosition {
        Self::new(self.measure_index, self.measure_position)
    }
}
impl From<rea_rs::Position> for RelativePosition {
    fn from(value: rea_rs::Position) -> Self {
        let project = Reaper::get().current_project();
        let measure = Measure::from_position(value, &project);
        Self {
            measure_index: measure.index,
            measure_position: Fraction::from(
                (value - measure.start).as_quarters(&project) / 4.0,
            ),
        }
    }
}
impl Into<rea_rs::Position> for RelativePosition {
    fn into(self) -> rea_rs::Position {
        let project = Reaper::get().current_project();
        let measure = Measure::from_index(self.measure_index, &project);
        let quarters = measure.start.as_quarters(&project)
            + (*self.measure_position.numer().expect("bad fraction") as f64
                / *self.measure_position.denom().expect("bad fraction")
                    as f64);
        rea_rs::Position::from_quarters(quarters, &project)
    }
}
impl From<AbsolutePosition> for RelativePosition {
    fn from(position: AbsolutePosition) -> Self {
        let rpr_pos: rea_rs::Position = position.into();
        Self::from(rpr_pos)
    }
}
impl Add for RelativePosition {
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.measure_index, rhs.measure_index);
        Self::new(self.measure_index, self.get_position() + rhs.get_position())
    }
    type Output = Self;
}
impl Sub for RelativePosition {
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.measure_index, rhs.measure_index);
        let new = self.get_position() + rhs.get_position();
        assert!(new.is_normal());
        assert!(new.is_sign_positive(), "resulted position is negative");
        Self::new(self.measure_index, new)
    }
    type Output = Self;
}

#[cfg(test)]
mod tests {
    use crate::primitives::position::{Fraction, RelativePosition};

    #[test]
    fn relative_position() {
        let a = RelativePosition::new(2, Fraction::new(1u32, 4u32));
        assert_eq!(
            a,
            RelativePosition {
                measure_index: 2,
                measure_position: Fraction::new(1u64, 4u16)
            }
        );
    }
    #[test]
    #[should_panic]
    fn relative_add_1() {
        let a = RelativePosition::new(2, Fraction::new(1u64, 4u64));
        let b = RelativePosition::new(3, Fraction::new(1u64, 4u64));
        let _ = a + b;
    }
    #[test]
    #[should_panic]
    fn relative_sub_1() {
        let a = RelativePosition::new(2, Fraction::new(2u64, 4u64));
        let b = RelativePosition::new(2, Fraction::new(1u64, 4u64));
        assert_eq!(a.clone() - b.clone(), b.clone());
        let _ = b - a;
    }
}
