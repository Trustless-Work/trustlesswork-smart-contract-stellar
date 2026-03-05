extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_dispute_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "test_dispute");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            amount,
            evidence: String::from_str(&env, "Initial evidence"),
            flags,
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert!(!escrow.milestones.get(0).unwrap().flags.disputed);

    escrow_approver.dispute_milestone(&0, &approver_address);

    let escrow_after_change = escrow_approver.get_escrow();
    assert!(
        escrow_after_change
            .milestones
            .get(0)
            .unwrap()
            .flags
            .disputed
    );

    usdc_token.1.mint(&approver_address, &(amount as i128));
    // Test block on distributing earnings during dispute
    let result =
        escrow_approver.try_release_milestone_funds(&release_signer_address, &platform, &0);
    assert!(result.is_err());

    let _ = escrow_approver.try_dispute_milestone(&0, &approver_address);

    let escrow_after_second_change = escrow_approver.get_escrow();
    assert!(
        escrow_after_second_change
            .milestones
            .get(0)
            .unwrap()
            .flags
            .disputed
    );
}

#[test]
fn test_dispute_resolution_process() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&admin, &(amount as i128));
    usdc_token
        .0
        .transfer(&admin, &escrow_approver.address, &(amount as i128));

    // Verify initial state
    let escrow_balance = usdc_token.0.balance(&escrow_approver.address);
    assert_eq!(escrow_balance, amount as i128);

    // Change milestone dispute flag
    escrow_approver.dispute_milestone(&0, &approver_address);

    // Verify milestone dispute flag changed
    let disputed_escrow = escrow_approver.get_escrow();
    let disputed_milestone = disputed_escrow.milestones.get(0).unwrap();
    assert_eq!(disputed_milestone.flags.disputed, true);

    // Resolve dispute
    let approver_amount: i128 = 40_000_000;
    let provider_amount: i128 = 60_000_000;
    let total_amount = approver_amount + provider_amount;

    let mut dist = Map::new(&env);
    dist.set(approver_address.clone(), approver_amount);
    dist.set(service_provider_address.clone(), provider_amount);
    escrow_approver.resolve_milestone_dispute(
        &dispute_resolver_address,
        &0, // milestone_index
        &trustless_work_address,
        &dist,
    );

    let expected_tw_fee = (total_amount * 30) / 10000; // 0.3%
    let expected_platform_fee = (total_amount * platform_fee as i128) / 10000;

    let expected_approver = approver_amount
        - (approver_amount * (expected_tw_fee + expected_platform_fee)) / total_amount;
    let expected_provider = provider_amount
        - (provider_amount * (expected_tw_fee + expected_platform_fee)) / total_amount;

    assert_eq!(usdc_token.0.balance(&escrow_approver.address), 0);
    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        expected_tw_fee
    );
    assert_eq!(
        usdc_token.0.balance(&platform),
        expected_platform_fee
    );
    assert_eq!(usdc_token.0.balance(&approver_address), expected_approver);
    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        expected_provider
    );

    let final_escrow = escrow_approver.get_escrow();
    let resolved_milestone = final_escrow.milestones.get(0).unwrap();
    assert_eq!(
        resolved_milestone.status,
        String::from_str(&env, "resolved")
    );
}

#[test]
fn test_cannot_release_after_dispute_resolved() {
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

    // Setup escrow with one milestone
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
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
            flags: flags.clone(),
            amount,
            evidence: String::from_str(&env, "e"),
            receiver: service_provider.clone(),
        },
    ];
    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones,
        trustline,
        receiver_memo: 0,
    };
    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Fund and open dispute then resolve
    usdc.1.mint(&client.address, &amount);
    client.dispute_milestone(&0, &approver);
    let mut dist = Map::new(&env);
    dist.set(approver.clone(), 40_000_000);
    dist.set(service_provider.clone(), 60_000_000);
    client.resolve_milestone_dispute(&dispute_resolver, &0, &trustless_work_address, &dist);

    // Try to release after resolved - should fail
    let bal_before = usdc.0.balance(&client.address);
    let res = client.try_release_milestone_funds(&release_signer, &platform, &0);
    assert!(
        res.is_err(),
        "Should not allow release after dispute-resolved"
    );
    assert_eq!(
        usdc.0.balance(&client.address),
        bal_before,
        "No funds should move on failed precondition"
    );
}

#[test]
fn test_cannot_dispute_resolve_after_released() {
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

    // Setup escrow with one milestone
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
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
            flags: flags.clone(),
            amount,
            evidence: String::from_str(&env, "e"),
            receiver: service_provider.clone(),
        },
    ];
    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee,
        milestones,
        trustline,
        receiver_memo: 0,
    };
    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Fund and mark approved then release
    usdc.1.mint(&client.address, &amount);
    client.approve_milestone(&0, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    // Try to dispute-resolve after released - should fail
    let bal_before = usdc.0.balance(&client.address);
    let mut dist = Map::new(&env);
    dist.set(approver.clone(), 40_000_000);
    dist.set(service_provider.clone(), 60_000_000);
    let res =
        client.try_resolve_milestone_dispute(&dispute_resolver, &0, &trustless_work_address, &dist);
    assert!(
        res.is_err(),
        "Should not allow dispute-resolution after release"
    );
    assert_eq!(
        usdc.0.balance(&client.address),
        bal_before,
        "No funds should move on failed precondition"
    );
}

#[test]
fn test_dispute_milestone() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    escrow_approver.dispute_milestone(&0, &approver_address);

    let escrow = escrow_approver.get_escrow();
    let milestone = escrow.milestones.get(0).unwrap();
    assert!(
        milestone.flags.disputed,
        "First milestone dispute flag should be true"
    );

    let milestone2 = escrow.milestones.get(1).unwrap();
    assert!(
        !milestone2.flags.disputed,
        "Second milestone dispute flag should remain false"
    );

    let result = escrow_approver.try_dispute_milestone(&5, &approver_address);
    assert!(result.is_err(), "Should fail with invalid milestone index");

    let result = escrow_approver.try_dispute_milestone(&0, &approver_address);
    assert!(
        result.is_err(),
        "Should fail when milestone is already in dispute"
    );
}

#[test]
fn test_change_dispute_flag_authorized_and_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform,
        release_signer,
        dispute_resolver,
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: 0,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client_1 = test_data.client;

    escrow_client_1.initialize_escrow(&escrow_properties);

    escrow_client_1.dispute_milestone(&0, &approver);

    let updated_escrow = escrow_client_1.get_escrow();
    assert!(
        updated_escrow.milestones.get(0).unwrap().flags.disputed,
        "Dispute flag should be set to true for authorized address"
    );

    let test_data_2 = create_escrow_contract(&env);
    let escrow_client_2 = test_data_2.client;

    escrow_client_2.initialize_escrow(&escrow_properties);

    let result = escrow_client_2.try_dispute_milestone(&0, &unauthorized);

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to change dispute flag"
    );
}

#[test]
fn test_resolve_dispute_rounding_edge_case() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    // Use values where floor division rounding causes a mismatch:
    // total = 100_003, TW fee (30 bps) = floor(100_003 * 30 / 10000) = 300,
    // platform fee (300 bps) = floor(100_003 * 300 / 10000) = 3000,
    // total_fees = 3300.
    // Per-recipient floor shares: floor(50_001 * 3300 / 100_003) = 1649, floor(50_002 * 3300 / 100_003) = 1650
    // sum(fee_shares) = 3299 < 3300, so the old code would over-distribute by 1.
    let total: i128 = 100_003;
    let platform_fee: u32 = 300; // 3%

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };

    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            amount: total,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "rounding_resolve"),
        title: String::from_str(&env, "Rounding Test"),
        description: String::from_str(&env, "Test floor division rounding in resolve_dispute"),
        roles,
        platform_fee,
        milestones,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    // Fund the escrow with exactly total
    usdc_token.1.mint(&client.address, &total);

    // Put milestone in dispute
    client.dispute_milestone(&0, &approver);

    // Distributions that trigger the rounding mismatch
    let mut distributions = Map::new(&env);
    distributions.set(approver.clone(), 50_001);
    distributions.set(service_provider.clone(), 50_002);

    // This must NOT revert (old code would fail here due to insufficient balance)
    let result = client.try_resolve_milestone_dispute(
        &dispute_resolver,
        &0,
        &trustless_work_address,
        &distributions,
    );
    assert!(result.is_ok(), "resolve_milestone_dispute should handle fee rounding correctly");

    // Verify contract has no negative balance and all funds were distributed
    let final_balance = usdc_token.0.balance(&client.address);
    assert!(final_balance >= 0, "Contract balance must be non-negative");

    // Verify the total outflows equal exactly the initial balance
    let tw_balance = usdc_token.0.balance(&trustless_work_address);
    let platform_balance = usdc_token.0.balance(&platform);
    let approver_balance = usdc_token.0.balance(&approver);
    let sp_balance = usdc_token.0.balance(&service_provider);

    let total_outflow = tw_balance + platform_balance + approver_balance + sp_balance;
    assert_eq!(
        total_outflow + final_balance,
        total,
        "Sum of all outflows plus remaining balance must equal the original total"
    );

    // Verify dispute was resolved
    let escrow = client.get_escrow();
    let milestone = escrow.milestones.get(0).unwrap();
    assert!(milestone.flags.resolved);
    assert!(!milestone.flags.disputed);
}

#[test]
fn test_withdraw_remaining_funds_rounding_edge_case() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestone_amount: i128 = 1_000_000;
    let platform_fee: u32 = 300; // 3%

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
    };

    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            amount: milestone_amount,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "rounding_withdraw"),
        title: String::from_str(&env, "Rounding Withdraw Test"),
        description: String::from_str(&env, "Test floor division rounding in withdraw"),
        roles,
        platform_fee,
        milestones: milestones.clone(),
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    // Fund and go through the full release flow so withdraw_remaining_funds is allowed
    usdc_token.1.mint(&client.address, &milestone_amount);

    let milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "Completed"),
            evidence: Some(String::from_str(&env, "Done")),
        },
    ];
    client.change_milestone_status(&milestone_updates, &service_provider);

    client.approve_milestone(&0, &approver);

    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    // Simulate remaining funds (e.g. from overfunding or rounding leftovers)
    let remaining: i128 = 100_003;
    usdc_token.1.mint(&client.address, &remaining);

    let balance_before = usdc_token.0.balance(&client.address);

    // Record initial balances
    let tw_before = usdc_token.0.balance(&trustless_work_address);
    let platform_before = usdc_token.0.balance(&platform);
    let a_before = usdc_token.0.balance(&recipient_a);
    let b_before = usdc_token.0.balance(&recipient_b);

    // Distributions that trigger rounding mismatch
    let mut distributions = Map::new(&env);
    distributions.set(recipient_a.clone(), 50_001);
    distributions.set(recipient_b.clone(), 50_002);

    let result = client.try_withdraw_remaining_funds(
        &dispute_resolver,
        &trustless_work_address,
        &distributions,
    );
    assert!(result.is_ok(), "withdraw_remaining_funds should handle fee rounding correctly");

    // Verify the contract didn't underflow
    let final_balance = usdc_token.0.balance(&client.address);
    assert!(final_balance >= 0, "Contract balance must be non-negative");

    // Verify total outflows from the withdraw operation
    let tw_delta = usdc_token.0.balance(&trustless_work_address) - tw_before;
    let platform_delta = usdc_token.0.balance(&platform) - platform_before;
    let a_delta = usdc_token.0.balance(&recipient_a) - a_before;
    let b_delta = usdc_token.0.balance(&recipient_b) - b_before;

    let total_withdrawn = tw_delta + platform_delta + a_delta + b_delta;
    let balance_used = balance_before - final_balance;
    assert_eq!(
        total_withdrawn, balance_used,
        "Total withdrawn must equal the contract balance decrease"
    );
}
