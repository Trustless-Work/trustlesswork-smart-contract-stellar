use crate::{
    error::ContractError,
    modules::{
        math::{BasicArithmetic, BasicMath},
        math::{SafeArithmetic, SafeMath},
    },
};

const TRUSTLESS_WORK_FEE_BPS: u32 = 30;
const BASIS_POINTS_DENOMINATOR: i128 = 10000;

pub trait FeeCalculatorTrait {
    fn calculate_total_fees(
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<(i128, i128, i128), ContractError>;
    fn calculate_net_share(
        share_amount: i128,
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<(i128, i128, i128), ContractError>;
}

#[derive(Clone)]
pub struct FeeCalculator;

impl FeeCalculatorTrait for FeeCalculator {
    #[inline]
    fn calculate_total_fees(
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<(i128, i128, i128), ContractError> {
        let trustless_work_fee = SafeMath::safe_mul_div(
            total_amount,
            TRUSTLESS_WORK_FEE_BPS,
            BASIS_POINTS_DENOMINATOR,
        )?;
        let platform_fee =
            SafeMath::safe_mul_div(total_amount, platform_fee_bps, BASIS_POINTS_DENOMINATOR)?;
        let total_fees = BasicMath::safe_add(trustless_work_fee, platform_fee)?;
        Ok((trustless_work_fee, platform_fee, total_fees))
    }

    #[inline]
    fn calculate_net_share(
        share_amount: i128,
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<(i128, i128, i128), ContractError> {
        if share_amount <= 0 || total_amount <= 0 || share_amount > total_amount {
            return Err(ContractError::AmountsToBeTransferredShouldBePositive);
        }
        let (trustless_work_fee, platform_fee, total_fees) =
            Self::calculate_total_fees(total_amount, platform_fee_bps)?;
        let share_total_fees = SafeMath::safe_mul_div(share_amount, total_fees as u32, total_amount)?;
        let trustless_share_fee = if total_fees == 0 {
            0
        } else {
            SafeMath::safe_mul_div(share_total_fees, trustless_work_fee as u32, total_fees)?
        };
        let platform_share_fee = if total_fees == 0 {
            0
        } else {
            SafeMath::safe_mul_div(share_total_fees, platform_fee as u32, total_fees)?
        };

        let net_share = BasicMath::safe_sub(share_amount, share_total_fees)?;
        Ok((net_share, trustless_share_fee, platform_share_fee))
    }
}
