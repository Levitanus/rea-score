use rea_rs::{
    errors::ReaperError, MidiEvent, MidiMessage, NotationMessage,
    NoteOnMessage, Position, ProbablyMutable, RawMidiMessage, Take,
};

use crate::{
    dom::get_edited_midi,
    notation::{message::MidiFuncs, NotationType},
    primitives::{
        position::Distance, AbsolutePosition, EventInfo, EventType,
        Note, Pitch, RelativePosition,
    },
};

use super::set_edited_midi;

pub fn parse_events<'a, T: ProbablyMutable>(
    events: impl Iterator<Item = MidiEvent<RawMidiMessage>> + Clone + 'a,
    take: &'a Take<T>,
) -> Result<Box<dyn Iterator<Item = ParsedEvent> + 'a>, ReaperError>
{
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
        let position =
            RelativePosition::from(AbsolutePosition::from(
                Position::from_ppq(note.start_in_ppq, take),
            ));
        let end_pos =
            RelativePosition::from(AbsolutePosition::from(
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
                .filter_map(|n| {
                    MidiFuncs::parse_notations(n.message().clone())
                })
                .flatten()
                .collect(),
        )
    });
    Ok(Box::new(parsed_events))
}

/// Apply notations to the given note_on events.
///
/// Assuming, that caller at first filtered note events, that should
/// be processed. Than calling this function, which builds `Vec` of
/// new events. Then, pushing new events to take\whatever.
pub fn notations_to_note_events(
    notations: Vec<NotationType>,
    note_events: Vec<MidiEvent<NoteOnMessage>>,
    all_events: impl IntoIterator<Item = MidiEvent<RawMidiMessage>>,
) -> Vec<MidiEvent<RawMidiMessage>> {
    let mut n_evts = Vec::new();

    // move existing notations from eventlist to `n_evts`
    let mut all_events: Vec<_> = all_events
        .into_iter()
        .filter_map(|ev| {
            for note in note_events.iter() {
                if ev.ppq_position() == note.ppq_position() {
                    if let Some(msg) =
                        NotationMessage::from_raw(ev.message().get_raw())
                    {
                        match msg.notation() {
                            rea_rs::Notation::Note {
                                note: nt,
                                channel: ch,
                                tokens: _tk,
                            } => {
                                if nt != note.message().note()
                                    || ch != note.message().channel()
                                {
                                    continue;
                                }
                                // The only case, when event is filtered
                                // NotationMessages are already processed by
                                // new notations.
                                n_evts.push(MidiEvent::with_new_message(
                                    ev,
                                    MidiFuncs::replace_notations(
                                        msg.clone(),
                                        notations.clone(),
                                    )
                                    .expect(
                                        "Can not aplly notations to message",
                                    ),
                                ));
                                return None;
                            }
                            _ => continue,
                        }
                    }
                }
            }
            Some(ev)
        })
        .collect();

    // building notations events for every desired note.
    // If notations event exists for note — it is just moved from old
    // vec.
    let n_evts = note_events
        .into_iter()
        .map(|ev| {
            for (idx, n_ev) in n_evts.iter().enumerate() {
                if n_ev.ppq_position() != ev.ppq_position() {
                    continue;
                }
                match n_ev.message().notation() {
                    rea_rs::Notation::Note {
                        channel,
                        note,
                        tokens: _,
                    } => {
                        if note != ev.message().note()
                            || channel != ev.message().channel()
                        {
                            continue;
                        }
                        let n_ev = n_ev.clone();
                        n_evts.swap_remove(idx);
                        return n_ev;
                    }
                    _ => continue,
                }
            }
            let msg =
                NotationMessage::from(rea_rs::Notation::Note {
                    channel: ev.message().channel(),
                    note: ev.message().note(),
                    tokens: Vec::new(),
                });
            MidiEvent::with_new_message(
                ev,
                MidiFuncs::replace_notations(msg, notations.clone())
                    .expect("Can not build notation message"),
            )
        })
        // convert notations events to raw events
        .map(|ev| {
            let msg = ev.message().as_raw_message();
            MidiEvent::with_new_message(ev, msg)
        });
    // add new notation events to all events
    all_events.extend(n_evts);
    // and sort everything by position, and, notes go first, then
    // notations.
    all_events.sort_by(|a, b| {
        match a.ppq_position().cmp(&b.ppq_position()) {
            std::cmp::Ordering::Equal => {
                match NoteOnMessage::from_raw(a.message().get_raw())
                {
                    Some(_) => std::cmp::Ordering::Less,
                    None => std::cmp::Ordering::Greater,
                }
            }
            x => x,
        }
    });

    all_events
}

pub fn notations_to_first_selected(
    notations: Vec<NotationType>,
) -> Result<(), ReaperError> {
    let events = get_edited_midi()?;
    let note_events = match events
        .clone()
        .filter_note_on()
        .filter(|ev| ev.selected())
        .next()
    {
        None => {
            return Err(ReaperError::UnsuccessfulOperation(
                "No selected notes.",
            ))
        }
        Some(ev) => vec![ev],
    };
    let events =
        notations_to_note_events(notations, note_events, events);
    set_edited_midi(events)
}
pub fn notations_to_first_and_last_selected(
    notations_to_first: Vec<NotationType>,
    notations_to_last: Vec<NotationType>,
) -> Result<(), ReaperError> {
    let events = get_edited_midi()?;
    let mut note_events =
        events.clone().filter_note_on().filter(|ev| ev.selected());
    let first_selected = match note_events.next() {
        None => {
            return Err(ReaperError::UnsuccessfulOperation(
                "No selected notes.",
            ))
        }
        Some(ev) => vec![ev],
    };
    let last_selected = match note_events.last() {
        None => {
            return Err(ReaperError::UnsuccessfulOperation(
                "No selected notes.",
            ))
        }
        Some(ev) => vec![ev],
    };
    let events = notations_to_note_events(
        notations_to_first,
        first_selected,
        events,
    );
    let events = notations_to_note_events(
        notations_to_last,
        last_selected,
        events,
    );
    set_edited_midi(events)
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
            .filter_map(|not| {
                match self.event.push_notation(not.clone()) {
                    Ok(_) => None,
                    Err(_) => Some(not),
                }
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
            EventInfo, EventType, Length, Note, Pitch,
            RelativePosition,
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
                EventType::Note(Note::new(Pitch::from_midi(
                    60, None, None,
                ))),
            ),
            vec![
                NotationType::Note(NoteNotations::NoteHead(
                    NoteHead::Cross,
                )),
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
