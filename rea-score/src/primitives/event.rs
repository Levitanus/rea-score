//! A smallest piece of music, that is held by Measure.
use std::collections::VecDeque;

use crate::{
    lilypond_render::{RenderSettings, RendersToLilypond},
    notation::{
        chord_notations::ChordNotations, note_notations::NoteNotations,
        NotationError, NotationRender, NotationSplitPosition, NotationType,
    },
};

use super::{
    normalize_fraction, Length, Pitch, RelativePosition, ResolvedPitch,
};

/// Can be considered as "Generic" Event.
///
/// EventInfo is more about position and length, while
/// EventType responds for Event-representation and rendering.
#[derive(Debug, PartialEq, Clone)]
pub struct EventInfo {
    pub position: RelativePosition,
    pub length: Length,
    pub event: EventType,
}
impl RendersToLilypond for EventInfo {
    fn render_lilypond(&self) -> String {
        let settings = Self::global_render_settings();
        let length = self.length.render_lilypond();
        self.event.render_lilypond(length, &settings)
    }
}
impl EventInfo {
    pub fn new(
        position: RelativePosition,
        length: Length,
        event: EventType,
    ) -> Self {
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
            && self.position.get_position() + self.length.get()
                > pos.get_position()
    }

    /// Find if outlasts other event (e.g. if end of self > end of other)
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
    /// assert_eq!(ev1.outlasts(&ev2), None);
    /// assert_eq!(ev2.outlasts(&ev1), Some(Length::from(eight)));
    /// assert_eq!(ev3.outlasts(&ev1), None);
    /// assert_eq!(ev1.outlasts(&ev3), None);
    /// ```
    pub fn outlasts(&self, other: &Self) -> Option<Length> {
        if self.position.get_measure_index()
            != other.position.get_measure_index()
        {
            return None;
        }
        let o_end = other.position.get_position() + other.length.get();
        let s_end = self.position.get_position() + self.length.get();
        if s_end <= o_end {
            return None;
        }
        Some(Length::from(s_end - o_end))
    }

    /// Find if overlaps other event (e.g. if one part of self == one part of
    /// other)
    ///
    /// # Note
    ///
    /// Works only in bounds of measure.
    ///
    /// # Example
    /// ```
    /// # use rea_score::primitives::{EventInfo, RelativePosition, Length, EventType};
    /// # use fraction::Fraction;
    /// let quarter = Fraction::new(1u64, 4u64);
    /// let eight = Fraction::new(1u64, 8u64);
    /// let ev1 = EventInfo::new(
    ///     RelativePosition::new(3, quarter+eight),
    ///     Length::from(eight),
    ///     Default::default()
    /// );
    /// let ev2 = EventInfo::new(
    ///     RelativePosition::new(3, quarter),
    ///     Length::from(quarter+eight),
    ///     Default::default()
    /// );
    /// let ev3 = EventInfo::new(
    ///     RelativePosition::new(3, eight),
    ///     Length::from(eight),
    ///     Default::default()
    /// );
    /// assert_eq!(ev1.overlaps(&ev2), true);
    /// assert_eq!(ev2.overlaps(&ev1), true);
    /// assert_eq!(ev3.overlaps(&ev1), false);
    /// assert_eq!(ev1.overlaps(&ev3), false);
    /// ```
    pub fn overlaps(&self, other: &Self) -> bool {
        if self.position.get_measure_index()
            != other.position.get_measure_index()
        {
            return false;
        }
        let o_end = other.position.get_position() + other.length.get();
        let s_end = self.position.get_position() + self.length.get();
        let o_start = other.position.get_position();
        let s_start = self.position.get_position();
        if o_end == s_end || o_start == s_start {
            return true;
        }
        match s_start < o_start {
            true => s_end > o_start,
            false => o_end > s_start,
        }
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
        let (l_evt, r_evt) = self.event.clone().split();
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
    pub fn cut_head_at_position(
        &mut self,
        position: &RelativePosition,
    ) -> Result<Self, String> {
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

    /// Get events, split by normalized length.
    pub fn with_normalized_length(&self) -> VecDeque<Self> {
        let lengths = normalize_fraction(self.length.get(), VecDeque::new());
        let len = lengths.len();
        let mut pos = self.position.clone();
        let mut events = VecDeque::new();
        if len == 1 {
            events.push_back(self.clone());
            return events;
        }
        let mut event = self.event.clone();
        for (idx, length) in lengths.into_iter().rev().enumerate() {
            let ev = match idx {
                x if x == (len - 1) => event.clone(),
                _ => {
                    let t = event.split();
                    let ev = t.0;
                    event = t.1;
                    ev
                }
            };
            let ev = EventInfo::new(pos.clone(), Length::from(length), ev);
            events.push_back(ev);
            pos.set_position(pos.get_position() + length);
        }
        events
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
    pub fn push_notation(
        &mut self,
        notation: NotationType,
    ) -> Result<(), NotationError> {
        self.event.push_notation(notation)
    }
}

/// Various types of events with concrete realizations
/// as variant args.
#[derive(Debug, PartialEq, Clone)]
pub enum EventType {
    /// I think, nothing in the crate likes Rest, so...
    /// Don't pass it ever to anywhere.
    Rest,
    Note(Note),
    Chord(Chord),
}
impl EventType {
    /// TODO! For now just clones.
    fn split(self) -> (Self, Self) {
        let a = match self.clone() {
            Self::Note(mut note) => {
                note.set_tie(true);
                note.remove_tail_notations();
                Self::Note(note)
            }
            Self::Chord(mut ch) => {
                ch.set_ties(true);
                ch.remove_tail_notations();
                Self::Chord(ch)
            }
            Self::Rest => Self::Rest,
        };
        let b = match self {
            Self::Note(mut note) => {
                note.remove_head_notations();
                Self::Note(note)
            }
            Self::Chord(mut ch) => {
                ch.remove_head_notations();
                Self::Chord(ch)
            }
            Self::Rest => Self::Rest,
        };
        (a, b)
    }
    pub fn push_notation(
        &mut self,
        notation: NotationType,
    ) -> Result<(), NotationError> {
        match self {
            Self::Note(note) => note.apply_notation(notation),
            Self::Chord(chord) => match notation {
                NotationType::Chord(n) => chord.apply_notation(n),
                n => Err(NotationError::UnexpectedNotation {
                    notation: format!("{:?}", n),
                    object: format!("{:?}", chord),
                }),
            },
            Self::Rest => todo!(),
        }
    }
    pub fn render_lilypond(
        &self,
        length_string: String,
        settings: &RenderSettings,
    ) -> String {
        match self {
            Self::Rest => format!("r{}", length_string),
            Self::Note(note) => note.render_lilypond(length_string, settings),
            Self::Chord(chord) => {
                chord.render_lilypond(length_string, settings)
            }
        }
    }
}
impl Default for EventType {
    fn default() -> Self {
        Self::Rest
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub pitch: Pitch,
    tie: bool,
    notations: Vec<NoteNotations>,
    chord_notations: Vec<ChordNotations>,
}
impl Note {
    pub fn new(pitch: Pitch) -> Self {
        Self {
            pitch,
            tie: false,
            notations: Vec::new(),
            chord_notations: Vec::new(),
        }
    }
    pub fn set_tie(&mut self, tie: bool) {
        self.tie = tie;
    }
    fn notation_error(&self, notation: NotationType) -> NotationError {
        NotationError::UnexpectedNotation {
            notation: format!("{:?}", notation),
            object: format!("{:?}", self),
        }
    }
    pub fn apply_notation(
        &mut self,
        notation: NotationType,
    ) -> Result<(), NotationError> {
        match notation {
            NotationType::Note(n) => match n {
                NoteNotations::NoteHead(head) => {
                    self.notations.push(NoteNotations::NoteHead(head));
                    Ok(())
                }
                NoteNotations::Voice(_) => Err(self.notation_error(notation)),
            },
            NotationType::Chord(n) => {
                self.chord_notations.push(n);
                Ok(())
            }
            NotationType::Event => Err(self.notation_error(notation)),
        }
    }
    pub fn remove_head_notations(&mut self) {
        self.notations = self
            .notations
            .iter()
            .filter(|nt| !nt.is_head())
            .map(|nt| nt.clone())
            .collect();
        self.chord_notations = self
            .chord_notations
            .iter()
            .filter(|nt| !nt.is_head())
            .map(|nt| nt.clone())
            .collect();
    }
    pub fn remove_tail_notations(&mut self) {
        self.notations = self
            .notations
            .iter()
            .filter(|nt| !nt.is_tail())
            .map(|nt| nt.clone())
            .collect();
        self.chord_notations = self
            .chord_notations
            .iter()
            .filter(|nt| !nt.is_tail())
            .map(|nt| nt.clone())
            .collect();
    }
    pub fn render_lilypond(
        &self,
        length_string: String,
        settings: &RenderSettings,
    ) -> String {
        let key = settings.key;
        let pitch = match self.pitch.resolve(&key) {
            ResolvedPitch::Name(s) => s,
            ResolvedPitch::Note(note) => {
                let n = note.note.to_string();
                let acc = note.accidental.to_string_by_note(note.note);
                let acc = match acc.as_str() {
                    "white" => "",
                    x => x.clone(),
                };
                let oct = match note.octave.raw() as i32 - 4 {
                    0 => "".to_string(),
                    x if x > 0 => "'".repeat(x as usize),
                    x => ",".repeat(x.abs() as usize),
                };
                format!("{}{}{}", n, acc, oct)
            }
        };
        let pitch = format!("{pitch}{length_string}");
        let s = self.notations.iter().fold(pitch, |p, n| n.render(p));
        let s = self.chord_notations.iter().fold(s, |p, n| n.render(p));
        let s = match self.tie {
            true => format!("{}~", s),
            false => s,
        };

        s
    }
}
impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.pitch.partial_cmp(&other.pitch) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.tie.partial_cmp(&other.tie)
    }
}

/// TODO: think on sorting events.
#[derive(Debug, PartialEq, Clone)]
pub struct Chord {
    notes: Vec<Note>,
    chord_notations: Vec<ChordNotations>,
}
impl Chord {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            chord_notations: Vec::new(),
        }
    }
    fn grab_chord_notations(&mut self, notations: &mut Vec<ChordNotations>) {
        notations
            .iter_mut()
            .map(|n| {
                if !self.chord_notations.contains(&n) {
                    self.chord_notations.push(n.clone())
                }
            })
            .count();
        notations.clear();
    }
    pub fn push(mut self, event: EventType) -> Result<Self, String> {
        match event {
            EventType::Rest => {
                Err(format!("Cannot push rest to chord! {:?}", event))
            }
            EventType::Note(mut note) => {
                self.grab_chord_notations(&mut note.chord_notations);
                self.notes.push(note);
                Ok(self)
            }
            EventType::Chord(mut chord) => {
                self.grab_chord_notations(&mut chord.chord_notations);
                self.notes.append(&mut chord.notes);
                Ok(self)
            }
        }
    }
    pub fn set_ties(&mut self, tie: bool) {
        self.notes.iter_mut().map(|n| n.set_tie(tie)).count();
    }
    pub fn remove_head_notations(&mut self) {
        self.chord_notations = self
            .chord_notations
            .iter()
            .filter(|nt| !nt.is_head())
            .map(|nt| nt.clone())
            .collect();
    }
    pub fn remove_tail_notations(&mut self) {
        self.chord_notations = self
            .chord_notations
            .iter()
            .filter(|nt| !nt.is_tail())
            .map(|nt| nt.clone())
            .collect();
    }
    pub fn apply_notation(
        &mut self,
        notation: ChordNotations,
    ) -> Result<(), NotationError> {
        match notation {
            ChordNotations::Dynamics(n) => {
                self.chord_notations.push(ChordNotations::Dynamics(n));
                Ok(())
            }
        }
    }

    pub fn render_lilypond(
        &self,
        length_string: String,
        settings: &RenderSettings,
    ) -> String {
        let note_string = self
            .notes
            .iter()
            .map(|n| n.render_lilypond("".to_string(), settings))
            .collect::<Vec<_>>();
        let s = format!("< {} >{length_string}", note_string.join(" "));
        let s = self.chord_notations.iter().fold(s, |p, n| n.render(p));
        s
    }
}
