use crate::error::EscrowError;
use soroban_sdk::{Env, U256};

pub struct SafeMath;

pub trait SafeArithmetic {
    fn safe_mul_div(amount: i128, multiplier: u32, divisor: i128) -> Result<i128, EscrowError>;
}

impl SafeArithmetic for SafeMath {
    fn safe_mul_div(amount: i128, multiplier: u32, divisor: i128) -> Result<i128, EscrowError> {
        if divisor == 0 {
            return Err(EscrowError::DivisionError);
        }
        // Compute amount * multiplier / divisor without overflowing the
        // intermediate product: (amount/divisor)*multiplier +
        // ((amount%divisor)*multiplier)/divisor. Mathematically identical to
        // (amount * multiplier) / divisor, but the split keeps each product in
        // range whenever the final result fits (multiplier <= divisor here).
        let multiplier: i128 = multiplier.into();
        let quotient = amount / divisor;
        let remainder = amount % divisor;

        let high = quotient
            .checked_mul(multiplier)
            .ok_or(EscrowError::Overflow)?;
        let low = remainder
            .checked_mul(multiplier)
            .ok_or(EscrowError::Overflow)?
            / divisor;

        high.checked_add(low).ok_or(EscrowError::Overflow)
    }
}


/// Computes `a * b / divisor` for non-negative values using a 256-bit
/// intermediate product, so the multiplication can never overflow even when
/// `a * b` would exceed `i128`. Unlike `safe_mul_div`, both factors here may be
/// full-range `i128` values (the bps split trick only works when the multiplier
/// is small relative to the divisor).
pub fn mul_div_wide(e: &Env, a: i128, b: i128, divisor: i128) -> Result<i128, EscrowError> {
    if divisor <= 0 {
        return Err(EscrowError::DivisionError);
    }
    if a < 0 || b < 0 {
        return Err(EscrowError::Underflow);
    }

    let product = U256::from_u128(e, a as u128).mul(&U256::from_u128(e, b as u128));
    let quotient = product.div(&U256::from_u128(e, divisor as u128));

    let value = quotient.to_u128().ok_or(EscrowError::Overflow)?;
    if value > i128::MAX as u128 {
        return Err(EscrowError::Overflow);
    }
    Ok(value as i128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_naive_for_normal_values() {
        // 30 bps and 300 bps of 100_000_000
        assert_eq!(SafeMath::safe_mul_div(100_000_000, 30, 10000).unwrap(), 300_000);
        assert_eq!(SafeMath::safe_mul_div(100_000_000, 300, 10000).unwrap(), 3_000_000);
        // floor behavior preserved
        assert_eq!(SafeMath::safe_mul_div(100_003, 300, 10000).unwrap(), 3000);
    }

    #[test]
    fn no_overflow_for_huge_amount() {
        // amount * multiplier would overflow i128 with the naive approach, but
        // the result fits (multiplier <= divisor), so this must succeed.
        let huge = i128::MAX;
        let result = SafeMath::safe_mul_div(huge, 30, 10000).unwrap();
        assert_eq!(result, huge / 10000 * 30 + (huge % 10000) * 30 / 10000);
    }

    #[test]
    fn zero_divisor_errors() {
        assert_eq!(
            SafeMath::safe_mul_div(100, 30, 0),
            Err(EscrowError::DivisionError)
        );
    }

    #[test]
    fn mul_div_wide_matches_naive_for_normal_values() {
        let e = Env::default();
        // Values small enough that the naive i128 product is fine: results must match.
        assert_eq!(mul_div_wide(&e, 50_000, 97_000, 100_000).unwrap(), 48_500);
        assert_eq!(mul_div_wide(&e, 1, 100, 3).unwrap(), 33); // floor preserved
        assert_eq!(mul_div_wide(&e, 0, 12_345, 100).unwrap(), 0);
    }

    #[test]
    fn mul_div_wide_no_overflow_on_18_decimal_scale() {
        let e = Env::default();
        // ~20 tokens at 18 decimals: a * b overflows i128 with the naive product,
        // but the true result fits because b <= divisor.
        let a: i128 = 20_000_000_000_000_000_000;
        let total: i128 = 20_000_000_000_000_000_000;
        let distributable: i128 = total - 42; // total minus fees
        assert!(a.checked_mul(distributable).is_none(), "precondition: naive product must overflow");

        let net = mul_div_wide(&e, a, distributable, total).unwrap();
        assert_eq!(net, distributable);
    }

    #[test]
    fn mul_div_wide_rejects_bad_inputs() {
        let e = Env::default();
        assert_eq!(mul_div_wide(&e, 10, 10, 0), Err(EscrowError::DivisionError));
        assert_eq!(mul_div_wide(&e, -1, 10, 5), Err(EscrowError::Underflow));
    }
}
