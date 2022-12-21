use rea_rs::{
    errors::ReaperError, MidiEvent, MidiMessage, Position, ProbablyMutable,
    RawMidiMessage, Take,
};

use crate::{
    notation::{notations_from_midi, NotationType},
    primitives::{
        position::Distance, AbsolutePosition, EventInfo, EventType, Length,
        Note, Pitch, RelativePosition,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedEvent {
    pub position: RelativePosition,
    pub length: Length,
    pub channel: u8,
    pub note: u8,
    pub event: EventInfo,
    pub notations: Vec<NotationType>,
}
impl ParsedEvent {
    pub fn new(
        position: RelativePosition,
        length: Length,
        channel: u8,
        note: u8,
        event: EventInfo,
        notations: Vec<NotationType>,
    ) -> Self {
        Self {
            position,
            length,
            channel,
            note,
            event,
            notations,
        }
    }
}

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
    let map = notes.map(move |note| {
        let position = RelativePosition::from(AbsolutePosition::from(
            Position::from_ppq(note.start_in_ppq, take),
        ));
        let end_pos = RelativePosition::from(AbsolutePosition::from(
            Position::from_ppq(note.end_in_ppq, take),
        ));
        let length = position.get_distance_as_length(&end_pos, None);
        let ev = EventInfo::new(
            position.clone(),
            length.clone(),
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
            position,
            length,
            note.channel,
            note.note,
            ev,
            not_n
                .filter_map(|n| notations_from_midi(n.message().clone()))
                .flatten()
                .collect(),
        )
    });
    Ok(Box::new(map))
}
