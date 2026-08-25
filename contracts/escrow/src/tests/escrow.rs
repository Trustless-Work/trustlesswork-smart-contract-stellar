extern crate std;

use crate::error::{EscrowError, ReleaseError};
use crate::events::handler::EscrowUpdated;
use crate::storage::types::{Dispute, Escrow, Milestone, MilestoneApprovals, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, testutils::Events as _, vec, Address, Env, Event as _, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

fn overflow_milestone(env: &Env, receiver: &Address, amount: i128) -> Milestone {
    Milestone {
        description: String::from_str(env, "Overflow milestone"),
        status: String::from_str(env, "Pending"),
        evidence: String::from_str(env, ""),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![env],
        },
        amount,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(env, ""),
            resolved: false,
        },
        released: false,
        receiver: receiver.clone(),
    }
}

#[test]
fn test_initialize_excrow() {
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

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

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
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let escrow_approver = test_data.client;

    let initialized_escrow = escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);
    assert_eq!(escrow.roles.approvers, escrow_properties.roles.approvers);
    assert_eq!(
        escrow.roles.service_providers,
        escrow_properties.roles.service_providers
    );
    assert_eq!(escrow.roles.platform, escrow_properties.roles.platform);
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
    let receiver_address = Address::generate(&env);

    let platform_fee = 3 * 100;

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

    let usdc_token = create_usdc_token(&env, &admin);

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

    let test_data = create_escrow_contract(&env, &escrow_admin);
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
            description: String::from_str(&env, "Second milestone updated"),
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

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles,
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow = escrow_approver.update_escrow(&escrow_admin, &updated_escrow_properties);

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
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
        escrow.receiver_memo,
        updated_escrow_properties.receiver_memo
    );

    // Try to update escrow properties without admin address (should fail)
    let non_admin = Address::generate(&env);
    let result = escrow_approver.try_update_escrow(&non_admin, &updated_escrow_properties);
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
    let receiver_address = Address::generate(&env);

    let platform_fee_valid = 50 * 100; // 50%
    let platform_fee_invalid = 100 * 100; // 100% (should fail because cap is 99%)

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
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
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let initial_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        platform_fee: platform_fee_valid,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&initial_escrow);

    // Attempt invalid update (no funds path so full modification allowed but platform_fee cap enforced)
    let invalid_update: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
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
    let receiver_address = Address::generate(&env);

    let platform_fee_invalid = 100 * 100; // 100%

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
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
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let invalid_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_invalid_init"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles,
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
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
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 1_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver_address.clone(),
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
                admin: escrow_admin,
                observers: vec![&env],
            },
            platform_fee: 300,
            milestones: milestones.clone(),
            trustline: trustline.clone(),
            receiver_memo: 0,
        }
    };

    // admin == approver must fail
    let test_data = create_escrow_contract(&env, &approver_address);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(approver_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == approver");

    // admin == service_provider must fail
    let test_data = create_escrow_contract(&env, &service_provider_address);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(service_provider_address.clone()));
    assert!(
        res.is_err(),
        "Init must fail when admin == service_provider"
    );

    // admin == release_signer must fail
    let test_data = create_escrow_contract(&env, &release_signer_address);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(release_signer_address.clone()));
    assert!(res.is_err(), "Init must fail when admin == release_signer");

    // admin == dispute_resolver must fail
    let test_data = create_escrow_contract(&env, &dispute_resolver_address);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(dispute_resolver_address.clone()));
    assert!(
        res.is_err(),
        "Init must fail when admin == dispute_resolver"
    );

    // distinct admin must succeed
    let escrow_admin = Address::generate(&env);
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&make_escrow(escrow_admin));
    assert!(res.is_ok(), "Init must succeed with distinct admin");
}

#[test]
fn test_role_limit_exceeded() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestone = Milestone {
        description: String::from_str(&env, "M1"),
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
    };

    let make_escrow = |approvers: soroban_sdk::Vec<Address>| Escrow {
        engagement_id: String::from_str(&env, "role_limit"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers,
            service_providers: vec![&env, Address::generate(&env)],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, milestone.clone()],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    // 5 approvers must succeed
    let five_approvers = vec![
        &env,
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data
        .client
        .try_initialize_escrow(&make_escrow(five_approvers));
    assert!(res.is_ok(), "Exactly 5 approvers must succeed");

    // 6 approvers must fail
    let six_approvers = vec![
        &env,
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data
        .client
        .try_initialize_escrow(&make_escrow(six_approvers));
    assert!(res.is_err(), "6 approvers must fail with RoleLimitExceeded");
}

#[test]
fn test_duplicate_address_in_role() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let approver = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestone = Milestone {
        description: String::from_str(&env, "M1"),
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
    };

    let escrow = Escrow {
        engagement_id: String::from_str(&env, "dup_role"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone(), approver.clone()], // duplicate!
            service_providers: vec![&env, Address::generate(&env)],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, milestone],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data.client.try_initialize_escrow(&escrow);
    assert!(res.is_err(), "Duplicate address in role must fail");
}

#[test]
fn test_dispute_resolver_role_overlap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let shared = Address::generate(&env); // will be both dispute_resolver and approver
    let release_signer = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestone = Milestone {
        description: String::from_str(&env, "M1"),
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
    };

    let make_escrow = |dispute_resolvers: soroban_sdk::Vec<Address>| Escrow {
        engagement_id: String::from_str(&env, "resolver_overlap"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, shared.clone()],
            service_providers: vec![&env, Address::generate(&env)],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers,
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, milestone.clone()],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    // dispute_resolver == approver must fail
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data
        .client
        .try_initialize_escrow(&make_escrow(vec![&env, shared.clone()]));
    assert!(
        res.is_err(),
        "dispute_resolver overlapping with approver must fail"
    );

    // distinct dispute_resolver must succeed
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data
        .client
        .try_initialize_escrow(&make_escrow(vec![&env, Address::generate(&env)]));
    assert!(res.is_ok(), "Non-overlapping dispute_resolver must succeed");
}

#[test]
fn test_initialize_escrow_without_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "no-milestones-init"),
        title: String::from_str(&env, "No Milestones Escrow"),
        description: String::from_str(&env, "Escrow initialized without milestones"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: platform.clone(),
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver.clone()],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let result = test_data.client.try_initialize_escrow(&escrow_properties);
    assert!(
        result.is_ok(),
        "Should be able to initialize escrow without milestones"
    );

    let escrow = test_data.client.get_escrow();
    assert!(escrow.milestones.is_empty());
}

#[test]
fn test_initialize_escrow_returns_overflow_for_milestone_total() {
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
    let usdc_token = create_usdc_token(&env, &admin);

    let escrow = Escrow {
        engagement_id: String::from_str(&env, "overflow-init"),
        title: String::from_str(&env, "Overflow init"),
        description: String::from_str(&env, ""),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider],
            platform,
            release_signers: vec![&env, release_signer],
            dispute_resolvers: vec![&env, dispute_resolver],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![
            &env,
            overflow_milestone(&env, &receiver, i128::MAX),
            overflow_milestone(&env, &receiver, 1),
        ],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    let result = client.try_initialize_escrow(&escrow);

    assert!(matches!(result, Err(Ok(EscrowError::Overflow))));
}

#[test]
fn test_release_funds_returns_overflow_for_milestone_total() {
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
    let trustless_work = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);

    let escrow = Escrow {
        engagement_id: String::from_str(&env, "overflow-release"),
        title: String::from_str(&env, "Overflow release"),
        description: String::from_str(&env, ""),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider],
            platform,
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, dispute_resolver],
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, overflow_milestone(&env, &receiver, i128::MAX)],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin).client;
    client.initialize_escrow(&escrow);

    client.manage_milestones(
        &escrow_admin,
        &vec![&env, overflow_milestone(&env, &receiver, 1)],
        &vec![&env],
    );
    client.approve_milestones(&vec![&env, 0, 1], &approver);

    let result = client.try_release_funds(&release_signer, &trustless_work, &vec![&env, 0, 1]);

    assert!(matches!(result, Err(Ok(ReleaseError::Overflow))));
}

#[test]
fn test_dispute_resolver_cannot_equal_platform() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let shared_address = Address::generate(&env); // will be both platform and dispute_resolver
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestone = Milestone {
        description: String::from_str(&env, "M1"),
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
    };

    let escrow = Escrow {
        engagement_id: String::from_str(&env, "platform_resolver_overlap"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: shared_address.clone(), // same as dispute_resolver
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, shared_address.clone()], // same as platform
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, milestone.clone()],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    // dispute_resolver == platform must fail
    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data.client.try_initialize_escrow(&escrow);
    assert!(
        matches!(res, Err(Ok(EscrowError::DisputeResolverOverlapsWithOtherRole))),
        "dispute_resolver == platform must be rejected with DisputeResolverOverlapsWithOtherRole"
    );

    // distinct dispute_resolver and platform must succeed
    let distinct_dispute_resolver = Address::generate(&env);
    let escrow_valid = Escrow {
        engagement_id: String::from_str(&env, "platform_resolver_distinct"),
        title: String::from_str(&env, "Test"),
        description: String::from_str(&env, "Test"),
        roles: Roles {
            approvers: vec![&env, approver.clone()],
            service_providers: vec![&env, service_provider.clone()],
            platform: shared_address.clone(), // different from dispute_resolver
            release_signers: vec![&env, release_signer.clone()],
            dispute_resolvers: vec![&env, distinct_dispute_resolver], // different from platform
            admin: escrow_admin.clone(),
            observers: vec![&env],
        },
        platform_fee: 0,
        milestones: vec![&env, milestone],
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let res = test_data.client.try_initialize_escrow(&escrow_valid);
    assert!(res.is_ok(), "Non-overlapping platform and dispute_resolver must succeed");
}

#[test]
fn test_update_escrow_event_reports_changed_fields() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let platform = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
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

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "update_escrow_event_test");
    let initial_escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Original Title"),
        description: String::from_str(&env, "Original Description"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env, &escrow_admin);
    let client = test_data.client;
    client.initialize_escrow(&initial_escrow);

    // Only title and description change; everything else is passed back
    // unchanged (milestones/roles/trustline/platform_fee/receiver_memo).
    let updated_escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "New Title"),
        description: String::from_str(&env, "New Description"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    client.update_escrow(&escrow_admin, &updated_escrow);

    let expected_event = EscrowUpdated {
        engagement_id: engagement_id.clone(),
        admin: escrow_admin.clone(),
        changed_fields: vec![
            &env,
            String::from_str(&env, "title"),
            String::from_str(&env, "description"),
        ],
    };

    assert_eq!(
        env.events().all(),
        std::vec![expected_event.to_xdr(&env, &client.address)],
        "EscrowUpdated should report exactly the fields that changed, in field-declaration order"
    );
}
