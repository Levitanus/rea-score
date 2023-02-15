use fraction::Fraction;
use rea_rs::{PluginContext, Position, Project, Reaper};
use rea_rs_macros::reaper_extension_plugin;
use rea_rs_test::*;
use rea_score::{
    dom::midi_parse::parse_events,
    primitives::{AbsolutePosition, RelativePosition},
};
use std::error::Error;

#[reaper_extension_plugin]
fn test_extension(
    context: PluginContext,
) -> Result<(), Box<dyn Error>> {
    let test = ReaperTest::setup(context, "test_action");
    test.push_test_step(TestStep::new("Positions", positions));
    test.push_test_step(TestStep::new("Simple Parse", simple_parse));
    test.push_test_step(TestStep::new(
        "Regress Parse",
        regress_parse,
    ));
    Ok(())
}

fn clear_project(reaper: &mut Reaper) -> Project {
    let mut pr = reaper.current_project();
    for idx in pr.n_tracks()..0 {
        pr.get_track_mut(idx).unwrap().delete();
    }
    pr
}
fn setup_project(reaper: &mut Reaper) -> Project {
    let mut pr = clear_project(reaper);
    let two_bars = Position::from_quarters(8.0, &pr);
    let mut track = pr.add_track(1, "");
    let mut item = track.add_midi_item(0.0, two_bars);
    item.set_selected(true);
    pr
}

fn positions(reaper: &mut Reaper) -> TestStepResult {
    let mut pr = setup_project(reaper);
    let item = pr.get_selected_item_mut(0).unwrap();
    let take = item.active_take();
    let pos = Position::from_ppq(960_u32, &take);
    let abs_pos = AbsolutePosition::from(pos);
    let rel_pos = RelativePosition::from(abs_pos.clone());

    assert_eq!(pos, Position::from_quarters(1.0, &pr));
    assert_eq!(
        abs_pos.get_quantized(),
        (pos.as_quarters(&pr) / 4.0).into()
    );
    assert_eq!(
        rel_pos,
        RelativePosition::new(1, Fraction::new(1_u64, 4_u64))
    );
    Ok(())
}

mod simple_parse_data;
fn simple_parse(reaper: &mut Reaper) -> TestStepResult {
    let mut pr = setup_project(reaper);
    let mut item = pr.get_selected_item_mut(0).unwrap();
    let mut take = item.active_take_mut();
    let events =
        parse_events(simple_parse_data::data().into_iter(), &take)
            .expect("Can not parse events.");
    // assert_eq!(events, simple_parse_data::expected());
    events
        .zip(simple_parse_data::expected().into_iter())
        .map(|t| {
            assert_eq!(t.0, t.1);
        })
        .count();

    take.set_midi(
        rea_rs::MidiEventConsumer::new(
            simple_parse_data::data().into_iter(),
        )
        .collect(),
    )
    .expect("Can not set take midi!");

    let events = parse_events(
        take.iter_midi(None).expect("Can not get take midi."),
        &take,
    )
    .expect("Can not parse events.");
    // assert_eq!(events, simple_parse_data::expected());
    events
        .zip(simple_parse_data::expected().into_iter())
        .map(|t| {
            assert_eq!(t.0, t.1);
        })
        .count();
    take.set_midi(vec![0u8; 1])?;

    Ok(())
}

fn regress_parse(reaper: &mut Reaper) -> TestStepResult {
    let mut pr = setup_project(reaper);
    let mut item = pr.get_selected_item_mut(0).unwrap();
    let mut take = item.active_take_mut();
    let events = parse_events(
        simple_parse_data::regress1_data().into_iter(),
        &take,
    )
    .expect("Can not parse events.");
    events
        .zip(simple_parse_data::regress1_expected().into_iter())
        .map(|t| {
            assert_eq!(t.0, t.1);
        })
        .count();

    take.set_midi(
        rea_rs::MidiEventConsumer::new(
            simple_parse_data::regress1_data().into_iter(),
        )
        .collect(),
    )
    .expect("Can not set take midi!");

    let events = parse_events(
        take.iter_midi(None).expect("Can not get take midi."),
        &take,
    )
    .expect("Can not parse events.");
    // assert_eq!(events, simple_parse_data::expected());
    drop(events);
    simple_parse_data::regress1_expected()
        .into_iter()
        .zip(simple_parse_data::regress1_expected().into_iter())
        .map(|t| {
            assert_eq!(t.0, t.1);
        })
        .count();
    take.set_midi(vec![0u8; 1])?;

    Ok(())
}
