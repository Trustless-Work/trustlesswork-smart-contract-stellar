#![cfg(test)]

extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};

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

fn create_escrow_contract<'a>(env: &Env) -> TestData {
    env.mock_all_auths();
    let client = EscrowContractClient::new(env, &env.register(EscrowContract {}, ()));

    TestData { client }
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
    escrow_approver.change_milestone_status(
        &0, // Milestone index
        &new_status,
        &new_evidence,
        &service_provider_address,
    );

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
    let result = escrow_approver.try_change_milestone_status(
        &invalid_index,
        &new_status,
        &new_evidence,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` with invalid index
    let result = escrow_approver.try_approve_milestone(&invalid_index, &approver_address);
    assert!(result.is_err());

    // Test only authorized party can perform the function
    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let result = escrow_approver.try_change_milestone_status(
        &(0),
        &new_status,
        &new_evidence,
        &unauthorized_address,
    );
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
    escrow_approver.approve_milestone(&0, &approver_address);
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
    escrow_approver.approve_milestone(&0, &approver_address);
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
    escrow_approver.approve_milestone(&0, &approver_address);
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
    client.approve_milestone(&0, &approver);
    client.approve_milestone(&1, &approver);
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
    client.approve_milestone(&0, &approver);
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
    client.approve_milestone(&0, &approver);
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
    client.approve_milestone(&0, &approver);
    client.approve_milestone(&1, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &1);

    assert_eq!(usdc.0.balance(&client.address), 0);

    // With empty distributions total == 0, we now expect an error (TotalAmountCannotBeZero)
    let dist: Map<Address, i128> = Map::new(&env);
    let res = client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert!(res.is_err(), "Expected error when total distribution amount is zero");
}
