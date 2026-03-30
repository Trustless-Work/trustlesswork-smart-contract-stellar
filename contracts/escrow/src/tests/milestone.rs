extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneStatusUpdate, Roles, Trustline};
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
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
            approved: false,
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
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
    escrow_client.approve_milestone(&0, &approver_address);
    let after_approval = escrow_client.get_escrow();
    assert!(after_approval.milestones.get(0).unwrap().approved);

    // Build updated properties with a new milestone appended (unapproved)
    let updated_milestones = vec![
        &env,
        after_approval.milestones.get(0).unwrap(),
        after_approval.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
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
    assert!(
        final_escrow.milestones.get(0).unwrap().approved,
        "Existing approved milestone should remain approved"
    );
    assert_eq!(
        final_escrow.milestones.get(1).unwrap(),
        after_approval.milestones.get(1).unwrap()
    );
    assert!(
        !final_escrow.milestones.get(2).unwrap().approved,
        "Appended milestone should start unapproved"
    );
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
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
    escrow_approver.approve_milestone(&(0), &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved);

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

    let result = escrow_approver.try_approve_milestone(&10u32, &approver_address);
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

    // Unauthorized approver should fail
    let result = escrow_approver.try_approve_milestone(&(0u32), &unauthorized_address);
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
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 3"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Evidence 2"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
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
