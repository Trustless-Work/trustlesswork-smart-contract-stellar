extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneApprovals, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_initialize_excrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let milestones = vec![
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

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(),
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
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones,
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    let initialized_escrow = escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);
    assert_eq!(escrow.roles.approvers, escrow_properties.roles.approvers);
    assert_eq!(
        escrow.roles.service_providers,
        escrow_properties.roles.service_providers
    );
    assert_eq!(
        escrow.roles.platform,
        escrow_properties.roles.platform
    );
    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signers,
        escrow_properties.roles.release_signers
    );
    assert_eq!(
        escrow.roles.dispute_resolvers,
        escrow_properties.roles.dispute_resolvers
    );
    assert_eq!(escrow.roles.receiver, escrow_properties.roles.receiver);
    assert_eq!(escrow.receiver_memo, escrow_properties.receiver_memo);

    let result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_update_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

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

    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_2");
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

    // Create a new updated escrow properties
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone updated"),
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
            description: String::from_str(&env, "Second milestone updated"),
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
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles,
        amount: amount * 2,
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow =
        escrow_approver.update_escrow(&escrow_admin, &updated_escrow_properties);

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
    assert_eq!(escrow.amount, updated_escrow_properties.amount);
    assert_eq!(escrow.platform_fee, updated_escrow_properties.platform_fee);
    assert_eq!(escrow.milestones, initial_milestones);
    assert_eq!(
        escrow.roles.release_signers,
        updated_escrow_properties.roles.release_signers
    );
    assert_eq!(
        escrow.roles.dispute_resolvers,
        updated_escrow_properties.roles.dispute_resolvers
    );
    assert_eq!(
        escrow.roles.receiver,
        updated_escrow_properties.roles.receiver
    );
    assert_eq!(
        escrow.receiver_memo,
        updated_escrow_properties.receiver_memo
    );

    // Try to update escrow properties without admin address (should fail)
    let non_admin = Address::generate(&env);
    let result =
        escrow_approver.try_update_escrow(&non_admin, &updated_escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_update_escrow_platform_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let amount: i128 = 10_000_000;
    let platform_fee_valid = 50 * 100; // 50%
    let platform_fee_invalid = 100 * 100; // 100% (should fail because cap is 99%)

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let (token_client, _admin_client) = create_usdc_token(&env, &admin);
    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let initial_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        amount,
        platform_fee: platform_fee_valid,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&initial_escrow);

    // Attempt invalid update (no funds path so full modification allowed but platform_fee cap enforced)
    let invalid_update: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        amount,
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let res = client.try_update_escrow(&escrow_admin, &invalid_update);
    assert!(
        res.is_err(),
        "Update should fail with platform fee > 99% cap"
    );
}

#[test]
fn test_initialize_escrow_platform_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let amount: i128 = 10_000_000;
    let platform_fee_invalid = 100 * 100; // 100%

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 100_000_000,
            released: false,
        },
    ];

    let (token_client, _admin_client) = create_usdc_token(&env, &admin);
    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let roles: Roles = Roles {
        approvers: vec![&env, approver_address.clone()],
        service_providers: vec![&env, service_provider_address.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer_address.clone()],
        dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
        receiver: service_provider_address.clone(),
        admin: escrow_admin.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let invalid_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_invalid_init"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles,
        amount,
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&invalid_escrow);
    assert!(
        res.is_err(),
        "Initialization should fail with platform fee > 99% cap"
    );
}

#[test]
fn test_admin_role_overlap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = Address::generate(&env);

    let (token_client, _admin_client) = create_usdc_token(&env, &admin);
    let trustline = Trustline {
        address: token_client.address.clone(),
    };
    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            approvals: MilestoneApprovals {
                quorum: 1,
                approval_count: 0,
                approvers: vec![&env],
            },
            amount: 1_000_000,
            released: false,
        },
    ];

    let make_escrow = |escrow_admin: Address| -> Escrow {
        Escrow {
            engagement_id: String::from_str(&env, "overlap_test"),
            title: String::from_str(&env, "Escrow"),
            description: String::from_str(&env, "Desc"),
            roles: Roles {
                approvers: vec![&env, approver_address.clone()],
                service_providers: vec![&env, service_provider_address.clone()],
                platform: platform.clone(),
                release_signers: vec![&env, release_signer_address.clone()],
                dispute_resolvers: vec![&env, dispute_resolver_address.clone()],
                receiver: receiver_address.clone(),
                admin: escrow_admin,
            },
            amount: 1_000_000,
            platform_fee: 300,
            milestones: milestones.clone(),
            flags: flags.clone(),
            trustline: trustline.clone(),
            receiver_memo: 0,
        }
    };

    // admin == approver must fail
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(approver_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == approver");

    // admin == service_provider must fail
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(service_provider_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == service_provider");

    // admin == release_signer must fail
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(release_signer_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == release_signer");

    // admin == receiver must fail
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(receiver_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == receiver");

    // admin == dispute_resolver must fail
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(dispute_resolver_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == dispute_resolver");

    // distinct admin must succeed
    let escrow_admin = Address::generate(&env);
    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(escrow_admin));
    assert!(res.is_ok(), "Init must succeed with distinct admin");
}
