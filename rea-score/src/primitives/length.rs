use std::ops::{Add, Sub};

use fraction::Fraction;
use reaper_medium::PositionInQuarterNotes;

use super::{limit_denominator, TimeSignature, LIMIT_DENOMINATOR};

#[derive(Debug, PartialOrd, Clone)]
pub struct Length {
    fraction: Fraction,
}
impl Length {
    pub fn get(&self) -> Fraction {
        limit_denominator(self.fraction, LIMIT_DENOMINATOR).unwrap()
    }
}
impl PartialEq for Length {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
    fn ne(&self, other: &Self) -> bool {
        self.get() != other.get()
    }
}
impl From<Fraction> for Length {
    fn from(value: Fraction) -> Self {
        Self { fraction: value }
    }
}
impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Self::from(Fraction::from(value))
    }
}
impl From<&TimeSignature> for Length {
    fn from(ts: &TimeSignature) -> Self {
        Self {
            fraction: Fraction::new(ts.numerator, ts.denominator),
        }
    }
}
impl From<PositionInQuarterNotes> for Length {
    fn from(value: PositionInQuarterNotes) -> Self {
        Self {
            fraction: Fraction::from(value.get() / 4.0),
        }
    }
}
impl Add for Length {
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            fraction: self.get() + rhs.get(),
        }
    }
    type Output = Self;
}
impl Sub for Length {
    fn sub(self, rhs: Self) -> Self::Output {
        let frac = self.get() - rhs.get();
        if frac.is_sign_negative() {
            panic!(
                "length can not be negative. left: {}, right: {}, result: {}",
                self.get(),
                rhs.get(),
                frac
            );
        }
        Self::from(frac)
    }
    type Output = Self;
}

#[cfg(test)]
mod tests {
    use fraction::Fraction;
    use reaper_medium::PositionInQuarterNotes;

    use crate::primitives::Length;

    #[test]
    fn length() {
        let a = Length::from(1.0);
        let b = Length::from(PositionInQuarterNotes::new(4.0));
        assert_eq!(a, b);
        assert_eq!(a.clone() + b.clone(), Length::from(2.0));
        assert_eq!(Length::from(1.0 / 129.0).get(), Fraction::new(1u64, 128u64));
        assert_eq!(Length::from(1.0 / 129.0), Length::from(1.0 / 128.0));
    }
    #[test]
    #[should_panic]
    fn length_negative_sub() {
        let _ = Length::from(1.0) - Length::from(2.0);
    }
}
