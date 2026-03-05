extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_append_milestones_with_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
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
        address: token_client.address.clone(),
    };

    let initial_milestones = vec![
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

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow (contract will hold funds)
    token_admin.mint(&release_signer_address, &amount);
    escrow_approver.fund_escrow(&release_signer_address, &initial_escrow_properties, &amount);

    // Now attempt to append new milestones while funds exist
    let updated_milestones = vec![
        &env,
        initial_escrow_properties.milestones.get(0).unwrap(),
        initial_escrow_properties.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 200_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

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
    // Non-milestone fields must remain unchanged
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
fn test_change_milestone_status_and_approved_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
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
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Change milestone status (valid case)
    let new_status = String::from_str(&env, "completed");
    let new_evidence = Some(String::from_str(&env, "New evidence"));
    let milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: new_status.clone(),
            evidence: new_evidence.clone(),
        },
    ];
    escrow_approver.change_milestone_status(&milestone_updates, &service_provider_address);

    // Verify milestone status change
    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, new_status);

    // Change milestone approved_flag (valid case)
    escrow_approver.approve_milestone(&(0), &approver_address);

    // Verify milestone approved_flag change
    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved);

    // Invalid index test
    let invalid_index = 10;
    let new_status = String::from_str(&env, "completed");

    // Test for `change_status` with invalid index
    let invalid_milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: invalid_index,
            status: new_status.clone(),
            evidence: new_evidence.clone(),
        },
    ];
    let result = escrow_approver
        .try_change_milestone_status(&invalid_milestone_updates, &service_provider_address);
    assert!(result.is_err());

    // Test for `change_approved_flag` with invalid index
    let result = escrow_approver.try_approve_milestone(&invalid_index, &approver_address);
    assert!(result.is_err());

    // Test only authorized party can perform the function
    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let valid_milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: new_status.clone(),
            evidence: new_evidence.clone(),
        },
    ];
    let result = escrow_approver
        .try_change_milestone_status(&valid_milestone_updates, &unauthorized_address);
    assert!(result.is_err());

    // Test for `change_approved_flag` by invalid approver
    let result = escrow_approver.try_approve_milestone(&(0), &unauthorized_address);
    assert!(result.is_err());

    //Escrow Test with no milestone
    let escrow_properties_v2: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: vec![&env],
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let new_escrow_approver = test_data.client;

    let init_result = new_escrow_approver.try_initialize_escrow(&escrow_properties_v2);
    assert!(
        init_result.is_err(),
        "Initialization should fail when no milestones are defined"
    );
}

#[test]
fn test_update_after_milestone_approved_append_new() {
    // Scenario: After approving an existing milestone (flags.approved = true),
    // we should still be able to append new milestones whose flags are all false.
    // Existing milestone flags must match exactly; new milestone flags must be false.
    let env = Env::default();
    env.mock_all_auths();

    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

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
        address: token_client.address.clone(),
    };

    let initial_milestones = vec![
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
        engagement_id: String::from_str(&env, "eng-approved-update"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee: 300, // 3%
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Approve the existing milestone -> flags.approved = true
    client.approve_milestone(&0, &approver);
    let after_approval = client.get_escrow();
    let approved_milestone = after_approval.milestones.get(0).unwrap();
    assert!(
        approved_milestone.flags.approved,
        "Milestone should be approved before update"
    );

    // Build updated escrow properties: keep existing milestone (with approved flag), append a new one with all flags false.
    let new_milestone = Milestone {
        description: String::from_str(&env, "m2"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, "e"),
        amount: 150_000,
        flags: flags.clone(), // all false
        receiver: service_provider.clone(),
    };
    let updated_milestones = vec![&env, approved_milestone.clone(), new_milestone.clone()];

    let updated_escrow = Escrow {
        engagement_id: esc.engagement_id.clone(),
        title: esc.title.clone(),
        description: esc.description.clone(),
        roles: esc.roles.clone(),
        platform_fee: esc.platform_fee, // unchanged
        milestones: updated_milestones.clone(),
        trustline: esc.trustline.clone(),
        receiver_memo: esc.receiver_memo,
    };

    // Perform update
    let res = client.try_update_escrow(&platform, &updated_escrow);
    assert!(res.is_ok(), "Update should succeed when appending new milestone with flags false while keeping existing approved milestone flags unchanged");

    let final_escrow = client.get_escrow();
    assert_eq!(final_escrow.milestones.len(), 2);
    assert!(
        final_escrow.milestones.get(0).unwrap().flags.approved,
        "Existing milestone approval flag must remain true"
    );
    let appended = final_escrow.milestones.get(1).unwrap();
    assert!(
        !appended.flags.approved
            && !appended.flags.released
            && !appended.flags.resolved
            && !appended.flags.disputed,
        "New milestone flags must all be false"
    );
}

#[test]
fn test_update_after_milestone_released_append_new() {
    // Scenario: After releasing an existing milestone (flags.released = true),
    // we should still be able to append new milestones whose flags are all false.
    // Existing milestone flags must match exactly; new milestone flags must be false.
    let env = Env::default();
    env.mock_all_auths();

    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_client, token_admin) = create_usdc_token(&env, &admin);

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
        address: token_client.address.clone(),
    };

    let amount: i128 = 100_000;
    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng-released-update"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee: 300, // 3%
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    // Fund contract and approve + release milestone 0
    token_admin.mint(&client.address, &amount);
    client.approve_milestone(&0, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    // Verify released flag
    let after_release = client.get_escrow();
    let released_milestone = after_release.milestones.get(0).unwrap();
    assert!(
        released_milestone.flags.released,
        "Milestone should be released before update"
    );

    // Build updated escrow properties: keep released milestone, append new one with all flags false
    let new_milestone = Milestone {
        description: String::from_str(&env, "m2"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, "e"),
        amount,
        flags: flags.clone(), // all false
        receiver: service_provider.clone(),
    };
    let updated_milestones = vec![&env, released_milestone.clone(), new_milestone.clone()];

    let updated_escrow = Escrow {
        engagement_id: esc.engagement_id.clone(),
        title: esc.title.clone(),
        description: esc.description.clone(),
        roles: esc.roles.clone(),
        platform_fee: esc.platform_fee,
        milestones: updated_milestones.clone(),
        trustline: esc.trustline.clone(),
        receiver_memo: esc.receiver_memo,
    };

    // Perform update
    let res = client.try_update_escrow(&platform, &updated_escrow);
    assert!(
        res.is_ok(),
        "Update should succeed when appending after a milestone was released"
    );

    let final_escrow = client.get_escrow();
    assert_eq!(final_escrow.milestones.len(), 2);
    assert!(
        final_escrow.milestones.get(0).unwrap().flags.released,
        "Existing milestone released flag must remain true"
    );
    let appended = final_escrow.milestones.get(1).unwrap();
    assert!(
        !appended.flags.approved
            && !appended.flags.released
            && !appended.flags.resolved
            && !appended.flags.disputed,
        "New milestone flags must all be false"
    );
}
