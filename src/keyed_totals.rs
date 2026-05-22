use std::collections::BTreeMap;

use crate::foundation::*;
use crate::inventory::*;

pub type AllocationKey = (Nat, Nat);

#[must_use]
pub const fn allocation_key_for_total(allocation: &Allocation) -> AllocationKey {
    allocation_key(allocation)
}

pub fn allocation_key_support(allocations: &[Allocation]) -> Vec<AllocationKey> {
    let mut keys: Vec<_> = allocations.iter().map(allocation_key_for_total).collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

pub fn allocation_quantity_for_key(
    allocations: &[Allocation],
    key: AllocationKey,
) -> DomainResult<Quantity> {
    checked_sum(
        allocations
            .iter()
            .filter(|allocation| allocation_key_for_total(allocation) == key)
            .map(Allocation::quantity),
        "allocation_quantity_for_key",
    )
}

pub fn allocation_quantity_by_key(
    allocations: &[Allocation],
) -> DomainResult<BTreeMap<AllocationKey, Quantity>> {
    let mut totals = BTreeMap::new();
    for allocation in allocations {
        let key = allocation_key_for_total(allocation);
        let current = totals.get(&key).copied().unwrap_or(0);
        totals.insert(
            key,
            checked_add(current, allocation.quantity(), "allocation_quantity_by_key")?,
        );
    }
    Ok(totals)
}
