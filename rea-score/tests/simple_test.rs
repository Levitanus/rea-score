use fraction::Fraction;

#[test]
fn limit_denominator() {
    let frac = Fraction::new(6_u32, 25_u32);
    println!("{:?}", frac);
    let (num, denom) = (frac.numer().unwrap(), frac.denom().unwrap());
    let (num, denom) = (num * 128, denom * 128);
    println!("{num}, {denom}");
    let mut diff = Fraction::from(1.0);
    let mut cur = Fraction::from(0.0);
    for i in 1_u64..=128_u64 {
        cur = Fraction::new(i, 128_u64);
        let cur_diff = (frac - cur).abs();
        if cur_diff > diff {
            break;
        } else {
            diff = cur_diff;
        }
    }
    println!("{:?}", cur);
}
