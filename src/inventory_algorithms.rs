use crate::event_replay::Timed;
use crate::foundation::*;
use crate::inventory::*;

pub fn timed_allocations_total(allocations: &[Allocation]) -> DomainResult<Timed<Quantity>> {
    Ok(Timed::new(
        allocations_total(allocations)?,
        allocations.len() as Nat,
    ))
}
