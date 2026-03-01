use num_bigint::BigInt;
use num_traits::{One, Zero};

/// Returns `true` if `n` is a prime number.
///
/// Uses trial division up to √n with the 6k±1 optimisation.
pub fn is_prime(n: &BigInt) -> bool {
    if *n <= BigInt::one() {
        return false;
    }
    let two = BigInt::from(2);
    let three = BigInt::from(3);
    if *n <= three {
        return true;
    }
    if n % &two == BigInt::zero() || n % &three == BigInt::zero() {
        return false;
    }
    let mut i = BigInt::from(5);
    loop {
        if &i * &i > *n {
            break;
        }
        if n % &i == BigInt::zero() {
            return false;
        }
        let i_plus_2 = &i + 2;
        if n % &i_plus_2 == BigInt::zero() {
            return false;
        }
        i += 6;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_primes() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        for p in primes {
            assert!(is_prime(&BigInt::from(p)), "{p} should be prime");
        }
    }

    #[test]
    fn test_non_primes() {
        let composites = [0, 1, 4, 6, 8, 9, 10, 12, 14, 15, 16, 25, 100];
        for c in composites {
            assert!(!is_prime(&BigInt::from(c)), "{c} should not be prime");
        }
    }

    #[test]
    fn test_negative() {
        assert!(!is_prime(&BigInt::from(-7)));
    }
}
