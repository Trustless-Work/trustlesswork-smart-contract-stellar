extern crate std;

use crate::storage::types::{
    Dispute, Escrow, Milestone, MilestoneApprovals, MilestoneStatusEntry, MilestoneStatusUpdate,
    MilestoneUpdate, Roles, Trustline,
};
use soroban_sdk::{
    testutils::Address as _, testutils::Events as _, vec, xdr::ToXdr, Address, BytesN, Env,
    Event as _, String,
};

use crate::events::handler::{MilestoneStatusChanged, MilestonesManaged};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_append_milestones_with_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);

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
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
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
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
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
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_approver.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Add a new milestone via manage_milestones
    let new_milestones_to_add = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    escrow_approver.manage_milestones(&escrow_admin, &new_milestones_to_add, &vec![&env]);

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
    assert_eq!(escrow.platform_fee, initial_escrow_properties.platform_fee);
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
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);

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
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
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
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
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
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_client.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Approve the first milestone
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_address);
    let after_approval = escrow_client.get_escrow();
    {
        let m = after_approval.milestones.get(0).unwrap();
        assert!(m.approvals.approval_count >= m.approvals.target);
    }

    // Add a new milestone via manage_milestones
    let new_milestones_to_add = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    escrow_client.manage_milestones(&escrow_admin, &new_milestones_to_add, &vec![&env]);
    let final_escrow = escrow_client.get_escrow();

    assert_eq!(final_escrow.milestones.len(), 3);
    {
        let m = final_escrow.milestones.get(0).unwrap();
        assert!(
            m.approvals.approval_count >= m.approvals.target,
            "Existing approved milestone should remain approved"
        );
    }
    assert_eq!(
        final_escrow.milestones.get(1).unwrap(),
        after_approval.milestones.get(1).unwrap()
    );
    {
        let m = final_escrow.milestones.get(2).unwrap();
        assert!(
            m.approvals.approval_count < m.approvals.target,
            "Appended milestone should start unapproved"
        );
    }
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
    assert_eq!(
        final_escrow.platform_fee,
        initial_escrow_properties.platform_fee
    );
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
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
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
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
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
    {
        let m = final_escrow.milestones.get(0).unwrap();
        assert!(m.approvals.approval_count >= m.approvals.target);
    }

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

    // Milestone 0 is already fully approved (target reached), further approval must fail
    let result = escrow_approver.try_approve_milestones(&vec![&env, 0u32], &unauthorized_address);
    assert!(result.is_err());
}

#[test]
fn test_change_milestone_status_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 3"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_status_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
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
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_revert_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
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
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "empty_batch_test"),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Empty batch must fail
    let empty_updates: soroban_sdk::Vec<MilestoneStatusUpdate> = soroban_sdk::vec![&env];
    let result =
        escrow_client.try_change_milestone_status(&empty_updates, &service_provider_address);
    assert!(result.is_err());
}

#[test]
fn test_target_requires_multiple_approvers() {
    // Verify that a milestone with target > 1 is not considered approved
    // until the required number of unique approvers have voted.
    let env = Env::default();
    env.mock_all_auths();

    let approver_a = Address::generate(&env);
    let approver_b = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone requiring target 2"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence"),
            approvals: MilestoneApprovals {
                target: 2,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_a.clone(), approver_b.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "target_test"),
        title: String::from_str(&env, "Target Test Escrow"),
        description: String::from_str(&env, "Test target approval"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&escrow_client.address, &amount);

    // First approval: approval_count becomes 1, target is 2 → not yet approved
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_a);

    let escrow_after_first = escrow_client.get_escrow();
    let m = escrow_after_first.milestones.get(0).unwrap();
    assert_eq!(m.approvals.approval_count, 1);
    assert!(
        m.approvals.approval_count < m.approvals.target,
        "Milestone must not be approved yet after only one vote"
    );

    // Release must fail — target not yet reached
    let result = escrow_client.try_release_funds(
        &release_signer_address,
        &trustless_work_address,
        &vec![&env, 0u32],
    );
    assert!(
        result.is_err(),
        "Release must fail when target is not reached"
    );

    // Approver A tries to vote again — must fail
    let result = escrow_client.try_approve_milestones(&vec![&env, 0u32], &approver_a);
    assert!(
        result.is_err(),
        "Double-voting by the same approver must be rejected"
    );

    // Second approval by a different address: approval_count becomes 2 == target
    escrow_client.approve_milestones(&vec![&env, 0u32], &approver_b);

    let escrow_after_second = escrow_client.get_escrow();
    let m2 = escrow_after_second.milestones.get(0).unwrap();
    assert_eq!(m2.approvals.approval_count, 2);
    assert!(
        m2.approvals.approval_count >= m2.approvals.target,
        "Milestone must be fully approved once target is reached"
    );
    assert_eq!(
        m2.approvals.approved_by.len(),
        2,
        "Both approvers must be recorded"
    );

    // Release must now succeed
    let result = escrow_client.try_release_funds(
        &release_signer_address,
        &trustless_work_address,
        &vec![&env, 0u32],
    );
    assert!(
        result.is_ok(),
        "Release must succeed after target is reached"
    );
}

#[test]
fn test_batch_approve_milestones_multiple_indices() {
    // Approve multiple milestones in a single call and verify each is updated.
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let platform_fee = 3 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 1"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 2"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, "Evidence 3"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
        },
    ];

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "batch_approve_test"),
        title: String::from_str(&env, "Batch Approve Test"),
        description: String::from_str(&env, "Test batch milestone approval"),
        roles: roles.clone(),
        platform_fee,
        milestones: milestones.clone(),
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&escrow_properties);

    // Approve milestones 0 and 2 in a single call, leave 1 unapproved
    escrow_client.approve_milestones(&vec![&env, 0u32, 2u32], &approver_address);

    let escrow = escrow_client.get_escrow();
    let m0 = escrow.milestones.get(0).unwrap();
    let m1 = escrow.milestones.get(1).unwrap();
    let m2 = escrow.milestones.get(2).unwrap();

    assert_eq!(
        m0.approvals.approval_count, 1,
        "Milestone 0 should have 1 approval"
    );
    assert!(
        m0.approvals.approval_count >= m0.approvals.target,
        "Milestone 0 should be approved"
    );

    assert_eq!(
        m1.approvals.approval_count, 0,
        "Milestone 1 should be untouched"
    );
    assert!(
        m1.approvals.approval_count < m1.approvals.target,
        "Milestone 1 should not be approved"
    );

    assert_eq!(
        m2.approvals.approval_count, 1,
        "Milestone 2 should have 1 approval"
    );
    assert!(
        m2.approvals.approval_count >= m2.approvals.target,
        "Milestone 2 should be approved"
    );

    // Empty batch must fail
    let empty: soroban_sdk::Vec<u32> = soroban_sdk::vec![&env];
    let result = escrow_client.try_approve_milestones(&empty, &approver_address);
    assert!(result.is_err(), "Empty batch must fail");

    // Batch with invalid index must revert entirely
    let result = escrow_client.try_approve_milestones(&vec![&env, 1u32, 99u32], &approver_address);
    assert!(result.is_err(), "Batch with invalid index must fail");
}

#[test]
fn test_manage_milestones() {
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
    let non_platform = Address::generate(&env);

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 50_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "manage_milestones_test"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones: initial_milestones.clone(),
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_base);

    // Add new milestones (no funds required)
    let to_add = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 25_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 25_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let no_updates: soroban_sdk::Vec<MilestoneUpdate> = soroban_sdk::vec![&env];
    client.manage_milestones(&escrow_admin, &to_add, &no_updates);

    let escrow = client.get_escrow();
    assert_eq!(
        escrow.milestones.len(),
        3,
        "Should have 3 milestones after adding 2"
    );
    assert_eq!(
        escrow.milestones.get(0).unwrap().description,
        String::from_str(&env, "Milestone 1"),
        "Original milestone must be preserved"
    );
    assert_eq!(
        escrow.milestones.get(2).unwrap().description,
        String::from_str(&env, "Milestone 3")
    );

    // Unauthorized caller must fail
    let result = client.try_manage_milestones(&non_platform, &to_add, &no_updates);
    assert!(
        result.is_err(),
        "Non-admin address must not manage milestones"
    );

    // Both lists empty must fail
    let empty_milestones: soroban_sdk::Vec<Milestone> = soroban_sdk::vec![&env];
    let result = client.try_manage_milestones(&escrow_admin, &empty_milestones, &no_updates);
    assert!(result.is_err(), "Both lists empty must fail");

    // Milestone with released=true must fail
    let already_released = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Already released"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 10_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: true,
            receiver: receiver.clone(),
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &already_released, &no_updates);
    assert!(result.is_err(), "Milestone with released=true must fail");

    // Milestone with target 0 must fail
    let bad_target = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Bad"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 0,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 10_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &bad_target, &no_updates);
    assert!(result.is_err(), "Milestone with target 0 must fail");

    // Updating description/amount without funds must succeed
    let updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            new_description: Some(String::from_str(&env, "Updated Milestone 1")),
            new_amount: Some(60_000_000i128),
        },
    ];
    client.manage_milestones(&escrow_admin, &empty_milestones, &updates);
    let after_update = client.get_escrow();
    assert_eq!(
        after_update.milestones.get(0).unwrap().description,
        String::from_str(&env, "Updated Milestone 1"),
        "Description must be updated when escrow has no funds"
    );
    assert_eq!(
        after_update.milestones.get(0).unwrap().amount,
        60_000_000i128,
        "Amount must be updated when escrow has no funds"
    );

    // Invalid milestone index must fail
    let bad_index = vec![
        &env,
        MilestoneUpdate {
            index: 99,
            new_description: Some(String::from_str(&env, "Bad")),
            new_amount: None,
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &empty_milestones, &bad_index);
    assert!(result.is_err(), "Invalid milestone index must fail");

    // update_escrow must NOT change milestones (no funds yet)
    let updated_escrow_props = Escrow {
        engagement_id: String::from_str(&env, "manage_milestones_test"),
        title: String::from_str(&env, "Updated Title"),
        description: String::from_str(&env, "Updated Desc"),
        roles: escrow_base.roles.clone(),
        platform_fee: escrow_base.platform_fee,
        milestones: initial_milestones.clone(),
        trustline: escrow_base.trustline.clone(),
        receiver_memo: 0,
    };
    client.update_escrow(&escrow_admin, &updated_escrow_props);

    let after_escrow_update = client.get_escrow();
    assert_eq!(
        after_escrow_update.milestones.len(),
        3,
        "update_escrow must not overwrite milestones"
    );
    assert_eq!(
        after_escrow_update.title,
        String::from_str(&env, "Updated Title")
    );

    // Fund the escrow — updating must now fail
    token_admin.mint(&approver, &100_000_000i128);
    let current_escrow = client.get_escrow();
    client.fund_escrow(&approver, &current_escrow, &100_000_000i128);

    let result = client.try_manage_milestones(&escrow_admin, &empty_milestones, &updates);
    assert!(
        result.is_err(),
        "Updating milestones must fail when escrow has funds"
    );
}

#[test]
fn test_add_milestones_after_init_without_milestones() {
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

    let (token_client, _) = create_usdc_token(&env, &admin);

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "no-milestones-then-add"),
        title: String::from_str(&env, "Deferred Milestones Escrow"),
        description: String::from_str(&env, "Init without milestones, add later"),
        roles: roles.clone(),
        platform_fee: 0,
        milestones: vec![&env],
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    let new_milestones = vec![
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
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let empty_updates = vec![&env];
    let result = client.try_manage_milestones(&escrow_admin, &new_milestones, &empty_updates);
    assert!(
        result.is_ok(),
        "Should be able to add milestones to an escrow initialized without them"
    );

    let escrow = client.get_escrow();
    assert_eq!(escrow.milestones.len(), 1);
}

#[test]
fn test_manage_milestones_rejects_oversized_strings() {
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

    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 50_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "manage_milestones_lengths"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones: initial_milestones,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_base);

    let long_desc = String::from_str(&env, std::str::from_utf8(&[b'a'; 501]).unwrap());
    let long_status = String::from_str(&env, std::str::from_utf8(&[b'a'; 51]).unwrap());
    let long_evidence = String::from_str(&env, std::str::from_utf8(&[b'a'; 501]).unwrap());
    let no_updates: soroban_sdk::Vec<MilestoneUpdate> = vec![&env];

    let base_milestone = Milestone {
        description: String::from_str(&env, "ok"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, ""),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![&env],
        },
        amount: 25_000_000,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(&env, ""),
            resolved: false,
        },
        released: false,
        receiver: receiver.clone(),
    };

    let mut bad_desc = base_milestone.clone();
    bad_desc.description = long_desc.clone();
    let result = client.try_manage_milestones(&escrow_admin, &vec![&env, bad_desc], &no_updates);
    assert!(result.is_err());

    let mut bad_status = base_milestone.clone();
    bad_status.status = long_status;
    let result = client.try_manage_milestones(&escrow_admin, &vec![&env, bad_status], &no_updates);
    assert!(result.is_err());

    let mut bad_evidence = base_milestone.clone();
    bad_evidence.evidence = long_evidence;
    let result =
        client.try_manage_milestones(&escrow_admin, &vec![&env, bad_evidence], &no_updates);
    assert!(result.is_err());

    let no_new: soroban_sdk::Vec<Milestone> = vec![&env];
    let bad_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            new_description: Some(long_desc),
            new_amount: None,
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &no_new, &bad_update);
    assert!(result.is_err());

    let escrow = client.get_escrow();
    assert_eq!(escrow.milestones.len(), 1);
    assert_eq!(
        escrow.milestones.get(0).unwrap().description,
        String::from_str(&env, "Milestone 1")
    );
}

#[test]
fn test_approve_milestones_unauthorized_fails_fast_before_duplicate_check() {
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
    let outsider = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "completed"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "fail_fast_auth"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones,
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_properties);

    // A non-approver submitting a batch with duplicate indices must be rejected
    // for authorization first, before the duplicate-check loop runs.
    let result = client.try_approve_milestones(&vec![&env, 0u32, 0u32], &outsider);
    assert_eq!(
        result.err(),
        Some(Ok(crate::error::MilestoneError::UnauthorizedApprover))
    );
}

#[test]
fn test_manage_milestones_rejects_non_positive_amount_update() {
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

    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 50_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "amount_update_guard"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones: initial_milestones,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow_base);

    let no_new: soroban_sdk::Vec<Milestone> = vec![&env];

    // Zero amount must be rejected.
    let zero_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            new_description: None,
            new_amount: Some(0),
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &no_new, &zero_update);
    assert!(result.is_err());

    // Negative amount must be rejected.
    let neg_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            new_description: None,
            new_amount: Some(-1),
        },
    ];
    let result = client.try_manage_milestones(&escrow_admin, &no_new, &neg_update);
    assert!(result.is_err());

    // Original amount unchanged.
    assert_eq!(client.get_escrow().milestones.get(0).unwrap().amount, 50_000_000);
}

#[test]
fn test_manage_milestones_event_carries_indices_and_hashes() {
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

    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

    let m1_description = String::from_str(&env, "Milestone 1");
    let initial_milestones = vec![
        &env,
        Milestone {
            description: m1_description.clone(),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 50_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "manage_milestones_event_test"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones: initial_milestones.clone(),
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_base);

    let new_description = String::from_str(&env, "Milestone 2");
    let to_add = vec![
        &env,
        Milestone {
            description: new_description.clone(),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 25_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];
    let updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            new_description: None,
            new_amount: Some(60_000_000),
        },
    ];

    client.manage_milestones(&escrow_admin, &to_add, &updates);

    let expected_hash: BytesN<32> = env.crypto().sha256(&new_description.to_xdr(&env)).to_bytes();
    let expected_event = MilestonesManaged {
        engagement_id: escrow_base.engagement_id.clone(),
        admin: escrow_admin.clone(),
        added_count: 1,
        updated_count: 1,
        updated_indices: vec![&env, 0u32],
        added_description_hashes: vec![&env, expected_hash],
    };

    assert_eq!(
        env.events().all(),
        std::vec![expected_event.to_xdr(&env, &client.address)],
        "emitted MilestonesManaged event did not carry the expected indices/hashes"
    );
}

#[test]
fn test_change_milestone_status_event_carries_evidence_hash() {
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

    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "evidence_hash_test"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Desc"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 300,
        milestones: initial_milestones,
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // One update carries new evidence, the other doesn't — the event must
    // distinguish the two rather than hashing an absent value.
    let new_evidence = String::from_str(&env, "Proof of delivery");
    let updates = vec![
        &env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(&env, "completed"),
            new_evidence: Some(new_evidence.clone()),
        },
        MilestoneStatusUpdate {
            milestone_index: 1,
            new_status: String::from_str(&env, "completed"),
            new_evidence: None,
        },
    ];

    client.change_milestone_status(&updates, &service_provider);

    let expected_hash: BytesN<32> = env.crypto().sha256(&new_evidence.to_xdr(&env)).to_bytes();
    let expected_event = MilestoneStatusChanged {
        engagement_id: escrow_properties.engagement_id.clone(),
        service_provider: service_provider.clone(),
        updates: vec![
            &env,
            MilestoneStatusEntry {
                index: 0,
                status: String::from_str(&env, "completed"),
                evidence_hash: Some(expected_hash),
            },
            MilestoneStatusEntry {
                index: 1,
                status: String::from_str(&env, "completed"),
                evidence_hash: None,
            },
        ],
    };

    assert_eq!(
        env.events().all(),
        std::vec![expected_event.to_xdr(&env, &client.address)],
        "MilestoneStatusChanged should hash evidence when present and report None when absent"
    );
}
