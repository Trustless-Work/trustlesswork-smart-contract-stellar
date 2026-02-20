use crate::storage::types::Roles;

#[inline]
pub fn roles_equal_excluding_observers(existing: &Roles, new: &Roles) -> bool {
    existing.approver == new.approver
        && existing.service_provider == new.service_provider
        && existing.platform == new.platform
        && existing.release_signer == new.release_signer
        && existing.dispute_resolver == new.dispute_resolver
        && existing.receiver == new.receiver
}
