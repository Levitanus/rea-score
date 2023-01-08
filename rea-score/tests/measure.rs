use itertools::Itertools;
use rea_rs::TimeSignature;
use rea_score::primitives::{
    EventInfo, Measure, Note, Pitch, RelativePosition,
};

#[test]
fn test_events_normalized_quarter_in_middle() {
    let c3 = Note::new(Pitch::from_midi(60, None, None));
    let mut measure = Measure::new(1, TimeSignature::new(4, 4));
    measure
        .insert(EventInfo::new(
            RelativePosition::new(1, 0.5.into()),
            0.25.into(),
            rea_score::primitives::EventType::Note(c3.clone()),
        ))
        .expect("Can not insert event");
    let events = vec![
        EventInfo::new(
            RelativePosition::new(1, 0.0.into()),
            0.5.into(),
            rea_score::primitives::EventType::Rest,
        ),
        EventInfo::new(
            RelativePosition::new(1, 0.5.into()),
            0.25.into(),
            rea_score::primitives::EventType::Note(c3.clone()),
        ),
        EventInfo::new(
            RelativePosition::new(1, 0.75.into()),
            0.25.into(),
            rea_score::primitives::EventType::Rest,
        ),
    ];
    measure
        .get_events_normalized()
        .expect("Can not get normalized events")
        .into_iter()
        .zip_eq(events)
        .map(|(a, b)| assert_eq!(a, b))
        .count();
}
#[test]
fn test_events_normalized_half() {
    let c3 = Note::new(Pitch::from_midi(60, None, None));
    let mut measure = Measure::new(1, TimeSignature::new(4, 4));
    measure
        .insert(EventInfo::new(
            RelativePosition::new(1, 0.0.into()),
            0.5.into(),
            rea_score::primitives::EventType::Note(c3.clone()),
        ))
        .expect("Can not insert event");
    let events = vec![
        EventInfo::new(
            RelativePosition::new(1, 0.0.into()),
            0.5.into(),
            rea_score::primitives::EventType::Note(c3.clone()),
        ),
        EventInfo::new(
            RelativePosition::new(1, 0.5.into()),
            0.5.into(),
            rea_score::primitives::EventType::Rest,
        ),
    ];
    measure
        .get_events_normalized()
        .expect("Can not get normalized events")
        .into_iter()
        .zip_eq(events)
        .map(|(a, b)| assert_eq!(a, b))
        .count();
}

#[test]
fn test_events_normalized_whole() {
    let c3 = Note::new(Pitch::from_midi(60, None, None));
    let mut measure = Measure::new(1, TimeSignature::new(4, 4));
    measure
        .insert(EventInfo::new(
            RelativePosition::new(1, 0.0.into()),
            1.0.into(),
            rea_score::primitives::EventType::Note(c3.clone()),
        ))
        .expect("Can not insert event");
    let events = vec![EventInfo::new(
        RelativePosition::new(1, 0.0.into()),
        1.0.into(),
        rea_score::primitives::EventType::Note(c3.clone()),
    )];
    measure
        .get_events_normalized()
        .expect("Can not get normalized events")
        .into_iter()
        .zip_eq(events)
        .map(|(a, b)| assert_eq!(a, b))
        .count();
}

// #[test]
// fn test_events_normalized_3_8_as_dotted() {
//     let c3 = Note::new(Pitch::from_midi(60, None, None));
//     let mut measure = Measure::new(1, TimeSignature::new(4, 4));
//     measure
//         .insert(EventInfo::new(
//             RelativePosition::new(1, 0.125.into()),
//             0.375.into(),
//             rea_score::primitives::EventType::Note(c3.clone()),
//         ))
//         .expect("Can not insert event");
//     let events = vec![
//         EventInfo::new(
//             RelativePosition::new(1, 0.0.into()),
//             0.125.into(),
//             rea_score::primitives::EventType::Rest,
//         ),
//         EventInfo::new(
//             RelativePosition::new(1, 0.125.into()),
//             0.375.into(),
//             rea_score::primitives::EventType::Note(c3.clone()),
//         ),
//         EventInfo::new(
//             RelativePosition::new(1, 0.5.into()),
//             0.5.into(),
//             rea_score::primitives::EventType::Rest,
//         ),
//     ];
//     measure
//         .get_events_normalized()
//         .expect("Can not get normalized events")
//         .iter()
//         .map(|ev| println!("{:?}", ev))
//         .count();
//     measure
//         .get_events_normalized()
//         .expect("Can not get normalized events")
//         .into_iter()
//         .zip_eq(events)
//         .enumerate()
//         .map(|(idx, (a, b))| assert_eq!(a, b, "failed eq at idx: {idx}"))
//         .count();
// }
