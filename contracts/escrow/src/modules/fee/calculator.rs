use crate::{
    error::EscrowError,
    modules::{
        math::{BasicArithmetic, BasicMath},
        math::{SafeArithmetic, SafeMath},
    },
};

const TRUSTLESS_WORK_FEE_BPS: u32 = 30;
const BASIS_POINTS_DENOMINATOR: i128 = 10000;

#[derive(Debug, Clone)]
pub struct StandardFeeResult {
    pub trustless_work_fee: i128,
    pub platform_fee: i128,
    pub receiver_amount: i128,
}

impl StandardFeeResult {
    /// The amount the fees were taken from.
    pub fn gross_amount(&self) -> i128 {
        self.trustless_work_fee + self.platform_fee + self.receiver_amount
    }
}

pub trait FeeCalculatorTrait {
    fn calculate_standard_fees(
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<StandardFeeResult, EscrowError>;
}

#[derive(Clone)]
pub struct FeeCalculator;

impl FeeCalculatorTrait for FeeCalculator {
    fn calculate_standard_fees(
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<StandardFeeResult, EscrowError> {
        let trustless_work_fee = SafeMath::safe_mul_div(
            total_amount,
            TRUSTLESS_WORK_FEE_BPS,
            BASIS_POINTS_DENOMINATOR,
        )?;
        let platform_fee =
            SafeMath::safe_mul_div(total_amount, platform_fee_bps, BASIS_POINTS_DENOMINATOR)?;

        let after_tw = BasicMath::safe_sub(total_amount, trustless_work_fee)?;
        let receiver_amount = BasicMath::safe_sub(after_tw, platform_fee)?;

        Ok(StandardFeeResult {
            trustless_work_fee,
            platform_fee,
            receiver_amount,
        })
    }
}
