use fraction::Fraction;
use rea_score::{
    self,
    reaper::{get_track_items, Measure, Position, TimeSignature},
};
use reaper_high::Reaper;
use reaper_levitanus as rlev;

use reaper_macros::reaper_extension_plugin;
use reaper_medium::{PositionInQuarterNotes, PositionInSeconds};
use rlev::ActionHook;
use std::error::Error;

fn assert_true(expr: bool, message_ok: &str, message_err: &str) {
    match expr {
        true => log::info!("{}", message_ok),
        false => log::error!("{}", message_err),
    }
}

fn test_position() {
    let position = Position::from_beats(PositionInQuarterNotes::new(5.0));
    let reference = Position {
        bar: 2,
        quarters_from_bar_start: PositionInQuarterNotes::new(1.0),
        quarters_from_bar_end: PositionInQuarterNotes::new(3.0),
        quarters_from_project_start: PositionInQuarterNotes::new(5.0),
    };
    assert_true(
        position == reference,
        "Position from beats is OK",
        &format!(
            "Position from beats Errored. Position returned: {:?}, reference: {:?}",
            position, reference
        ),
    );

    let reaper = Reaper::get().medium_reaper();
    let item = reaper
        .get_media_item(reaper_medium::ProjectContext::CurrentProject, 0)
        .unwrap();
    unsafe {
        let take = reaper.get_active_take(item).unwrap();
        let position = Position::from_ppq(take, 100.0);
        assert_true(
            position.bar == 2,
            "Position from PPQ: bar check is OK",
            &format!(
                "Position from PPQ Errored. Bar returned: {:?}, reference: {:?}",
                position.bar, 2
            ),
        );
        assert_true(
            position.quarters_from_bar_start < PositionInQuarterNotes::new(1.0),
            "Position from PPQ: beats from bar start check is OK",
            &format!(
                "Position from PPQ Errored. Beats from start: {:?}, reference: {:?}",
                position.quarters_from_bar_start, "< 1"
            ),
        );
        assert_true(
            position.quarters_from_bar_end > PositionInQuarterNotes::new(3.0),
            "Position from PPQ: beats from bar start check is OK",
            &format!(
                "Position from PPQ Errored. Beats from start: {:?}, reference: {:?}",
                position.quarters_from_bar_end, "> 3"
            ),
        );
    }
}

fn test_measure() {
    let measure_2 = Measure::from_index(2);
    let reference_2 = Measure {
        start_qn: PositionInQuarterNotes::new(4.0),
        end_qn: PositionInQuarterNotes::new(8.0),
        time_signature: TimeSignature::new(4, 4),
        start_time: PositionInSeconds::new(2.0),
    };
    assert_true(
        measure_2 == reference_2,
        "2nd Measure OK",
        &format!(
            "2nd measure Errored. got: {:?}, reference: {:?}",
            measure_2, reference_2
        ),
    );
    let measure_5 = Measure::from_index(5);
    let reference_5 = Measure {
        start_qn: PositionInQuarterNotes::new(16.0),
        end_qn: PositionInQuarterNotes::new(19.5),
        time_signature: TimeSignature::new(7, 8),
        start_time: PositionInSeconds::new(8.0),
    };
    assert_true(
        measure_5 == reference_5,
        "5th Measure OK",
        &format!(
            "5th measure Errored. got: {:?}, reference: {:?}",
            measure_5, reference_5
        ),
    );
    let measure_6 = Measure::from_index(6);
    let reference_6 = Measure {
        start_qn: PositionInQuarterNotes::new(19.5),
        end_qn: PositionInQuarterNotes::new(24.0),
        time_signature: TimeSignature::new(9, 8),
        start_time: PositionInSeconds::new(10.333333333333),
    };
    assert_true(
        measure_6 == reference_6,
        "6th Measure OK",
        &format!(
            "6th measure Errored. got: {:?}, reference: {:?}",
            measure_6, reference_6
        ),
    );
}

fn test_items() {
    let rpr = Reaper::get().medium_reaper();
    let pr = reaper_medium::ProjectContext::CurrentProject;
    let track = rpr.get_track(pr, 1).unwrap();
    let items = get_track_items(track);
    assert_true(
        items.is_some(),
        "test items: returned Vec.",
        "test items: returned None.",
    );
    let items = items.unwrap();
    assert_true(
        items.len() == 2,
        &format!("test_items: Got Vec with 2 items: {:?}", items),
        &format!("test items: got bad Vec: {:?}", items),
    );
}

fn test(_flag: i32) {
    log::info!("running rea-score integration test");

    test_position();
    test_measure();
    test_items();
}

struct TestAction {}

impl ActionHook for TestAction {
    fn actions() -> &'static mut Vec<rlev::Action> {
        static mut ACTIONS: Vec<rlev::Action> = Vec::new();
        unsafe {
            ACTIONS.push(rlev::Action::new(
                "REASCRORE_TESTS",
                "Reascore (Rust): integration test",
                test,
            ));
        }
        unsafe { &mut ACTIONS }
    }
}

impl reaper_medium::HookCommand for TestAction {
    fn call(command_id: reaper_medium::CommandId, flag: i32) -> bool {
        TestAction::call_actions(command_id, flag)
    }
}

#[reaper_extension_plugin(
    name = "reascore_integration_test",
    support_email_address = "pianoist@ya.ru"
)]
fn plugin_main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut session = reaper_high::Reaper::get().medium_session();
    TestAction::register(&mut session)?;
    log::debug!("hi from reascore test");
    Ok(())
}
