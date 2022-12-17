//! A smallest piece of music, that is held by Measure.
use super::{Length, Pitch, RelativePosition};

/// Can be considered as "Generic" Event.
/// 
/// EventInfo is more about position and length, while
/// EventType responds for Event-representation and rendering.
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

/// Various types of events with concrete realizations
/// as variant args.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum EventType {
    /// I think, nothing in the crate likes Rest, so...
    /// Don't pass it ever to anywhere.
    Rest,
    Note(Note),
    Chord(Chord),
}
impl EventType {
    /// TODO! For now just clones.
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

/// TODO: think on sorting events.
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
