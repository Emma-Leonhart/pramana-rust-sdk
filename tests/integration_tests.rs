use num_bigint::BigInt;
use num_traits::Zero;
use pramana_sdk::{Gauss, Gint, Qi, Zi, is_prime};

// ── Gint tests ─────────────────────────────────────────────────

#[test]
fn gint_construction_and_properties() {
    let g = Gint::new(3, 4);
    assert_eq!(*g.real(), BigInt::from(3));
    assert_eq!(*g.imag(), BigInt::from(4));
    assert!(!g.is_real());
    assert!(!g.is_purely_imaginary());
    assert!(!g.is_zero());
    assert!(!g.is_one());
    assert!(g.is_gaussian_integer());
}

#[test]
fn gint_constants() {
    assert!(Gint::zero().is_zero());
    assert!(Gint::one().is_one());
    assert!(Gint::one().is_unit());
    assert!(Gint::minus_one().is_unit());
    assert!(Gint::i().is_unit());
    assert!(Gint::i().is_purely_imaginary());
}

#[test]
fn gint_arithmetic() {
    let a = Gint::new(3, 4);
    let b = Gint::new(1, 2);

    // Addition
    assert_eq!(a.clone() + b.clone(), Gint::new(4, 6));

    // Subtraction
    assert_eq!(a.clone() - b.clone(), Gint::new(2, 2));

    // Multiplication: (3+4i)(1+2i) = 3+6i+4i+8i² = -5+10i
    assert_eq!(a.clone() * b.clone(), Gint::new(-5, 10));

    // Negation
    assert_eq!(-a.clone(), Gint::new(-3, -4));
}

#[test]
fn gint_conjugate_and_norm() {
    let g = Gint::new(3, 4);
    assert_eq!(g.conjugate(), Gint::new(3, -4));
    assert_eq!(g.norm(), BigInt::from(25)); // 9 + 16
}

#[test]
fn gint_power() {
    let i = Gint::i();
    assert_eq!(i.pow(2), Gint::new(-1, 0));  // i² = -1
    assert_eq!(i.pow(4), Gint::new(1, 0));   // i⁴ = 1
}

#[test]
fn gint_units_and_associates() {
    let units = Gint::units();
    assert_eq!(units.len(), 4);
    for u in &units {
        assert!(u.is_unit());
    }

    let g = Gint::new(3, 4);
    let assocs = g.associates();
    assert_eq!(assocs.len(), 3);
    for a in &assocs {
        assert!(g.is_associate(a));
    }
}

#[test]
fn gint_gcd() {
    let a = Gint::new(11, 3);
    let b = Gint::new(1, 8);
    let g = Gint::gcd(&a, &b);
    assert!(!g.is_zero());
}

#[test]
fn gint_xgcd() {
    let a = Gint::new(11, 3);
    let b = Gint::new(1, 8);
    let (g, x, y) = Gint::xgcd(&a, &b);
    // Verify Bezout's identity: a*x + b*y = gcd
    assert_eq!(a * x + b * y, g);
}

#[test]
fn gint_modified_divmod() {
    let a = Gint::new(11, 3);
    let b = Gint::new(1, 8);
    let (q, r) = Gint::modified_divmod(&a, &b);
    // Verify a = q*b + r
    assert_eq!(Gint::new(11, 3), q * b + r);
}

#[test]
fn gint_gaussian_prime() {
    assert!(Gint::is_gaussian_prime(&Gint::from_real(3)));
    assert!(Gint::is_gaussian_prime(&Gint::from_real(7)));
    assert!(Gint::is_gaussian_prime(&Gint::new(1, 1)));
    assert!(!Gint::is_gaussian_prime(&Gint::from_real(5)));
    assert!(!Gint::is_gaussian_prime(&Gint::from_real(2)));
    assert!(!Gint::is_gaussian_prime(&Gint::zero()));
    assert!(!Gint::is_gaussian_prime(&Gint::one()));
}

#[test]
fn gint_congruent_modulo() {
    let a = Gint::new(5, 3);
    let b = Gint::new(2, 1);
    let c = Gint::new(3, 2);
    assert!(Gint::congruent_modulo(&a, &b, &c));
}

#[test]
fn gint_relatively_prime() {
    let a = Gint::new(3, 0);
    let b = Gint::new(0, 5);
    // 3 and 5i: gcd should be a unit since 3 and 5 are coprime
    // (and 3 ≡ 3 mod 4, so 3 is a Gaussian prime; 5 = (2+i)(2-i))
    let g = Gint::gcd(&a, &b);
    assert!(g.is_unit());
    assert!(Gint::is_relatively_prime(&a, &b));
}

#[test]
fn gint_display() {
    assert_eq!(Gint::new(3, 4).to_string(), "3 + 4i");
    assert_eq!(Gint::new(3, -4).to_string(), "3 - 4i");
    assert_eq!(Gint::new(0, 1).to_string(), "i");
    assert_eq!(Gint::new(0, -1).to_string(), "-i");
    assert_eq!(Gint::new(5, 0).to_string(), "5");
    assert_eq!(Gint::zero().to_string(), "0");
}

#[test]
fn gint_pramana_identity() {
    let g = Gint::new(3, 4);
    assert_eq!(g.pramana_key(), "3,1,4,1");
    assert_eq!(g.pramana_label(), "pra:num:3,1,4,1");

    // Deterministic
    let a = Gint::new(3, 4);
    let b = Gint::new(3, 4);
    assert_eq!(a.pramana_id(), b.pramana_id());

    // Different values produce different UUIDs
    let c = Gint::new(4, 3);
    assert_ne!(a.pramana_id(), c.pramana_id());
}

#[test]
fn gint_to_gauss_conversion() {
    let gi = Gint::new(3, 4);
    let g = gi.to_gauss();
    assert!(g.is_gaussian_integer());
    assert_eq!(*g.a(), BigInt::from(3));
    assert_eq!(*g.c(), BigInt::from(4));
}

// ── Gauss tests ────────────────────────────────────────────────

#[test]
fn gauss_construction_and_normalization() {
    // Automatic reduction
    let g = Gauss::new(6, 4, 3, 9);
    assert_eq!(*g.a(), BigInt::from(3));
    assert_eq!(*g.b(), BigInt::from(2));
    assert_eq!(*g.c(), BigInt::from(1));
    assert_eq!(*g.d(), BigInt::from(3));
}

#[test]
fn gauss_constants() {
    assert!(Gauss::zero().is_zero());
    assert!(Gauss::one().is_one());
    assert!(Gauss::i().is_purely_imaginary());
    assert!(Gauss::minus_one().is_negative());
}

#[test]
fn gauss_arithmetic() {
    // Addition: 1/2 + 1/3 = 5/6
    let a = Gauss::new(1, 2, 0, 1);
    let b = Gauss::new(1, 3, 0, 1);
    let sum = a + b;
    assert_eq!(*sum.a(), BigInt::from(5));
    assert_eq!(*sum.b(), BigInt::from(6));

    // Multiplication: (1+i)² = 2i
    let c = Gauss::from_ints(1, 1);
    let sq = c.clone() * c;
    assert!(sq.a().is_zero());
    assert_eq!(*sq.c(), BigInt::from(2));

    // Division: (2+2i) / (1+i) = 2
    let num = Gauss::from_ints(2, 2);
    let den = Gauss::from_ints(1, 1);
    let quot = num / den;
    assert_eq!(*quot.a(), BigInt::from(2));
    assert!(quot.is_integer());
}

#[test]
fn gauss_conjugate_and_norm() {
    let g = Gauss::from_ints(3, 4);
    assert_eq!(*g.conjugate().c(), BigInt::from(-4));

    let norm = g.magnitude_squared();
    assert_eq!(*norm.a(), BigInt::from(25));
    assert!(norm.is_integer());
}

#[test]
fn gauss_reciprocal() {
    // 1/(1+i) = (1-i)/2 = 1/2 - 1/2i
    let g = Gauss::from_ints(1, 1);
    let inv = g.reciprocal().unwrap();
    assert_eq!(*inv.a(), BigInt::from(1));
    assert_eq!(*inv.b(), BigInt::from(2));
    assert_eq!(*inv.c(), BigInt::from(-1));
    assert_eq!(*inv.d(), BigInt::from(2));
}

#[test]
fn gauss_pow() {
    let g = Gauss::from_ints(1, 1);

    // (1+i)^0 = 1
    assert!(g.pow(0).unwrap().is_one());

    // (1+i)^2 = 2i
    let sq = g.pow(2).unwrap();
    assert!(sq.a().is_zero());
    assert_eq!(*sq.c(), BigInt::from(2));

    // (1+i)^(-1) = 1/2 - 1/2i
    let inv = g.pow(-1).unwrap();
    assert_eq!(*inv.a(), BigInt::from(1));
    assert_eq!(*inv.b(), BigInt::from(2));
}

#[test]
fn gauss_is_properties() {
    let real = Gauss::from_int(5);
    assert!(real.is_real());
    assert!(real.is_integer());
    assert!(real.is_gaussian_integer());
    assert!(real.is_positive());

    let frac = Gauss::new(1, 2, 0, 1);
    assert!(frac.is_real());
    assert!(!frac.is_integer());
    assert!(!frac.is_gaussian_integer());

    let complex = Gauss::from_ints(1, 1);
    assert!(!complex.is_real());
    assert!(complex.is_gaussian_integer());
}

#[test]
fn gauss_parse() {
    let g = Gauss::parse("3,2,1,4").unwrap();
    assert_eq!(*g.a(), BigInt::from(3));
    assert_eq!(*g.b(), BigInt::from(2));
    assert_eq!(*g.c(), BigInt::from(1));
    assert_eq!(*g.d(), BigInt::from(4));

    let g2 = Gauss::from_pramana("pra:num:3,2,1,4").unwrap();
    assert_eq!(g, g2);
}

#[test]
fn gauss_display_formats() {
    let g = Gauss::new(3, 2, 1, 4);
    assert_eq!(g.to_raw_string(), "<3,2,1,4>");

    assert_eq!(Gauss::from_int(5).to_string(), "5");
    assert_eq!(Gauss::i().to_string(), "i");
    assert_eq!(Gauss::from_ints(3, 4).to_string(), "3 + 4i");
    assert_eq!(Gauss::from_ints(3, -4).to_string(), "3 - 4i");
}

#[test]
fn gauss_pramana_identity() {
    let g = Gauss::new(3, 2, 1, 4);
    assert_eq!(g.pramana_key(), "3,2,1,4");
    assert_eq!(g.pramana_label(), "pra:num:3,2,1,4");

    // Deterministic
    let a = Gauss::new(3, 2, 1, 4);
    let b = Gauss::new(3, 2, 1, 4);
    assert_eq!(a.pramana_id(), b.pramana_id());

    // Equivalent fractions produce same ID (normalization)
    let c = Gauss::new(6, 4, 2, 8);
    assert_eq!(a.pramana_id(), c.pramana_id());
}

#[test]
fn gauss_floor_ceil_truncate() {
    let g = Gauss::new(7, 2, 5, 3);
    // 7/2 = 3.5, floor = 3; 5/3 ≈ 1.67, floor = 1
    let f = Gauss::floor(&g);
    assert_eq!(*f.real(), BigInt::from(3));
    assert_eq!(*f.imag(), BigInt::from(1));

    let c = Gauss::ceiling(&g);
    assert_eq!(*c.real(), BigInt::from(4));
    assert_eq!(*c.imag(), BigInt::from(2));

    let t = Gauss::truncate(&g);
    assert_eq!(*t.real(), BigInt::from(3));
    assert_eq!(*t.imag(), BigInt::from(1));
}

#[test]
fn gauss_sign_and_abs() {
    assert_eq!(Gauss::sign(&Gauss::from_int(5)).unwrap(), 1);
    assert_eq!(Gauss::sign(&Gauss::from_int(-3)).unwrap(), -1);
    assert_eq!(Gauss::sign(&Gauss::from_int(0)).unwrap(), 0);
    assert!(Gauss::sign(&Gauss::from_ints(1, 1)).is_err());
}

#[test]
fn gauss_min_max_clamp() {
    let a = Gauss::from_int(3);
    let b = Gauss::from_int(7);
    assert_eq!(Gauss::min(&a, &b), a);
    assert_eq!(Gauss::max(&a, &b), b);

    let val = Gauss::from_int(10);
    let clamped = Gauss::clamp(&val, &a, &b);
    assert_eq!(clamped, b);
}

#[test]
fn gauss_from_f64() {
    let g = Gauss::from_f64(0.5, 0.25);
    assert_eq!(*g.a(), BigInt::from(1));
    assert_eq!(*g.b(), BigInt::from(2));
    assert_eq!(*g.c(), BigInt::from(1));
    assert_eq!(*g.d(), BigInt::from(4));
}

#[test]
fn gauss_from_gint() {
    let gi = Gint::new(3, 4);
    let g: Gauss = gi.into();
    assert_eq!(*g.a(), BigInt::from(3));
    assert_eq!(*g.c(), BigInt::from(4));
    assert!(g.is_gaussian_integer());
}

#[test]
fn gauss_try_into_gint() {
    let g = Gauss::from_ints(3, 4);
    let gi = Gint::try_from(&g).unwrap();
    assert_eq!(*gi.real(), BigInt::from(3));
    assert_eq!(*gi.imag(), BigInt::from(4));

    let frac = Gauss::new(1, 2, 0, 1);
    assert!(Gint::try_from(&frac).is_err());
}

// ── Aliases ────────────────────────────────────────────────────

#[test]
fn type_aliases_work() {
    let _z: Zi = Gint::new(1, 2);
    let _q: Qi = Gauss::new(1, 2, 3, 4);
}

// ── Number theory ──────────────────────────────────────────────

#[test]
fn test_is_prime() {
    assert!(is_prime(&BigInt::from(2)));
    assert!(is_prime(&BigInt::from(3)));
    assert!(is_prime(&BigInt::from(17)));
    assert!(!is_prime(&BigInt::from(1)));
    assert!(!is_prime(&BigInt::from(4)));
    assert!(!is_prime(&BigInt::from(100)));
}

// ── Cross-language consistency ─────────────────────────────────

#[test]
fn cross_language_pramana_key_one() {
    // The integer 1 should always produce key "1,1,0,1"
    let g = Gauss::from_int(1);
    assert_eq!(g.pramana_key(), "1,1,0,1");
    let gi = Gint::one();
    assert_eq!(gi.pramana_key(), "1,1,0,1");
    // And they should produce the same UUID
    assert_eq!(g.pramana_id(), gi.pramana_id());
}

#[test]
fn cross_language_pramana_key_i() {
    let g = Gauss::i();
    assert_eq!(g.pramana_key(), "0,1,1,1");
    let gi = Gint::i();
    assert_eq!(gi.pramana_key(), "0,1,1,1");
    assert_eq!(g.pramana_id(), gi.pramana_id());
}
