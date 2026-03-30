extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneApprovals, MilestoneStatusUpdate, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_append_milestones_with_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = service_provider_address.clone();

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: receiver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let engagement_id = String::from_str(&env, "append_with_funds");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_approver.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Build updated properties with milestones appended, all other fields identical
    let updated_milestones = vec![
        &env,
        initial_escrow_properties.milestones.get(0).unwrap(),
        initial_escrow_properties.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.update_escrow(&platform, &updated_escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.milestones.len(), 3);
    assert_eq!(
        escrow.milestones.get(0).unwrap(),
        initial_escrow_properties.milestones.get(0).unwrap()
    );
    assert_eq!(
        escrow.milestones.get(1).unwrap(),
        initial_escrow_properties.milestones.get(1).unwrap()
    );
    // Ensure non-milestone properties unchanged
    assert_eq!(
        escrow.engagement_id,
        initial_escrow_properties.engagement_id
    );
    assert_eq!(escrow.title, initial_escrow_properties.title);
    assert_eq!(escrow.description, initial_escrow_properties.description);
    assert!(escrow.roles == initial_escrow_properties.roles);
    assert_eq!(escrow.amount, initial_escrow_properties.amount);
    assert_eq!(escrow.platform_fee, initial_escrow_properties.platform_fee);
    assert!(escrow.flags == initial_escrow_properties.flags);
    assert!(escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(
        escrow.receiver_memo,
        initial_escrow_properties.receiver_memo
    );
}

#[test]
fn test_append_milestones_with_funds_and_existing_approved() {
    // This test validates that after approving an existing milestone, the contract still allows
    // appending new milestones (while keeping existing ones unchanged) when the escrow has funds.
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = service_provider_address.clone();

    let amount: i128 = 50_000_000;
    let platform_fee = 3 * 100;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: receiver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let engagement_id = String::from_str(&env, "append_with_funds_and_approved");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Approved"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_client.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Approve the first milestone
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_address);
    let after_approval = escrow_client.get_escrow();
    { let m = after_approval.milestones.get(0).unwrap(); assert!(m.approvals.approval_count >= m.approvals.quorum); }

    // Build updated properties with a new milestone appended (unapproved)
    let updated_milestones = vec![
        &env,
        after_approval.milestones.get(0).unwrap(),
        after_approval.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Approved"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_client.update_escrow(&platform, &updated_escrow_properties);
    let final_escrow = escrow_client.get_escrow();

    assert_eq!(final_escrow.milestones.len(), 3);
    { let m = final_escrow.milestones.get(0).unwrap(); assert!(m.approvals.approval_count >= m.approvals.quorum, "Existing approved milestone should remain approved"); }
    assert_eq!(
        final_escrow.milestones.get(1).unwrap(),
        after_approval.milestones.get(1).unwrap()
    );
    { let m = final_escrow.milestones.get(2).unwrap(); assert!(m.approvals.approval_count < m.approvals.quorum, "Appended milestone should start unapproved"); }
    // Ensure other properties unchanged
    assert_eq!(
        final_escrow.engagement_id,
        initial_escrow_properties.engagement_id
    );
    assert_eq!(final_escrow.title, initial_escrow_properties.title);
    assert_eq!(
        final_escrow.description,
        initial_escrow_properties.description
    );
    assert!(final_escrow.roles == initial_escrow_properties.roles);
    assert_eq!(final_escrow.amount, initial_escrow_properties.amount);
    assert_eq!(
        final_escrow.platform_fee,
        initial_escrow_properties.platform_fee
    );
    assert!(final_escrow.flags == initial_escrow_properties.flags);
    assert!(final_escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(
        final_escrow.receiver_memo,
        initial_escrow_properties.receiver_memo
    );
}

#[test]
fn test_change_milestone_status_and_approved() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Change milestone status for a single milestone via batch (valid case)
    let updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(&env, "completed"),
            new_evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    escrow_approver.change_milestone_status(&updates, &service_provider_address);

    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().status,
        String::from_str(&env, "completed")
    );
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "New evidence")
    );

    // Change milestone approved (valid case)
    escrow_approver.approve_milestones(&vec![&env, 0u32], &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    { let m = final_escrow.milestones.get(0).unwrap(); assert!(m.approvals.approval_count >= m.approvals.quorum); }

    // Invalid index in batch should fail
    let invalid_updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 10,
            new_status: String::from_str(&env, "completed"),
            new_evidence: None,
        },
    ];
    let result =
        escrow_approver.try_change_milestone_status(&invalid_updates, &service_provider_address);
    assert!(result.is_err());

    let result = escrow_approver.try_approve_milestones(&vec![&env, 10u32], &approver_address);
    assert!(result.is_err());

    let unauthorized_address = Address::generate(&env);

    // Unauthorized service provider should fail
    let unauth_updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(&env, "completed"),
            new_evidence: None,
        },
    ];
    let result =
        escrow_approver.try_change_milestone_status(&unauth_updates, &unauthorized_address);
    assert!(result.is_err());

    // Milestone 0 is already fully approved (quorum reached), further approval must fail
    let result =
        escrow_approver.try_approve_milestones(&vec![&env, 0u32], &unauthorized_address);
    assert!(result.is_err());
}

#[test]
fn test_change_milestone_status_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 3"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_status_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Update milestones 0 and 2 in a single batch call
    let updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(&env, "completed"),
            new_evidence: Some(String::from_str(&env, "Proof for milestone 1")),
        },
        MilestoneStatusUpdate {
            milestone_index: 2,
            new_status: String::from_str(&env, "completed"),
            new_evidence: Some(String::from_str(&env, "Proof for milestone 3")),
        },
    ];

    escrow_client.change_milestone_status(&updates, &service_provider_address);

    let updated_escrow = escrow_client.get_escrow();

    // Milestone 0 updated
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().status,
        String::from_str(&env, "completed")
    );
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Proof for milestone 1")
    );

    // Milestone 1 unchanged
    assert_eq!(
        updated_escrow.milestones.get(1).unwrap().status,
        String::from_str(&env, "in-progress")
    );

    // Milestone 2 updated
    assert_eq!(
        updated_escrow.milestones.get(2).unwrap().status,
        String::from_str(&env, "completed")
    );
    assert_eq!(
        updated_escrow.milestones.get(2).unwrap().evidence,
        String::from_str(&env, "Proof for milestone 3")
    );
}

#[test]
fn test_batch_milestone_status_reverts_on_invalid_index() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_revert_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Batch where second index (99) does not exist — entire operation must fail
    let updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(&env, "completed"),
            new_evidence: None,
        },
        MilestoneStatusUpdate {
            milestone_index: 99,
            new_status: String::from_str(&env, "completed"),
            new_evidence: None,
        },
    ];

    let result = escrow_client.try_change_milestone_status(&updates, &service_provider_address);
    assert!(result.is_err());

    // Verify milestone 0 was NOT updated (operation was reverted)
    let escrow = escrow_client.get_escrow();
    assert_eq!(
        escrow.milestones.get(0).unwrap().status,
        String::from_str(&env, "in-progress")
    );
}

#[test]
fn test_batch_milestone_status_empty_batch_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "empty_batch_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Empty batch must fail
    let empty_updates: soroban_sdk::Vec<MilestoneStatusUpdate> =
        soroban_sdk::vec![&env];
    let result = escrow_client.try_change_milestone_status(&empty_updates, &service_provider_address);
    assert!(result.is_err());
}

#[test]
fn test_quorum_requires_multiple_approvers() {
    // Verify that a milestone with quorum > 1 is not considered approved
    // until the required number of unique approvers have voted.
    let env = Env::default();
    env.mock_all_auths();

    let approver_a = Address::generate(&env);
    let approver_b = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone requiring quorum 2"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence"),
            approvals: MilestoneApprovals {
                quorum: 2,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_a.clone(), approver_b.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "quorum_test"),
        title: String::from_str(&env, "Quorum Test Escrow"),
        description: String::from_str(&env, "Test quorum approval"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: milestones.clone(),
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
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&escrow_client.address, &amount);

    // First approval: approval_count becomes 1, quorum is 2 → not yet approved
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_a);

    let escrow_after_first = escrow_client.get_escrow();
    let m = escrow_after_first.milestones.get(0).unwrap();
    assert_eq!(m.approvals.approval_count, 1);
    assert!(
        m.approvals.approval_count < m.approvals.quorum,
        "Milestone must not be approved yet after only one vote"
    );

    // Release must fail — quorum not yet reached
    let result =
        escrow_client.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);
    assert!(result.is_err(), "Release must fail when quorum is not reached");

    // Approver A tries to vote again — must fail
    let result = escrow_client.try_approve_milestones(&vec![&env, 0u32], &approver_a);
    assert!(result.is_err(), "Double-voting by the same approver must be rejected");

    // Second approval by a different address: approval_count becomes 2 == quorum
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_b);

    let escrow_after_second = escrow_client.get_escrow();
    let m2 = escrow_after_second.milestones.get(0).unwrap();
    assert_eq!(m2.approvals.approval_count, 2);
    assert!(
        m2.approvals.approval_count >= m2.approvals.quorum,
        "Milestone must be fully approved once quorum is reached"
    );
    assert_eq!(m2.approvals.approvers.len(), 2, "Both approvers must be recorded");

    // Release must now succeed
    let result =
        escrow_client.try_release_funds(&release_signer_address, &trustless_work_address, &vec![&env, 0u32]);
    assert!(result.is_ok(), "Release must succeed after quorum is reached");
}

#[test]
fn test_batch_approve_milestones_multiple_indices() {
    // Approve multiple milestones in a single call and verify each is updated.
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 3"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_approve_test"),
        title: String::from_str(&env, "Batch Approve Test"),
        description: String::from_str(&env, "Test batch milestone approval"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: milestones.clone(),
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
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Approve milestones 0 and 2 in a single call, leave 1 unapproved
    escrow_client.approve_milestones(&vec![&env, 0u32, 2u32], &approver_address);

    let escrow = escrow_client.get_escrow();
    let m0 = escrow.milestones.get(0).unwrap();
    let m1 = escrow.milestones.get(1).unwrap();
    let m2 = escrow.milestones.get(2).unwrap();

    assert_eq!(m0.approvals.approval_count, 1, "Milestone 0 should have 1 approval");
    assert!(m0.approvals.approval_count >= m0.approvals.quorum, "Milestone 0 should be approved");

    assert_eq!(m1.approvals.approval_count, 0, "Milestone 1 should be untouched");
    assert!(m1.approvals.approval_count < m1.approvals.quorum, "Milestone 1 should not be approved");

    assert_eq!(m2.approvals.approval_count, 1, "Milestone 2 should have 1 approval");
    assert!(m2.approvals.approval_count >= m2.approvals.quorum, "Milestone 2 should be approved");

    // Empty batch must fail
    let empty: soroban_sdk::Vec<u32> = soroban_sdk::vec![&env];
    let result = escrow_client.try_approve_milestones(&empty, &approver_address);
    assert!(result.is_err(), "Empty batch must fail");

    // Batch with invalid index must revert entirely
    let result =
        escrow_client.try_approve_milestones(&vec![&env, 1u32, 99u32], &approver_address);
    assert!(result.is_err(), "Batch with invalid index must fail");
}
