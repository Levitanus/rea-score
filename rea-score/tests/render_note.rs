use musical_note::Key;

use rea_score::{
    lilypond_render::RenderSettings,
    notation::{
        chord_notations::ChordNotations,
        note_notations::{NoteHead, NoteNotations},
        NotationType,
    },
    primitives::{Note, Pitch},
};

#[test]
fn test() {
    assert_eq!(
        &Note::new(Pitch::from_midi(60, None, None)).render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::White,
                musical_note::Scale::Major,
            ))
        ),
        "c'"
    );
    assert_eq!(
        &Note::new(Pitch::from_midi(60 - 36, None, None)).render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::White,
                musical_note::Scale::Major,
            ))
        ),
        "c,,"
    );
    assert_eq!(
        &Note::new(Pitch::from_midi(61, None, None)).render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::Sharp,
                musical_note::Scale::Major,
            ))
        ),
        "cis'"
    );
    assert_eq!(
        &Note::new(Pitch::from_midi(
            63 + 12,
            Some(musical_note::Accidental::Flat),
            None,
        ))
        .render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::Sharp,
                musical_note::Scale::Major,
            ))
        ),
        "es''"
    );

    let mut note = Note::new(Pitch::from_midi(
        63 + 12,
        Some(musical_note::Accidental::Flat),
        None,
    ));
    note.apply_notation(NotationType::Note(NoteNotations::NoteHead(
        NoteHead::Cross,
    )))
    .expect("can not apply notation");
    note.apply_notation(NotationType::Chord(ChordNotations::Dynamics(
        "f".to_string(),
    )))
    .expect("can not apply notation");
    note.set_tie(true);
    assert_eq!(
        &note.render_lilypond(
            "".to_string(),
            &RenderSettings::new(Key::new(
                musical_note::NoteName::C,
                musical_note::Accidental::Sharp,
                musical_note::Scale::Major,
            ))
        ),
        r"\override NoteHead.style = #'cross es''\f~"
    );
}
