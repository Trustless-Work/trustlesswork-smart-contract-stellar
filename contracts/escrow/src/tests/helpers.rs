extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use soroban_sdk::{token, xdr, Address, BytesN, Env};
use std::rc::Rc;
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

pub fn create_usdc_token<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    let admin_client = TokenAdminClient::new(e, &sac.address());
    add_fee_account_trustline(e, &admin_client);
    (TokenClient::new(e, &sac.address()), admin_client)
}

// The hardcoded fee address is a G-account: it must exist and trust the asset
// before it can receive fees.
fn add_fee_account_trustline(e: &Env, token: &TokenAdminClient) {
    let address = crate::modules::fee::trustless_work_address(e);
    let xdr::ScAddress::Account(account_id) = xdr::ScAddress::from(&address) else {
        unreachable!()
    };
    let key = Rc::new(xdr::LedgerKey::Account(xdr::LedgerKeyAccount {
        account_id: account_id.clone(),
    }));
    if e.host().get_ledger_entry(&key).unwrap().is_none() {
        let entry = Rc::new(xdr::LedgerEntry {
            data: xdr::LedgerEntryData::Account(xdr::AccountEntry {
                account_id,
                balance: 0,
                flags: 0,
                home_domain: Default::default(),
                inflation_dest: None,
                num_sub_entries: 0,
                seq_num: xdr::SequenceNumber(0),
                thresholds: xdr::Thresholds([1; 4]),
                signers: xdr::VecM::default(),
                ext: xdr::AccountEntryExt::V0,
            }),
            last_modified_ledger_seq: 0,
            ext: xdr::LedgerEntryExt::V0,
        });
        e.host().add_ledger_entry(&key, &entry, None).unwrap();
    }
    token.mock_all_auths().trust(&address);
}

pub struct TestData<'a> {
    pub client: EscrowContractClient<'a>,
}

pub fn create_escrow_contract<'a>(env: &Env, admin: &Address) -> TestData<'a> {
    env.mock_all_auths();
    let wasm_hash = BytesN::from_array(env, &[0u8; 32]);
    let client =
        EscrowContractClient::new(env, &env.register(EscrowContract {}, (admin, &wasm_hash)));
    TestData { client }
}
