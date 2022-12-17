//! Main "ruler" for making voices and moving through score.
use std::collections::HashMap;

use rea_rs::TimeSignature;

use super::{
    position::{AbsolutePosition, RelativePosition},
    Length,
};
pub type TimeMapMeasures = HashMap<u32, MeasureInfo>;

/// Represents area of timeline, that should be exported.
///
/// Considered to be used as reference for building voices, navigating
/// through them and converting positions from absolute to relative.
#[derive(Debug)]
pub struct TimeMap {
    /// indexes are measure numbers on timeline (1-based)
    measures: TimeMapMeasures,
    /// start measure of TimeMap
    begin: u32,
    /// end measure of TimeMap
    end: u32,
    /// start position of the first measure in map
    start_position: AbsolutePosition,
}
impl TimeMap {
    /// be careful with start position
    pub fn new(measures: HashMap<u32, MeasureInfo>, start_position: AbsolutePosition) -> Self {
        let (mut begin, mut end) = (0u32, 0u32);
        for idx in measures.keys() {
            if (begin, end) == (0, 0) {
                (begin, end) = (*idx, *idx);
            }
            if idx < &begin {
                begin = *idx;
            }
            if idx > &end {
                end = *idx;
            }
        }
        Self {
            measures,
            begin,
            end,
            start_position,
        }
    }
    /// Get absolute position of measure start.
    /// Index is 1-based.
    pub fn get_absolute_position_of_measure(&self, measure_index: u32) -> AbsolutePosition {
        let mut counted_abs = self.start_position.clone();
        for idx in self.begin..measure_index {
            let measure = &self.measures[&idx];
            counted_abs += measure.length.clone();
        }
        counted_abs
    }
    /// Get measure under given position.
    ///
    ///  # Returns
    /// MeasureInfo block and absolute position of its start.
    pub fn get_measure_from_absolute_position(
        &self,
        absolute: &AbsolutePosition,
    ) -> Option<(MeasureInfo, AbsolutePosition)> {
        let mut counted_abs = self.start_position.clone();
        for idx in self.begin..self.end + 1 {
            let measure = &self.measures[&idx];
            let last_measure_pos = counted_abs.clone();
            counted_abs += measure.length.clone();
            if counted_abs > *absolute {
                return Some((measure.clone(), last_measure_pos));
            }
        }
        None
    }

    pub fn pos_relative_from_absolute(
        &self,
        absolute: &AbsolutePosition,
    ) -> Option<RelativePosition> {
        match self.get_measure_from_absolute_position(absolute) {
            Some((measure, measure_start)) => Some(RelativePosition::new(
                measure.index,
                absolute.get() - measure_start.get(),
            )),
            None => None,
        }
    }
    pub fn pos_absolute_from_relative(&self, relative: &RelativePosition) -> AbsolutePosition {
        let measure_index = relative.get_measure_index();
        let m_pos = self.get_absolute_position_of_measure(measure_index);
        let relative_pos = relative.get_position();
        AbsolutePosition::from(m_pos.get() + relative_pos)
    }
    pub fn get_measure_info(&self, measure_index: u32) -> MeasureInfo {
        self.measures[&measure_index].clone()
    }
    pub fn get(&self) -> &HashMap<u32, MeasureInfo> {
        &self.measures
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct MeasureInfo {
    pub index: u32,
    pub time_signature: TimeSignature,
    pub length: Length,
}
impl MeasureInfo {
    pub fn new(index: u32, time_signature: TimeSignature) -> Self {
        let length = Length::from(&time_signature);
        Self {
            index,
            time_signature,
            length,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fraction::Fraction;
    use rea_rs::TimeSignature;

    use crate::primitives::{
        position::{AbsolutePosition, RelativePosition},
        Length,
    };

    use super::{MeasureInfo, TimeMap, TimeMapMeasures};

    fn measures_from_ts(info: HashMap<u32, TimeSignature>) -> TimeMapMeasures {
        let mut measures = HashMap::new();
        for (idx, time_signature) in info {
            let length = Length::from(&time_signature);
            measures.insert(
                idx,
                MeasureInfo {
                    index: idx,
                    time_signature,
                    length,
                },
            );
        }
        measures
    }

    fn time_map_1() -> TimeMap {
        let info = HashMap::from([
            (1, TimeSignature::new(4, 4)),
            (2, TimeSignature::new(4, 4)),
            (3, TimeSignature::new(4, 4)),
            (4, TimeSignature::new(7, 8)),
            (5, TimeSignature::new(9, 8)),
            (6, TimeSignature::new(4, 4)),
        ]);

        TimeMap::new(measures_from_ts(info), 0.0.into())
    }

    fn time_map_2() -> TimeMap {
        let info = HashMap::from([
            (3, TimeSignature::new(4, 4)),
            (4, TimeSignature::new(7, 8)),
            (5, TimeSignature::new(9, 8)),
            (6, TimeSignature::new(4, 4)),
        ]);

        TimeMap::new(measures_from_ts(info), 2.0.into())
    }

    #[test]
    fn test_contents() {
        let time_map = time_map_1();
        assert_eq!(time_map.begin, 1);
        assert_eq!(time_map.end, 6);
        assert_eq!(time_map.get().len(), 6);
        let time_signature = TimeSignature::new(4, 4);
        assert_eq!(
            time_map.get_measure_info(2),
            MeasureInfo {
                index: 2,
                length: Length::from(&time_signature),
                time_signature
            }
        );
        let time_signature = TimeSignature::new(7, 8);
        assert_eq!(
            time_map.get_measure_info(4),
            MeasureInfo {
                index: 4,
                length: Length::from(&time_signature),
                time_signature
            }
        );
        let time_signature = TimeSignature::new(9, 8);
        assert_eq!(
            time_map.get_measure_info(5),
            MeasureInfo {
                index: 5,
                length: Length::from(&time_signature),
                time_signature
            }
        );
    }
    #[test]
    fn test_measure_position() {
        let time_map = time_map_1();
        let time_map_2 = time_map_2();
        assert_eq!(
            time_map.get_absolute_position_of_measure(1),
            AbsolutePosition::from(0.0)
        );
        assert_eq!(
            time_map.get_absolute_position_of_measure(2),
            AbsolutePosition::from(1.0)
        );
        assert_eq!(
            time_map_2.get_absolute_position_of_measure(3),
            AbsolutePosition::from(2.0)
        );
        assert_eq!(
            time_map.get_absolute_position_of_measure(4),
            AbsolutePosition::from(3.0)
        );
        assert_eq!(
            time_map_2.get_absolute_position_of_measure(4),
            AbsolutePosition::from(3.0)
        );

        for time_map in [time_map, time_map_2].iter() {
            let position_5 =
                AbsolutePosition::from(Fraction::from(3.0) + Fraction::new(7u64, 8u64));
            assert_eq!(time_map.get_absolute_position_of_measure(5), position_5);
            let position_6 = AbsolutePosition::from(position_5.get() + Fraction::new(9u64, 8u64));
            assert_eq!(time_map.get_absolute_position_of_measure(6), position_6);
        }
    }

    #[test]
    fn test_converter() {
        env_logger::init();
        let time_map = time_map_2();
        let absolute = AbsolutePosition::from(Fraction::new(8 * 3 + 7 + 3 as u64, 8 as u64));
        let relative = RelativePosition::new(5, Fraction::new(3u64, 8u64));
        assert_eq!(&time_map.pos_absolute_from_relative(&relative), &absolute);
        assert_eq!(
            &time_map.pos_relative_from_absolute(&absolute).unwrap(),
            &relative
        );
        //
        let absolute = AbsolutePosition::from(Fraction::new(8 * 3 + 7 + 9 + 3 as u64, 8 as u64));
        let relative = RelativePosition::new(6, Fraction::new(3u64, 8u64));
        assert_eq!(&time_map.pos_absolute_from_relative(&relative), &absolute);
        assert_eq!(
            &time_map.pos_relative_from_absolute(&absolute).unwrap(),
            &relative
        );
    }
}
