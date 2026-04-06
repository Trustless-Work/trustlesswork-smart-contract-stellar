use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, Vec};

use crate::error::EscrowError;
use super::StandardFeeResult;
use crate::modules::math::{BasicArithmetic, BasicMath};

pub fn calculate_and_distribute_fees(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    trustless_work_address: &Address,
    platform_address: &Address,
    fee_result: &StandardFeeResult,
    distributions: &Map<Address, i128>,
    total: i128,
) -> Result<(), EscrowError> {
    let mut actual_trustless_fees = 0i128;
    let mut actual_platform_fees = 0i128;
    let mut net_distributions: Vec<(Address, i128)> = Vec::new(e);

    for (addr, amount) in distributions.iter() {
        if amount > 0 {
            let recipient_trustless_fee = BasicMath::safe_div(
                BasicMath::safe_mul(amount, fee_result.trustless_work_fee)?,
                total,
            )?;
            let recipient_platform_fee = BasicMath::safe_div(
                BasicMath::safe_mul(amount, fee_result.platform_fee)?,
                total,
            )?;

            let total_recipient_fee =
                BasicMath::safe_add(recipient_trustless_fee, recipient_platform_fee)?;
            let net_amount = BasicMath::safe_sub(amount, total_recipient_fee)?;

            actual_trustless_fees =
                BasicMath::safe_add(actual_trustless_fees, recipient_trustless_fee)?;
            actual_platform_fees =
                BasicMath::safe_add(actual_platform_fees, recipient_platform_fee)?;

            if net_amount > 0 {
                net_distributions.push_back((addr.clone(), net_amount));
            }
        }
    }

    // Assign any remainder from integer-division truncation to the last recipient
    // to guarantee sum(all transfers) == total and no funds are stranded.
    if !net_distributions.is_empty() {
        let mut total_net: i128 = 0;
        for (_, net_amount) in net_distributions.iter() {
            total_net = BasicMath::safe_add(total_net, net_amount)?;
        }
        let total_fees = BasicMath::safe_add(actual_trustless_fees, actual_platform_fees)?;
        let total_out = BasicMath::safe_add(total_fees, total_net)?;
        let remainder = BasicMath::safe_sub(total, total_out)?;
        if remainder > 0 {
            let last_idx = net_distributions.len() - 1;
            let (last_addr, last_amount) = net_distributions.get(last_idx).unwrap();
            net_distributions.set(last_idx, (last_addr, BasicMath::safe_add(last_amount, remainder)?));
        }
    }

    if actual_trustless_fees > 0 {
        token_client.transfer(contract_address, trustless_work_address, &actual_trustless_fees);
    }
    if actual_platform_fees > 0 {
        token_client.transfer(contract_address, platform_address, &actual_platform_fees);
    }

    for (addr, net_amount) in net_distributions.iter() {
        token_client.transfer(contract_address, &addr, &net_amount);
    }

    Ok(())
}
