use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, Vec};

use super::StandardFeeResult;
use crate::error::EscrowError;
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
) -> Result<Vec<(Address, i128)>, EscrowError> {
    if total <= 0 {
        return Err(EscrowError::TotalAmountCannotBeZero);
    }

    // Use pre-computed global fees to guarantee exact fee payment regardless of
    // how many recipients there are (per-recipient flooring would under-collect fees).
    let global_trustless_fee = fee_result.trustless_work_fee;
    let global_platform_fee = fee_result.platform_fee;
    let total_fees = BasicMath::safe_add(global_trustless_fee, global_platform_fee)?;
    let distributable = BasicMath::safe_sub(total, total_fees)?;

    // Distribute the net amount pro-rata to each recipient.
    let mut net_distributions: Vec<(Address, i128)> = Vec::new(e);
    for (addr, amount) in distributions.iter() {
        if amount > 0 {
            let net_amount =
                BasicMath::safe_div(BasicMath::safe_mul(distributable, amount)?, total)?;
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
        let total_out = BasicMath::safe_add(total_fees, total_net)?;
        let remainder = BasicMath::safe_sub(total, total_out)?;
        if remainder > 0 {
            let last_idx = net_distributions.len() - 1;
            let (last_addr, last_amount) = net_distributions.get(last_idx).unwrap();
            net_distributions.set(
                last_idx,
                (last_addr, BasicMath::safe_add(last_amount, remainder)?),
            );
        }
    }

    if global_trustless_fee > 0 {
        token_client.transfer(
            contract_address,
            trustless_work_address,
            &global_trustless_fee,
        );
    }
    if global_platform_fee > 0 {
        token_client.transfer(contract_address, platform_address, &global_platform_fee);
    }

    for (addr, net_amount) in net_distributions.iter() {
        token_client.transfer(contract_address, &addr, &net_amount);
    }

    Ok(net_distributions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{token, Address, Env};

    #[test]
    fn rejects_zero_total() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let sac = e.register_stellar_asset_contract_v2(admin);
        let token_client = token::Client::new(&e, &sac.address());

        let contract = Address::generate(&e);
        let tw = Address::generate(&e);
        let platform = Address::generate(&e);
        let fee_result = StandardFeeResult {
            trustless_work_fee: 0,
            platform_fee: 0,
            receiver_amount: 0,
        };
        let distributions: Map<Address, i128> = Map::new(&e);

        let result = calculate_and_distribute_fees(
            &e,
            &token_client,
            &contract,
            &tw,
            &platform,
            &fee_result,
            &distributions,
            0,
        );
        assert_eq!(result.err(), Some(EscrowError::TotalAmountCannotBeZero));
    }
}
