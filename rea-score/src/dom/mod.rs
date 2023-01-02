use crate::{
    lilypond_render::RendersToLilypond,
    primitives::{EventInfo, Measure, TimeMap},
};
use itertools::Itertools;
use rea_rs::{errors::ReaperError, Immutable, Position, Track};
use std::{collections::HashMap, error::Error, sync::Arc};

use self::midi_parse::{parse_events, ParsedEvent};

pub mod midi_parse;

#[derive(Debug)]
pub struct Voice {
    pub time_map: Arc<TimeMap>,
    pub index: u8,
    pub measures: HashMap<u32, Measure>,
}
impl Voice {
    pub fn insert_event(&mut self, event: EventInfo) -> Result<(), String> {
        let index = event.position.get_measure_index();
        let head = self
            .measures
            .get_mut(&index)
            .ok_or(format!("Can not find measure {}", index))?
            .insert(event)?;
        match head {
            None => Ok(()),
            Some(head) => self.insert_event(head),
        }
    }
}
impl From<Arc<TimeMap>> for Voice {
    fn from(time_map: Arc<TimeMap>) -> Self {
        let mut measures = HashMap::new();
        for (idx, measure) in time_map.get().iter() {
            measures.insert(*idx, Measure::from(measure));
        }
        Self {
            time_map,
            measures,
            index: 0,
        }
    }
}
impl RendersToLilypond for Voice {
    fn render_lilypond(&self) -> String {
        self.measures
            .iter()
            .sorted_by(|(idx1, _), (idx2, _)| Ord::cmp(idx1, idx2))
            .map(|(idx, measure)| {
                let ts = match idx {
                    1 => measure.get_time_signature().render_lilypond(),
                    x => match self.measures.get(&(x - 1)) {
                        None => measure.get_time_signature().render_lilypond(),
                        Some(m) => match m.get_time_signature()
                            == measure.get_time_signature()
                        {
                            true => "".to_string(),
                            false => {
                                measure.get_time_signature().render_lilypond()
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
                format! {
                    "% bar{idx}\n{ts} {events} |",
                }
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
            pos >= start_pos && pos <= end_pos
        });

        events.extend(parse_events(evts, &take)?);
    }
    Ok(events)
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
                        (1, MeasureInfo::new(1, TimeSignature::new(4, 4))),
                        (2, MeasureInfo::new(2, TimeSignature::new(4, 4))),
                        (3, MeasureInfo::new(3, TimeSignature::new(4, 4))),
                        (4, MeasureInfo::new(4, TimeSignature::new(7, 8))),
                        (5, MeasureInfo::new(5, TimeSignature::new(9, 8))),
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
            voice_1.measures.get(&2).unwrap(),
            &Measure::new(2, TimeSignature::new(4, 4))
        );
        assert_eq!(
            voice_1.measures.get(&4).unwrap(),
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
