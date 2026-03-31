use crate::error::EscrowError;

pub struct SafeMath;

pub trait SafeArithmetic {
    fn safe_mul_div(amount: i128, multiplier: u32, divisor: i128) -> Result<i128, EscrowError>;
}

impl SafeArithmetic for SafeMath {
    fn safe_mul_div(amount: i128, multiplier: u32, divisor: i128) -> Result<i128, EscrowError> {
        amount
            .checked_mul(multiplier.into())
            .ok_or(EscrowError::Overflow)?
            .checked_div(divisor)
            .ok_or(EscrowError::DivisionError)
    }
}
