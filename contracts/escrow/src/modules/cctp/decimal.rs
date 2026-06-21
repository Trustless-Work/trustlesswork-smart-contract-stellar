use crate::modules::cctp::constants::STELLAR_TO_CCTP_DECIMAL_FACTOR;

/// Truncates a 7-decimal Stellar USDC amount to 6-decimal CCTP burn units (floor).
#[inline]
pub fn truncate_to_6_decimals(amount_7dec: i128) -> i128 {
    (amount_7dec / STELLAR_TO_CCTP_DECIMAL_FACTOR) * STELLAR_TO_CCTP_DECIMAL_FACTOR
}

/// Returns the 7th-decimal remainder after CCTP truncation.
#[inline]
pub fn cctp_remainder(amount_7dec: i128) -> i128 {
    amount_7dec - truncate_to_6_decimals(amount_7dec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_exact_6_decimals() {
        assert_eq!(truncate_to_6_decimals(1_0000000), 1_0000000);
        assert_eq!(cctp_remainder(1_0000000), 0);
    }

    #[test]
    fn truncate_with_7th_decimal_remainder() {
        assert_eq!(truncate_to_6_decimals(1_0000003), 1_0000000);
        assert_eq!(cctp_remainder(1_0000003), 3);
    }

    #[test]
    fn truncate_zero() {
        assert_eq!(truncate_to_6_decimals(0), 0);
        assert_eq!(cctp_remainder(0), 0);
    }

    #[test]
    fn truncate_sub_stroop_amount() {
        assert_eq!(truncate_to_6_decimals(5), 0);
        assert_eq!(cctp_remainder(5), 5);
    }
}
