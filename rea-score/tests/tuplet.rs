use fraction::Fraction;
use rea_score::{
    lilypond_render::RendersToLilypond,
    primitives::{
        event::Tuplet, EventInfo, Length, Note, Pitch,
        RelativePosition,
    },
};

#[test]
fn triplet() {
    let start = RelativePosition::new(1, Fraction::new(1_u8, 4_u8));
    let quarter = EventInfo::new(
        start.clone(),
        Length::from(Fraction::new(2_u8, 12_u8)),
        rea_score::primitives::EventType::Note(Note::new(
            Pitch::from_midi(60, None, None),
        )),
    );
    let eight = EventInfo::new(
        RelativePosition::new(
            1,
            start.position() + Fraction::new(2_u8, 12_u8),
        ),
        Length::from(Fraction::new(1_u8, 12_u8)),
        rea_score::primitives::EventType::Note(Note::new(
            Pitch::from_midi(60, None, None),
        )),
    );
    let triplet = EventInfo::new(
        start.clone(),
        Length::from(Fraction::new(1_u8, 4_u8)),
        rea_score::primitives::EventType::Tuplet(Tuplet::new(
            Fraction::new(3_u8, 2_u8),
            vec![quarter, eight],
        )),
    );
    assert_eq!(
        triplet.render_lilypond(),
        r"\tuplet 3/2 { c'4 c'8 }"
    );

    let start = RelativePosition::new(1, Fraction::new(1_u8, 4_u8));
    let a = EventInfo::new(
        RelativePosition::new(1, Fraction::new(3_u8, 12_u8)),
        Length::from(Fraction::new(1_u8, 12_u8)),
        rea_score::primitives::EventType::Note(Note::new(
            Pitch::from_midi(60, None, None),
        )),
    );
    let b = EventInfo::new(
        RelativePosition::new(1, Fraction::new(5_u8, 12_u8)),
        Length::from(Fraction::new(1_u8, 12_u8)),
        rea_score::primitives::EventType::Note(Note::new(
            Pitch::from_midi(60, None, None),
        )),
    );
    let c = EventInfo::new(
        RelativePosition::new(1, Fraction::new(5_u8, 12_u8)),
        Length::from(Fraction::new(1_u8, 12_u8)),
        rea_score::primitives::EventType::Note(Note::new(
            Pitch::from_midi(62, None, None),
        )),
    );
    let triplet = EventInfo::new(
        start,
        Length::from(Fraction::new(1_u8, 4_u8)),
        rea_score::primitives::EventType::Tuplet(Tuplet::new(
            Fraction::new(3_u8, 2_u8),
            vec![a, b, c],
        )),
    );
    assert_eq!(
        triplet.render_lilypond(),
        r"\tuplet 3/2 { c'8 r8 < c' d' >8 }"
    );
}
