extern crate std;

use crate::storage::types::{Dispute, Escrow, Milestone, MilestoneApprovals, MilestoneStatusUpdate, Roles, Trustline};
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
    let receiver_address = Address::generate(&env);

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
                approvers: vec![&env]},
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_escrow_fund");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0};

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
    let receiver_address = Address::generate(&env);

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
                approvers: vec![&env]},
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_escrow_insufficient_funds");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0};

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
    let receiver_address = Address::generate(&env);
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
                approvers: vec![&env]},
            amount: 50_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 50_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_escrow_1");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.approve_milestones(&vec![&env, 1u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32, 1u32]);

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
        usdc_token.0.balance(&receiver_address),
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
    let receiver_address = Address::generate(&env);
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
                approvers: vec![&env]},
            amount: 50_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 50_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_incomplete_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: incomplete_milestones.clone(),
        trustline,
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));
    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    // Try to distribute earnings with incomplete milestones (should fail)
    let result =
        escrow_approver.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32, 1u32]);
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
    let receiver_address = service_provider_address.clone();
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
                approvers: vec![&env]},
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);

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
    let receiver_address = Address::generate(&env);

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
                approvers: vec![&env]},
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);

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
        usdc_token.0.balance(&receiver_address),
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
fn test_batch_release_partial_then_full() {
    // Release milestone 0 first, then milestone 1, verifying partial and full release logic.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let platform_fee = 5 * 100; // 5%

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Evidence A"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 40_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Evidence B"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 60_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "batch_release_test"),
        title: String::from_str(&env, "Batch Release Test"),
        description: String::from_str(&env, "Test batch milestone release"),
        roles,
        platform_fee,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);
    usdc_token.1.mint(&client.address, &amount);

    client.approve_milestones(&vec![&env, 0u32], &approver_address);
    client.approve_milestones(&vec![&env, 1u32], &approver_address);

    // Release only milestone 0
    client.release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);

    let m0_amount: i128 = 40_000_000;
    let tw_fee_0 = (m0_amount * 30) / 10000;
    let platform_fee_0 = (m0_amount * platform_fee as i128) / 10000;
    let receiver_0 = m0_amount - tw_fee_0 - platform_fee_0;

    assert_eq!(usdc_token.0.balance(&trustless_work_address), tw_fee_0);
    assert_eq!(usdc_token.0.balance(&platform), platform_fee_0);
    assert_eq!(usdc_token.0.balance(&receiver_address), receiver_0);

    // Escrow should not be fully released yet
    let escrow_after_first = client.get_escrow();
    assert!(!escrow_after_first.milestones.iter().all(|m| m.released));
    assert!(escrow_after_first.milestones.get(0).unwrap().released);
    assert!(!escrow_after_first.milestones.get(1).unwrap().released);

    // Trying to release milestone 0 again must fail
    let result = client.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);
    assert!(result.is_err(), "Re-releasing an already released milestone must fail");

    // Release milestone 1
    client.release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 1u32]);

    let m1_amount: i128 = 60_000_000;
    let tw_fee_1 = (m1_amount * 30) / 10000;
    let platform_fee_1 = (m1_amount * platform_fee as i128) / 10000;
    let receiver_1 = m1_amount - tw_fee_1 - platform_fee_1;

    assert_eq!(usdc_token.0.balance(&trustless_work_address), tw_fee_0 + tw_fee_1);
    assert_eq!(usdc_token.0.balance(&platform), platform_fee_0 + platform_fee_1);
    assert_eq!(usdc_token.0.balance(&receiver_address), receiver_0 + receiver_1);
    assert_eq!(usdc_token.0.balance(&client.address), 0);

    // Now all milestones released — escrow flag should be set
    let escrow_final = client.get_escrow();
    assert!(escrow_final.milestones.iter().all(|m| m.released));
}

#[test]
fn test_release_unapproved_milestone_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver_address.clone()},
    ];

    let roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "unapproved_release_test"),
        title: String::from_str(&env, "Unapproved Release Test"),
        description: String::from_str(&env, ""),
        roles,
        platform_fee: 300,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    usdc_token.1.mint(&client.address, &100_000_000);

    let result = client.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);
    assert!(result.is_err(), "Releasing unapproved milestone must fail");
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
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_amount: i128 = 1_000_000;
    let platform_fee: u32 = 300; // 3%

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(), observers: vec![&env]};

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approvers: vec![&env]},
            amount: 1_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
            released: false,
            receiver: receiver.clone()},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "rounding_withdraw"),
        title: String::from_str(&env, "Rounding Withdraw Test"),
        description: String::from_str(&env, "Test floor division rounding in withdraw"),
        roles,
        platform_fee,
        milestones: milestones.clone(),
        trustline: Trustline {
            address: usdc_token.0.address.clone()},
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    // Fund the escrow
    usdc_token.1.mint(&approver, &escrow_amount);
    client.fund_escrow(&approver, &escrow_properties, &escrow_amount);

    // Put escrow into dispute and then resolve it so withdraw_remaining_funds is allowed.
    // withdraw_remaining_funds requires at least one milestone to have been disputed.
    client.dispute_milestones(&approver, &vec![&env, 0u32], &String::from_str(&env, "Work disputed"));

    let mut resolve_dist = Map::new(&env);
    resolve_dist.set(approver.clone(), escrow_amount / 2);
    resolve_dist.set(service_provider.clone(), escrow_amount - escrow_amount / 2);
    client.resolve_dispute(&dispute_resolver, &trustless_work_address, &vec![&env, 0u32], &resolve_dist);

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
    let milestone_amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&dual_signer, &milestone_amount);

    let roles = Roles {
        approvers: vec![&env, dual_signer.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, dual_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestone = Milestone {
        description: String::from_str(&env, "Deliver project"),
        status: String::from_str(&env, "Completed"),
        evidence: String::from_str(&env, "Work done"),
        approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
        amount: milestone_amount,
        dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
        released: false,
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "approve-and-release-test"),
        title: String::from_str(&env, "Approve and Release Test"),
        description: String::from_str(&env, ""),
        roles,
        platform_fee: 0,
        milestones: vec![&env, milestone],
        trustline: Trustline { address: token_client.address.clone() },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    client.fund_escrow(&dual_signer, &escrow_properties, &milestone_amount);

    client.approve_and_release_milestones(&dual_signer, &trustless_work, &vec![&env, 0u32]);

    assert!(client.get_escrow().milestones.get(0).unwrap().released);
    assert_eq!(token_client.balance(&client.address), 0);
    let tw_fee = milestone_amount * 30 / 10_000;
    assert_eq!(token_client.balance(&receiver), milestone_amount - tw_fee);
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
    let milestone_amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&approver, &milestone_amount);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let milestone = Milestone {
        description: String::from_str(&env, "Deliver project"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, ""),
        approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
        amount: milestone_amount,
        dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
        released: false,
        receiver: receiver.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "only-approver-test"),
        title: String::from_str(&env, "Only Approver Test"),
        description: String::from_str(&env, ""),
        roles,
        platform_fee: 0,
        milestones: vec![&env, milestone],
        trustline: Trustline { address: token_client.address.clone() },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);
    client.fund_escrow(&approver, &escrow_properties, &milestone_amount);

    // approver no es release_signer → debe fallar
    let result = client.try_approve_and_release_milestones(&approver, &trustless_work, &vec![&env, 0u32]);
    assert!(result.is_err());
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
    let milestone_amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    token_admin.mint(&approver, &milestone_amount);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    // 1. Init sin milestones
    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "full-flow-no-milestones"),
        title: String::from_str(&env, "Full Flow Test"),
        description: String::from_str(&env, "Complete lifecycle starting without milestones"),
        roles: roles.clone(),
        platform_fee: 0,
        milestones: vec![&env],
        trustline: Trustline { address: token_client.address.clone() },
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
        approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
        amount: milestone_amount,
        dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false },
        released: false,
        receiver: receiver.clone(),
    };
    let escrow_with_milestones = client.manage_milestones(
        &escrow_admin,
        &vec![&env, new_milestone],
        &vec![&env],
    );
    assert_eq!(escrow_with_milestones.milestones.len(), 1);

    // 3. Fondear el escrow
    client.fund_escrow(&approver, &escrow_with_milestones, &milestone_amount);
    assert_eq!(token_client.balance(&client.address), milestone_amount);

    // 4. Service provider marca el trabajo como completado
    let status_update = MilestoneStatusUpdate {
        milestone_index: 0,
        new_status: String::from_str(&env, "Completed"),
        new_evidence: Some(String::from_str(&env, "Delivered on time")),
    };
    client.change_milestone_status(&vec![&env, status_update], &service_provider);

    // 5. Approver aprueba el milestone
    client.approve_milestones(&vec![&env, 0u32], &approver);

    let milestone = client.get_escrow().milestones.get(0).unwrap();
    assert!(milestone.approvals.approval_count >= milestone.approvals.target);

    // 6. Release funds del milestone
    client.release_funds(&release_signer, &trustless_work, &vec![&env, 0u32]);

    assert_eq!(token_client.balance(&client.address), 0);
    // TW cobra 30 bps (0.3%), platform_fee=0 → receiver recibe el resto
    let tw_fee = milestone_amount * 30 / 10_000;
    assert_eq!(token_client.balance(&trustless_work), tw_fee);
    assert_eq!(token_client.balance(&receiver), milestone_amount - tw_fee);
    assert!(client.get_escrow().milestones.get(0).unwrap().released);
}
