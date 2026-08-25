extern crate std;

use crate::error::EscrowError;
use crate::storage::types::{
    DataKey, Dispute, Escrow, Milestone, MilestoneApprovals, MilestoneStatusUpdate, Roles,
    Trustline,
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &amount);

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: _receiver_address.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_fund");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balances
    assert_eq!(usdc_token.0.balance(&approver_address), amount);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), 0);

    let deposit_amount = amount / 2;

    let test_fund = escrow_approver.try_fund_escrow(&approver_address, &escrow_properties, &0);
    assert!(test_fund.is_err());

    escrow_approver.fund_escrow(&approver_address, &escrow_properties, &deposit_amount);

    // Check balances after deposit
    assert_eq!(
        usdc_token.0.balance(&approver_address),
        amount - deposit_amount
    );
    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        deposit_amount
    );

    // Deposit remaining amount
    escrow_approver.fund_escrow(&approver_address, &escrow_properties, &deposit_amount);

    assert_eq!(usdc_token.0.balance(&approver_address), 0);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), amount);
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    // Only mint a small amount to the approver
    let small_amount: i128 = 1_000_000;
    usdc_token.1.mint(&approver_address, &small_amount);

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: _receiver_address.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_insufficient_funds");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balance
    assert_eq!(usdc_token.0.balance(&approver_address), small_amount);

    // Try to deposit more than the approver has (should fail)
    let result = escrow_approver.try_fund_escrow(&approver_address, &escrow_properties, &amount);
    assert!(result.is_err());

    // Verify balances didn't change
    assert_eq!(usdc_token.0.balance(&approver_address), small_amount);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), 0);
}

#[test]
fn test_release_funds_successful_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: _receiver_address.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_1");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.approve_milestones(&vec![&env, 1u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&_receiver_address),
        receiver_amount,
        "Receiver received incorrect amount"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        0,
        "Service Provider should have zero balance when using separate receiver"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

// Scenario: Milestones incomplete
#[test]
fn test_release_funds_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id_incomplete_milestones = String::from_str(&env, "test_incomplete_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let incomplete_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_incomplete_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: incomplete_milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));
    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    // Try to distribute earnings with incomplete milestones (should fail)
    let result =
        escrow_approver.try_release_funds(&release_signer_address, &trustless_work_address);
    assert!(result.is_err());
}

#[test]
fn test_release_funds_same_receiver_as_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    // Use service_provider_address as receiver to test same-address case
    let _receiver_address = service_provider_address.clone();
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: _receiver_address.clone(), // Set to service_provider to test same-address case
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let service_provider_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        service_provider_amount,
        "Service Provider should receive funds when receiver is set to same address"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_release_funds_invalid_receiver_fallback() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    // Create a valid but separate receiver address
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: _receiver_address.clone(), // Different receiver address than service provider
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    // Funds should go to the receiver (not service provider)
    assert_eq!(
        usdc_token.0.balance(&_receiver_address),
        receiver_amount,
        "Receiver should receive funds when set to a different address than service provider"
    );

    // The service provider should not receive funds when a different receiver is set
    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        0,
        "Service provider should not receive funds when a different receiver is set"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_withdraw_remaining_funds_rounding_edge_case() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_amount: i128 = 1_000_000;
    let platform_fee: u32 = 300; // 3%

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: service_provider.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "rounding_withdraw"),
        title: String::from_str(&env, "Rounding Withdraw Test"),
        description: String::from_str(&env, "Test floor division rounding in withdraw"),
        roles,
        amount: escrow_amount,
        platform_fee,
        milestones: milestones.clone(),
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    // Fund the escrow, open a dispute, resolve it, then mint additional funds
    // so that withdraw_remaining_funds is allowed (requires a prior dispute)
    usdc_token.1.mint(&approver, &escrow_amount);
    client.fund_escrow(&approver, &escrow_properties, &escrow_amount);

    client.dispute_escrow(
        &service_provider,
        &String::from_str(&env, "Payment dispute"),
    );

    // Resolve the dispute (sends all funds out), then mint remaining funds for the test
    let mut resolve_distributions = Map::new(&env);
    resolve_distributions.set(service_provider.clone(), escrow_amount);
    client.resolve_dispute(
        &dispute_resolver,
        &trustless_work_address,
        &resolve_distributions,
    );

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
    assert!(
        result.is_ok(),
        "withdraw_remaining_funds should handle fee rounding correctly"
    );

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

#[test]
fn test_full_flow_init_without_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let trustless_work = Address::generate(&env);
    let amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&approver, &amount);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: receiver.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    // 1. Init sin milestones
    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "full-flow-no-milestones"),
        title: String::from_str(&env, "Full Flow Test"),
        description: String::from_str(&env, "Complete lifecycle starting without milestones"),
        roles: roles.clone(),
        amount,
        platform_fee: 0,
        milestones: vec![&env],
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    assert!(client.get_escrow().milestones.is_empty());

    // 2. Agregar milestone
    let new_milestone = Milestone {
        description: String::from_str(&env, "Deliver project"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, ""),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![&env],
        },
    };
    let escrow_with_milestones =
        client.manage_milestones(&escrow_admin, &vec![&env, new_milestone], &vec![&env]);
    assert_eq!(escrow_with_milestones.milestones.len(), 1);

    // 3. Fondear el escrow
    client.fund_escrow(&approver, &escrow_with_milestones, &amount);
    assert_eq!(token_client.balance(&client.address), amount);

    // 4. Service provider marca el trabajo como completado
    let status_update = MilestoneStatusUpdate {
        milestone_index: 0,
        new_status: String::from_str(&env, "Completed"),
        new_evidence: Some(String::from_str(&env, "Delivered on time")),
    };
    client.change_milestone_status(&vec![&env, status_update], &service_provider);

    // 5. Approver aprueba el milestone
    client.approve_milestones(&vec![&env, 0u32], &approver);

    let escrow_before_release = client.get_escrow();
    let milestone = escrow_before_release.milestones.get(0).unwrap();
    assert!(milestone.approvals.approval_count >= milestone.approvals.target);

    // 6. Release funds
    client.release_funds(&release_signer, &trustless_work);

    assert!(client.get_escrow().released);
    assert_eq!(token_client.balance(&client.address), 0);
    // TW cobra 30 bps (0.3%), platform_fee=0 → receiver recibe el resto
    let tw_fee = amount * 30 / 10_000;
    assert_eq!(token_client.balance(&trustless_work), tw_fee);
    assert_eq!(token_client.balance(&receiver), amount - tw_fee);
}

#[test]
fn test_approve_and_release_milestones_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let dual_signer = Address::generate(&env); // approver + release_signer
    let service_provider = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let trustless_work = Address::generate(&env);
    let amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&dual_signer, &amount);

    let roles = Roles {
        approvers: vec![&env, dual_signer.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, dual_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: receiver.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestone = Milestone {
        description: String::from_str(&env, "Deliver project"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, ""),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![&env],
        },
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "approve-and-release-test"),
        title: String::from_str(&env, "Approve and Release Test"),
        description: String::from_str(&env, ""),
        roles,
        amount,
        platform_fee: 0,
        milestones: vec![&env, milestone],
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    client.fund_escrow(&dual_signer, &escrow_properties, &amount);

    client.approve_and_release_milestones(&dual_signer, &trustless_work, &vec![&env, 0u32]);

    assert!(client.get_escrow().released);
    assert_eq!(token_client.balance(&client.address), 0);
    let tw_fee = amount * 30 / 10_000;
    assert_eq!(token_client.balance(&receiver), amount - tw_fee);
}

#[test]
fn test_approve_and_release_milestones_only_approver_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let approver = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let trustless_work = Address::generate(&env);
    let amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&approver, &amount);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: receiver.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestone = Milestone {
        description: String::from_str(&env, "Deliver project"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, ""),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![&env],
        },
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "only-approver-test"),
        title: String::from_str(&env, "Only Approver Test"),
        description: String::from_str(&env, ""),
        roles,
        amount,
        platform_fee: 0,
        milestones: vec![&env, milestone],
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    client.fund_escrow(&approver, &escrow_properties, &amount);

    // approver no es release_signer → debe fallar
    let result =
        client.try_approve_and_release_milestones(&approver, &trustless_work, &vec![&env, 0u32]);
    assert!(result.is_err());
}

#[test]
fn test_fund_escrow_accumulation_overflow_is_controlled() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver, &amount);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: receiver.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "fund_overflow"),
        title: String::from_str(&env, "Fund Overflow"),
        description: String::from_str(&env, "Checked-add regression"),
        roles,
        amount,
        platform_fee: 0,
        milestones,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);

    // Force the stored cumulative funded amount to the i128 ceiling so the next
    // deposit's addition would overflow.
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::FundedAmount, &i128::MAX);
    });

    // A valid 1-unit deposit (approver holds the balance) must surface a
    // controlled Overflow error instead of trapping the host.
    let result = client.try_fund_escrow(&approver, &escrow_properties, &1i128);
    assert_eq!(result, Err(Ok(EscrowError::Overflow)));
}

#[test]
fn test_withdraw_remaining_funds_rejects_partial_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let recipient = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_amount: i128 = 1_000_000;
    let platform_fee: u32 = 300;

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: service_provider.clone(),
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "partial_withdraw"),
        title: String::from_str(&env, "Partial Withdraw Test"),
        description: String::from_str(&env, "Partial withdrawals must be rejected"),
        roles,
        amount: escrow_amount,
        platform_fee,
        milestones,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&approver, &escrow_amount);
    client.fund_escrow(&approver, &escrow_properties, &escrow_amount);

    client.dispute_escrow(&service_provider, &String::from_str(&env, "dispute"));

    let mut resolve_distributions = Map::new(&env);
    resolve_distributions.set(service_provider.clone(), escrow_amount);
    client.resolve_dispute(
        &dispute_resolver,
        &trustless_work_address,
        &resolve_distributions,
    );

    // Leftover balance available to withdraw.
    let remaining: i128 = 100_000;
    usdc_token.1.mint(&client.address, &remaining);

    // A partial withdrawal (total < current_balance) must now be rejected so
    // fees can't be floored away across many small calls.
    let mut partial = Map::new(&env);
    partial.set(recipient.clone(), remaining / 2);
    let result =
        client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &partial);
    assert!(result.is_err());
    assert_eq!(usdc_token.0.balance(&client.address), remaining);

    // A full-balance withdrawal still succeeds.
    let mut full = Map::new(&env);
    full.set(recipient.clone(), remaining);
    client.withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &full);
    assert_eq!(usdc_token.0.balance(&client.address), 0);
}

fn released_escrow_fixture(
    env: &Env,
) -> (
    crate::contract::EscrowContractClient<'static>,
    (
        soroban_sdk::token::Client<'static>,
        soroban_sdk::token::StellarAssetClient<'static>,
    ),
    Address, // approver (authorized signer)
    Address, // release_signer
    Address, // dispute_resolver
    Address, // trustless_work
    i128,    // amount
) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let escrow_admin = Address::generate(env);
    let approver = Address::generate(env);
    let service_provider = Address::generate(env);
    let platform = Address::generate(env);
    let release_signer = Address::generate(env);
    let dispute_resolver = Address::generate(env);
    let receiver = Address::generate(env);
    let trustless_work = Address::generate(env);

    let usdc = create_usdc_token(env, &admin);
    let amount: i128 = 100_000_000;

    let milestones = vec![
        env,
        Milestone {
            description: String::from_str(env, "M1"),
            status: String::from_str(env, "Completed"),
            evidence: String::from_str(env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![env],
            },
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(env, "released_fixture"),
        title: String::from_str(env, "Test"),
        description: String::from_str(env, "Desc"),
        roles: Roles {
            approvers: vec![env, approver.clone()],
            service_providers: vec![env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![env, release_signer.clone()],
            dispute_resolvers: vec![env, dispute_resolver.clone()],
            receiver: receiver.clone(),
            admin: escrow_admin.clone(),
            observers: vec![env],
        },
        amount,
        platform_fee: 300,
        milestones,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);

    usdc.1.mint(&approver, &amount);
    client.fund_escrow(&approver, &escrow_properties, &amount);
    client.approve_milestones(&vec![env, 0u32], &approver);
    client.release_funds(&release_signer, &trustless_work);

    (
        client,
        usdc,
        approver,
        release_signer,
        dispute_resolver,
        trustless_work,
        amount,
    )
}

#[test]
fn test_dispute_escrow_after_release_is_rejected() {
    let env = Env::default();
    let (client, _usdc, approver, _rs, _dr, _tw, _amount) = released_escrow_fixture(&env);

    // The escrow is released; opening a dispute must now be rejected.
    let result = client.try_dispute_escrow(&approver, &String::from_str(&env, "too late"));
    assert_eq!(
        result.err(),
        Some(Ok(crate::error::EscrowError::EscrowAlreadyReleased))
    );
}

#[test]
fn test_dispute_resolver_can_sweep_surplus_after_release() {
    let env = Env::default();
    let (client, usdc, _approver, _rs, dispute_resolver, trustless_work, _amount) =
        released_escrow_fixture(&env);

    // Contract holds nothing after a clean release.
    assert_eq!(usdc.0.balance(&client.address), 0);

    // Surplus lands in the contract (e.g. overfunding or a stray transfer).
    let surplus: i128 = 5_000_000;
    usdc.1.mint(&client.address, &surplus);

    // The dispute_resolver can sweep it via withdraw_remaining_funds WITHOUT
    // opening a dispute, since the escrow is released.
    let recipient = Address::generate(&env);
    let mut distributions = soroban_sdk::Map::new(&env);
    distributions.set(recipient.clone(), surplus);
    client.withdraw_remaining_funds(&dispute_resolver, &trustless_work, &distributions);

    // Surplus fully swept; nothing stranded.
    assert_eq!(usdc.0.balance(&client.address), 0);
    assert!(usdc.0.balance(&recipient) > 0);
}

#[test]
fn test_resolve_dispute_with_18_decimal_scale_amounts() {
    // Regression: the per-recipient pro-rata net used a naive i128 product
    // (amount * distributable), which overflows on 18-decimal trustlines and
    // made resolve_dispute revert permanently. It must now succeed.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    // ~20 tokens on an 18-decimal token: amount * distributable overflows i128.
    let amount: i128 = 20_000_000_000_000_000_000;
    assert!(
        amount.checked_mul(amount).is_none(),
        "precondition: naive product must overflow i128"
    );

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "wide_amounts"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: receiver.clone(),
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        amount,
        platform_fee: 300,
        milestones,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);
    usdc_token.1.mint(&client.address, &amount);

    client.dispute_escrow(&approver, &String::from_str(&env, "dispute"));

    // resolve_dispute requires total == current_balance in single-release.
    let mut distributions = Map::new(&env);
    distributions.set(receiver.clone(), amount);

    client.resolve_dispute(&dispute_resolver, &trustless_work, &distributions);

    assert!(client.get_escrow().dispute.resolved);
    assert_eq!(usdc_token.0.balance(&client.address), 0);
    assert!(usdc_token.0.balance(&receiver) > 0);
    assert!(usdc_token.0.balance(&trustless_work) > 0);
    assert!(usdc_token.0.balance(&platform) > 0);
}

#[test]
fn test_extend_contract_ttl_covers_funded_amount_and_instance() {
    use soroban_sdk::testutils::storage::{Instance, Persistent};
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);
    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver, &amount);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "ttl_coverage"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: receiver.clone(),
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        amount,
        platform_fee: 300,
        milestones,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);
    client.fund_escrow(&approver, &escrow_properties, &amount);

    // fund_escrow leaves TTLs at the ledger max; advance until the remaining
    // TTL drops below the 17280-ledger extend threshold, otherwise extend_ttl
    // is a no-op and the test would prove nothing.
    env.ledger().with_mut(|li| li.sequence_number += 6_300_000);

    let (funded_before, instance_before) = env.as_contract(&client.address, || {
        (
            env.storage()
                .persistent()
                .get_ttl(&crate::storage::types::DataKey::FundedAmount),
            env.storage().instance().get_ttl(),
        )
    });

    client.extend_contract_ttl(&escrow_admin, &1_000_000u32);

    let (funded_after, instance_after) = env.as_contract(&client.address, || {
        (
            env.storage()
                .persistent()
                .get_ttl(&crate::storage::types::DataKey::FundedAmount),
            env.storage().instance().get_ttl(),
        )
    });

    assert!(
        funded_after > funded_before,
        "FundedAmount TTL must be extended (before {funded_before}, after {funded_after})"
    );
    assert!(
        instance_after > instance_before,
        "instance TTL must be extended (before {instance_before}, after {instance_after})"
    );
}
