use crate::error::EscrowError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_naive_for_normal_values() {
        // 30 bps and 300 bps of 100_000_000
        assert_eq!(
            SafeMath::safe_mul_div(100_000_000, 30, 10000).unwrap(),
            300_000
        );
        assert_eq!(
            SafeMath::safe_mul_div(100_000_000, 300, 10000).unwrap(),
            3_000_000
        );
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
}
