use rea_rs::{
    errors::ReaperError, MidiEvent, MidiMessage, Position, ProbablyMutable,
    RawMidiMessage, Take,
};

use crate::{
    notation::{notations_from_midi, NotationType},
    primitives::{
        position::Distance, AbsolutePosition, EventInfo, EventType, Note,
        Pitch, RelativePosition,
    },
};

pub fn parse_events<'a, T: ProbablyMutable>(
    events: impl Iterator<Item = MidiEvent<RawMidiMessage>> + Clone + 'a,
    take: &'a Take<T>,
) -> Result<Box<dyn Iterator<Item = ParsedEvent> + 'a>, ReaperError> {
    let notes = rea_rs::FilterNotes::new(events.clone());
    let notations = events
        .filter_map(|ev| {
            let msg = ev.message().get_raw();
            Some(rea_rs::MidiEvent::with_new_message(
                ev,
                rea_rs::NotationMessage::from_raw(msg)?,
            ))
        })
        .collect::<Vec<_>>();
    let parsed_events = notes.map(move |note| {
        let position = RelativePosition::from(AbsolutePosition::from(
            Position::from_ppq(note.start_in_ppq, take),
        ));
        let end_pos = RelativePosition::from(AbsolutePosition::from(
            Position::from_ppq(note.end_in_ppq, take),
        ));
        let length = position.get_distance_as_length(&end_pos, None);
        let ev = EventInfo::new(
            position,
            length,
            EventType::Note(Note::new(Pitch::from_midi(
                note.note, None, None,
            ))),
        );
        let not_n = notations.clone().into_iter().filter(|not| {
            if not.ppq_position() != note.start_in_ppq {
                return false;
            }
            match not.message().notation().clone() {
                rea_rs::Notation::Note {
                    channel: ch,
                    note: nt,
                    tokens: _,
                } => ch == note.channel && nt == note.note,
                rea_rs::Notation::Track(_) => false,
                rea_rs::Notation::Unknown(_) => false,
            }
        });
        ParsedEvent::new(
            note.channel,
            note.note,
            ev,
            not_n
                .filter_map(|n| notations_from_midi(n.message().clone()))
                .flatten()
                .collect(),
        )
    });
    Ok(Box::new(parsed_events))
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedEvent {
    pub channel: u8,
    pub note: u8,
    pub event: EventInfo,
    pub notations: Vec<NotationType>,
}
impl ParsedEvent {
    pub fn new(
        channel: u8,
        note: u8,
        event: EventInfo,
        notations: Vec<NotationType>,
    ) -> Self {
        Self {
            channel,
            note,
            event,
            notations,
        }
    }

    /// Push to event every notations possible, leaving those, that
    /// can not be applied to a single event.
    pub fn apply_single_notations(mut self) -> Self {
        self.notations = self
            .notations
            .into_iter()
            .filter_map(|not| match self.event.push_notation(not.clone()) {
                Ok(_) => None,
                Err(_) => Some(not),
            })
            .collect();
        self
    }
}

#[cfg(test)]
mod tests {
    use fraction::Fraction;

    use crate::{
        notation::{
            chord_notations::ChordNotations,
            note_notations::{NoteHead, NoteNotations},
            NotationType,
        },
        primitives::{
            EventInfo, EventType, Length, Note, Pitch, RelativePosition,
        },
    };

    use super::ParsedEvent;

    #[test]
    fn test_apply_single_notations() {
        let quarter = Fraction::new(1u64, 4u64);
        let ev = ParsedEvent::new(
            1,
            60,
            EventInfo::new(
                RelativePosition::new(1, quarter),
                Length::from(quarter * 2),
                EventType::Note(Note::new(Pitch::from_midi(60, None, None))),
            ),
            vec![
                NotationType::Note(NoteNotations::NoteHead(NoteHead::Cross)),
                NotationType::Note(NoteNotations::Voice(1)),
                NotationType::Chord(ChordNotations::Dynamics(
                    r"\f".to_string(),
                )),
            ],
        );
        assert_eq!(ev.notations.len(), 3);
        let ev = ev.apply_single_notations();
        assert_eq!(ev.notations.len(), 1);
        assert_eq!(
            ev.notations[0],
            NotationType::Note(NoteNotations::Voice(1))
        );
    }
}
