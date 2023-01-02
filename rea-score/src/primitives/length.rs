use std::ops::{Add, Sub};

use fraction::Fraction;
use rea_rs::{Position, Reaper, TimeSignature};

use crate::lilypond_render::RendersToLilypond;

use super::{limit_denominator, normalize_fraction, LIMIT_DENOMINATOR};

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
impl From<Position> for Length {
    fn from(value: Position) -> Self {
        Self {
            fraction: Fraction::from(
                value.as_quarters(&Reaper::get().current_project()) / 4.0,
            ),
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
impl RendersToLilypond for Length {
    fn render_lilypond(&self) -> String {
        match self.get().numer().unwrap() {
            1_u64 => format!(
                "{}",
                self.get().denom().expect("No Denominator in Length")
            ),
            3_u64 => match *self.get().denom().expect("No Denominator in Length"){
                    x if x >1=>format!("{}.",x/2),
                    _ => "\\breve.".to_string()
                },
            _ => panic!("Invalid Length to render: {:?}. What happens if normalize it? : {:?}", self, normalize_fraction(self.get(), Vec::new().into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use fraction::Fraction;

    use crate::primitives::Length;

    #[test]
    fn length() {
        let a = Length::from(1.0);
        let b = Length::from(Fraction::from(1.0));
        assert_eq!(a, b);
        assert_eq!(a.clone() + b.clone(), Length::from(2.0));
        assert_eq!(
            Length::from(1.0 / 129.0).get(),
            Fraction::new(1u64, 128u64)
        );
        assert_eq!(Length::from(1.0 / 129.0), Length::from(1.0 / 128.0));
    }
    #[test]
    #[should_panic]
    fn length_negative_sub() {
        let _ = Length::from(1.0) - Length::from(2.0);
    }
}
