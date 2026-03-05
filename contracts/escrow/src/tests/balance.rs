extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_withdraw_remaining_funds_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc = create_usdc_token(&env, &admin);

    let platform_fee = 3 * 100; // 3%
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };

    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };
    let trustline = Trustline {
        address: usdc.0.address.clone(),
    };
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "m2"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Fund contract with 250_000 so after releasing 2x100_000 there are 50_000 remaining
    usdc.1.mint(&client.address, &250_000);

    // Approve and release both milestones
    client.approve_milestone(&0, &approver);
    client.approve_milestone(&1, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &1);

    // Sanity: contract balance should be 50_000 now
    let contract_balance_before = usdc.0.balance(&client.address);
    assert_eq!(contract_balance_before, 50_000);

    // Build distributions below remaining balance so fees also fit:
    // send 10k to TW, 5k to platform, 33k to receiver => total = 48,000
    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(trustless_work_address.clone(), 10_000);
    dist.set(platform.clone(), 5_000);
    dist.set(service_provider.clone(), 33_000);

    // Capture balances before
    let tw_before = usdc.0.balance(&trustless_work_address);
    let platform_before = usdc.0.balance(&platform);
    let receiver_before = usdc.0.balance(&service_provider);

    client.withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);

    // Fees are computed over the total distribution (48,000). Net amounts are distribution - proportional fee share.
    let total_dist = 48_000i128;
    let tw_fee = (total_dist * 30) / 10000; // 0.3% => 144
    let platform_fee_amount = (total_dist * platform_fee as i128) / 10000; // 3% => 1440
    let total_fees = tw_fee + platform_fee_amount; // 1584

    // Proportional fee share per beneficiary
    let fee_share_tw = (10_000 * total_fees) / total_dist; // 330
    let fee_share_platform = (5_000 * total_fees) / total_dist; // 165
    let fee_share_receiver = (33_000 * total_fees) / total_dist; // 1089

    let net_tw = 10_000 - fee_share_tw; // 9,670 + fee payment 144 => balance increase 9,814 vs original model 10,144
    let net_platform = 5_000 - fee_share_platform; // 4,835 + platform fee 1440 => 6,275 total increase
    let net_receiver = 33_000 - fee_share_receiver; // 31,911

    // Contract leftover = 50,000 - total_dist (because fees + nets == total_dist)
    let expected_leftover = 50_000 - total_dist; // 2,000

    assert_eq!(usdc.0.balance(&client.address), expected_leftover);
    assert_eq!(
        usdc.0.balance(&trustless_work_address),
        tw_before + net_tw + tw_fee
    );
    assert_eq!(
        usdc.0.balance(&platform),
        platform_before + net_platform + platform_fee_amount
    );
    assert_eq!(
        usdc.0.balance(&service_provider),
        receiver_before + net_receiver
    );
}

#[test]
fn test_withdraw_remaining_funds_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let platform_fee = 3 * 100;
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };
    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };
    let trustline = Trustline {
        address: usdc.0.address.clone(),
    };
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];
    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };
    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Process the single milestone fully and leave leftover of 10_000
    usdc.1.mint(&client.address, &110_000);
    client.approve_milestone(&0, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    // Attacker provides any distributions but is not resolver
    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(service_provider.clone(), 10_000);
    let res = client.try_withdraw_remaining_funds(&attacker, &trustless_work_address, &dist);
    assert!(res.is_err(), "Only dispute_resolver should be allowed");
}

#[test]
fn test_withdraw_remaining_funds_not_fully_processed() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let platform_fee = 3 * 100;
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };
    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };
    let trustline = Trustline {
        address: usdc.0.address.clone(),
    };
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "m2"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];
    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };
    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    usdc.1.mint(&client.address, &220_000);
    // Process only first milestone; second remains pending
    client.approve_milestone(&0, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    // Try withdraw while second milestone not processed
    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(service_provider.clone(), 10_000);
    let res =
        client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert!(
        res.is_err(),
        "Should fail when not all milestones are processed"
    );
}

#[test]
fn test_withdraw_remaining_funds_zero_balance_ok() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let platform_fee = 3 * 100;
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };
    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };
    let trustline = Trustline {
        address: usdc.0.address.clone(),
    };
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "m2"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];
    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };
    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Fund exactly the total milestones 200_000; after releases, no leftover
    usdc.1.mint(&client.address, &200_000);
    client.approve_milestone(&0, &approver);
    client.approve_milestone(&1, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &1);

    assert_eq!(usdc.0.balance(&client.address), 0);

    // With empty distributions total == 0, we now expect an error (TotalAmountCannotBeZero)
    let dist: Map<Address, i128> = Map::new(&env);
    let res =
        client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert!(
        res.is_err(),
        "Expected error when total distribution amount is zero"
    );
}
