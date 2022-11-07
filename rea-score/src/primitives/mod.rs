use std::collections::VecDeque;

use fraction::Fraction;

use crate::reaper;
pub use reaper::TimeSignature;

pub mod fraction_tools;
pub mod length;
pub mod pitch;
pub mod position;
pub mod time_map;

pub use fraction_tools::{limit_denominator, normalize_fraction};
pub use length::Length;
pub use pitch::{midi_to_note, Accidental, Key, NoteName, Octave, Pitch, ResolvedPitch, Scale};
pub use position::{AbsolutePosition, RelativeDistance, RelativePosition};
pub use time_map::{MeasureInfo, TimeMap};

static LIMIT_DENOMINATOR: u64 = 128;

#[derive(Debug, PartialEq)]
pub struct Measure {
    index: u32,
    time_signature: TimeSignature,
    events: VecDeque<EventInfo>,
    length: Length,
}

impl Measure {
    pub fn new(index: u32, time_signature: TimeSignature) -> Self {
        let length = Length::from(&time_signature);
        let mut events = VecDeque::new();
        let pos = RelativePosition::new(index, Fraction::from(0.0));
        events.push_back(EventInfo {
            position: pos,
            length: length.clone(),
            event: EventType::Rest,
        });
        Self {
            index,
            time_signature,
            events,
            length,
        }
    }

    pub fn get_events(&self) -> &VecDeque<EventInfo> {
        &self.events
    }

    /// insert event to the measure, resolving how to place it
    /// with other events.
    ///
    /// # Returns
    /// - None, if everything OK
    /// - Some(EventInfo), if something should be inserted to the next measure.
    /// - Err(String), if something goes wrong.
    ///
    /// # Example
    /// ```
    /// use rea_score::primitives::{Chord, EventInfo, Length, Measure, RelativePosition, TimeSignature,EventType, Note, Pitch};
    /// use fraction::Fraction;
    /// let mut m1 = Measure::new(2, TimeSignature::new(4, 4));
    /// let mut pos = RelativePosition::new(2, Fraction::from(0.0));
    /// let _1_8 = Fraction::new(1u64, 8u64);
    /// let _1_4 = Fraction::new(1u64, 4u64);
    /// let c3 = Note {
    ///     pitch: Pitch::from_midi(60, None, None),
    /// };
    /// let d3 = Note {
    ///     pitch: Pitch::from_midi(62, None, None),
    /// };
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
    ///         EventType::Note(c3.clone()),
    ///     ),
    ///     EventInfo::new(
    ///         pos.set_position(_1_4 + _1_8).clone(),
    ///         Length::from(_1_8),
    ///         EventType::Chord(
    ///             Chord::new()
    ///                 .push(EventType::Note(d3.clone()))
    ///                 .unwrap()
    ///                 .push(EventType::Note(c3.clone()))
    ///                 .unwrap(),
    ///         ),
    ///     ),
    ///     EventInfo::new(
    ///         pos.set_position(_1_4 + _1_4).clone(),
    ///         Length::from(_1_4 + _1_8),
    ///         EventType::Note(c3.clone()),
    ///     ),
    /// ];
    /// for (res, exp) in m1.get_events().iter().zip(expected_events.iter()) {
    ///     assert_eq!(res, exp);
    /// }
    ///
    /// let head = m1.insert(EventInfo::new(
    ///     pos.set_position(_1_4 * 3).clone(),
    ///     Length::from(_1_4 * 2),
    ///     EventType::Note(d3.clone()),
    /// ));
    /// assert_eq!(
    ///     head.unwrap(),
    ///     Some(EventInfo::new(
    ///         RelativePosition::new(3, Fraction::from(0.0)),
    ///         Length::from(_1_4),
    ///         EventType::Note(d3.clone()),
    ///     ))
    /// );
    /// ```
    pub fn insert(&mut self, event: EventInfo) -> Result<Option<EventInfo>, String> {
        // let mut event = event;
        let mut idx = self
            .events
            .iter()
            .position(|evt| evt.contains_pos(&event.position))
            .ok_or("Can not find place for event")?;
        let (event, append_to_self) = self.resolve_event_overlaps(event, idx)?;

        // be sure, length and position of event and current are equal
        let mut current = &mut self.events[idx];
        if current.position != event.position {
            let head = current.cut_head_at_position(&event.position)?;
            idx += 1;
            self.events.insert(idx, head);
            current = &mut self.events[idx];
        }

        // Now, when current event and event are equal at position and length,
        // and we are sure, everything else is correctly splitted,
        // we can replace old event by the new one, which is constructed below.
        let new_event = match &current.event {
            EventType::Rest => event.event,
            EventType::Chord(chord) => EventType::Chord(chord.clone().push(event.event)?),
            EventType::Note(note) => EventType::Chord(
                Chord::new()
                    .push(EventType::Note(note.clone()))?
                    .push(event.event)?,
            ),
        };
        current.set_event(new_event);

        // make sure, nothing is lost:
        match append_to_self {
            None => Ok(None),
            Some(head) => {
                // if head is longer then measure, insert our part recursively,
                // and return head to the caller, to insert to the next measure.
                if head.get_end_position().get_position() > self.length.get() {
                    let mut current = head;
                    // head =
                    //     current.cut_head(Length::from(self.length.get() - current.length.get()))?;
                    let mut head = current.cut_head_at_position(&RelativePosition::new(
                        self.index,
                        self.length.get(),
                    ))?;
                    head.position.set_position(Fraction::from(0.0));
                    head.position.set_measure_index(self.index + 1);
                    // If still something left — things go bad,
                    // and it's time to return Err.
                    match self.insert(current)? {
                        None => Ok(Some(head)),
                        Some(unexpected) => {
                            Err(format!("unexpected head of event found: {:?}", unexpected))
                        }
                    }
                } else {
                    // if head is part of our measure — recursively insert it.
                    match self.insert(head)? {
                        None => Ok(None),
                        Some(unexpected) => {
                            Err(format!("unexpected head of event found: {:?}", unexpected))
                        }
                    }
                }
            }
        }
    }

    /// cuts head from inserted event, or from the current measure event,
    /// depends on their overlaps.
    ///
    /// # Returns
    /// (event, head), where:
    /// - event: EventInfo is given event, but, possibly, truncated.
    /// - head: Option<EventInfo> is cut part, from given event, or from one,
    /// being present in measure already.
    ///
    /// # Side-effect
    /// can make event at idx shorter.
    fn resolve_event_overlaps(
        &mut self,
        mut event: EventInfo,
        idx: usize,
    ) -> Result<(EventInfo, Option<EventInfo>), String> {
        let mut append_to_self: Option<EventInfo> = None;
        let current = &mut self.events[idx];
        match event.overlaps(&current) {
            Some(len) => {
                let head = event.cut_head(len)?;
                append_to_self = Some(head);
            }
            None => {
                if let Some(len) = current.overlaps(&event) {
                    let head = current.cut_head(len)?;
                    // idx += 1;
                    // self.events.push_back(head);
                    self.events.insert(idx + 1, head);
                }
            }
        }
        Ok((event, append_to_self))
    }

    pub fn get(&self, pos: &RelativePosition) -> Option<&EventInfo> {
        for ev in self.events.iter() {
            if ev.contains_pos(pos) {
                return Some(&ev);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct EventInfo {
    pub position: RelativePosition,
    pub length: Length,
    pub event: EventType,
    // event: Box<dyn Event>
}
impl EventInfo {
    pub fn new(position: RelativePosition, length: Length, event: EventType) -> Self {
        return Self {
            position,
            length,
            event,
        };
    }

    /// True if given position in bounds of event.
    ///
    /// # Example
    /// ```
    /// # use rea_score::primitives::{EventInfo, RelativePosition, Length, EventType};
    /// # use fraction::Fraction;
    /// let quarter = Fraction::new(1u64, 4u64);
    /// let eight = Fraction::new(1u64, 8u64);
    /// let ev1 = EventInfo::new(
    ///     RelativePosition::new(3, quarter.clone()),
    ///     Length::from(quarter.clone()),
    ///     Default::default()
    /// );
    /// let _3_8 = RelativePosition::new(3, quarter+eight);
    /// let _1_8 = RelativePosition::new(3, eight);
    /// assert!(ev1.contains_pos(&_3_8));
    /// assert!(!ev1.contains_pos(&_1_8));
    /// ```
    pub fn contains_pos(&self, pos: &RelativePosition) -> bool {
        if pos.get_measure_index() != self.position.get_measure_index() {
            return false;
        }
        self.position.get_position() <= pos.get_position()
            && self.position.get_position() + self.length.get() > pos.get_position()
    }

    /// Find if overlaps other event (e.g. if end of self > end of other)
    ///
    /// # Returns
    /// - None if ends are equal or other is longer, or if events are from
    /// different measures
    /// - Length — that part, which overlaps other event.
    ///
    /// # Example
    /// ```
    /// # use rea_score::primitives::{EventInfo, RelativePosition, Length, EventType};
    /// # use fraction::Fraction;
    /// let quarter = Fraction::new(1u64, 4u64);
    /// let eight = Fraction::new(1u64, 8u64);
    /// let ev1 = EventInfo::new(
    ///     RelativePosition::new(3, quarter),
    ///     Length::from(quarter),
    ///     Default::default()
    /// );
    /// let ev2 = EventInfo::new(
    ///     RelativePosition::new(3, quarter),
    ///     Length::from(quarter+eight),
    ///     Default::default()
    /// );
    /// let ev3 = EventInfo::new(
    ///     RelativePosition::new(3, quarter+eight),
    ///     Length::from(eight),
    ///     Default::default()
    /// );
    /// assert_eq!(ev1.overlaps(&ev2), None);
    /// assert_eq!(ev2.overlaps(&ev1), Some(Length::from(eight)));
    /// assert_eq!(ev3.overlaps(&ev1), None);
    /// assert_eq!(ev1.overlaps(&ev3), None);
    /// ```
    pub fn overlaps(&self, other: &Self) -> Option<Length> {
        if self.position.get_measure_index() != other.position.get_measure_index() {
            return None;
        }
        let o_end = other.position.get_position() + other.length.get();
        let s_end = self.position.get_position() + self.length.get();
        if s_end <= o_end {
            return None;
        }
        Some(Length::from(s_end - o_end))
    }

    /// Split event, truncate length and return new "head" event,
    /// of given lengths.
    ///
    /// # Example
    /// ```
    /// # use rea_score::primitives::{EventInfo, Length, RelativePosition};
    /// # use fraction::Fraction;
    /// let _1_4 = Fraction::new(1u64, 4u64);
    /// let _1_8 = Fraction::new(1u64, 8u64);
    /// let mut ev1 = EventInfo::new(
    ///     RelativePosition::new(3, _1_4),
    ///     Length::from(_1_4),
    ///     Default::default(),
    /// );
    /// let mut ev2 = EventInfo::new(
    ///     RelativePosition::new(3, _1_4 + _1_8),
    ///     Length::from(_1_8),
    ///     Default::default(),
    /// );
    /// assert_eq!(ev1.cut_head(Length::from(_1_8)).unwrap(), ev2);
    /// ev2.position.set_position(_1_4);
    /// assert_eq!(ev1, ev2);
    /// ```
    pub fn cut_head(&mut self, head_length: Length) -> Result<Self, String> {
        let (l_evt, r_evt) = self.event.split();
        if self.length < head_length {
            return Err(format!(
                "Trying to cut head bigger, than body: head: {:?}, body: {:?}",
                head_length, self.length
            ));
        }
        let (l_len, r_len) = (
            Length::from(self.length.get() - head_length.get()),
            head_length,
        );
        let mut r_pos = self.position.clone();
        r_pos.set_position(self.position.get_position() + l_len.get());
        self.set_event(l_evt).set_length(l_len);
        let mut head = self.clone();
        head.set_event(r_evt).set_length(r_len).set_position(r_pos);
        Ok(head)
    }

    /// Split event, truncate length and return new "head" event,
    /// of given lengths.
    ///
    /// # Example
    /// ```
    /// # use rea_score::primitives::{EventInfo, Length, RelativePosition};
    /// # use fraction::Fraction;
    /// let _1_4 = Fraction::new(1u64, 4u64);
    /// let _1_8 = Fraction::new(1u64, 8u64);
    /// let mut ev1 = EventInfo::new(
    ///     RelativePosition::new(3, _1_4),
    ///     Length::from(_1_4),
    ///     Default::default(),
    /// );
    /// let mut ev2 = EventInfo::new(
    ///     RelativePosition::new(3, _1_4 + _1_8),
    ///     Length::from(_1_8),
    ///     Default::default(),
    /// );
    /// assert_eq!(ev1.cut_head_at_position(&ev2.position).unwrap(), ev2);
    /// ev2.position.set_position(_1_4);
    /// assert_eq!(ev1, ev2);
    /// ```
    pub fn cut_head_at_position(&mut self, position: &RelativePosition) -> Result<Self, String> {
        if position < &self.position {
            return Err(format!(
                "can not cut at negative position. self: {:?}, given: {:?}",
                self.position, position
            ));
        }
        let s_end = self.position.get_position() + self.length.get();
        let head_length = s_end - position.get_position();
        self.cut_head(Length::from(head_length))
    }

    pub fn set_length(&mut self, length: Length) -> &mut Self {
        self.length = length;
        self
    }
    pub fn set_position(&mut self, position: RelativePosition) -> &mut Self {
        self.position = position;
        self
    }
    pub fn set_event(&mut self, event: EventType) -> &mut Self {
        self.event = event;
        self
    }
    pub fn get_end_position(&self) -> RelativePosition {
        let mut pos = self.position.clone();
        pos.set_position(pos.get_position() + self.length.get());
        pos
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum EventType {
    Rest,
    Note(Note),
    Chord(Chord),
}
impl EventType {
    fn split(&self) -> (Self, Self) {
        (self.clone(), self.clone())
    }
}
impl Default for EventType {
    fn default() -> Self {
        Self::Rest
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Note {
    pub pitch: Pitch,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Chord {
    notes: Vec<Note>,
}
impl Chord {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }
    pub fn push(mut self, event: EventType) -> Result<Self, String> {
        match event {
            EventType::Rest => Err(format!("Cannot push rest to chord! {:?}", event)),
            EventType::Note(note) => {
                self.notes.push(note);
                Ok(self)
            }
            EventType::Chord(mut chord) => {
                self.notes.append(&mut chord.notes);
                Ok(self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Chord, EventInfo, Length, Measure, RelativePosition, TimeSignature};
    use crate::primitives::{EventType, Note, Pitch};
    use fraction::Fraction;

    #[test]
    fn test_measure() {
        let mut m1 = Measure::new(2, TimeSignature::new(4, 4));
        let mut pos = RelativePosition::new(2, Fraction::from(0.0));
        let _1_8 = Fraction::new(1u64, 8u64);
        let _1_4 = Fraction::new(1u64, 4u64);
        let c3 = Note {
            pitch: Pitch::from_midi(60, None, None),
        };
        let d3 = Note {
            pitch: Pitch::from_midi(62, None, None),
        };
        let ev1 = EventInfo::new(
            RelativePosition::new(2, _1_4 + _1_8),
            Length::from(_1_8),
            EventType::Note(d3.clone()),
        );
        let ev2 = EventInfo::new(
            RelativePosition::new(2, _1_4),
            Length::from(_1_4 + _1_4 + _1_8),
            EventType::Note(c3.clone()),
        );
        m1.insert(ev1).unwrap();
        let head = m1.insert(ev2).unwrap();
        assert_eq!(head, None);

        let expected_events = vec![
            EventInfo::new(pos.clone(), Length::from(_1_4), EventType::Rest),
            EventInfo::new(
                pos.set_position(_1_4).clone(),
                Length::from(_1_8),
                EventType::Note(c3.clone()),
            ),
            EventInfo::new(
                pos.set_position(_1_4 + _1_8).clone(),
                Length::from(_1_8),
                EventType::Chord(
                    Chord::new()
                        .push(EventType::Note(d3.clone()))
                        .unwrap()
                        .push(EventType::Note(c3.clone()))
                        .unwrap(),
                ),
            ),
            EventInfo::new(
                pos.set_position(_1_4 + _1_4).clone(),
                Length::from(_1_4 + _1_8),
                EventType::Note(c3.clone()),
            ),
        ];
        for (res, exp) in m1.events.iter().zip(expected_events.iter()) {
            assert_eq!(res, exp);
        }

        let head = m1.insert(EventInfo::new(
            pos.set_position(_1_4 * 3).clone(),
            Length::from(_1_4 * 2),
            EventType::Note(d3.clone()),
        ));
        assert_eq!(
            head.unwrap(),
            Some(EventInfo::new(
                RelativePosition::new(3, Fraction::from(0.0)),
                Length::from(_1_4),
                EventType::Note(d3.clone()),
            ))
        );
    }
}
