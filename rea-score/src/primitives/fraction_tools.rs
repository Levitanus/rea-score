//! Tools for optimizing fractions as musical lengths.

use std::collections::VecDeque;

use fraction::Fraction;

// /// Truncate (quantize) Fraction to the provided denominator.
// ///
// /// By default library uses 1/128.
// pub fn limit_denominator(
//     frac: Fraction,
//     limit: u64,
// ) -> Result<Fraction, String> {
//     if limit < 1 {
//         return Err(format!(
//             "denominator shouldn't be less that one. input:{}",
//             limit
//         ));
//     }
//     if frac.denom().unwrap() <= &limit {
//         return Ok(Fraction::from(frac));
//     } else {
//         let (mut p0, mut q0, mut p1, mut q1) = (0, 1, 1, 0);
//         let (mut n, mut d) =
//             (frac.numer().unwrap().clone(), frac.denom().unwrap().clone());
//         let mut count: u16 = 0;
//         loop {
//             if count > 1000 {
//                 return Err(String::from("Probably, infinite loop."));
//             }
//             let a = n / d;
//             let q2 = q0 + a * q1;
//             if q2 > limit {
//                 break;
//             }
//             (p0, q0, p1, q1) = (p1, q1, p0 + a * p1, q2);
//             // let d1 = n - a * 4;
//             (n, d) = (d, n - a * d);
//             count += 1;
//         }

//         let k = (limit - q0) / q1;
//         let bound1 = Fraction::new(p0 + k * p1, q0 + k * q1);
//         let bound2 = Fraction::new(p1, q1);
//         if (bound2 - frac).abs() <= (bound1 - frac).abs() {
//             return Ok(bound2);
//         } else {
//             return Ok(bound1);
//         }
//     }
// }

/// Truncate (quantize) Fraction to the provided denominator.
///
/// By default library uses 1/128.
pub fn limit_denominator(
    frac: Fraction,
    limit: u64,
) -> Result<Fraction, String> {
    let (mut num, mut denom) = (
        *frac
            .numer()
            .ok_or("Can not get numerator from fraction".to_string())?
            as f64,
        *frac
            .denom()
            .ok_or("Can not get denominator from fraction".to_string())?
            as f64,
    );
    let limit_f64 = limit as f64;
    num *= limit_f64 / denom;
    denom *= limit_f64 / denom;
    Ok(Fraction::new(num.round() as u32, denom.round() as u32))
}
// pub fn limit_denominator(
//     frac: Fraction,
//     limit: u64,
// ) -> Result<Fraction, String> {
//     if limit < 1 {
//         return Err(format!(
//             "denominator shouldn't be less that one. input:{}",
//             limit
//         ));
//     }
//     let mut diff = Fraction::from(1.0);
//     let int = frac.floor();
//     if int == frac {
//         return Ok(frac);
//     }
//     let mut last = frac;
//     let mut cur: Fraction;
//     let mut i = 0_u64;
//     loop {
//         cur = Fraction::new(i, limit) + int;
//         let cur_diff = (frac - cur).abs();
//         if cur_diff > diff {
//             break;
//         } else {
//             diff = cur_diff;
//         }
//         if i > 1000 {
//             panic!("Probably, infinite loop")
//         }
//         last = cur;
//         i += 1u64;
//     }
//     Ok(last)
// }

fn power_of_two(num: u64) -> Option<u64> {
    if num > 1u64 {
        for i in 1u64..num {
            if 2u64.pow(i as u32) >= num {
                return Some(2u64.pow(i as u32 - 1));
            }
        }
    } else if num == 1 | 0 {
        return Some(num);
    }
    None
}

/// Split complex fraction by simple fractions, that could be interpreted as
/// musical lengths.
///
/// # Returns
///
/// Vector of fractions, started with the smallest, up to the largest.
///
/// # Example
///
/// ```
/// # use fraction::Fraction;
/// # use std::collections::VecDeque;
/// # use rea_score::primitives::normalize_fraction;
/// assert_eq!(
///     normalize_fraction(Fraction::new(13u64, 16u64), VecDeque::new()),
///         vec![
///             Fraction::new(1u64, 16u64),
///             Fraction::new(1u64, 4u64),
///             Fraction::new(1u64, 2u64)
///         ]
/// );
/// ```
pub fn normalize_fraction(
    frac: Fraction,
    mut head: VecDeque<Fraction>,
) -> VecDeque<Fraction> {
    let num = frac.numer().unwrap();
    let den = frac.denom().unwrap();
    if *num == 0 {
        return head;
    }

    if den == &1u64 || num < &5u64 {
        head.push_back(frac);
        return head;
    }
    if num == &power_of_two(*num).unwrap() {
        head.push_back(frac);
        return head;
    }
    let num_nr = power_of_two(*num).unwrap();

    let whole = Fraction::new(num_nr, *den);
    let remainder = Fraction::new(num - num_nr, *den);
    if remainder.numer().unwrap() > &3u64 {
        head.push_back(whole);
        return normalize_fraction(remainder, head);
    }
    head.push_front(whole);
    head.push_front(remainder);
    head
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use fraction::Fraction;

    use crate::primitives::{
        limit_denominator, normalize_fraction, LIMIT_DENOMINATOR,
    };

    #[test]
    fn test_limit_denominator() {
        assert_eq!(
            limit_denominator(Fraction::new(1u64, 129u64), 128).unwrap(),
            Fraction::new(1u64, 128u64)
        );
        assert_eq!(
            limit_denominator(Fraction::from(3.141592653589793), 10).unwrap(),
            Fraction::new(31u64, 10u64)
        );
        assert_eq!(
            limit_denominator(Fraction::from(3.141592653589793), 100).unwrap(),
            Fraction::new(157u64, 50u64)
        );
        assert_eq!(
            limit_denominator(Fraction::new(6_u64, 25_u64), LIMIT_DENOMINATOR)
                .unwrap(),
            Fraction::new(31_u64, 128_u64)
        );
    }

    #[test]
    fn test_normalize_fraction() {
        assert_eq!(
            normalize_fraction(Fraction::new(5u64, 8u64), VecDeque::new()),
            vec![Fraction::new(1u64, 8u64), Fraction::new(1u64, 2u64)]
        );
        assert_eq!(
            normalize_fraction(Fraction::new(13u64, 16u64), VecDeque::new()),
            vec![
                Fraction::new(1u64, 16u64),
                Fraction::new(1u64, 4u64),
                Fraction::new(1u64, 2u64)
            ]
        );
    }
}
