use std::{collections::HashMap, sync::Arc};

use crate::primitives::{EventInfo, Measure, TimeMap};

#[derive(Debug)]
pub struct Voice {
    pub time_map: Arc<TimeMap>,
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
            Some(head) => {
                println!("got head: {:?}", head);
                self.insert_event(head)
            }
        }
    }
}
impl From<Arc<TimeMap>> for Voice {
    fn from(time_map: Arc<TimeMap>) -> Self {
        let mut measures = HashMap::new();
        for (idx, measure) in time_map.get().iter() {
            measures.insert(*idx, Measure::from(measure));
        }
        Self { time_map, measures }
    }
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
