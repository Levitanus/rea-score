use std::{collections::HashMap, sync::Arc};

use crate::primitives::{Measure, TimeMap};

pub struct Voice {
    time_map: Arc<TimeMap>,
    measures: HashMap<u32, Measure>,
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
    use once_cell::sync::OnceCell;
    use rea_rs::TimeSignature;
    use std::sync::Arc;

    use crate::primitives::{AbsolutePosition, MeasureInfo, TimeMap, TimeMapMeasures};
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
    fn test_voice() {
        let voice_1 = Voice::from(get_time_map());
    }
}
