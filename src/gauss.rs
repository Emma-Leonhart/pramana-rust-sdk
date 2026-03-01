use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{PramanaError, PramanaResult};
use crate::gint::Gint;
use crate::pramana_id::{pramana_label, pramana_url, pramana_uuid};

/// A Gaussian rational: an element of **Q**\[i\], stored as
/// `A/B + (C/D)i` with A, B, C, D ∈ **Z**, B > 0, D > 0,
/// and both fractions in lowest terms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gauss {
    a: BigInt, // real numerator
    b: BigInt, // real denominator (> 0)
    c: BigInt, // imaginary numerator
    d: BigInt, // imaginary denominator (> 0)
}

// ── Internal helpers ───────────────────────────────────────────

/// Normalize a fraction: reduce to lowest terms, ensure positive denominator.
fn normalize(num: BigInt, den: BigInt) -> (BigInt, BigInt) {
    if num.is_zero() {
        return (BigInt::zero(), BigInt::one());
    }
    let g = num_integer::Integer::gcd(&num, &den);
    let (mut n, mut d) = (num / &g, den / &g);
    if d.is_negative() {
        n = -n;
        d = -d;
    }
    (n, d)
}

/// Add two fractions: a/b + c/d.
fn add_frac(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt) -> (BigInt, BigInt) {
    let num = a * d + c * b;
    let den = b * d;
    normalize(num, den)
}

/// Subtract two fractions: a/b - c/d.
fn sub_frac(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt) -> (BigInt, BigInt) {
    let num = a * d - c * b;
    let den = b * d;
    normalize(num, den)
}

/// Multiply two fractions: (a/b) * (c/d).
fn mul_frac(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt) -> (BigInt, BigInt) {
    normalize(a * c, b * d)
}

/// Divide two fractions: (a/b) / (c/d).
fn div_frac(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt) -> (BigInt, BigInt) {
    normalize(a * d, b * c)
}

/// Convert a double to a fraction via continued-fraction expansion.
fn double_to_fraction(value: f64) -> (BigInt, BigInt) {
    if value == 0.0 {
        return (BigInt::zero(), BigInt::one());
    }
    let negative = value < 0.0;
    let val = value.abs();

    let mut h0 = BigInt::one();
    let mut h1 = BigInt::zero();
    let mut k0 = BigInt::zero();
    let mut k1 = BigInt::one();
    let mut x = val;

    for _ in 0..64 {
        let a_i = x.floor() as i64;
        let a_big = BigInt::from(a_i);

        let h2 = &a_big * &h0 + &h1;
        let k2 = &a_big * &k0 + &k1;

        h1 = h0;
        h0 = h2;
        k1 = k0;
        k0 = k2.clone();

        // Check convergence
        if !k2.is_zero() {
            let approx = h0.to_string().parse::<f64>().unwrap_or(0.0)
                / k2.to_string().parse::<f64>().unwrap_or(1.0);
            if (approx - val).abs() < 1e-15 * val.max(1.0) {
                break;
            }
        }

        let frac = x - a_i as f64;
        if frac.abs() < 1e-15 {
            break;
        }
        x = 1.0 / frac;
    }

    let num = if negative { -h0 } else { h0 };
    normalize(num, k0)
}

// ── Construction ───────────────────────────────────────────────

impl Gauss {
    /// Create a new Gaussian rational `a/b + (c/d)i`, automatically normalized.
    pub fn new(a: i64, b: i64, c: i64, d: i64) -> Self {
        Self::from_bigints(
            BigInt::from(a),
            BigInt::from(b),
            BigInt::from(c),
            BigInt::from(d),
        )
    }

    /// Create from BigInts, with automatic normalization.
    pub fn from_bigints(a: BigInt, b: BigInt, c: BigInt, d: BigInt) -> Self {
        assert!(!b.is_zero(), "real denominator cannot be zero");
        assert!(!d.is_zero(), "imaginary denominator cannot be zero");
        let (a, b) = normalize(a, b);
        let (c, d) = normalize(c, d);
        Self { a, b, c, d }
    }

    /// Create from a single integer.
    pub fn from_int(value: i64) -> Self {
        Self::new(value, 1, 0, 1)
    }

    /// Create from a BigInt.
    pub fn from_bigint(value: BigInt) -> Self {
        Self::from_bigints(value, BigInt::one(), BigInt::zero(), BigInt::one())
    }

    /// Create from two integers (real and imaginary parts).
    pub fn from_ints(real: i64, imag: i64) -> Self {
        Self::new(real, 1, imag, 1)
    }

    /// Create from two f64 values via continued-fraction approximation.
    pub fn from_f64(real: f64, imag: f64) -> Self {
        let (a, b) = double_to_fraction(real);
        let (c, d) = double_to_fraction(imag);
        Self { a, b, c, d }
    }

    /// Create from polar coordinates (approximate).
    pub fn from_polar(magnitude: f64, phase: f64) -> Self {
        let re = magnitude * phase.cos();
        let im = magnitude * phase.sin();
        Self::from_f64(re, im)
    }

    /// The additive identity `0`.
    pub fn zero() -> Self {
        Self::new(0, 1, 0, 1)
    }

    /// The multiplicative identity `1`.
    pub fn one() -> Self {
        Self::new(1, 1, 0, 1)
    }

    /// `-1`.
    pub fn minus_one() -> Self {
        Self::new(-1, 1, 0, 1)
    }

    /// The imaginary unit `i`.
    pub fn i() -> Self {
        Self::new(0, 1, 1, 1)
    }

    /// Alias for `i()`.
    pub fn eye() -> Self {
        Self::i()
    }

    /// The four units of **Q**\[i\]: `[1, -1, i, -i]`.
    pub fn units() -> [Self; 4] {
        [
            Self::one(),
            Self::minus_one(),
            Self::i(),
            Self::new(0, 1, -1, 1),
        ]
    }

    /// Random Gaussian rational in a range.
    pub fn random(re1: i64, re2: i64, im1: i64, im2: i64) -> Self {
        let mut rng = rand::thread_rng();
        Self::from_ints(rng.gen_range(re1..re2), rng.gen_range(im1..im2))
    }

    /// Parse from canonical `"A,B,C,D"` format.
    pub fn parse(s: &str) -> PramanaResult<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err(PramanaError::ParseError(
                "expected 4 comma-separated integers".into(),
            ));
        }
        let a: BigInt = parts[0]
            .trim()
            .parse()
            .map_err(|e| PramanaError::ParseError(format!("invalid A: {e}")))?;
        let b: BigInt = parts[1]
            .trim()
            .parse()
            .map_err(|e| PramanaError::ParseError(format!("invalid B: {e}")))?;
        let c: BigInt = parts[2]
            .trim()
            .parse()
            .map_err(|e| PramanaError::ParseError(format!("invalid C: {e}")))?;
        let d: BigInt = parts[3]
            .trim()
            .parse()
            .map_err(|e| PramanaError::ParseError(format!("invalid D: {e}")))?;
        if b.is_zero() || d.is_zero() {
            return Err(PramanaError::ZeroDenominator);
        }
        Ok(Self::from_bigints(a, b, c, d))
    }

    /// Parse from Pramana label format `"pra:num:A,B,C,D"`.
    pub fn from_pramana(s: &str) -> PramanaResult<Self> {
        let stripped = s
            .strip_prefix("pra:num:")
            .ok_or_else(|| PramanaError::ParseError("expected 'pra:num:' prefix".into()))?;
        Self::parse(stripped)
    }
}

// ── Properties ─────────────────────────────────────────────────

impl Gauss {
    /// Real numerator.
    pub fn a(&self) -> &BigInt {
        &self.a
    }
    /// Real denominator (always > 0).
    pub fn b(&self) -> &BigInt {
        &self.b
    }
    /// Imaginary numerator.
    pub fn c(&self) -> &BigInt {
        &self.c
    }
    /// Imaginary denominator (always > 0).
    pub fn d(&self) -> &BigInt {
        &self.d
    }

    /// `true` if imaginary part is zero.
    pub fn is_real(&self) -> bool {
        self.c.is_zero()
    }

    /// `true` if real is zero and imaginary is non-zero.
    pub fn is_purely_imaginary(&self) -> bool {
        self.a.is_zero() && !self.c.is_zero()
    }

    /// `true` if both parts are zero.
    pub fn is_zero(&self) -> bool {
        self.a.is_zero() && self.c.is_zero()
    }

    /// `true` if equals `1`.
    pub fn is_one(&self) -> bool {
        self.a.is_one() && self.b.is_one() && self.c.is_zero()
    }

    /// `true` if this is a real integer (imag = 0, real denominator = 1).
    pub fn is_integer(&self) -> bool {
        self.c.is_zero() && self.b.is_one()
    }

    /// `true` if both denominators are 1.
    pub fn is_gaussian_integer(&self) -> bool {
        self.b.is_one() && self.d.is_one()
    }

    /// `true` if real and positive.
    pub fn is_positive(&self) -> bool {
        self.is_real() && self.a.is_positive()
    }

    /// `true` if real and negative.
    pub fn is_negative(&self) -> bool {
        self.is_real() && self.a.is_negative()
    }

    /// Complex conjugate: `A/B - (C/D)i`.
    pub fn conjugate(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            c: -self.c.clone(),
            d: self.d.clone(),
        }
    }

    /// Exact magnitude squared: `(A/B)² + (C/D)²` as a Gaussian rational.
    pub fn magnitude_squared(&self) -> Self {
        let (re_num, re_den) = mul_frac(&self.a, &self.b, &self.a, &self.b);
        let (im_num, im_den) = mul_frac(&self.c, &self.d, &self.c, &self.d);
        let (num, den) = add_frac(&re_num, &re_den, &im_num, &im_den);
        Self::from_bigints(num, den, BigInt::zero(), BigInt::one())
    }

    /// Alias for `magnitude_squared()`.
    pub fn norm(&self) -> Self {
        self.magnitude_squared()
    }

    /// Approximate magnitude as f64.
    pub fn magnitude(&self) -> f64 {
        let re = self.real_f64();
        let im = self.imag_f64();
        (re * re + im * im).sqrt()
    }

    /// Approximate phase (argument) in radians.
    pub fn phase(&self) -> f64 {
        self.imag_f64().atan2(self.real_f64())
    }

    /// Real part as a Gaussian rational (imaginary = 0).
    pub fn real_part(&self) -> Self {
        Self::from_bigints(
            self.a.clone(),
            self.b.clone(),
            BigInt::zero(),
            BigInt::one(),
        )
    }

    /// Imaginary coefficient as a Gaussian rational (imaginary = 0).
    pub fn imaginary_part(&self) -> Self {
        Self::from_bigints(
            self.c.clone(),
            self.d.clone(),
            BigInt::zero(),
            BigInt::one(),
        )
    }

    /// Reciprocal `1/z`, using `conjugate / |z|²`.
    pub fn reciprocal(&self) -> PramanaResult<Self> {
        if self.is_zero() {
            return Err(PramanaError::DivisionByZero);
        }
        let conj = self.conjugate();
        let mag_sq = self.magnitude_squared();
        // Divide conjugate by magnitude_squared (which is purely real)
        let (new_a, new_b) = div_frac(&conj.a, &conj.b, &mag_sq.a, &mag_sq.b);
        let (new_c, new_d) = div_frac(&conj.c, &conj.d, &mag_sq.a, &mag_sq.b);
        Ok(Self::from_bigints(new_a, new_b, new_c, new_d))
    }

    /// Alias for `reciprocal()`.
    pub fn inverse(&self) -> PramanaResult<Self> {
        self.reciprocal()
    }

    /// Real part as f64 approximation.
    pub fn real_f64(&self) -> f64 {
        bigint_to_f64(&self.a) / bigint_to_f64(&self.b)
    }

    /// Imaginary part as f64 approximation.
    pub fn imag_f64(&self) -> f64 {
        bigint_to_f64(&self.c) / bigint_to_f64(&self.d)
    }

    /// The three non-trivial associates: `[self·(-1), self·i, self·(-i)]`.
    pub fn associates(&self) -> [Self; 3] {
        [
            -self.clone(),
            self.clone() * Self::i(),
            self.clone() * Self::new(0, 1, -1, 1),
        ]
    }

    /// `true` if `other` is an associate of `self`.
    pub fn is_associate(&self, other: &Self) -> bool {
        self.associates().iter().any(|a| a == other)
    }
}

// ── Pramana identity ───────────────────────────────────────────

impl Gauss {
    /// Canonical key: `"A,B,C,D"`.
    pub fn pramana_key(&self) -> String {
        format!("{},{},{},{}", self.a, self.b, self.c, self.d)
    }

    /// Deterministic UUID v5 for this value.
    pub fn pramana_id(&self) -> Uuid {
        pramana_uuid(&self.pramana_key())
    }

    /// Canonical Pramana label: `"pra:num:A,B,C,D"`.
    pub fn pramana_label(&self) -> String {
        pramana_label(&self.pramana_key())
    }

    /// Entity URL using UUID.
    pub fn pramana_url(&self) -> String {
        pramana_url(&self.pramana_id())
    }
}

// ── Arithmetic operators ───────────────────────────────────────

impl Add for Gauss {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (ra, rb) = add_frac(&self.a, &self.b, &rhs.a, &rhs.b);
        let (rc, rd) = add_frac(&self.c, &self.d, &rhs.c, &rhs.d);
        Self {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Add for &Gauss {
    type Output = Gauss;
    fn add(self, rhs: Self) -> Gauss {
        let (ra, rb) = add_frac(&self.a, &self.b, &rhs.a, &rhs.b);
        let (rc, rd) = add_frac(&self.c, &self.d, &rhs.c, &rhs.d);
        Gauss {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Sub for Gauss {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let (ra, rb) = sub_frac(&self.a, &self.b, &rhs.a, &rhs.b);
        let (rc, rd) = sub_frac(&self.c, &self.d, &rhs.c, &rhs.d);
        Self {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Sub for &Gauss {
    type Output = Gauss;
    fn sub(self, rhs: Self) -> Gauss {
        let (ra, rb) = sub_frac(&self.a, &self.b, &rhs.a, &rhs.b);
        let (rc, rd) = sub_frac(&self.c, &self.d, &rhs.c, &rhs.d);
        Gauss {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Mul for Gauss {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // (a/b + c/d i)(e/f + g/h i)
        // real = a/b * e/f - c/d * g/h
        // imag = a/b * g/h + c/d * e/f
        let (ae_n, ae_d) = mul_frac(&self.a, &self.b, &rhs.a, &rhs.b);
        let (cg_n, cg_d) = mul_frac(&self.c, &self.d, &rhs.c, &rhs.d);
        let (ag_n, ag_d) = mul_frac(&self.a, &self.b, &rhs.c, &rhs.d);
        let (ce_n, ce_d) = mul_frac(&self.c, &self.d, &rhs.a, &rhs.b);

        let (ra, rb) = sub_frac(&ae_n, &ae_d, &cg_n, &cg_d);
        let (rc, rd) = add_frac(&ag_n, &ag_d, &ce_n, &ce_d);

        Self {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Mul for &Gauss {
    type Output = Gauss;
    fn mul(self, rhs: Self) -> Gauss {
        self.clone() * rhs.clone()
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Div for Gauss {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("division by zero");
        }
        // z1 / z2 = z1 * conj(z2) / |z2|²
        let conj = rhs.conjugate();
        let numerator = self * conj;
        let mag_sq = rhs.magnitude_squared();
        // mag_sq is purely real: a/b + 0i
        let (ra, rb) = div_frac(&numerator.a, &numerator.b, &mag_sq.a, &mag_sq.b);
        let (rc, rd) = div_frac(&numerator.c, &numerator.d, &mag_sq.a, &mag_sq.b);
        Self {
            a: ra,
            b: rb,
            c: rc,
            d: rd,
        }
    }
}

impl Div for &Gauss {
    type Output = Gauss;
    fn div(self, rhs: Self) -> Gauss {
        self.clone() / rhs.clone()
    }
}

impl Rem for Gauss {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("modulo by zero");
        }
        let quotient = &self / &rhs;
        let floored = Gauss::floor(&quotient);
        self - floored.to_gauss() * rhs
    }
}

impl Neg for Gauss {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            a: -self.a,
            b: self.b,
            c: -self.c,
            d: self.d,
        }
    }
}

impl Neg for &Gauss {
    type Output = Gauss;
    fn neg(self) -> Gauss {
        Gauss {
            a: -self.a.clone(),
            b: self.b.clone(),
            c: -self.c.clone(),
            d: self.d.clone(),
        }
    }
}

// ── Exponentiation ─────────────────────────────────────────────

impl Gauss {
    /// Integer exponentiation. Negative exponents use the reciprocal.
    pub fn pow(&self, exp: i32) -> PramanaResult<Self> {
        if exp < 0 {
            let inv = self.reciprocal()?;
            return inv.pow(-exp);
        }
        let exp = exp as u32;
        if exp == 0 {
            return Ok(Self::one());
        }
        let mut result = Self::one();
        let mut base = self.clone();
        let mut e = exp;
        while e > 0 {
            if e & 1 == 1 {
                result = result * base.clone();
            }
            base = base.clone() * base;
            e >>= 1;
        }
        Ok(result)
    }
}

// ── Static math functions ──────────────────────────────────────

impl Gauss {
    /// Absolute value: for real values returns `|real|`; for complex values
    /// returns the exact magnitude squared.
    pub fn abs(value: &Self) -> Self {
        if value.is_real() {
            Self::from_bigints(
                value.a.abs(),
                value.b.clone(),
                BigInt::zero(),
                BigInt::one(),
            )
        } else {
            value.magnitude_squared()
        }
    }

    /// Sign of the real part: -1, 0, or 1. Returns error for complex values.
    pub fn sign(value: &Self) -> PramanaResult<i32> {
        if !value.is_real() {
            return Err(PramanaError::NotReal);
        }
        if value.a.is_zero() {
            Ok(0)
        } else if value.a.is_positive() {
            Ok(1)
        } else {
            Ok(-1)
        }
    }

    /// Floor: largest Gaussian integer ≤ value (component-wise).
    pub fn floor(value: &Self) -> Gint {
        Gint::new(
            floor_rational(&value.a, &value.b),
            floor_rational(&value.c, &value.d),
        )
    }

    /// Ceiling: smallest Gaussian integer ≥ value (component-wise).
    pub fn ceiling(value: &Self) -> Gint {
        Gint::new(
            ceil_rational(&value.a, &value.b),
            ceil_rational(&value.c, &value.d),
        )
    }

    /// Truncate toward zero (component-wise).
    pub fn truncate(value: &Self) -> Gint {
        Gint::new(
            trunc_rational(&value.a, &value.b),
            trunc_rational(&value.c, &value.d),
        )
    }

    /// Minimum by real part, then imaginary.
    pub fn min(a: &Self, b: &Self) -> Self {
        if a <= b {
            a.clone()
        } else {
            b.clone()
        }
    }

    /// Maximum by real part, then imaginary.
    pub fn max(a: &Self, b: &Self) -> Self {
        if a >= b {
            a.clone()
        } else {
            b.clone()
        }
    }

    /// Clamp a value between min and max.
    pub fn clamp(value: &Self, min_val: &Self, max_val: &Self) -> Self {
        Self::min(&Self::max(value, min_val), max_val)
    }

    /// Exact absolute value for real values only.
    pub fn exact_abs(value: &Self) -> PramanaResult<Self> {
        if !value.is_real() {
            return Err(PramanaError::NotReal);
        }
        Ok(Self::from_bigints(
            value.a.abs(),
            value.b.clone(),
            BigInt::zero(),
            BigInt::one(),
        ))
    }
}

// ── Equality & Ordering ────────────────────────────────────────

impl PartialEq for Gauss {
    fn eq(&self, other: &Self) -> bool {
        // Since both are normalized, direct comparison works
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
    }
}

impl Eq for Gauss {}

impl PartialOrd for Gauss {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Gauss {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by real part first (cross-multiply to avoid division), then imaginary
        let real_cmp = (&self.a * &other.b).cmp(&(&other.a * &self.b));
        match real_cmp {
            Ordering::Equal => (&self.c * &other.d).cmp(&(&other.c * &self.d)),
            ord => ord,
        }
    }
}

impl std::hash::Hash for Gauss {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
        self.c.hash(state);
        self.d.hash(state);
    }
}

// ── Display ────────────────────────────────────────────────────

impl fmt::Display for Gauss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let real_str = format_fraction(&self.a, &self.b);
        let imag_str = format_imag_fraction(&self.c, &self.d);

        if self.c.is_zero() {
            write!(f, "{real_str}")
        } else if self.a.is_zero() {
            if self.c.is_negative() {
                let pos_imag = format_imag_fraction(&(-&self.c), &self.d);
                return write!(f, "-{pos_imag}");
            }
            write!(f, "{imag_str}")
        } else if self.c.is_negative() {
            let pos_imag = format_imag_fraction(&(-&self.c), &self.d);
            write!(f, "{real_str} - {pos_imag}")
        } else {
            write!(f, "{real_str} + {imag_str}")
        }
    }
}

impl Gauss {
    /// Raw vector form: `"<A,B,C,D>"`.
    pub fn to_raw_string(&self) -> String {
        format!("<{},{},{},{}>", self.a, self.b, self.c, self.d)
    }

    /// Improper fraction form: `"A/B + C/Di"`.
    pub fn to_improper_string(&self) -> String {
        let real = if self.b.is_one() {
            format!("{}", self.a)
        } else {
            format!("{}/{}", self.a, self.b)
        };

        if self.c.is_zero() {
            return real;
        }

        let imag_abs = self.c.abs();
        let imag = if self.d.is_one() {
            if imag_abs.is_one() {
                "i".to_string()
            } else {
                format!("{imag_abs}i")
            }
        } else if imag_abs.is_one() {
            format!("i/{}", self.d)
        } else {
            format!("{imag_abs}/{}i", self.d)
        };

        if self.a.is_zero() {
            if self.c.is_negative() {
                format!("-{imag}")
            } else {
                imag
            }
        } else if self.c.is_negative() {
            format!("{real} - {imag}")
        } else {
            format!("{real} + {imag}")
        }
    }

    /// Mixed-number form, e.g. `"3 & 1/2 + 3/4 i"`.
    pub fn to_mixed_string(&self) -> String {
        let real = mixed_fraction(&self.a, &self.b);
        let imag = mixed_fraction(&self.c.abs(), &self.d);

        if self.c.is_zero() {
            return real;
        }

        let imag_part = if self.c.abs().is_one() && self.d.is_one() {
            "i".to_string()
        } else {
            format!("{imag} i")
        };

        if self.a.is_zero() {
            if self.c.is_negative() {
                format!("-{imag_part}")
            } else {
                imag_part
            }
        } else if self.c.is_negative() {
            format!("{real} - {imag_part}")
        } else {
            format!("{real} + {imag_part}")
        }
    }

    /// Decimal string approximation.
    pub fn to_decimal_string(&self, precision: usize) -> String {
        let re = self.real_f64();
        let im = self.imag_f64();

        if im == 0.0 {
            format!("{re:.prec$}", prec = precision)
        } else if re == 0.0 {
            format!("{im:.prec$}i", prec = precision)
        } else if im < 0.0 {
            format!(
                "{re:.prec$} - {im:.prec$}i",
                re = re,
                im = -im,
                prec = precision
            )
        } else {
            format!(
                "{re:.prec$} + {im:.prec$}i",
                re = re,
                im = im,
                prec = precision
            )
        }
    }
}

// ── Conversions ────────────────────────────────────────────────

impl From<i32> for Gauss {
    fn from(v: i32) -> Self {
        Self::from_int(v as i64)
    }
}

impl From<i64> for Gauss {
    fn from(v: i64) -> Self {
        Self::from_int(v)
    }
}

impl From<BigInt> for Gauss {
    fn from(v: BigInt) -> Self {
        Self::from_bigint(v)
    }
}

impl From<f64> for Gauss {
    fn from(v: f64) -> Self {
        Self::from_f64(v, 0.0)
    }
}

impl From<Gint> for Gauss {
    fn from(g: Gint) -> Self {
        g.to_gauss()
    }
}

impl TryFrom<&Gauss> for BigInt {
    type Error = PramanaError;
    fn try_from(g: &Gauss) -> PramanaResult<Self> {
        if !g.is_integer() {
            return Err(PramanaError::NotInteger);
        }
        Ok(g.a.clone())
    }
}

impl TryFrom<&Gauss> for i64 {
    type Error = PramanaError;
    fn try_from(g: &Gauss) -> PramanaResult<Self> {
        if !g.is_integer() {
            return Err(PramanaError::NotInteger);
        }
        g.a.clone()
            .try_into()
            .map_err(|_| PramanaError::Overflow("value too large for i64".into()))
    }
}

impl TryFrom<&Gauss> for f64 {
    type Error = PramanaError;
    fn try_from(g: &Gauss) -> PramanaResult<Self> {
        if !g.is_real() {
            return Err(PramanaError::NotReal);
        }
        Ok(g.real_f64())
    }
}

// ── Private helpers ────────────────────────────────────────────

fn bigint_to_f64(n: &BigInt) -> f64 {
    n.to_string().parse::<f64>().unwrap_or(f64::NAN)
}

fn floor_rational(num: &BigInt, den: &BigInt) -> BigInt {
    num.div_floor(den)
}

fn ceil_rational(num: &BigInt, den: &BigInt) -> BigInt {
    num.div_ceil(den)
}

fn trunc_rational(num: &BigInt, den: &BigInt) -> BigInt {
    num / den
}

fn format_fraction(num: &BigInt, den: &BigInt) -> String {
    if den.is_one() {
        format!("{num}")
    } else {
        format!("{num}/{den}")
    }
}

fn format_imag_fraction(num: &BigInt, den: &BigInt) -> String {
    let abs_num = num.abs();
    if den.is_one() {
        if abs_num.is_one() {
            "i".to_string()
        } else {
            format!("{num}i")
        }
    } else if abs_num.is_one() {
        if num.is_negative() {
            format!("-i/{den}")
        } else {
            format!("i/{den}")
        }
    } else {
        format!("{num}/{den}i")
    }
}

fn mixed_fraction(num: &BigInt, den: &BigInt) -> String {
    if den.is_one() {
        return format!("{num}");
    }
    let abs_num = num.abs();
    let whole = &abs_num / den;
    let remainder = &abs_num % den;

    if remainder.is_zero() {
        if num.is_negative() {
            format!("-{whole}")
        } else {
            format!("{whole}")
        }
    } else if whole.is_zero() {
        if num.is_negative() {
            format!("-{remainder}/{den}")
        } else {
            format!("{remainder}/{den}")
        }
    } else {
        let sign = if num.is_negative() { "-" } else { "" };
        format!("{sign}{whole} & {remainder}/{den}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let g = Gauss::new(3, 2, 1, 4);
        assert_eq!(*g.a(), BigInt::from(3));
        assert_eq!(*g.b(), BigInt::from(2));
        assert_eq!(*g.c(), BigInt::from(1));
        assert_eq!(*g.d(), BigInt::from(4));
    }

    #[test]
    fn test_normalization() {
        let g = Gauss::new(6, 4, 3, 9);
        assert_eq!(*g.a(), BigInt::from(3));
        assert_eq!(*g.b(), BigInt::from(2));
        assert_eq!(*g.c(), BigInt::from(1));
        assert_eq!(*g.d(), BigInt::from(3));
    }

    #[test]
    fn test_negative_denominator() {
        let g = Gauss::new(3, -2, 1, -4);
        assert_eq!(*g.a(), BigInt::from(-3));
        assert_eq!(*g.b(), BigInt::from(2));
        assert_eq!(*g.c(), BigInt::from(-1));
        assert_eq!(*g.d(), BigInt::from(4));
    }

    #[test]
    fn test_zero_one() {
        assert!(Gauss::zero().is_zero());
        assert!(Gauss::one().is_one());
        assert!(Gauss::i().is_purely_imaginary());
    }

    #[test]
    fn test_addition() {
        let a = Gauss::new(1, 2, 1, 3);
        let b = Gauss::new(1, 3, 1, 4);
        let sum = a + b;
        // 1/2 + 1/3 = 5/6, 1/3 + 1/4 = 7/12
        assert_eq!(*sum.a(), BigInt::from(5));
        assert_eq!(*sum.b(), BigInt::from(6));
        assert_eq!(*sum.c(), BigInt::from(7));
        assert_eq!(*sum.d(), BigInt::from(12));
    }

    #[test]
    fn test_subtraction() {
        let a = Gauss::new(1, 1, 0, 1);
        let b = Gauss::new(1, 1, 0, 1);
        assert!((a - b).is_zero());
    }

    #[test]
    fn test_multiplication() {
        // (1 + i) * (1 + i) = 2i
        let a = Gauss::from_ints(1, 1);
        let result = a.clone() * a;
        assert_eq!(*result.a(), BigInt::from(0));
        assert_eq!(*result.c(), BigInt::from(2));
    }

    #[test]
    fn test_division() {
        // (2 + 2i) / (1 + i) = 2
        let a = Gauss::from_ints(2, 2);
        let b = Gauss::from_ints(1, 1);
        let result = a / b;
        assert_eq!(*result.a(), BigInt::from(2));
        assert!(result.is_integer());
    }

    #[test]
    fn test_conjugate() {
        let g = Gauss::from_ints(3, 4);
        let conj = g.conjugate();
        assert_eq!(*conj.c(), BigInt::from(-4));
    }

    #[test]
    fn test_reciprocal() {
        let g = Gauss::from_ints(1, 1);
        let inv = g.reciprocal().unwrap();
        // 1/(1+i) = (1-i)/2 = 1/2 - 1/2 i
        assert_eq!(*inv.a(), BigInt::from(1));
        assert_eq!(*inv.b(), BigInt::from(2));
        assert_eq!(*inv.c(), BigInt::from(-1));
        assert_eq!(*inv.d(), BigInt::from(2));
    }

    #[test]
    fn test_pow_positive() {
        let g = Gauss::from_ints(1, 1);
        // (1+i)^2 = 2i
        let result = g.pow(2).unwrap();
        assert!(result.a().is_zero());
        assert_eq!(*result.c(), BigInt::from(2));
    }

    #[test]
    fn test_pow_negative() {
        let g = Gauss::from_ints(1, 1);
        // (1+i)^(-1) = 1/2 - 1/2 i
        let result = g.pow(-1).unwrap();
        assert_eq!(*result.a(), BigInt::from(1));
        assert_eq!(*result.b(), BigInt::from(2));
    }

    #[test]
    fn test_parse() {
        let g = Gauss::parse("3,2,1,4").unwrap();
        assert_eq!(*g.a(), BigInt::from(3));
        assert_eq!(*g.b(), BigInt::from(2));
    }

    #[test]
    fn test_from_pramana() {
        let g = Gauss::from_pramana("pra:num:3,2,1,4").unwrap();
        assert_eq!(*g.a(), BigInt::from(3));
        assert_eq!(*g.b(), BigInt::from(2));
    }

    #[test]
    fn test_pramana_key() {
        let g = Gauss::new(3, 2, 1, 4);
        assert_eq!(g.pramana_key(), "3,2,1,4");
    }

    #[test]
    fn test_pramana_id_deterministic() {
        let a = Gauss::new(3, 2, 1, 4);
        let b = Gauss::new(3, 2, 1, 4);
        assert_eq!(a.pramana_id(), b.pramana_id());
    }

    #[test]
    fn test_pramana_id_normalized() {
        // 6/4 = 3/2, so these should produce the same ID
        let a = Gauss::new(3, 2, 1, 4);
        let b = Gauss::new(6, 4, 2, 8);
        assert_eq!(a.pramana_id(), b.pramana_id());
    }

    #[test]
    fn test_display() {
        assert_eq!(Gauss::from_int(5).to_string(), "5");
        assert_eq!(Gauss::i().to_string(), "i");
        assert_eq!(Gauss::new(0, 1, -1, 1).to_string(), "-i");
        assert_eq!(Gauss::new(1, 2, 0, 1).to_string(), "1/2");
        assert_eq!(Gauss::from_ints(3, 4).to_string(), "3 + 4i");
        assert_eq!(Gauss::from_ints(3, -4).to_string(), "3 - 4i");
    }

    #[test]
    fn test_raw_string() {
        let g = Gauss::new(3, 2, 1, 4);
        assert_eq!(g.to_raw_string(), "<3,2,1,4>");
    }

    #[test]
    fn test_floor() {
        let g = Gauss::new(7, 2, 5, 3);
        let floored = Gauss::floor(&g);
        assert_eq!(*floored.real(), BigInt::from(3));
        assert_eq!(*floored.imag(), BigInt::from(1));
    }

    #[test]
    fn test_ceiling() {
        let g = Gauss::new(7, 2, 5, 3);
        let ceiled = Gauss::ceiling(&g);
        assert_eq!(*ceiled.real(), BigInt::from(4));
        assert_eq!(*ceiled.imag(), BigInt::from(2));
    }

    #[test]
    fn test_magnitude_squared() {
        let g = Gauss::from_ints(3, 4);
        let mag = g.magnitude_squared();
        assert_eq!(*mag.a(), BigInt::from(25));
        assert!(mag.is_integer());
    }

    #[test]
    fn test_from_f64() {
        let g = Gauss::from_f64(0.5, 0.25);
        assert_eq!(*g.a(), BigInt::from(1));
        assert_eq!(*g.b(), BigInt::from(2));
        assert_eq!(*g.c(), BigInt::from(1));
        assert_eq!(*g.d(), BigInt::from(4));
    }

    #[test]
    fn test_from_gint() {
        let gi = Gint::new(3, 4);
        let g: Gauss = gi.into();
        assert_eq!(*g.a(), BigInt::from(3));
        assert_eq!(*g.c(), BigInt::from(4));
        assert!(g.is_gaussian_integer());
    }

    #[test]
    fn test_is_properties() {
        let real = Gauss::from_int(5);
        assert!(real.is_real());
        assert!(real.is_integer());
        assert!(real.is_positive());
        assert!(!real.is_negative());

        let neg = Gauss::from_int(-3);
        assert!(neg.is_negative());

        let complex = Gauss::from_ints(1, 1);
        assert!(!complex.is_real());
        assert!(!complex.is_integer());
        assert!(complex.is_gaussian_integer());
    }
}
