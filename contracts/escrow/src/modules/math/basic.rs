use crate::error::ContractError;

pub struct BasicMath;

pub trait BasicArithmetic {
    fn safe_add(a: i128, b: i128) -> Result<i128, ContractError>;
    fn safe_sub(a: i128, b: i128) -> Result<i128, ContractError>;
    fn safe_mul(a: i128, b: i128) -> Result<i128, ContractError>;
    fn safe_div(a: i128, b: i128) -> Result<i128, ContractError>;
}

impl BasicArithmetic for BasicMath {
    fn safe_add(a: i128, b: i128) -> Result<i128, ContractError> {
        a.checked_add(b).ok_or(ContractError::Overflow)
    }

    fn safe_sub(a: i128, b: i128) -> Result<i128, ContractError> {
        a.checked_sub(b).ok_or(ContractError::Underflow)
    }

    fn safe_mul(a: i128, b: i128) -> Result<i128, ContractError> {
        a.checked_mul(b).ok_or(ContractError::Overflow)
    }

    fn safe_div(a: i128, b: i128) -> Result<i128, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionError);
        }
        a.checked_div(b).ok_or(ContractError::Overflow)
    }
}
