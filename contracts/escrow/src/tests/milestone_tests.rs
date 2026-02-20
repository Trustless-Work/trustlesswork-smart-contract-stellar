#![cfg(test)]

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};
use crate::tests::helpers::{create_escrow_contract, create_usdc_token};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

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
        observers: vec![&env],
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

    // Change milestone status (valid case)
    let milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    escrow_approver.change_milestone_status(
        &milestone_updates,
        &service_provider_address,
    );

    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "New evidence")
    );

    // Change milestone approved (valid case)
    let milestone_indices = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved);

    let invalid_index = 10;

    let invalid_updates = vec![
        &env,
        MilestoneUpdate {
            index: invalid_index,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &invalid_updates,
        &service_provider_address,
    );
    assert!(result.is_err());

    let invalid_indices = vec![&env, invalid_index];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err());

    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let valid_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &valid_updates,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_approved` by invalid approver
    let valid_indices = vec![&env, 0];
    let result = escrow_approver.try_approve_milestones(&valid_indices, &unauthorized_address);
    assert!(result.is_err());

    // Test changing multiple milestones at once
    let multiple_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    
    escrow_approver.change_milestone_status(
        &multiple_updates,
        &service_provider_address,
    );

    let batch_updated_escrow = escrow_approver.get_escrow();
    assert_eq!(batch_updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(batch_updated_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(
        batch_updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
    assert_eq!(
        batch_updated_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );

    // Test with empty status
    let empty_status_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, ""),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &empty_status_update,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test with different status and evidence for each milestone
    let different_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "Evidence for milestone 0")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "in-progress"),
            evidence: None,
        },
    ];
    
    escrow_approver.change_milestone_status(
        &different_updates,
        &service_provider_address,
    );

    let final_check_escrow = escrow_approver.get_escrow();
    assert_eq!(final_check_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));
    assert_eq!(final_check_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "in-progress"));
    assert_eq!(
        final_check_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Evidence for milestone 0")
    );
    // Milestone 1 should keep its previous evidence since we passed None
    assert_eq!(
        final_check_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
}

#[test]
fn test_approve_multiple_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 1000;
    let platform_fee: u32 = 300;

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
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
        Milestone {
            description: String::from_str(&env, "Third milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let engagement_id = String::from_str(&env, "test-multiple-milestones");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform: platform.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
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
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Multiple Milestones"),
        description: String::from_str(&env, "Test approving multiple milestones at once"),
        roles,
        amount,
        platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &amount);

    // Test 1: Aprobar múltiples milestones a la vez (0 y 1)
    let milestone_indices = vec![&env, 0, 1];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let escrow_after_approval = escrow_approver.get_escrow();
    assert!(escrow_after_approval.milestones.get(0).unwrap().approved, "Milestone 0 should be approved");
    assert!(escrow_after_approval.milestones.get(1).unwrap().approved, "Milestone 1 should be approved");
    assert!(!escrow_after_approval.milestones.get(2).unwrap().approved, "Milestone 2 should not be approved");

    // Test 2: Aprobar el último milestone
    let milestone_indices = vec![&env, 2];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let escrow_after_all_approved = escrow_approver.get_escrow();
    assert!(escrow_after_all_approved.milestones.get(2).unwrap().approved, "Milestone 2 should be approved");

    // Test 3: Intentar aprobar con un índice que no existe (debe fallar)
    let invalid_indices = vec![&env, 10];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err(), "Should fail with non-existent index");

    // Test 4: Intentar aprobar múltiples índices donde uno es inválido (debe fallar)
    let mixed_indices = vec![&env, 0, 99];
    let result = escrow_approver.try_approve_milestones(&mixed_indices, &approver_address);
    assert!(result.is_err(), "Should fail when any index is invalid");
}
