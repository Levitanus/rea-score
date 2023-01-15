use crate::{
    lilypond_render::RendersToLilypond,
    primitives::{EventInfo, Measure, TimeMap},
};
use itertools::Itertools;
use rea_rs::{
    errors::ReaperError, Immutable, MidiEvent, MidiEventBuilder,
    MidiEventConsumer, MidiMessage, NoteOffMessage, Position, RawMidiMessage,
    Reaper, Track,
};
use std::{collections::HashMap, error::Error, sync::Arc};

use self::midi_parse::{parse_events, ParsedEvent};

pub mod midi_parse;

#[derive(Debug)]
pub struct Voice {
    pub time_map: Arc<TimeMap>,
    pub index: u8,
    pub begin_measure: u32,
    measures: Vec<Measure>,
}
impl Voice {
    pub fn insert_event(
        &mut self,
        mut event: EventInfo,
    ) -> Result<(), String> {
        let mut index = event.position.get_measure_index();
        if index < self.begin_measure {
            eprintln!("Event from previous measures. Probably, tied note.");
            index = self.begin_measure;
            event.position.set_measure_index(index);
            event.position.set_position(0.0.into());
        }
        let head = self
            .measures
            .get_mut((index - self.begin_measure) as usize)
            .ok_or(format!(
                "Can not find measure {}\nrequired by event: {:#?}",
                index, event
            ))?
            .insert(event)?;
        match head {
            None => Ok(()),
            Some(head) => self.insert_event(head),
        }
    }
    pub fn get_measure(&self, index: u32) -> Option<&Measure> {
        self.measures.get((index - self.begin_measure) as usize)
    }
    pub fn get_measure_mut(&mut self, index: u32) -> Option<&mut Measure> {
        self.measures.get_mut((index - self.begin_measure) as usize)
    }
}
impl From<Arc<TimeMap>> for Voice {
    fn from(time_map: Arc<TimeMap>) -> Self {
        let mut measures = Vec::new();
        for measure in time_map.get().iter() {
            measures.push(Measure::from(measure));
        }
        let begin_measure = time_map.begin_measure();
        Self {
            time_map,
            index: 0,
            begin_measure,
            measures,
        }
    }
}
impl RendersToLilypond for Voice {
    fn render_lilypond(&self) -> String {
        self.measures
            .iter()
            .map(|measure| {
                let ts = match measure.index() {
                    x if x == self.begin_measure => {
                        measure.time_signature().render_lilypond()
                    }
                    x => match self
                        .measures
                        .get((x - self.begin_measure - 1) as usize)
                    {
                        None => measure.time_signature().render_lilypond(),
                        Some(m) => match m.time_signature()
                            == measure.time_signature()
                        {
                            true => "".to_string(),
                            false => {
                                measure.time_signature().render_lilypond()
                            }
                        },
                    },
                };
                let events = measure
                    .get_events_normalized()
                    .expect("Can not get normalized events")
                    .iter()
                    .map(|ev| ev.render_lilypond())
                    .join(" ");
                format!("% bar{}\n{ts} {events} |", measure.index())
            })
            .join(" ")
    }
}

#[derive(Debug)]
pub struct Staff {
    pub time_map: Arc<TimeMap>,
    pub index: u8,
    pub voices: Vec<Voice>,
}
impl Staff {
    pub fn new(time_map: Arc<TimeMap>, index: u8, voices: Vec<Voice>) -> Self {
        Self {
            time_map,
            index,
            voices,
        }
    }
}
impl RendersToLilypond for Staff {
    fn render_lilypond(&self) -> String {
        if self.voices.len() == 1 {
            self.voices[0].render_lilypond()
        } else {
            todo!()
        }
    }
}

#[derive(Debug)]
pub struct Part {
    pub time_map: Arc<TimeMap>,
    pub staves: Vec<Staff>,
}
impl Part {
    pub fn new(time_map: Arc<TimeMap>, staves: Vec<Staff>) -> Self {
        Self { time_map, staves }
    }
}
impl RendersToLilypond for Part {
    fn render_lilypond(&self) -> String {
        if self.staves.len() == 1 {
            return self.staves[0].render_lilypond();
        }
        format!(
            "<< {} >>",
            self.staves
                .iter()
                .map(|staff| staff.render_lilypond())
                .join(" ")
        )
    }
}

pub fn parse_track_in_bounds(
    track: Track<Immutable>,
    start_pos: impl Into<Position>,
    end_pos: impl Into<Position>,
) -> Result<Part, Box<dyn Error>> {
    let (start_pos, end_pos) = (start_pos.into(), end_pos.into());
    let events = get_track_midi_in_bounds(track, start_pos, end_pos)?
        .into_iter()
        .map(|ev| ev.apply_single_notations());
    // println!("events: {:?}", events.clone().collect_vec());
    let time_map = Arc::new(TimeMap::build_from_bounds(start_pos, end_pos));
    let voices = voices_from_events(events, time_map.clone())?;
    // println!("voices: {:?}", voices);
    let staves = staves_from_voices(voices, time_map.clone());
    // println!("staves: {:?}", staves);
    Ok(Part::new(time_map.clone(), staves))
}
fn staves_from_voices(
    voices: Vec<Voice>,
    time_map: Arc<TimeMap>,
) -> Vec<Staff> {
    let st: HashMap<u8, Staff> =
        voices.into_iter().fold(HashMap::new(), |mut map, voice| {
            let st_idx = match voice.index {
                1..=4 => 1,
                5..=8 => 2,
                9..=12 => 3,
                13..=16 => 4,
                _ => panic!("Can not place voice with index {}", voice.index),
            };
            match map.get_mut(&st_idx) {
                None => {
                    map.insert(
                        st_idx,
                        Staff::new(time_map.clone(), st_idx, vec![voice]),
                    );
                }
                Some(staff) => staff.voices.push(voice),
            };
            map
        });
    st.into_iter()
        .sorted_by(|(a, _), (b, _)| Ord::cmp(&b, &a))
        .map(|(_, v)| v)
        .collect()
}

fn voices_from_events(
    events: impl Iterator<Item = ParsedEvent>,
    time_map: Arc<TimeMap>,
) -> Result<Vec<Voice>, String> {
    let voices: Result<HashMap<u8, Voice>, String> =
        events.fold(Ok(HashMap::new()), |voices, ev| {
            let idx = ev.channel;
            let mut voices = voices?;
            match voices.get_mut(&idx) {
                None => {
                    let mut v = Voice::from(time_map.clone());
                    v.index = idx;
                    voices.insert(idx, v);
                    voices.get_mut(&idx).unwrap().insert_event(ev.event)?
                }
                Some(vc) => vc.insert_event(ev.event)?,
            }
            Ok(voices)
        });
    let voices = match voices {
        Ok(v) => v,
        Err(err) => return Err(err),
    };
    Ok(voices
        .into_iter()
        .sorted_by(|(a, _), (b, _)| Ord::cmp(&b, &a))
        .map(|(_, v)| v)
        .collect())
}
fn get_track_midi_in_bounds(
    track: Track<Immutable>,
    start_pos: impl Into<Position>,
    end_pos: impl Into<Position>,
) -> Result<Vec<ParsedEvent>, ReaperError> {
    let start_pos = start_pos.into();
    let end_pos = end_pos.into();
    let n_items = track.n_items();
    let mut events = Vec::new();
    for idx in 0..n_items {
        let item = track.get_item(idx).expect("Should be item here");
        if !(item.position() <= end_pos && item.end_position() >= start_pos) {
            continue;
        }
        let take = item.active_take();
        let evts = take.iter_midi(None)?.filter(|ev| {
            let pos = Position::from_ppq(ev.ppq_position(), &take);
            if pos == start_pos
                && NoteOffMessage::from_raw(ev.message().get_raw()).is_some()
            {
                return false;
            }
            pos >= start_pos && pos <= end_pos
        });

        events.extend(parse_events(evts, &take)?);
    }
    Ok(events)
}

pub fn get_edited_midi() -> Result<MidiEventBuilder, ReaperError> {
    let rpr = Reaper::get();
    let mut pr = rpr.current_project();
    match rpr.active_midi_editor() {
        Some(mut e) => e.item_mut(&pr).active_take().iter_midi(None),
        None => pr
            .get_selected_item_mut(0)
            .ok_or(ReaperError::InvalidObject(
                "No opened editor and no selected item found.",
            ))?
            .active_take()
            .iter_midi(None),
    }
}

pub fn set_edited_midi(
    events: Vec<MidiEvent<RawMidiMessage>>,
) -> Result<(), ReaperError> {
    let rpr = Reaper::get();
    let mut pr = rpr.current_project();
    match rpr.active_midi_editor() {
        Some(mut e) => {
            let mut item = e.item_mut(&pr);
            let mut take = item.active_take_mut();
            take.set_midi(
                MidiEventConsumer::new(events.into_iter()).collect(),
            )?;
            take.sort_midi();
        }
        None => {
            let mut item = pr.get_selected_item_mut(0).ok_or(
                ReaperError::InvalidObject(
                    "No opened editor and no selected item found.",
                ),
            )?;
            let mut take = item.active_take_mut();
            take.set_midi(
                MidiEventConsumer::new(events.into_iter()).collect(),
            )?;
            take.sort_midi();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fraction::Fraction;
    use musical_note::Accidental;
    use once_cell::sync::OnceCell;
    use rea_rs::TimeSignature;
    use std::sync::Arc;

    use crate::primitives::{
        AbsolutePosition, EventInfo, EventType, Length, Measure, MeasureInfo,
        Note, Pitch, RelativePosition, TimeMap, TimeMapMeasures,
    };
    static TIME_MAP: OnceCell<Arc<TimeMap>> = OnceCell::new();

    use super::Voice;
    fn get_time_map() -> Arc<TimeMap> {
        match TIME_MAP.get() {
            None => {
                let tm = Arc::new(TimeMap::new(
                    TimeMapMeasures::from([
                        MeasureInfo::new(1, TimeSignature::new(4, 4)),
                        MeasureInfo::new(2, TimeSignature::new(4, 4)),
                        MeasureInfo::new(3, TimeSignature::new(4, 4)),
                        MeasureInfo::new(4, TimeSignature::new(7, 8)),
                        MeasureInfo::new(5, TimeSignature::new(9, 8)),
                    ]),
                    AbsolutePosition::from(0.0),
                ));
                TIME_MAP.set(tm.clone()).unwrap();
                tm
            }
            Some(time_map) => time_map.clone(),
        }
    }

    #[test]
    fn test_voice() -> Result<(), String> {
        env_logger::init();
        let mut voice_1 = Voice::from(get_time_map());
        // println!("voice:contents before: {:#?}", voice_1);

        assert_eq!(
            voice_1.get_measure(2).unwrap(),
            &Measure::new(2, TimeSignature::new(4, 4))
        );
        assert_eq!(
            voice_1.get_measure(4).unwrap(),
            &Measure::new(4, TimeSignature::new(7, 8))
        );
        let c = EventInfo::new(
            RelativePosition::new(3, Fraction::new(6_u64, 8_u64)),
            Length::from(Fraction::new(5_u64, 8_u64)),
            EventType::Note(Note::new(Pitch::from_midi(60, None, None))),
        );
        let es = EventInfo::new(
            RelativePosition::new(3, Fraction::new(6_u64, 8_u64)),
            Length::from(Fraction::new(5_u64, 8_u64)),
            EventType::Note(Note::new(Pitch::from_midi(
                63,
                Accidental::Flat.into(),
                None,
            ))),
        );
        let g = EventInfo::new(
            RelativePosition::new(4, Fraction::new(0_u64, 8_u64)),
            Length::from(Fraction::new(3_u64, 8_u64)),
            EventType::Note(Note::new(Pitch::from_midi(67, None, None))),
        );
        voice_1.insert_event(c)?;
        voice_1.insert_event(g)?;
        voice_1.insert_event(es)?;

        Ok(())
    }
}
