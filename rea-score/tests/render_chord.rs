use musical_note::Key;

use rea_score::{
    lilypond_render::RenderSettings,
    notation::{
        chord_notations::ChordNotations,
        note_notations::{NoteHead, NoteNotations},
        NotationType,
    },
    primitives::{Chord, EventType, Note, Pitch},
};

#[test]
fn test() {
    let mut cis = Note::new(Pitch::from_midi(61, None, None));
    cis.apply_notation(NotationType::Note(NoteNotations::NoteHead(
        NoteHead::Cross,
    )))
    .expect("can not apply notation");
    cis.apply_notation(NotationType::Chord(ChordNotations::Dynamics(
        "f".to_string(),
    )))
    .expect("can not apply notation");
    cis.set_tie(true);

    let mut des = Note::new(Pitch::from_midi(
        61 + 12,
        Some(musical_note::Accidental::Flat),
        None,
    ));
    des.apply_notation(NotationType::Note(NoteNotations::NoteHead(
        NoteHead::Default,
    )))
    .expect("can not apply notation");
    des.apply_notation(NotationType::Chord(ChordNotations::Dynamics(
        "f".to_string(),
    )))
    .expect("can not apply notation");
    des.set_tie(true);

    let chord = Chord::new();
    let chord = chord.push(EventType::Note(cis)).expect("can not push");
    let chord = chord.push(EventType::Note(des)).expect("can not push");

    assert_eq!(
        &chord.render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::Sharp,
                musical_note::Scale::Major,
            ))
        ),
        "< \\override NoteHead.style = #'cross cis'~ \
        \\override NoteHead.style = #'default des''~ >\\f"
    );
}
