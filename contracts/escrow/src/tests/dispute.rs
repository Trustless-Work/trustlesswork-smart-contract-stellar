extern crate std;

use crate::storage::types::{Dispute, Escrow, Milestone, MilestoneApprovals, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_dispute_management() {
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

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "test_dispute");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

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
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

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

    let escrow = escrow_approver.get_escrow();
    assert!(!escrow.milestones.iter().any(|m| m.dispute.is_disputed));

    escrow_approver.dispute_milestones(&approver_address, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

    let escrow_after_change = escrow_approver.get_escrow();
    assert!(escrow_after_change.milestones.iter().any(|m| m.dispute.is_disputed));

    usdc_token.1.mint(&approver_address, &(amount as i128));
    // Test block on distributing earnings during dispute
    let result =
        escrow_approver.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);
    assert!(result.is_err());

    let _ = escrow_approver.try_dispute_milestones(&approver_address, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

    let escrow_after_second_change = escrow_approver.get_escrow();
    assert!(escrow_after_second_change.milestones.iter().any(|m| m.dispute.is_disputed));
}

#[test]
fn test_dispute_resolution_process() {
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
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(), observers: vec![&env]};

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone()};

    let engagement_id = String::from_str(&env, "test_dispute_resolution");
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
        .0
        .transfer(&approver_address, &escrow_approver.address, &amount);

    escrow_approver.dispute_milestones(&approver_address, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

    let escrow_with_dispute = escrow_approver.get_escrow();
    assert!(escrow_with_dispute.milestones.iter().any(|m| m.dispute.is_disputed));

    // Try to resolve dispute with incorrect dispute resolver (should fail)
    let mut wrong_dist = Map::new(&env);
    wrong_dist.set(approver_address.clone(), 50_000_000);
    wrong_dist.set(service_provider_address.clone(), 50_000_000);
    let result = escrow_approver.try_resolve_dispute(
        &approver_address,
        &trustless_work_address,
        &wrong_dist,
    );
    assert!(result.is_err());

    let approver_funds: i128 = 50_000_000;
    let insufficient_receiver_funds: i128 = 40_000_000;

    let mut incorrect_dist = Map::new(&env);
    incorrect_dist.set(approver_address.clone(), approver_funds);
    incorrect_dist.set(
        service_provider_address.clone(),
        insufficient_receiver_funds,
    );
    let incorrect_dispute_resolution_result = escrow_approver.try_resolve_dispute(
        &dispute_resolver_address,
        &trustless_work_address,
        &incorrect_dist,
    );

    assert!(incorrect_dispute_resolution_result.is_err());

    let empty_dist = Map::new(&env);
    let dispute_resolution_with_incorrect_funds = escrow_approver.try_resolve_dispute(
        &dispute_resolver_address,
        &trustless_work_address,
        &empty_dist,
    );

    assert!(dispute_resolution_with_incorrect_funds.is_err());

    // Resolve dispute with correct dispute resolver (50/50 split)
    let receiver_funds: i128 = 50_000_000;

    let mut ok_dist = Map::new(&env);
    ok_dist.set(approver_address.clone(), approver_funds);
    ok_dist.set(service_provider_address.clone(), receiver_funds);
    escrow_approver.resolve_dispute(&dispute_resolver_address, &trustless_work_address, &ok_dist);

    // Verify dispute was resolved
    let escrow_after_resolution = escrow_approver.get_escrow();
    assert!(!escrow_after_resolution.milestones.iter().any(|m| m.dispute.is_disputed));
    assert!(escrow_after_resolution.milestones.iter().any(|m| m.dispute.resolved));

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let remaining_amount = total_amount - (trustless_work_commission + platform_commission);

    let platform_amount = platform_commission;
    let service_provider_amount = (remaining_amount * receiver_funds) / total_amount;
    let approver_amount = (remaining_amount * approver_funds) / total_amount;

    // Check balances
    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform),
        platform_amount,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        service_provider_amount,
        "Service provider amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&approver_address),
        approver_amount,
        "Approver amount is incorrect"
    );
}

#[test]
fn test_dispute_milestones_authorized_and_unauthorized() {
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
    let unauthorized = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: receiver.clone(),
        admin: escrow_admin.clone(), observers: vec![&env]};

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
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "engagement_001"),
        title: String::from_str(&env, "Escrow for test"),
        description: String::from_str(&env, "Test for dispute flag"),
        roles,
        platform_fee: 0,
        milestones,
        trustline: Trustline {
            address: usdc_token.0.address.clone()},
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client_1 = test_data.client;

    escrow_client_1.initialize_escrow(&escrow_base);
    escrow_client_1.dispute_milestones(&approver, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

    let updated_escrow = escrow_client_1.get_escrow();
    assert!(
        updated_escrow.milestones.iter().any(|m| m.dispute.is_disputed),
        "Dispute flag should be set to true for authorized address"
    );

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client_2 = test_data.client;

    escrow_client_2.initialize_escrow(&escrow_base);
    let result = escrow_client_2.try_dispute_milestones(&unauthorized, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

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
    let escrow_admin = Address::generate(&env);
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
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        receiver: service_provider.clone(),
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
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "rounding_resolve"),
        title: String::from_str(&env, "Rounding Test"),
        description: String::from_str(&env, "Test floor division rounding in resolve_dispute"),
        roles,
        platform_fee,
        milestones,
        trustline: Trustline {
            address: usdc_token.0.address.clone()},
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    // Fund the escrow with exactly total
    usdc_token.1.mint(&client.address, &total);

    // Put escrow in dispute
    client.dispute_milestones(&approver, &vec![&env, 0u32], &String::from_str(&env, "Work not done"));

    // Distributions that trigger the rounding mismatch
    let mut distributions = Map::new(&env);
    distributions.set(approver.clone(), 50_001);
    distributions.set(service_provider.clone(), 50_002);

    // This must NOT revert (old code would fail here due to insufficient balance)
    let result = client.try_resolve_dispute(
        &dispute_resolver,
        &trustless_work_address,
        &distributions,
    );
    assert!(result.is_ok(), "resolve_dispute should handle fee rounding correctly");

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
    assert!(escrow.milestones.iter().any(|m| m.dispute.resolved));
    assert!(!escrow.milestones.iter().any(|m| m.dispute.is_disputed));
}

#[test]
fn test_dispute_milestones_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 40_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 30_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 30_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "batch_dispute"),
        title: String::from_str(&env, "Batch Dispute Test"),
        description: String::from_str(&env, "Test batch milestone dispute"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: service_provider.clone(),
            admin: escrow_admin.clone(), observers: vec![&env]},
        platform_fee: 300,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Dispute milestones 0 and 2 in batch
    client.dispute_milestones(&approver, &vec![&env, 0u32, 2u32], &String::from_str(&env, "Dispute reason"));

    let escrow = client.get_escrow();
    assert!(escrow.milestones.iter().any(|m| m.dispute.is_disputed), "Escrow-level disputed flag must be set");
    assert!(escrow.milestones.get(0).unwrap().dispute.is_disputed, "Milestone 0 must be disputed");
    assert!(!escrow.milestones.get(1).unwrap().dispute.is_disputed, "Milestone 1 must NOT be disputed");
    assert!(escrow.milestones.get(2).unwrap().dispute.is_disputed, "Milestone 2 must be disputed");
}

#[test]
fn test_dispute_milestones_invalid_index_reverts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Only milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "invalid_idx"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: service_provider.clone(),
            admin: escrow_admin.clone(), observers: vec![&env]},
        platform_fee: 0,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Index 5 does not exist → must revert
    let result = client.try_dispute_milestones(&approver, &vec![&env, 5u32], &String::from_str(&env, "Dispute reason"));
    assert!(result.is_err(), "Disputing a non-existent milestone index must revert");
}

#[test]
fn test_dispute_milestones_already_disputed_reverts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "double_dispute"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: service_provider.clone(),
            admin: escrow_admin.clone(), observers: vec![&env]},
        platform_fee: 0,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // First dispute succeeds
    client.dispute_milestones(&approver, &vec![&env, 0u32], &String::from_str(&env, "Dispute reason"));

    // Disputing the same milestone again must revert
    let result = client.try_dispute_milestones(&approver, &vec![&env, 0u32], &String::from_str(&env, "Dispute reason"));
    assert!(result.is_err(), "Re-disputing an already disputed milestone must revert");
}

#[test]
fn test_dispute_milestones_unauthorized_reverts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals { target: 1, approval_count: 0, approvers: vec![&env] },
            amount: 100_000_000,
            dispute: Dispute { is_disputed: false, reason: String::from_str(&env, ""), resolved: false }, released: false},
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "unauth_dispute"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            receiver: service_provider.clone(),
            admin: escrow_admin.clone(), observers: vec![&env]},
        platform_fee: 0,
        milestones,
        trustline: Trustline { address: usdc_token.0.address.clone() },
        receiver_memo: 0};

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Unauthorized address must revert
    let result = client.try_dispute_milestones(&unauthorized, &vec![&env, 0u32], &String::from_str(&env, "Dispute reason"));
    assert!(result.is_err(), "Unauthorized address must not be able to dispute milestones");

    // Dispute resolver must also be blocked
    let result = client.try_dispute_milestones(&dispute_resolver, &vec![&env, 0u32], &String::from_str(&env, "Dispute reason"));
    assert!(result.is_err(), "Dispute resolver must not be able to dispute milestones");
}
