extern crate std;

use crate::contract::EscrowContract;
use crate::modules::cctp::decimal::{cctp_remainder, truncate_to_6_decimals};
use crate::modules::cctp::release::release_receiver_amount_via_cctp_with_messenger;
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::storage::types::{CrossChainReceiver, Escrow, Flags, Milestone, Roles, Trustline , CrossChainReceiverOption};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token, vec, Address, BytesN, Env, String,
};
use token::Client as TokenClient;

use super::helpers::{create_escrow_contract, create_usdc_token};

#[contract]
pub struct MockTokenMessenger;

#[contractimpl]
impl MockTokenMessenger {
    pub fn deposit_for_burn(
        e: Env,
        caller: Address,
        amount: i128,
        _destination_domain: u32,
        _mint_recipient: BytesN<32>,
        burn_token: Address,
        _destination_caller: BytesN<32>,
        _max_fee: i128,
        _min_finality_threshold: u32,
    ) {
        caller.require_auth();
        let token_client = TokenClient::new(&e, &burn_token);
        token_client.transfer(&caller, &e.current_contract_address(), &amount);
    }
}

fn base_escrow(env: &Env, usdc: &TokenClient, amount: i128) -> Escrow {
    let approver = Address::generate(env);
    let service_provider = Address::generate(env);
    let platform = Address::generate(env);
    let release_signer = Address::generate(env);
    let dispute_resolver = Address::generate(env);
    let receiver = Address::generate(env);

    Escrow {
        engagement_id: String::from_str(env, "cctp_test"),
        title: String::from_str(env, "CCTP Test"),
        description: String::from_str(env, "Cross-chain release test"),
        roles: Roles {
            approver,
            service_provider,
            platform,
            release_signer,
            dispute_resolver,
            receiver,
        },
        amount,
        platform_fee: 300,
        milestones: vec![
            env,
            Milestone {
                description: String::from_str(env, "M1"),
                status: String::from_str(env, "Done"),
                evidence: String::from_str(env, "Evidence"),
                approved: false,
            },
        ],
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc.address.clone(),
        },
        receiver_memo: 0,
        cross_chain_receiver: CrossChainReceiverOption::None,
    }
}

fn evm_recipient(env: &Env, byte: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[31] = byte;
    BytesN::from_array(env, &bytes)
}

fn validate_cross_chain_receiver(
    receiver: &Option<CrossChainReceiver>,
) -> Result<(), crate::error::ContractError> {
    use crate::error::ContractError;
    use crate::modules::cctp::constants::is_valid_cctp_destination_domain;

    if receiver.is_none() {
        return Ok(());
    }

    let receiver = receiver.as_ref().unwrap();

    if !is_valid_cctp_destination_domain(receiver.destination_domain) {
        return Err(ContractError::InvalidCctpDestinationDomain);
    }

    if receiver.recipient.to_array() == [0u8; 32] {
        return Err(ContractError::InvalidCctpRecipient);
    }

    Ok(())
}

#[test]
fn test_validate_rejects_invalid_cctp_domain() {
    let env = Env::default();
    let receiver = Some(CrossChainReceiver {
        destination_domain: 999,
        recipient: evm_recipient(&env, 0x01),
    });
    let result = validate_cross_chain_receiver(&receiver);
    assert_eq!(
        result,
        Err(crate::error::ContractError::InvalidCctpDestinationDomain)
    );
}

#[test]
fn test_validate_rejects_zero_cctp_recipient() {
    let env = Env::default();
    let receiver = Some(CrossChainReceiver {
        destination_domain: 6,
        recipient: BytesN::from_array(&env, &[0u8; 32]),
    });
    let result = validate_cross_chain_receiver(&receiver);
    assert_eq!(
        result,
        Err(crate::error::ContractError::InvalidCctpRecipient)
    );
}

#[test]
fn test_initialize_rejects_invalid_cross_chain_domain() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);
    let mut escrow = base_escrow(&env, &usdc.0, 1_0000000);
escrow.cross_chain_receiver = CrossChainReceiverOption::Some(
    CrossChainReceiver {
        destination_domain: 999,
        recipient: evm_recipient(&env, 0x01),
    }
);

    let client = create_escrow_contract(&env).client;
    let result = client.try_initialize_escrow(&escrow);
    assert!(result.is_err());
}

#[test]
fn test_decimal_truncation_for_cross_chain_amount() {
    assert_eq!(truncate_to_6_decimals(1_0000003), 1_0000000);
    assert_eq!(cctp_remainder(1_0000003), 3);
}

#[test]
fn test_cross_chain_release_burns_and_sends_remainder() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);
    let amount: i128 = 1_0000003;
    usdc.1.mint(&admin, &amount);

    let escrow_contract = env.register(EscrowContract, ());
    let contract_address = escrow_contract.clone();
    usdc.1.mint(&contract_address, &amount);

    let mock_messenger = env.register(MockTokenMessenger, ());
    let stellar_receiver = Address::generate(&env);

    let cross_chain = CrossChainReceiver {
        destination_domain: 6,
        recipient: evm_recipient(&env, 0xAB),
    };

    release_receiver_amount_via_cctp_with_messenger(
        &env,
        &usdc.0,
        &contract_address,
        &mock_messenger,
        &usdc.0.address,
        amount,
        &cross_chain,
        &stellar_receiver,
    )
    .unwrap();

    assert_eq!(usdc.0.balance(&mock_messenger), 1_0000000);
    assert_eq!(usdc.0.balance(&stellar_receiver), 3);
    assert_eq!(usdc.0.balance(&contract_address), 0);
}

#[test]
fn test_cross_chain_release_fee_deductions_match_stellar_path() {
    let amount: i128 = 100_000_000;
    let platform_fee_bps: u32 = 500;

    let fees = FeeCalculator::calculate_standard_fees(amount, platform_fee_bps).unwrap();
    let tw_fee = amount * 30 / 10_000;
    let platform_fee = amount * platform_fee_bps as i128 / 10_000;
    let receiver_amount = amount - tw_fee - platform_fee;

    assert_eq!(fees.trustless_work_fee, tw_fee);
    assert_eq!(fees.platform_fee, platform_fee);
    assert_eq!(fees.receiver_amount, receiver_amount);
}

#[test]
fn test_release_funds_stellar_path_unchanged_with_default_cross_chain() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);
    let amount: i128 = 100_000_000;
    let platform_fee_bps: u32 = 500;

    let mut escrow = base_escrow(&env, &usdc.0, amount);
    escrow.platform_fee = platform_fee_bps;
    escrow.roles.release_signer = Address::generate(&env);
    let release_signer = escrow.roles.release_signer.clone();
    let receiver = escrow.roles.receiver.clone();
    let platform = escrow.roles.platform.clone();
    let tw_address = Address::generate(&env);

    usdc.1.mint(&admin, &amount);

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow);
    client.fund_escrow(&admin, &escrow, &amount);

    for i in 0..escrow.milestones.len() {
        client.approve_milestone(&i, &escrow.roles.approver);
    }

    client.release_funds(&release_signer, &tw_address);

    let tw_fee = amount * 30 / 10_000;
    let platform_fee = amount * platform_fee_bps as i128 / 10_000;
    let receiver_amount = amount - tw_fee - platform_fee;

    assert_eq!(usdc.0.balance(&tw_address), tw_fee);
    assert_eq!(usdc.0.balance(&platform), platform_fee);
    assert_eq!(usdc.0.balance(&receiver), receiver_amount);
    assert_eq!(usdc.0.balance(&client.address), 0);
}
