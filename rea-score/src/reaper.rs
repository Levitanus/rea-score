use reaper_high::Reaper;

use reaper_medium::{
    MediaItem, MediaItemTake, MediaTrack, PositionInPPQ, PositionInQuarterNotes, PositionInSeconds,
};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Position {
    pub bar: i32,
    pub quarters_from_bar_start: PositionInQuarterNotes,
    pub quarters_from_bar_end: PositionInQuarterNotes,
    pub quarters_from_project_start: PositionInQuarterNotes,
}

impl Position {
    pub fn from_beats(beats: PositionInQuarterNotes) -> Self {
        let rpr = Reaper::get().medium_reaper();
        let result =
            rpr.time_map_qn_to_measure(reaper_medium::ProjectContext::CurrentProject, beats);
        Position {
            bar: result.measure_index,
            quarters_from_bar_start: beats - result.start_qn,
            quarters_from_bar_end: result.end_qn - beats,
            quarters_from_project_start: beats,
        }
    }

    pub fn from_ppq(take: MediaItemTake, ppq: f64) -> Self {
        let rpr = Reaper::get().medium_reaper();
        unsafe {
            let qn = rpr.midi_get_proj_qn_from_ppq_pos(take, PositionInPPQ::new(ppq));
            Self::from_beats(qn)
        }
    }

    pub fn from_seconds(seconds: f64) -> Self {
        let rpr = Reaper::get().medium_reaper();
        let qn = rpr.time_map_2_time_to_qn(
            reaper_medium::ProjectContext::CurrentProject,
            PositionInSeconds::new(seconds),
        );
        Self::from_beats(qn)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct TimeSignature {
    pub numerator: u32,
    pub denominator: u32,
}
impl TimeSignature {
    pub fn from_reaper(r_time_sig: reaper_medium::TimeSignature) -> Self {
        Self {
            numerator: r_time_sig.numerator.get(),
            denominator: r_time_sig.denominator.get(),
        }
    }
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Measure {
    pub start_qn: PositionInQuarterNotes,
    pub end_qn: PositionInQuarterNotes,
    pub time_signature: TimeSignature,
    pub start_time: PositionInSeconds,
}

impl Measure {
    pub fn from_index(index: u32) -> Self {
        let rpr = Reaper::get().medium_reaper();
        let measure_info = rpr.time_map_get_measure_info(
            reaper_medium::ProjectContext::CurrentProject,
            index as i32 - 1,
        );
        Self {
            start_qn: measure_info.start_qn,
            end_qn: measure_info.end_qn,
            time_signature: TimeSignature::from_reaper(measure_info.time_signature),
            start_time: measure_info.start_time,
        }
    }
}

pub fn get_active_take(item: reaper_medium::MediaItem) -> Option<MediaItemTake> {
    let rpr = Reaper::get().medium_reaper();
    let valid = rpr.validate_ptr_2(reaper_medium::ProjectContext::CurrentProject, item);
    unsafe {
        match valid {
            true => rpr.get_active_take(item),
            false => None,
        }
    }
}

pub fn get_track_items(track: MediaTrack) -> Option<Vec<MediaItem>> {
    let rpr = Reaper::get().medium_reaper();
    let pr = reaper_medium::ProjectContext::CurrentProject;
    let valid = rpr.validate_ptr_2(pr, track);
    if valid == false {
        return None;
    }
    let mut out: Vec<MediaItem> = Vec::new();
    for idx in 0..rpr.count_media_items(pr) {
        let item = rpr.get_media_item(pr, idx);
        if item.is_none(){
            log::error!("get_track_items loop found None at index {:?}", idx);
            break;
        }
        out.push(item.unwrap());
    }
    Some(out)
}
