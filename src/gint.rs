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
use crate::gauss::Gauss;
use crate::number_theory::is_prime;
use crate::pramana_id::{pramana_label, pramana_url, pramana_uuid};

/// A Gaussian integer: an element of **Z**\[i\], i.e. `real + imag·i`
/// where both `real` and `imag` are arbitrary-precision integers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gint {
    real: BigInt,
    imag: BigInt,
}

// ── Construction ───────────────────────────────────────────────

impl Gint {
    /// Create a new Gaussian integer.
    pub fn new(real: impl Into<BigInt>, imag: impl Into<BigInt>) -> Self {
        Self {
            real: real.into(),
            imag: imag.into(),
        }
    }

    /// Create a purely real Gaussian integer.
    pub fn from_real(real: impl Into<BigInt>) -> Self {
        Self {
            real: real.into(),
            imag: BigInt::zero(),
        }
    }

    /// The additive identity `0`.
    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    /// The multiplicative identity `1`.
    pub fn one() -> Self {
        Self::new(1, 0)
    }

    /// `-1`.
    pub fn minus_one() -> Self {
        Self::new(-1, 0)
    }

    /// The imaginary unit `i`.
    pub fn i() -> Self {
        Self::new(0, 1)
    }

    /// Alias for `i()`.
    pub fn eye() -> Self {
        Self::i()
    }

    /// `1 + i`.
    pub fn two() -> Self {
        Self::new(1, 1)
    }

    /// The four units of **Z**\[i\]: `[1, -1, i, -i]`.
    pub fn units() -> [Self; 4] {
        [Self::one(), Self::minus_one(), Self::i(), Self::new(0, -1)]
    }

    /// Random Gaussian integer with real in `[re1, re2)` and imag in `[im1, im2)`.
    pub fn random(re1: i64, re2: i64, im1: i64, im2: i64) -> Self {
        let mut rng = rand::thread_rng();
        Self::new(rng.gen_range(re1..re2), rng.gen_range(im1..im2))
    }

    /// Create from a 2-element slice `[real, imag]`.
    pub fn from_array(arr: &[BigInt]) -> PramanaResult<Self> {
        if arr.len() != 2 {
            return Err(PramanaError::ParseError(
                "array must have exactly 2 elements".into(),
            ));
        }
        Ok(Self::new(arr[0].clone(), arr[1].clone()))
    }
}

// ── Properties ─────────────────────────────────────────────────

impl Gint {
    /// Real part.
    pub fn real(&self) -> &BigInt {
        &self.real
    }

    /// Imaginary part.
    pub fn imag(&self) -> &BigInt {
        &self.imag
    }

    /// `true` if imaginary part is zero.
    pub fn is_real(&self) -> bool {
        self.imag.is_zero()
    }

    /// `true` if real part is zero and imaginary part is non-zero.
    pub fn is_purely_imaginary(&self) -> bool {
        self.real.is_zero() && !self.imag.is_zero()
    }

    /// `true` if both parts are zero.
    pub fn is_zero(&self) -> bool {
        self.real.is_zero() && self.imag.is_zero()
    }

    /// `true` if this equals `1`.
    pub fn is_one(&self) -> bool {
        self.real.is_one() && self.imag.is_zero()
    }

    /// Always `true` for `Gint` (by definition it is an integer).
    pub fn is_integer(&self) -> bool {
        self.imag.is_zero()
    }

    /// Always `true` for `Gint`.
    pub fn is_gaussian_integer(&self) -> bool {
        true
    }

    /// `true` if real part is positive and imaginary is zero.
    pub fn is_positive(&self) -> bool {
        self.imag.is_zero() && self.real.is_positive()
    }

    /// `true` if real part is negative and imaginary is zero.
    pub fn is_negative(&self) -> bool {
        self.imag.is_zero() && self.real.is_negative()
    }

    /// Complex conjugate: `real - imag·i`.
    pub fn conjugate(&self) -> Self {
        Self::new(self.real.clone(), -self.imag.clone())
    }

    /// Gaussian norm: `real² + imag²`.
    pub fn norm(&self) -> BigInt {
        &self.real * &self.real + &self.imag * &self.imag
    }

    /// `true` if the norm is 1, i.e. this is one of `{1, -1, i, -i}`.
    pub fn is_unit(&self) -> bool {
        self.norm().is_one()
    }

    /// Convert to a `Gauss` (Gaussian rational with denominators 1).
    pub fn to_gauss(&self) -> Gauss {
        Gauss::from_bigints(
            self.real.clone(),
            BigInt::one(),
            self.imag.clone(),
            BigInt::one(),
        )
    }

    /// The three non-trivial associates: `[self·(-1), self·i, self·(-i)]`.
    pub fn associates(&self) -> [Self; 3] {
        [
            -self.clone(),
            self.clone() * Self::i(),
            self.clone() * Self::new(0, -1),
        ]
    }

    /// `true` if `other` is an associate of `self`.
    pub fn is_associate(&self, other: &Self) -> bool {
        self.associates().iter().any(|a| a == other)
    }
}

// ── Pramana identity ───────────────────────────────────────────

impl Gint {
    /// Canonical key: `"real,1,imag,1"`.
    pub fn pramana_key(&self) -> String {
        format!("{},1,{},1", self.real, self.imag)
    }

    /// Deterministic UUID v5 for this value.
    pub fn pramana_id(&self) -> Uuid {
        pramana_uuid(&self.pramana_key())
    }

    /// Canonical Pramana label: `"pra:num:real,1,imag,1"`.
    pub fn pramana_label(&self) -> String {
        pramana_label(&self.pramana_key())
    }

    /// Entity URL using UUID.
    pub fn pramana_url(&self) -> String {
        pramana_url(&self.pramana_id())
    }
}

// ── Arithmetic operators ───────────────────────────────────────

impl Add for Gint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.real + rhs.real, self.imag + rhs.imag)
    }
}

impl Add for &Gint {
    type Output = Gint;
    fn add(self, rhs: Self) -> Gint {
        Gint::new(&self.real + &rhs.real, &self.imag + &rhs.imag)
    }
}

impl Sub for Gint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.real - rhs.real, self.imag - rhs.imag)
    }
}

impl Sub for &Gint {
    type Output = Gint;
    fn sub(self, rhs: Self) -> Gint {
        Gint::new(&self.real - &rhs.real, &self.imag - &rhs.imag)
    }
}

impl Mul for Gint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // (a+bi)(c+di) = (ac-bd) + (ad+bc)i
        Self::new(
            &self.real * &rhs.real - &self.imag * &rhs.imag,
            &self.real * &rhs.imag + &self.imag * &rhs.real,
        )
    }
}

impl Mul for &Gint {
    type Output = Gint;
    fn mul(self, rhs: Self) -> Gint {
        Gint::new(
            &self.real * &rhs.real - &self.imag * &rhs.imag,
            &self.real * &rhs.imag + &self.imag * &rhs.real,
        )
    }
}

impl Neg for Gint {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.real, -self.imag)
    }
}

impl Neg for &Gint {
    type Output = Gint;
    fn neg(self) -> Gint {
        Gint::new(-&self.real, -&self.imag)
    }
}

/// Floor division for Gaussian integers (rounding toward nearest integer).
impl Div for Gint {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::floor_div(&self, &rhs)
    }
}

impl Div for &Gint {
    type Output = Gint;
    fn div(self, rhs: Self) -> Gint {
        Gint::floor_div(self, rhs)
    }
}

impl Rem for Gint {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        let (_, r) = Self::modified_divmod(&self, &rhs);
        r
    }
}

impl Rem for &Gint {
    type Output = Gint;
    fn rem(self, rhs: Self) -> Gint {
        let (_, r) = Gint::modified_divmod(self, rhs);
        r
    }
}

// ── Equality & Ordering ────────────────────────────────────────

impl PartialEq for Gint {
    fn eq(&self, other: &Self) -> bool {
        self.real == other.real && self.imag == other.imag
    }
}

impl Eq for Gint {}

impl PartialOrd for Gint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Gint {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.real.cmp(&other.real) {
            Ordering::Equal => self.imag.cmp(&other.imag),
            ord => ord,
        }
    }
}

impl std::hash::Hash for Gint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.real.hash(state);
        self.imag.hash(state);
    }
}

// ── Display ────────────────────────────────────────────────────

impl fmt::Display for Gint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.imag.is_zero() {
            write!(f, "{}", self.real)
        } else if self.real.is_zero() {
            if self.imag.is_one() {
                write!(f, "i")
            } else if self.imag == BigInt::from(-1) {
                write!(f, "-i")
            } else {
                write!(f, "{}i", self.imag)
            }
        } else if self.imag.is_one() {
            write!(f, "{} + i", self.real)
        } else if self.imag == BigInt::from(-1) {
            write!(f, "{} - i", self.real)
        } else if self.imag.is_negative() {
            write!(f, "{} - {}i", self.real, -&self.imag)
        } else {
            write!(f, "{} + {}i", self.real, self.imag)
        }
    }
}

impl Gint {
    /// Raw form: `"(real, imag)"`.
    pub fn to_raw_string(&self) -> String {
        format!("({}, {})", self.real, self.imag)
    }
}

// ── Conversions ────────────────────────────────────────────────

impl From<i32> for Gint {
    fn from(v: i32) -> Self {
        Self::from_real(v)
    }
}

impl From<i64> for Gint {
    fn from(v: i64) -> Self {
        Self::from_real(v)
    }
}

impl From<BigInt> for Gint {
    fn from(v: BigInt) -> Self {
        Self::from_real(v)
    }
}

impl TryFrom<&Gauss> for Gint {
    type Error = PramanaError;
    fn try_from(g: &Gauss) -> PramanaResult<Self> {
        if !g.is_gaussian_integer() {
            return Err(PramanaError::NotGaussianInteger);
        }
        Ok(Self::new(g.a().clone(), g.c().clone()))
    }
}

// ── Number-theoretic methods ───────────────────────────────────

impl Gint {
    /// Floor division, rounding toward the nearest Gaussian integer.
    pub fn floor_div(a: &Gint, b: &Gint) -> Gint {
        let norm_b = b.norm();
        if norm_b.is_zero() {
            panic!("division by zero");
        }
        // Multiply numerator by conjugate of denominator
        let conj = b.conjugate();
        let product = a * &conj;
        // Round to nearest integer
        let re = round_div(&product.real, &norm_b);
        let im = round_div(&product.imag, &norm_b);
        Gint::new(re, im)
    }

    /// Modified division: returns `(quotient, remainder)` such that
    /// `a = quotient * b + remainder` and `norm(remainder) < norm(b)`.
    pub fn modified_divmod(a: &Gint, b: &Gint) -> (Gint, Gint) {
        let q = Self::floor_div(a, b);
        let r = a - &(&q * b);
        (q, r)
    }

    /// Greatest common divisor using the Euclidean algorithm.
    pub fn gcd(a: &Gint, b: &Gint) -> Gint {
        let mut x = a.clone();
        let mut y = b.clone();
        while !y.is_zero() {
            let (_, r) = Self::modified_divmod(&x, &y);
            x = y;
            y = r;
        }
        x
    }

    /// Extended GCD: returns `(gcd, x, y)` such that `a*x + b*y = gcd`.
    pub fn xgcd(alpha: &Gint, beta: &Gint) -> (Gint, Gint, Gint) {
        let mut old_r = alpha.clone();
        let mut r = beta.clone();
        let mut old_s = Gint::one();
        let mut s = Gint::zero();
        let mut old_t = Gint::zero();
        let mut t = Gint::one();

        while !r.is_zero() {
            let (q, remainder) = Self::modified_divmod(&old_r, &r);
            old_r = r;
            r = remainder;

            let new_s = old_s.clone() - q.clone() * s.clone();
            old_s = s;
            s = new_s;

            let new_t = old_t.clone() - q * t.clone();
            old_t = t;
            t = new_t;
        }

        (old_r, old_s, old_t)
    }

    /// `true` if `a ≡ b (mod c)`.
    pub fn congruent_modulo(a: &Gint, b: &Gint, c: &Gint) -> bool {
        let diff = a - b;
        let (_, rem) = Self::modified_divmod(&diff, c);
        rem.is_zero()
    }

    /// `true` if `gcd(a, b)` is a unit.
    pub fn is_relatively_prime(a: &Gint, b: &Gint) -> bool {
        Self::gcd(a, b).is_unit()
    }

    /// `true` if `x` is a Gaussian prime.
    ///
    /// A Gaussian integer is prime if:
    /// - It is zero → not prime
    /// - It is a unit → not prime
    /// - If real part is zero: |imag| is a rational prime ≡ 3 (mod 4)
    /// - If imag part is zero: |real| is a rational prime ≡ 3 (mod 4)
    /// - Otherwise: its norm is a rational prime
    pub fn is_gaussian_prime(x: &Gint) -> bool {
        if x.is_zero() || x.is_unit() {
            return false;
        }
        let n = x.norm();
        if x.real.is_zero() {
            let abs_imag = x.imag.abs();
            return is_prime(&abs_imag) && &abs_imag % BigInt::from(4) == BigInt::from(3);
        }
        if x.imag.is_zero() {
            let abs_real = x.real.abs();
            return is_prime(&abs_real) && &abs_real % BigInt::from(4) == BigInt::from(3);
        }
        is_prime(&n)
    }

    /// If the larger norm divides evenly by the smaller, return the quotient.
    pub fn norms_divide(a: &Gint, b: &Gint) -> Option<BigInt> {
        let na = a.norm();
        let nb = b.norm();
        if na.is_zero() || nb.is_zero() {
            return None;
        }
        let (big, small) = if na >= nb { (na, nb) } else { (nb, na) };
        let (q, r) = big.div_rem(&small);
        if r.is_zero() {
            Some(q)
        } else {
            None
        }
    }

    /// Integer exponentiation. Negative exponents are not supported for `Gint`.
    pub fn pow(&self, exp: u32) -> Self {
        if exp == 0 {
            return Self::one();
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
        result
    }
}

/// Rounds `a / b` to the nearest integer, rounding half toward positive infinity.
fn round_div(a: &BigInt, b: &BigInt) -> BigInt {
    if b.is_zero() {
        panic!("division by zero in round_div");
    }
    // Use the formula: floor((2*a + b) / (2*b)) for positive b
    let two = BigInt::from(2);
    let (b_sign, b_abs) = if b.is_negative() {
        (BigInt::from(-1), -b)
    } else {
        (BigInt::one(), b.clone())
    };
    let a_adj = a * &b_sign; // make divisor positive
    let numerator = &two * &a_adj + &b_abs;
    let denominator = &two * &b_abs;
    numerator.div_floor(&denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let g = Gint::new(3, 4);
        assert_eq!(*g.real(), BigInt::from(3));
        assert_eq!(*g.imag(), BigInt::from(4));
    }

    #[test]
    fn test_zero_one_i() {
        assert!(Gint::zero().is_zero());
        assert!(Gint::one().is_one());
        assert!(Gint::i().is_purely_imaginary());
        assert_eq!(Gint::i().imag(), &BigInt::one());
    }

    #[test]
    fn test_addition() {
        let a = Gint::new(1, 2);
        let b = Gint::new(3, 4);
        assert_eq!(a + b, Gint::new(4, 6));
    }

    #[test]
    fn test_multiplication() {
        let a = Gint::new(1, 2);
        let b = Gint::new(3, 4);
        // (1+2i)(3+4i) = 3+4i+6i+8i² = 3+10i-8 = -5+10i
        assert_eq!(a * b, Gint::new(-5, 10));
    }

    #[test]
    fn test_conjugate() {
        let g = Gint::new(3, 4);
        assert_eq!(g.conjugate(), Gint::new(3, -4));
    }

    #[test]
    fn test_norm() {
        let g = Gint::new(3, 4);
        assert_eq!(g.norm(), BigInt::from(25));
    }

    #[test]
    fn test_is_unit() {
        assert!(Gint::one().is_unit());
        assert!(Gint::minus_one().is_unit());
        assert!(Gint::i().is_unit());
        assert!(Gint::new(0, -1).is_unit());
        assert!(!Gint::new(1, 1).is_unit());
    }

    #[test]
    fn test_display() {
        assert_eq!(Gint::new(3, 4).to_string(), "3 + 4i");
        assert_eq!(Gint::new(3, -4).to_string(), "3 - 4i");
        assert_eq!(Gint::new(0, 1).to_string(), "i");
        assert_eq!(Gint::new(0, -1).to_string(), "-i");
        assert_eq!(Gint::new(5, 0).to_string(), "5");
        assert_eq!(Gint::new(3, 1).to_string(), "3 + i");
        assert_eq!(Gint::new(3, -1).to_string(), "3 - i");
    }

    #[test]
    fn test_gcd() {
        let a = Gint::new(11, 3);
        let b = Gint::new(1, 8);
        let g = Gint::gcd(&a, &b);
        // GCD result multiplied by a unit should divide both
        assert!(!g.is_zero());
    }

    #[test]
    fn test_gaussian_prime() {
        // 3 is a Gaussian prime (3 ≡ 3 mod 4)
        assert!(Gint::is_gaussian_prime(&Gint::from_real(3)));
        // 2 = -i(1+i)² is NOT a Gaussian prime
        assert!(!Gint::is_gaussian_prime(&Gint::from_real(2)));
        // 1+i has norm 2 which is prime
        assert!(Gint::is_gaussian_prime(&Gint::new(1, 1)));
    }

    #[test]
    fn test_pramana_id_deterministic() {
        let a = Gint::new(3, 4);
        let b = Gint::new(3, 4);
        assert_eq!(a.pramana_id(), b.pramana_id());
    }

    #[test]
    fn test_pramana_key() {
        let g = Gint::new(3, 4);
        assert_eq!(g.pramana_key(), "3,1,4,1");
    }

    #[test]
    fn test_pramana_label() {
        let g = Gint::new(3, 4);
        assert_eq!(g.pramana_label(), "pra:num:3,1,4,1");
    }

    #[test]
    fn test_modified_divmod() {
        let a = Gint::new(11, 3);
        let b = Gint::new(1, 8);
        let (q, r) = Gint::modified_divmod(&a, &b);
        // Verify a = q*b + r
        assert_eq!(a, q * b + r);
    }

    #[test]
    fn test_xgcd() {
        let a = Gint::new(11, 3);
        let b = Gint::new(1, 8);
        let (g, x, y) = Gint::xgcd(&a, &b);
        // Verify a*x + b*y = g
        assert_eq!(a.clone() * x + b.clone() * y, g);
    }
}
