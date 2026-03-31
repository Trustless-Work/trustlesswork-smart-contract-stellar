use crate::error::EscrowError;

pub struct BasicMath;

pub trait BasicArithmetic {
    fn safe_add(a: i128, b: i128) -> Result<i128, EscrowError>;
    fn safe_sub(a: i128, b: i128) -> Result<i128, EscrowError>;
    fn safe_mul(a: i128, b: i128) -> Result<i128, EscrowError>;
    fn safe_div(a: i128, b: i128) -> Result<i128, EscrowError>;
}

impl BasicArithmetic for BasicMath {
    fn safe_add(a: i128, b: i128) -> Result<i128, EscrowError> {
        a.checked_add(b).ok_or(EscrowError::Overflow)
    }

    fn safe_sub(a: i128, b: i128) -> Result<i128, EscrowError> {
        a.checked_sub(b).ok_or(EscrowError::Underflow)
    }

    fn safe_mul(a: i128, b: i128) -> Result<i128, EscrowError> {
        a.checked_mul(b).ok_or(EscrowError::Overflow)
    }

    fn safe_div(a: i128, b: i128) -> Result<i128, EscrowError> {
        if b == 0 {
            return Err(EscrowError::DivisionError);
        }
        a.checked_div(b).ok_or(EscrowError::Overflow)
    }
}
