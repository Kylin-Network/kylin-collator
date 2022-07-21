// Yes clippy this stuff looks like a computer science text book ໒( ݓ Ĺ̯ ݓ )७
#![allow(clippy::many_single_char_names)]
use statrs::function::beta::{beta, ln_beta};
use std::mem::swap;

const MAX_GAMMA: f64 = 171.624_376_956_302_7;
const MIN_LOG: f64 = -708.396_418_532_264_1; // log(2**-1022)
const MAX_LOG: f64 = 709.782_712_893_384; // log(2**1024)
const MACHINE_EPSILON: f64 = 1.110_223_024_625_156_5E-16; // maximum relative precision of f64 (2^-53)
const BIG: f64 = 4.503_599_627_370_496E15;
const BIG_INVERSE: f64 = 2.220_446_049_250_313E-16;

/// Computes the integral from minus infinity to t of the Student
/// t distribution with integer k > 0 degrees of freedom
pub fn stdtr(k: i64, t: f64) -> f64 {
    assert!(k > 0);
    if t == 0. {
        return 0.5;
    }
    let rk = k as f64;
    if t < -2. {
        let z = rk / (rk + t * t);
        let p = 0.5 * inc_beta(0.5 * rk, 0.5, z);
        return p;
    }
    // compute integral from -t to + t
    let x = match t {
        t if t < 0. => -t,
        _ => t,
    };
    let z = 1.0 + (x * x) / rk;
    if k % 2 != 0 {
        // computation for odd k
        let xsqk = x / rk.sqrt();
        let mut p = xsqk.atan();
        if k > 1 {
            let mut f = 1.0;
            let mut tz = 1.0;
            let mut j = 3;
            while j <= k - 2 && tz / f > MACHINE_EPSILON {
                tz *= (j - 1) as f64 / (z * (j as f64));
                f += tz;
                j += 2;
            }
            p += f * xsqk / z;
        }
        p *= 2.0 / std::f64::consts::PI;
        if t < 0. {
            p = -p;
        }
        p = 0.5 + 0.5 * p;
        p
    } else {
        // computation for even k
        let mut f = 1.0;
        let mut tz = 1.0;
        let mut j = 2;
        while j <= k - 2 && tz / f > MACHINE_EPSILON {
            tz *= (j - 1) as f64 / (z * (j as f64));
            f += tz;
            j += 2;
        }
        let mut p = f * x / (z * rk).sqrt();
        if t < 0. {
            p = -p;
        }
        p = 0.5 + 0.5 * p;
        p
    }
}

/// Returns incomplete beta integral of the arguments, evaluated
/// from zero to x.
pub fn inc_beta(a: f64, b: f64, x: f64) -> f64 {
    assert!(a > 0. && b > 0.);
    assert!((0. ..=1.0).contains(&x));
    if x == 0.0 {
        return 0.0;
    }
    if x as i64 == 1 {
        return 1.0;
    }
    if b * x <= 1.0 && x <= 0.95 {
        return pseries(a, b, x);
    }
    let mut x = x;
    let mut a = a;
    let mut b = b;
    let mut w = 1. - x;
    let mut xc = x;
    let mut was_swapped = false;
    // Swap a and b if x is greater than mean
    if x > a / (a + b) {
        was_swapped = true;
        swap(&mut a, &mut b);
        x = w;
        if b * x <= 1.0 && x <= 0.95 {
            let mut t = pseries(a, b, x);
            if t <= MACHINE_EPSILON {
                t = 1. - MACHINE_EPSILON;
            } else {
                t = 1. - t;
            }
            return t;
        }
    } else {
        xc = w;
    }
    let y = x * (a + b - 2.0) - (a - 1.0);
    if y < 0. {
        w = inc_bcf(a, b, x);
    } else {
        w = inc_bd(a, b, x) / xc;
    }
    let mut y = a * x.ln();
    let mut t = b * xc.ln();
    if a + b < MAX_GAMMA && y.abs() < MAX_LOG && t.abs() < MAX_LOG {
        t = xc.powf(b);
        t *= x.powf(a);
        t /= a;
        t *= w;
        t *= 1.0 / beta(a, b);
    } else {
        y += t - ln_beta(a, b);
        y += (w / a).ln();
        if y < MIN_LOG {
            t = 0.;
        } else {
            t = y.exp();
        }
    }
    if was_swapped {
        if t <= MACHINE_EPSILON {
            t = 1. - MACHINE_EPSILON;
        } else {
            t = 1. - t;
        }
    }
    t
}

/// Power series for incomplete beta integral.
fn pseries(a: f64, b: f64, x: f64) -> f64 {
    assert!(a > 0. && b > 0. && x > 0. && x < 1.);
    let a_inverse = 1. / a;
    let mut u = (1. - b) * x;
    let mut v = u / (a + 1.0);
    let t1 = v;
    let mut t = u;
    let mut n = 2.0;
    let mut s = 0.0;
    let z = MACHINE_EPSILON * a_inverse;
    while v.abs() > z {
        u = (n - b) * x / n;
        t *= u;
        v = t / (a + n);
        s += v;
        n += 1.0;
    }
    s += t1;
    s += a_inverse;
    u = a * x.ln();
    if (a + b) < MAX_GAMMA && u.abs() < MAX_LOG {
        t = 1.0 / beta(a, b);
        s = s * t * x.powf(a);
    } else {
        t = -ln_beta(a, b) + u + s.ln();
        if t < MIN_LOG {
            s = 0.0;
        } else {
            s = t.exp();
        }
    }
    s
}

/// Helper function for inc_beta
fn inc_bcf(a: f64, b: f64, x: f64) -> f64 {
    let mut k1 = a;
    let mut k2 = a + b;
    let mut k3 = a;
    let mut k4 = a + 1.0;
    let mut k5 = 1.0;
    let mut k6 = b - 1.0;
    let mut k7 = k4;
    let mut k8 = a + 2.0;
    let mut pkm2 = 0.0;
    let mut qkm2 = 1.0;
    let mut pkm1 = 1.0;
    let mut qkm1 = 1.0;
    let mut r = 1.0;
    let mut t;
    let mut answer = 1.0;
    let threshold = 3.0 * MACHINE_EPSILON;
    for _n in 0..300 {
        let xk = -(x * k1 * k2) / (k3 * k4);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        pkm2 = pkm1;
        pkm1 = pk;
        qkm2 = qkm1;
        qkm1 = qk;
        let xk = (x * k5 * k6) / (k7 * k8);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        pkm2 = pkm1;
        pkm1 = pk;
        qkm2 = qkm1;
        qkm1 = qk;
        if qk != 0. {
            r = pk / qk;
        }
        if r != 0. {
            t = ((answer - r) / r).abs();
            answer = r;
        } else {
            t = 1.0;
        }
        if t < threshold {
            return answer;
        }
        k1 += 1.0;
        k2 += 1.0;
        k3 += 2.0;
        k4 += 2.0;
        k5 += 1.0;
        k6 -= 1.0;
        k7 += 2.0;
        k8 += 2.0;
        if qk.abs() + pk.abs() > BIG {
            pkm2 *= BIG_INVERSE;
            pkm1 *= BIG_INVERSE;
            qkm2 *= BIG_INVERSE;
            qkm1 *= BIG_INVERSE;
        }
        if qk.abs() < BIG_INVERSE || pk.abs() < BIG_INVERSE {
            pkm2 *= BIG;
            pkm1 *= BIG;
            qkm2 *= BIG;
            qkm1 *= BIG;
        }
    }
    answer
}

/// Helper function for inc_beta
fn inc_bd(a: f64, b: f64, x: f64) -> f64 {
    let mut k1 = a;
    let mut k2 = b - 1.0;
    let mut k3 = a;
    let mut k4 = a + 1.0;
    let mut k5 = 1.0;
    let mut k6 = a + b;
    let mut k7 = a + 1.0;
    let mut k8 = a + 2.0;
    let mut pkm2 = 0.0;
    let mut qkm2 = 1.0;
    let mut pkm1 = 1.0;
    let mut qkm1 = 1.0;
    let z = x / (1.0 - x);
    let mut t;
    let mut answer = 1.0;
    let mut r = 1.0;
    let threshold = 3.0 * MACHINE_EPSILON;
    for _n in 0..300 {
        let xk = -(z * k1 * k2) / (k3 * k4);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        pkm2 = pkm1;
        pkm1 = pk;
        qkm2 = qkm1;
        qkm1 = qk;
        let xk = (z * k5 * k6) / (k7 * k8);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        pkm2 = pkm1;
        pkm1 = pk;
        qkm2 = qkm1;
        qkm1 = qk;
        if qk != 0. {
            r = pk / qk;
        }
        if r != 0. {
            t = ((answer - r) / r).abs();
            answer = r;
        } else {
            t = 1.0;
        }
        if t < threshold {
            return answer;
        }
        k1 += 1.0;
        k2 -= 1.0;
        k3 += 2.0;
        k4 += 2.0;
        k5 += 1.0;
        k6 += 1.0;
        k7 += 2.0;
        k8 += 2.0;
        if qk.abs() + pk.abs() > BIG {
            pkm2 *= BIG_INVERSE;
            pkm1 *= BIG_INVERSE;
            qkm2 *= BIG_INVERSE;
            qkm1 *= BIG_INVERSE;
        }
        if qk.abs() < BIG_INVERSE || pk.abs() < BIG_INVERSE {
            pkm2 *= BIG;
            pkm1 *= BIG;
            qkm2 *= BIG;
            qkm1 *= BIG;
        }
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use stdtr;

    fn assert_almost_equal(a: f64, b: f64) {
        if (a - b).abs() > 1.0E-14 {
            panic!("{:?} vs {:?}", a, b);
        }
    }

    #[test]
    fn test_inc_beta() {
        assert_eq!(inc_beta(1.0, 2.0, 0.0), 0.0);
        assert_eq!(inc_beta(1.0, 2.0, 1.0), 1.0);
        assert_almost_equal(inc_beta(1.0, 2.0, 0.2), 0.36);
        assert_almost_equal(inc_beta(5.0, 2.0, 0.5), 0.109375);
        // b * x > 1
        // x > a / (a + b)
        // a * x <= 1.0 && x <= 0.95
        assert_almost_equal(inc_beta(1.0, 3.0, 0.6), 0.9359999999999999);
        // a * x > 1.0 && x <= 0.95
        assert_almost_equal(inc_beta(4.0, 3.0, 0.6), 0.54432);

        assert_almost_equal(inc_beta(2.0, 3.0, 0.5), 0.6875);
    }

    #[quickcheck]
    fn qc_inc_beta(a: f64, b: f64, x: f64) -> TestResult {
        if !(a > 0. && b > 0.) {
            return TestResult::discard();
        } else if x < 0. || x > 1. {
            return TestResult::discard();
        }
        let passed = (inc_beta(a, b, x) - stdtr::unchecked_inc_beta(a, b, x)).abs() < 1.0E-12;
        TestResult::from_bool(passed)
    }

    #[quickcheck]
    fn qc_stdtr(k: i32, t: f64) -> TestResult {
        if k <= 0 {
            return TestResult::discard();
        }
        let passed = (stdtr(k.into(), t) - stdtr::unchecked_stdr(k, t)).abs() < 1.0E-14;
        TestResult::from_bool(passed)
    }
}
