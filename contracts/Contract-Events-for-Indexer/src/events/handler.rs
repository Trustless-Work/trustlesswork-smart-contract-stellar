use crate::storage::types::Escrow;
use soroban_sdk::{symbol_short, vec, Env, IntoVal, String, Val};

// ------ Escrows
pub fn escrows_by_contract_id(e: &Env, escrow_id: String, escrow: Escrow) {
    let topics = (symbol_short!("p_by_spdr"),);

    let escrow_id_val: Val = escrow_id.into_val(e);
    let escrow_val: Val = escrow.into_val(e);

    let event_payload = vec![e, escrow_id_val, escrow_val];
    e.events().publish(topics, event_payload);
}
