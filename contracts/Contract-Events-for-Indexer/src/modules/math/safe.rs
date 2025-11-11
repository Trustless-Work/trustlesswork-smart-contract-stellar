use crate::error::ContractError;

pub struct SafeMath;

pub trait SafeArithmetic {
    fn safe_mul_div(amount: i128, multiplier: i128, divisor: i128) -> Result<i128, ContractError>;
}

impl SafeArithmetic for SafeMath {
    fn safe_mul_div(amount: i128, multiplier: i128, divisor: i128) -> Result<i128, ContractError> {
        amount
            .checked_mul(multiplier)
            .ok_or(ContractError::Overflow)?
            .checked_div(divisor)
            .ok_or(ContractError::DivisionError)
    }
}
