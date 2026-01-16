#![cfg(test)]

extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};

use soroban_sdk::{testutils::Address as _, token, vec, Address, Env, Map, String};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_usdc_token<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(e, &sac.address()),
        TokenAdminClient::new(e, &sac.address()),
    )
}

struct TestData<'a> {
    client: EscrowContractClient<'a>,
}

fn create_escrow_contract<'a>(env: &Env) -> TestData<'a> {
    env.mock_all_auths();
    let client = EscrowContractClient::new(env, &env.register(EscrowContract {}, ()));

    TestData { client }
}

#[test]
fn test_initialize_escrow_rejects_platform_fee_exceeding_aggregate_cap() {
    let env = Env::default();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "agg-cap");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline { address: usdc_token.0.address.clone() };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    // Platform fee that makes (platform + TW) > 100%. TW fee is 30 bps.
    let platform_fee_bps_over = (10_000 - 30) + 1;

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        platform_fee: platform_fee_bps_over,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    let res = client.try_initialize_escrow(&escrow_properties);
    assert!(res.is_err());
}

#[test]
fn test_change_escrow_rejects_platform_fee_exceeding_aggregate_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false, approved: false };

    let trustline: Trustline = Trustline { address: usdc_token.0.address.clone() };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    // Start with a valid platform fee
    let base_platform_fee = 3 * 100;
    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "E1"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        platform_fee: base_platform_fee,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Now attempt to change to an invalid platform fee (aggregate > 100%). TW fee is 30 bps.
    let over_platform_fee = (10_000 - 30) + 1;
    let updated_escrow_properties = Escrow {
        platform_fee: over_platform_fee,
        ..escrow_properties
    };

    let res = client.try_update_escrow(&platform_address, &updated_escrow_properties);
    assert!(res.is_err());
}

#[test]
fn test_initialize_escrow() {
    let env = Env::default();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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

    let escrow = escrow_approver.get_escrow();

    assert_eq!(escrow.engagement_id, engagement_id.clone());
    assert_eq!(escrow.roles.approver, escrow_properties.roles.approver);
    assert_eq!(
        escrow.roles.service_provider,
        escrow_properties.roles.service_provider
    );
    assert_eq!(
        escrow.roles.platform_address,
        escrow_properties.roles.platform_address
    );
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        escrow_properties.roles.dispute_resolver
    );

    let result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_update_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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

    let engagement_id = String::from_str(&env, "test_escrow_2");
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

    // Create a new updated escrow properties (no funds: can modify any field)
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles: roles.clone(),
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow =
        escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
    assert_eq!(escrow.platform_fee, updated_escrow_properties.platform_fee);
    assert_eq!(escrow.milestones, updated_escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        updated_escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        updated_escrow_properties.roles.dispute_resolver
    );
    for (i, _) in escrow.milestones.iter().enumerate() {
        assert_eq!(
            escrow.milestones.get(i as u32).unwrap().receiver,
            updated_escrow_properties.milestones.get(i as u32).unwrap().receiver
        );
    }
    assert_eq!(
        escrow.receiver_memo,
        updated_escrow_properties.receiver_memo
    );

    // Try to update escrow properties without platform address (should fail)
    let non_platform_address = Address::generate(&env);
    let result =
        escrow_approver.try_update_escrow(&non_platform_address, &updated_escrow_properties);
    assert!(result.is_err());
}

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
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false, approved: false };

    let trustline: Trustline = Trustline { address: token_client.address.clone() };

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
    assert_eq!(escrow.milestones.get(0).unwrap(), initial_escrow_properties.milestones.get(0).unwrap());
    assert_eq!(escrow.milestones.get(1).unwrap(), initial_escrow_properties.milestones.get(1).unwrap());
    // Non-milestone fields must remain unchanged
    assert_eq!(escrow.engagement_id, initial_escrow_properties.engagement_id);
    assert_eq!(escrow.title, initial_escrow_properties.title);
    assert_eq!(escrow.description, initial_escrow_properties.description);
    assert!(escrow.roles == initial_escrow_properties.roles);
    assert_eq!(escrow.platform_fee, initial_escrow_properties.platform_fee);
    assert!(escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(escrow.receiver_memo, initial_escrow_properties.receiver_memo);
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
        observers: vec![&env],
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

    // Verify milestone status change
    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));

    // Change milestone approved_flag (valid case)
    let milestone_indexes = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indexes, &approver_address);

    // Verify milestone approved_flag change
    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved);

    // Invalid index test
    let invalid_index = 10;

    // Test for `change_status` with invalid index
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

    // Test for `change_approved_flag` with invalid index
    let invalid_indices = vec![&env, invalid_index];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err());

    // Test only authorized party can perform the function
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

    // Test for `change_approved_flag` by invalid approver
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

    // Test with negative index
    let negative_update = vec![
        &env,
        MilestoneUpdate {
            index: -1,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &negative_update,
        &service_provider_address,
    );
    assert!(result.is_err());

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
fn test_release_milestone_funds_successful() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");
    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let initial_contract_balance = usdc_token.0.balance(&escrow_approver.address);

    // Approve the milestone before releasing funds
    let milestone_indexes = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indexes, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &(0));

    let total_amount = milestones.get(0).unwrap().amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * (platform_fee as i128)) / (10000 as i128);
    let service_provider_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        service_provider_amount,
        "Service Provider received incorrect amount"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        initial_contract_balance - total_amount,
        "Contract balance is incorrect after claiming earnings"
    );
}

// // //test claim escrow earnings in failure scenarios
// // // Scenario 1: Escrow with no milestones:

#[test]
fn test_release_milestone_funds_no_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: vec![&env],
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    let init_result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(
        init_result.is_err(),
        "Initialization should fail when no milestones are defined"
    );
}

// // // Scenario 2: Milestones incomplete
#[test]
fn test_release_milestone_funds_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Try to claim earnings with incomplete milestones (should fail)
    let result = escrow_approver.try_release_milestone_funds(
        &release_signer_address,
        &platform_address,
        &(0),
    );
    assert!(
        result.is_err(),
        "Should fail when milestones are not completed"
    );
    assert!(
        result.is_err(),
        "Should fail when milestones are not completed"
    );
}

#[test]
fn test_release_milestone_funds_same_receiver_as_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    // Use service_provider_address as receiver to test same-address case
    let _receiver_address = service_provider_address.clone();

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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
            status: String::from_str(&env, "Completed"),
            amount,
            evidence: String::from_str(&env, "Initial evidence"),
            flags,
            receiver: _receiver_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
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

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    // Approve before release
    let milestone_indexes = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indexes, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &0);

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
        usdc_token.0.balance(&platform_address),
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
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    // Create a valid but separate receiver address
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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
            status: String::from_str(&env, "Completed"),
            amount,
            evidence: String::from_str(&env, "Initial evidence"),
            flags,
            receiver: _receiver_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
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

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    // Approve before release
    let milestone_indexes = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indexes, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &0);

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
        usdc_token.0.balance(&platform_address),
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
fn test_dispute_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "test_dispute");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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
        escrow_approver.try_release_milestone_funds(&release_signer_address, &platform_address, &0);
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
    let platform_address = Address::generate(&env);
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
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
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
    escrow_approver.dispute_milestone(&(0 as i128), &approver_address);

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
        usdc_token.0.balance(&platform_address),
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
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
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
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
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
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
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
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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

    usdc_token
        .1
        .mint(&release_signer_address, &(amount as i128));

    let amount_to_deposit: i128 = 100_000_000;

    escrow_approver.fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    let expected_result_amount: i128 = 100_000_000;

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        expected_result_amount,
        "Escrow balance is incorrect"
    );
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let signer_funds: i128 = 100_000;
    usdc_token
        .1
        .mint(&release_signer_address, &(signer_funds as i128));

    let amount_to_deposit: i128 = 180_000;

    let result = escrow_approver.try_fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    assert!(
        result.is_err(),
        "Should fail when the signer has insufficient funds"
    );
}

#[test]
fn test_fund_escrow_dispute_flag_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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

    let amount_to_deposit: i128 = 80_000;

    let result = escrow_approver.try_fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    assert!(
        result.is_err(),
        "Should fail when the dispute approved_flag is true"
    );
}

#[test]
fn test_dispute_milestone() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
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
        observers: vec![&env],
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

    escrow_approver.dispute_milestone(&(0 as i128), &approver_address);

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

    let result = escrow_approver.try_dispute_milestone(&(5 as i128), &approver_address);
    assert!(result.is_err(), "Should fail with invalid milestone index");

    let result = escrow_approver.try_dispute_milestone(&(0 as i128), &approver_address);
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
    let platform_address = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address,
        release_signer,
        dispute_resolver,
        observers: vec![&env],
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
        observers: vec![&env],
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
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
    let milestone_indexes_1 = vec![&env, 1];
    client.approve_milestones(&milestone_indexes_1, &approver);
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
        observers: vec![&env],
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
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
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
        observers: vec![&env],
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
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
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
        observers: vec![&env],
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
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
    let milestone_indexes_1 = vec![&env, 1];
    client.approve_milestones(&milestone_indexes_1, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &1);

    assert_eq!(usdc.0.balance(&client.address), 0);

    // With empty distributions total == 0, we now expect an error (TotalAmountCannotBeZero)
    let dist: Map<Address, i128> = Map::new(&env);
    let res = client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert!(res.is_err(), "Expected error when total distribution amount is zero");
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
            observers: vec![&env],
        };
        let flags = Flags { disputed: false, released: false, resolved: false, approved: false };
        let trustline = Trustline { address: token_client.address.clone() };

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
        let milestone_indexes = vec![&env, 0];
        client.approve_milestones(&milestone_indexes, &approver);
        let after_approval = client.get_escrow();
        let approved_milestone = after_approval.milestones.get(0).unwrap();
        assert!(approved_milestone.flags.approved, "Milestone should be approved before update");

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
        assert!(final_escrow.milestones.get(0).unwrap().flags.approved, "Existing milestone approval flag must remain true");
        let appended = final_escrow.milestones.get(1).unwrap();
        assert!(!appended.flags.approved && !appended.flags.released && !appended.flags.resolved && !appended.flags.disputed, "New milestone flags must all be false");
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
            observers: vec![&env],
        };
        let flags = Flags { disputed: false, released: false, resolved: false, approved: false };
        let trustline = Trustline { address: token_client.address.clone() };

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
        let milestone_indexes = vec![&env, 0];
        client.approve_milestones(&milestone_indexes, &approver);
        client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

        // Verify released flag
        let after_release = client.get_escrow();
        let released_milestone = after_release.milestones.get(0).unwrap();
        assert!(released_milestone.flags.released, "Milestone should be released before update");

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
        assert!(res.is_ok(), "Update should succeed when appending after a milestone was released");

        let final_escrow = client.get_escrow();
        assert_eq!(final_escrow.milestones.len(), 2);
        assert!(final_escrow.milestones.get(0).unwrap().flags.released, "Existing milestone released flag must remain true");
        let appended = final_escrow.milestones.get(1).unwrap();
        assert!(!appended.flags.approved && !appended.flags.released && !appended.flags.resolved && !appended.flags.disputed, "New milestone flags must all be false");
    }

#[test]
fn test_approve_multiple_milestones_at_once() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
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

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 1"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 2"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 3"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 4"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 4"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "test_multiple_approval"),
        title: String::from_str(&env, "Test Multiple Approval"),
        description: String::from_str(&env, "Testing approval of multiple milestones at once"),
        roles: roles.clone(),
        platform_fee: 300, // 3%
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    token_admin.mint(&client.address, &100_000);

    // Verify initial state - no milestones approved
    let initial_escrow = client.get_escrow();
    assert!(!initial_escrow.milestones.get(0).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(1).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(2).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(3).unwrap().flags.approved);

    // Test 1: Approve first two milestones at once
    let indices_batch_1 = vec![&env, 0, 1];
    client.approve_milestones(&indices_batch_1, &approver);

    let after_first_batch = client.get_escrow();
    assert!(after_first_batch.milestones.get(0).unwrap().flags.approved, "Milestone 0 should be approved");
    assert!(after_first_batch.milestones.get(1).unwrap().flags.approved, "Milestone 1 should be approved");
    assert!(!after_first_batch.milestones.get(2).unwrap().flags.approved, "Milestone 2 should not be approved yet");
    assert!(!after_first_batch.milestones.get(3).unwrap().flags.approved, "Milestone 3 should not be approved yet");

    // Test 2: Approve remaining milestones at once
    let indices_batch_2 = vec![&env, 2, 3];
    client.approve_milestones(&indices_batch_2, &approver);

    let after_second_batch = client.get_escrow();
    assert!(after_second_batch.milestones.get(0).unwrap().flags.approved);
    assert!(after_second_batch.milestones.get(1).unwrap().flags.approved);
    assert!(after_second_batch.milestones.get(2).unwrap().flags.approved, "Milestone 2 should now be approved");
    assert!(after_second_batch.milestones.get(3).unwrap().flags.approved, "Milestone 3 should now be approved");

    // Test 3: Try to approve already approved milestones (should fail)
    let already_approved_indices = vec![&env, 0, 1];
    let result = client.try_approve_milestones(&already_approved_indices, &approver);
    assert!(result.is_err(), "Should fail when trying to approve already approved milestones");

    // Test 4: Try to approve with negative index (should fail)
    let negative_indices = vec![&env, -1];
    let result = client.try_approve_milestones(&negative_indices, &approver);
    assert!(result.is_err(), "Should fail with negative index");

    // Test 5: Try to approve with non-existent index (should fail)
    let invalid_indices = vec![&env, 10];
    let result = client.try_approve_milestones(&invalid_indices, &approver);
    assert!(result.is_err(), "Should fail with non-existent index");

    // Test 6: Try to approve multiple milestones where one is invalid (should fail and not approve any)
    // Create a new escrow for this test
    let test_data_2 = create_escrow_contract(&env);
    let client_2 = test_data_2.client;
    client_2.initialize_escrow(&escrow_properties);

    let mixed_indices = vec![&env, 0, 1, 99]; // 99 doesn't exist
    let result = client_2.try_approve_milestones(&mixed_indices, &approver);
    assert!(result.is_err(), "Should fail when any index in the batch is invalid");

    // Verify that NONE of the milestones were approved (atomic operation)
    let after_failed_attempt = client_2.get_escrow();
    assert!(!after_failed_attempt.milestones.get(0).unwrap().flags.approved, "Milestone 0 should not be approved after failed batch");
    assert!(!after_failed_attempt.milestones.get(1).unwrap().flags.approved, "Milestone 1 should not be approved after failed batch");

    // Test 7: Try to approve with unauthorized address (should fail)
    let unauthorized = Address::generate(&env);
    let valid_indices = vec![&env, 0];
    let result = client_2.try_approve_milestones(&valid_indices, &unauthorized);
    assert!(result.is_err(), "Should fail when approver is not authorized");

    // Test 8: Approve all remaining milestones at once in client_2
    let all_indices = vec![&env, 0, 1, 2, 3];
    client_2.approve_milestones(&all_indices, &approver);

    let final_escrow = client_2.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(1).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(2).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(3).unwrap().flags.approved);
}

#[test]
fn test_milestone_index_wraparound_prevention() {
    let env = Env::default();
    let approver = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "wraparound-test");

    let roles: Roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        approved: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 0"),
            amount: 1000,
            receiver: approver.clone(),
            status: String::from_str(&env, "pending"),
            flags: flags.clone(),
            evidence: String::from_str(&env, ""),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            amount: 2000,
            receiver: approver.clone(),
            status: String::from_str(&env, "pending"),
            flags: flags.clone(),
            evidence: String::from_str(&env, ""),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Testing wraparound prevention"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Test 1: Try to approve with an index that would wrap around (2^32 + 0 = 0 after truncation)
    let wraparound_index_zero: i128 = (u32::MAX as i128) + 1; // This would wrap to 0 if using unsafe casting
    let wraparound_indices = vec![&env, wraparound_index_zero];
    let result = client.try_approve_milestones(&wraparound_indices, &approver);
    assert!(
        result.is_err(),
        "Should fail with index that would wrap around to 0"
    );

    // Test 2: Try to approve with an index that would wrap around (2^32 + 1 = 1 after truncation)
    let wraparound_index_one: i128 = (u32::MAX as i128) + 2; // This would wrap to 1 if using unsafe casting
    let wraparound_indices = vec![&env, wraparound_index_one];
    let result = client.try_approve_milestones(&wraparound_indices, &approver);
    assert!(
        result.is_err(),
        "Should fail with index that would wrap around to 1"
    );

    // Test 3: Try to update milestone status with wraparound index
    let wraparound_update = vec![
        &env,
        MilestoneUpdate {
            index: wraparound_index_zero,
            status: String::from_str(&env, "completed"),
            evidence: None,
        },
    ];
    let result = client.try_change_milestone_status(&wraparound_update, &service_provider_address);
    assert!(
        result.is_err(),
        "Should fail when updating milestone with wraparound index"
    );

    // Test 4: Try to dispute with wraparound index
    let result = client.try_dispute_milestone(&wraparound_index_zero, &approver);
    assert!(
        result.is_err(),
        "Should fail when disputing milestone with wraparound index"
    );

    // Test 5: Verify that legitimate indices still work
    let valid_indices = vec![&env, 0, 1];
    client.approve_milestones(&valid_indices, &approver);
    let escrow = client.get_escrow();
    assert!(escrow.milestones.get(0).unwrap().flags.approved);
    assert!(escrow.milestones.get(1).unwrap().flags.approved);

    // Test 6: Try with index equal to i128::MAX (should fail)
    let max_index = i128::MAX;
    let max_indices = vec![&env, max_index];
    let result = client.try_approve_milestones(&max_indices, &approver);
    assert!(
        result.is_err(),
        "Should fail with i128::MAX as index"
    );
}
