use fraction::Fraction;
use rea_rs::{
    MidiEvent, Notation, NotationMessage, NoteOffMessage, NoteOnMessage,
    RawMidiMessage,
};
use rea_score::{
    dom::midi_parse::ParsedEvent,
    notation::{
        chord_notations::ChordNotations,
        note_notations::{NoteHead, NoteNotations},
        NotationType,
    },
    primitives::{
        EventInfo, EventType, Length, Note, Pitch, RelativePosition,
    },
};

pub fn data() -> Vec<MidiEvent<RawMidiMessage>> {
    let cc_sh = rea_rs::CcShapeKind::Square;

    vec![
        rea_rs::MidiEvent::new(
            960,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOnMessage::new(1, 60, 43)),
        ),
        rea_rs::MidiEvent::new(
            960,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NotationMessage::from(Notation::Note {
                channel: 1,
                note: 60,
                tokens: vec![
                    "text".to_string(),
                    r"ReaScore|note-head:cross|dyn:mf".to_string(),
                ],
            })),
        ),
        rea_rs::MidiEvent::new(
            960 * 2,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOnMessage::new(1, 61, 46)),
        ),
        rea_rs::MidiEvent::new(
            960 * 2,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOnMessage::new(1, 63, 46)),
        ),
        rea_rs::MidiEvent::new(
            960 * 2,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NotationMessage::from(Notation::Note {
                channel: 1,
                note: 63,
                tokens: vec![
                    "text".to_string(),
                    r"ReaScore|note-head:cross|dyn:f".to_string(),
                ],
            })),
        ),
        rea_rs::MidiEvent::new(
            960 * 3,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOffMessage::new(1, 60, 43)),
        ),
        rea_rs::MidiEvent::new(
            960 * 3,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOffMessage::new(1, 61, 43)),
        ),
        rea_rs::MidiEvent::new(
            960 * 3,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOffMessage::new(1, 63, 43)),
        ),
    ]
}

pub fn expected() -> Vec<ParsedEvent> {
    let quarter = Fraction::new(1u64, 4u64);
    let notes = [
        Note::new(Pitch::from_midi(60, None, None)),
        Note::new(Pitch::from_midi(61, None, None)),
        Note::new(Pitch::from_midi(63, None, None)),
    ];
    let positions = [
        RelativePosition::new(1, quarter),
        RelativePosition::new(1, quarter * 2),
        RelativePosition::new(1, quarter * 2),
    ];
    let lengths = [
        Length::from(quarter * 2),
        Length::from(quarter * 1),
        Length::from(quarter * 1),
    ];

    vec![
        ParsedEvent::new(
            1,
            notes[0].pitch.midi(),
            EventInfo::new(
                positions[0].clone(),
                lengths[0].clone(),
                EventType::Note(notes[0].clone()),
            ),
            vec![
                NotationType::Note(NoteNotations::NoteHead(NoteHead::Cross)),
                NotationType::Chord(ChordNotations::Dynamics(
                    "mf".to_string(),
                )),
            ],
        ),
        ParsedEvent::new(
            1,
            notes[1].pitch.midi(),
            EventInfo::new(
                positions[1].clone(),
                lengths[1].clone(),
                EventType::Note(notes[1].clone()),
            ),
            Vec::new(),
        ),
        ParsedEvent::new(
            1,
            notes[2].pitch.midi(),
            EventInfo::new(
                positions[2].clone(),
                lengths[2].clone(),
                EventType::Note(notes[2].clone()),
            ),
            vec![
                NotationType::Note(NoteNotations::NoteHead(NoteHead::Cross)),
                NotationType::Chord(ChordNotations::Dynamics("f".to_string())),
            ],
        ),
    ]
}

//

pub fn regress1_data() -> Vec<MidiEvent<RawMidiMessage>> {
    let cc_sh = rea_rs::CcShapeKind::Square;

    vec![
        rea_rs::MidiEvent::new(
            960 * 3,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOnMessage::new(1, 60, 43)),
        ),
        rea_rs::MidiEvent::new(
            960 * 4,
            false,
            false,
            cc_sh,
            RawMidiMessage::from_msg(NoteOffMessage::new(1, 60, 43)),
        ),
    ]
}

pub fn regress1_expected() -> Vec<ParsedEvent> {
    let quarter = Fraction::new(1u64, 4u64);
    let notes = [Note::new(Pitch::from_midi(60, None, None))];
    let positions = [RelativePosition::new(1, Fraction::new(3u64, 4u64))];
    let lengths = [Length::from(quarter)];

    let events = vec![ParsedEvent::new(
        1,
        notes[0].pitch.midi(),
        EventInfo::new(
            positions[0].clone(),
            lengths[0].clone(),
            EventType::Note(notes[0].clone()),
        ),
        Vec::new(),
    )];
    events
}
