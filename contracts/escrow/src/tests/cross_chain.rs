extern crate std;

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, vec, Address, Bytes, Env,
};

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};
use soroban_sdk::String as SorobanString;

use super::helpers::{create_escrow_contract, create_usdc_token};

/// Mock TokenMessenger contract for testing
/// Accepts deposit_for_burn calls and records the parameters
#[contract]
pub struct MockTokenMessenger;

#[contractimpl]
impl MockTokenMessenger {
    pub fn deposit_for_burn(
        _env: Env,
        _amount: i128,
        _destination_domain: u32,
        _mint_recipient: Bytes,
        _token_address: Address,
    ) {
        // In tests, we just verify this function can be called
        // The actual CCTP burn would happen on real testnet
    }
}

#[test]
fn test_cross_chain_invalid_domain_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: SorobanString::from_str(&env, "Invalid domain"),
            status: SorobanString::from_str(&env, "Pending"),
            evidence: SorobanString::from_str(&env, ""),
            approved: false,
            cross_chain_destination_domain: Some(999),
            cross_chain_recipient: Some(Bytes::from_array(&env, &[1u8; 32])),
        },
    ];

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: SorobanString::from_str(&env, "invalid_domain"),
        title: SorobanString::from_str(&env, "Test"),
        description: SorobanString::from_str(&env, "Test"),
        roles,
        amount: 100_000_000,
        platform_fee: 0,
        milestones,
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    let result = client.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err(), "Should reject invalid CCTP domain");
}

#[test]
fn test_cross_chain_zero_recipient_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: SorobanString::from_str(&env, "Zero recipient"),
            status: SorobanString::from_str(&env, "Pending"),
            evidence: SorobanString::from_str(&env, ""),
            approved: false,
            cross_chain_destination_domain: Some(6),
            cross_chain_recipient: Some(Bytes::from_array(&env, &[0u8; 32])),
        },
    ];

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: SorobanString::from_str(&env, "zero_recipient"),
        title: SorobanString::from_str(&env, "Test"),
        description: SorobanString::from_str(&env, "Test"),
        roles,
        amount: 100_000_000,
        platform_fee: 0,
        milestones,
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    let result = client.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err(), "Should reject zero-byte CCTP recipient");
}

#[test]
fn test_cross_chain_missing_recipient_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: SorobanString::from_str(&env, "Missing recipient"),
            status: SorobanString::from_str(&env, "Pending"),
            evidence: SorobanString::from_str(&env, ""),
            approved: false,
            cross_chain_destination_domain: Some(6),
            cross_chain_recipient: None,
        },
    ];

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: SorobanString::from_str(&env, "missing_recipient"),
        title: SorobanString::from_str(&env, "Test"),
        description: SorobanString::from_str(&env, "Test"),
        roles,
        amount: 100_000_000,
        platform_fee: 0,
        milestones,
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    let result = client.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err(), "Should reject missing CCTP recipient");
}

#[test]
fn test_cross_chain_backwards_compatible() {
    // When cross_chain fields are None, release must behave identically to before
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let trustless_work = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver, &amount);

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: SorobanString::from_str(&env, "Backwards compatible"),
            status: SorobanString::from_str(&env, "Completed"),
            evidence: SorobanString::from_str(&env, "Evidence"),
            approved: false,
            cross_chain_destination_domain: None,
            cross_chain_recipient: None,
        },
    ];

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: SorobanString::from_str(&env, "backwards_compat"),
        title: SorobanString::from_str(&env, "Test"),
        description: SorobanString::from_str(&env, "Test backwards compatibility"),
        roles,
        amount,
        platform_fee,
        milestones,
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&client.address, &amount);

    client.approve_milestone(&0, &approver);
    client.release_funds(&release_signer, &trustless_work);

    let tw_fee = (amount * 30) / 10000;
    let platform_commission = (amount * platform_fee as i128) / 10000;
    let receiver_amount = amount - tw_fee - platform_commission;

    assert_eq!(usdc_token.0.balance(&trustless_work), tw_fee);
    assert_eq!(usdc_token.0.balance(&platform), platform_commission);
    assert_eq!(usdc_token.0.balance(&receiver), receiver_amount);
    assert_eq!(usdc_token.0.balance(&client.address), 0);
}


