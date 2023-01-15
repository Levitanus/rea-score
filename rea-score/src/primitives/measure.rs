//! Measure contains events of one voice.
//!
//! Under construction Measure is filled with EventType:Rest,
//! so, measure is never empty.
//!  
//! By inserting new events, you modifying events inside Measure,
//! splitting them and changing their type (rest become notes,
//! notes become chords).
//!
//! See `Measure::insert` for examples.
//!
//! # Developer Note
//! Measure operates lengths and positions, but not handle
//! event splitting. So, if some event-specific functional
//! broken — go to the event module.

use std::collections::VecDeque;

use fraction::Fraction;
use rea_rs::TimeSignature;

use super::{
    container::Container, EventInfo, Length, MeasureInfo,
    RelativePosition,
};

#[derive(Debug, PartialEq)]
pub struct Measure {
    index: u32,
    time_signature: TimeSignature,
    container: Container,
}
impl From<&MeasureInfo> for Measure {
    fn from(measure: &MeasureInfo) -> Self {
        Self::new(measure.index, measure.time_signature.clone())
    }
}
impl Measure {
    pub fn new(index: u32, time_signature: TimeSignature) -> Self {
        let length = Length::from(&time_signature);
        // let mut events = VecDeque::new();
        let position =
            RelativePosition::new(index, Fraction::from(0.0));
        // events.push_back(EventInfo::new(
        //     position.clone(),
        //     length.clone(),
        //     EventType::Rest,
        // ));
        Self {
            index,
            time_signature,
            container: Container::empty(position, length),
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn length(&self) -> &Length {
        self.container.length()
    }
    pub fn events(&self) -> &VecDeque<EventInfo> {
        self.container.events()
    }
    pub fn length_mut(&mut self) -> &mut Length {
        self.container.length_mut()
    }
    pub fn events_mut(&mut self) -> &mut VecDeque<EventInfo> {
        self.container.events_mut()
    }
    pub fn time_signature(&self) -> &TimeSignature {
        &self.time_signature
    }

    /// Get events, split and tied based on the time signature.
    pub fn get_events_normalized(
        &self,
    ) -> Result<Vec<EventInfo>, String> {
        let mut ts_events = Vec::new();
        (0..self.time_signature.numerator)
            .map(|idx| {
                let g = Fraction::new(
                    idx as u64,
                    self.time_signature.denominator as u64,
                );
                let position = RelativePosition::new(self.index, g);
                let length = Length::from(Fraction::new(
                    1_u64,
                    self.time_signature.denominator as u64,
                ));
                ts_events.push(EventInfo::new(
                    position,
                    length,
                    Default::default(),
                ))
            })
            .count();
        let mut events = Vec::new();
        for event in self.events() {
            let mut event = event.clone();
            for ts_event in ts_events.iter() {
                if !ts_event.overlaps(&event) {
                    continue;
                }
                match event.position == ts_event.position {
                    false => match event.outlasts(&ts_event) {
                        None => {
                            events.extend(
                                event.with_normalized_length(),
                            );
                            break;
                        }
                        Some(length) => match event.position
                            > ts_event.position
                        {
                            false => continue,
                            true => {
                                let head = event.cut_head(length)?;
                                events.extend(
                                    event.with_normalized_length(),
                                );
                                event = head;
                            }
                        },
                    },
                    true => {
                        let ev_end = event.position.position()
                            + event.length.get();
                        let ts_ev_end = ts_event.position.position()
                            + ts_event.length.get();
                        match ev_end == ts_ev_end {
                            true => {
                                events.extend(
                                    event.with_normalized_length(),
                                );
                                break;
                            }
                            false => {
                                match event.outlasts(ts_event) {
                                    None => {
                                        events.extend(
                                        event.with_normalized_length(),
                                    );
                                        break;
                                    }
                                    Some(_length) => {
                                        // println!(
                                        //     "Event:\n----{:?}\
                                        // noutlasts
                                        // ts:\n----{:?}",
                                        //     event, ts_event
                                        // );
                                        // let head =
                                        // event.cut_head(length)?;
                                        // events
                                        //     .extend(head.
                                        // with_normalized_length())
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(events)
    }

    /// insert event to the measure, resolving how to place it
    /// with other events.
    ///
    /// # Returns
    /// - None, if everything OK
    /// - Some(EventInfo), if something should be inserted to the
    ///   next measure.
    /// - Err(String), if something goes wrong.
    ///
    /// # Example
    /// ```
    /// use rea_score::primitives::{Chord, EventInfo, Length, Measure, RelativePosition, EventType, Note, Pitch};
    /// use rea_rs::TimeSignature;
    /// use fraction::Fraction;
    /// let mut m1 = Measure::new(2, TimeSignature::new(4, 4));
    /// let mut pos = RelativePosition::new(2, Fraction::from(0.0));
    /// let _1_8 = Fraction::new(1u64, 8u64);
    /// let _1_4 = Fraction::new(1u64, 4u64);
    /// let c3 = Note::new(
    ///     Pitch::from_midi(60, None, None),
    /// );
    /// let mut c3_tied = c3.clone();
    /// c3_tied.set_tie(true);
    ///
    /// let d3 = Note::new(
    ///     Pitch::from_midi(62, None, None),
    /// );
    /// let mut d3_tied = d3.clone();
    /// d3_tied.set_tie(true);
    ///
    /// let ev1 = EventInfo::new(
    ///     RelativePosition::new(2, _1_4 + _1_8),
    ///     Length::from(_1_8),
    ///     EventType::Note(d3.clone()),
    /// );
    /// let ev2 = EventInfo::new(
    ///     RelativePosition::new(2, _1_4),
    ///     Length::from(_1_4 + _1_4 + _1_8),
    ///     EventType::Note(c3.clone()),
    /// );
    /// m1.insert(ev1).unwrap();
    /// let head = m1.insert(ev2).unwrap();
    /// assert_eq!(head, None);
    ///
    /// let expected_events = vec![
    ///     EventInfo::new(pos.clone(), Length::from(_1_4), EventType::Rest),
    ///     EventInfo::new(
    ///         pos.set_position(_1_4).clone(),
    ///         Length::from(_1_8),
    ///         EventType::Note(c3_tied.clone()),
    ///     ),
    ///     EventInfo::new(
    ///         pos.set_position(_1_4 + _1_8).clone(),
    ///         Length::from(_1_8),
    ///         EventType::Chord(
    ///             Chord::new()
    ///                 .push(EventType::Note(d3.clone()))
    ///                 .unwrap()
    ///                 .push(EventType::Note(c3_tied.clone()))
    ///                 .unwrap(),
    ///         ),
    ///     ),
    ///     EventInfo::new(
    ///         pos.set_position(_1_4 + _1_4).clone(),
    ///         Length::from(_1_4 + _1_8),
    ///         EventType::Note(c3.clone()),
    ///     ),
    /// ];
    /// for (res, exp) in m1.events().iter().zip(expected_events.iter()) {
    ///     assert_eq!(res, exp);
    /// }
    ///
    /// let head = m1.insert(EventInfo::new(
    ///     pos.set_position(_1_4 * 3).clone(),
    ///     Length::from(_1_4 * 2),
    ///     EventType::Note(d3.clone()),
    /// ));
    ///
    /// assert_eq!(
    ///     head.unwrap(),
    ///     Some(EventInfo::new(
    ///         RelativePosition::new(3, Fraction::from(0.0)),
    ///         Length::from(_1_4),
    ///         EventType::Note(d3),
    ///     ))
    /// );
    /// ```
    pub fn insert(
        &mut self,
        event: EventInfo,
    ) -> Result<Option<EventInfo>, String> {
        self.container.insert(event)
    }

    pub fn get(&self, pos: &RelativePosition) -> Option<&EventInfo> {
        for ev in self.events().iter() {
            if ev.contains_pos(pos) {
                return Some(&ev);
            }
        }
        None
    }
}
