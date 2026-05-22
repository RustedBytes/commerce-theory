use std::collections::HashSet;

use crate::foundation::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StockState {
    pub(crate) sku: Sku,
    pub(crate) total: Quantity,
    pub(crate) reserved: Quantity,
}

impl StockState {
    pub const fn try_new(sku: Sku, total: Quantity, reserved: Quantity) -> DomainResult<Self> {
        if reserved > total {
            return Err(ValidationError::Invariant("reserved stock exceeds total"));
        }
        Ok(Self {
            sku,
            total,
            reserved,
        })
    }

    #[must_use]
    pub const fn sku(&self) -> Sku {
        self.sku
    }

    #[must_use]
    pub const fn total(&self) -> Quantity {
        self.total
    }

    #[must_use]
    pub const fn reserved(&self) -> Quantity {
        self.reserved
    }
}

#[must_use]
pub const fn available_stock(s: &StockState) -> Quantity {
    nat_sub(s.total, s.reserved)
}

#[must_use]
pub const fn can_reserve(s: &StockState, q: Quantity) -> bool {
    q <= available_stock(s)
}

pub fn reserve_stock(s: &StockState, q: Quantity) -> DomainResult<StockState> {
    if !can_reserve(s, q) {
        return Err(ValidationError::Invariant(
            "reservation exceeds available stock",
        ));
    }
    StockState::try_new(s.sku, s.total, checked_add(s.reserved, q, "reserve_stock")?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VersionedStock {
    pub(crate) stock: StockState,
    pub(crate) version: Nat,
}

impl VersionedStock {
    pub fn try_new(
        sku: Sku,
        total: Quantity,
        reserved: Quantity,
        version: Nat,
    ) -> DomainResult<Self> {
        Ok(Self {
            stock: StockState::try_new(sku, total, reserved)?,
            version,
        })
    }

    #[must_use]
    pub const fn from_stock(stock: StockState, version: Nat) -> Self {
        Self { stock, version }
    }

    #[must_use]
    pub const fn stock(&self) -> StockState {
        self.stock
    }

    #[must_use]
    pub const fn version(&self) -> Nat {
        self.version
    }
}

pub fn reserve_versioned_stock(
    s: &VersionedStock,
    q: Quantity,
    expected_version: Nat,
) -> DomainResult<VersionedStock> {
    if expected_version != s.version {
        return Err(ValidationError::Invariant("stock version mismatch"));
    }
    Ok(VersionedStock {
        stock: reserve_stock(&s.stock, q)?,
        version: checked_add(s.version, 1, "reserve_versioned_stock")?,
    })
}

domain_struct! {
    pub struct Warehouse {
        id: Id,
        name: String,
    }
}

domain_struct! {
    pub struct BinLocation {
        warehouse: Warehouse,
        bin_id: Id,
    }
}

domain_struct! {
    pub struct BinStock {
        sku: Sku,
        location: BinLocation,
        quantity: Quantity,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PickTask {
    pub(crate) sku: Sku,
    pub(crate) requested: Quantity,
    pub(crate) bin: BinStock,
}

impl PickTask {
    pub fn try_new(sku: Sku, requested: Quantity, bin: BinStock) -> DomainResult<Self> {
        if requested > bin.quantity {
            return Err(ValidationError::Invariant("pick exceeds bin quantity"));
        }
        Ok(Self {
            sku,
            requested,
            bin,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PackTask {
    pub(crate) picked: Quantity,
    pub(crate) packed: Quantity,
}

impl PackTask {
    pub const fn try_new(
        source_quantity: Quantity,
        packed_quantity: Quantity,
    ) -> DomainResult<Self> {
        if packed_quantity > source_quantity {
            return Err(ValidationError::Invariant("packed exceeds picked"));
        }
        Ok(Self {
            picked: source_quantity,
            packed: packed_quantity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarehouseShipment {
    pub(crate) packed: Quantity,
    pub(crate) shipped: Quantity,
}

impl WarehouseShipment {
    pub const fn try_new(packed: Quantity, shipped: Quantity) -> DomainResult<Self> {
        if shipped > packed {
            return Err(ValidationError::Invariant("shipped exceeds packed"));
        }
        Ok(Self { packed, shipped })
    }
}

domain_struct! {
    pub struct InventoryNode {
        warehouse: Warehouse,
        stock: StockState,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Allocation {
    pub(crate) node: InventoryNode,
    pub(crate) quantity: Quantity,
}

impl Allocation {
    pub fn try_new(node: InventoryNode, quantity: Quantity) -> DomainResult<Self> {
        if quantity > available_stock(&node.stock) {
            return Err(ValidationError::Invariant(
                "allocation exceeds available stock",
            ));
        }
        Ok(Self { node, quantity })
    }

    #[must_use]
    pub const fn node(&self) -> &InventoryNode {
        &self.node
    }

    #[must_use]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }
}

pub fn allocations_total(allocations: &[Allocation]) -> DomainResult<Quantity> {
    checked_sum(allocations.iter().map(|a| a.quantity), "allocations_total")
}

pub fn allocations_available_total(allocations: &[Allocation]) -> DomainResult<Quantity> {
    checked_sum(
        allocations.iter().map(|a| available_stock(&a.node.stock)),
        "allocations_available_total",
    )
}

#[must_use]
pub const fn allocation_key(a: &Allocation) -> (Nat, Nat) {
    (a.node.warehouse.id, a.node.stock.sku.value())
}

#[must_use]
pub fn allocation_keys_distinct(allocations: &[Allocation]) -> bool {
    let mut seen = HashSet::new();
    allocations.iter().all(|a| seen.insert(allocation_key(a)))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FulfillmentPlan {
    pub(crate) requested: Quantity,
    pub(crate) allocations: Vec<Allocation>,
}

impl FulfillmentPlan {
    pub fn try_new(requested: Quantity, allocations: Vec<Allocation>) -> DomainResult<Self> {
        if allocations_total(&allocations)? != requested {
            return Err(ValidationError::Invariant(
                "allocations must exactly cover request",
            ));
        }
        Ok(Self {
            requested,
            allocations,
        })
    }

    #[must_use]
    pub const fn requested(&self) -> Quantity {
        self.requested
    }

    #[must_use]
    pub fn allocations(&self) -> &[Allocation] {
        &self.allocations
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistinctFulfillmentPlan {
    pub(crate) requested: Quantity,
    pub(crate) allocations: Vec<Allocation>,
}

impl DistinctFulfillmentPlan {
    pub fn try_new(requested: Quantity, allocations: Vec<Allocation>) -> DomainResult<Self> {
        if allocations_total(&allocations)? != requested {
            return Err(ValidationError::Invariant(
                "allocations must exactly cover request",
            ));
        }
        if !allocation_keys_distinct(&allocations) {
            return Err(ValidationError::Invariant(
                "allocation keys must be distinct",
            ));
        }
        Ok(Self {
            requested,
            allocations,
        })
    }
}

pub const fn release_reserved_stock(s: &StockState, q: Quantity) -> DomainResult<StockState> {
    if q > s.reserved {
        return Err(ValidationError::InventoryInvariantFailed);
    }
    StockState::try_new(s.sku, s.total, nat_sub(s.reserved, q))
}

pub const fn confirm_reserved_shipment(s: &StockState, q: Quantity) -> DomainResult<StockState> {
    if q > s.reserved {
        return Err(ValidationError::InventoryInvariantFailed);
    }
    StockState::try_new(s.sku, nat_sub(s.total, q), nat_sub(s.reserved, q))
}

#[must_use]
pub fn compare_and_swap_reserve(
    s: &VersionedStock,
    q: Quantity,
    expected_version: Nat,
) -> Option<VersionedStock> {
    reserve_versioned_stock(s, q, expected_version).ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReservationAttempt {
    pub(crate) stock: VersionedStock,
    pub(crate) quantity: Quantity,
    pub(crate) expected_version: Nat,
}

impl ReservationAttempt {
    #[must_use]
    pub const fn new(stock: VersionedStock, quantity: Quantity, expected_version: Nat) -> Self {
        Self {
            stock,
            quantity,
            expected_version,
        }
    }
}

#[must_use]
pub fn commit_reservation_attempt(attempt: &ReservationAttempt) -> Option<VersionedStock> {
    compare_and_swap_reserve(&attempt.stock, attempt.quantity, attempt.expected_version)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConcurrentReservationConflict {
    pub(crate) first: ReservationAttempt,
    pub(crate) second: ReservationAttempt,
}

impl ConcurrentReservationConflict {
    pub fn try_new(first: ReservationAttempt, second: ReservationAttempt) -> DomainResult<Self> {
        if first.stock.stock.sku != second.stock.stock.sku
            || first.expected_version != second.expected_version
        {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self { first, second })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReservationStatus {
    Active,
    Expired,
    Confirmed,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimedReservation {
    pub(crate) stock: StockState,
    pub(crate) quantity: Quantity,
    pub(crate) reserved_at: Timestamp,
    pub(crate) expires_at: Timestamp,
    pub(crate) status: ReservationStatus,
}

impl TimedReservation {
    pub fn try_new(
        stock: StockState,
        quantity: Quantity,
        reserved_at: Timestamp,
        expires_at: Timestamp,
        status: ReservationStatus,
    ) -> DomainResult<Self> {
        if expires_at < reserved_at || quantity > stock.reserved {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            stock,
            quantity,
            reserved_at,
            expires_at,
            status,
        })
    }
}

#[must_use]
pub fn reservation_expired_at(now: Timestamp, reservation: &TimedReservation) -> bool {
    reservation.expires_at < now
}

#[must_use]
pub fn reservation_active_at(now: Timestamp, reservation: &TimedReservation) -> bool {
    reservation.status == ReservationStatus::Active && now <= reservation.expires_at
}

pub fn release_expired_reservation(
    reservation: &TimedReservation,
    now: Timestamp,
) -> DomainResult<StockState> {
    if !reservation_expired_at(now, reservation) {
        return Err(ValidationError::InventoryInvariantFailed);
    }
    release_reserved_stock(&reservation.stock, reservation.quantity)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackorderRequest {
    pub(crate) sku: Sku,
    pub(crate) requested: Quantity,
    pub(crate) available_now: Quantity,
    pub(crate) backordered: Quantity,
}

impl BackorderRequest {
    pub fn try_new(
        sku: Sku,
        requested: Quantity,
        available_now: Quantity,
        backordered: Quantity,
    ) -> DomainResult<Self> {
        if requested != checked_add(available_now, backordered, "backorder quantity")? {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            sku,
            requested,
            available_now,
            backordered,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreorderWindow {
    pub(crate) sku: Sku,
    pub(crate) opens_at: Timestamp,
    pub(crate) closes_at: Timestamp,
    pub(crate) capacity: Quantity,
}

impl PreorderWindow {
    pub fn try_new(
        sku: Sku,
        opens_at: Timestamp,
        closes_at: Timestamp,
        capacity: Quantity,
    ) -> DomainResult<Self> {
        if closes_at < opens_at {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            sku,
            opens_at,
            closes_at,
            capacity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreorderReservation {
    pub(crate) window: PreorderWindow,
    pub(crate) quantity: Quantity,
    pub(crate) reserved_at: Timestamp,
}

impl PreorderReservation {
    pub fn try_new(
        window: PreorderWindow,
        quantity: Quantity,
        reserved_at: Timestamp,
    ) -> DomainResult<Self> {
        if quantity > window.capacity
            || reserved_at < window.opens_at
            || reserved_at > window.closes_at
        {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            window,
            quantity,
            reserved_at,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerialNumber {
    value: Nat,
}

impl SerialNumber {
    #[must_use]
    pub const fn new(value: Nat) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn value(self) -> Nat {
        self.value
    }
}

domain_struct! {
    pub struct SerializedInventoryUnit {
        sku: Sku,
        serial: SerialNumber,
        warehouse: Warehouse,
        reserved: bool,
    }
}

#[must_use]
pub fn serial_numbers_distinct(units: &[SerializedInventoryUnit]) -> bool {
    let mut seen = HashSet::new();
    units.iter().all(|unit| seen.insert(unit.serial.value()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerializedInventorySet {
    pub(crate) units: Vec<SerializedInventoryUnit>,
}

impl SerializedInventorySet {
    pub fn try_new(units: Vec<SerializedInventoryUnit>) -> DomainResult<Self> {
        if !serial_numbers_distinct(&units) {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self { units })
    }
}

domain_struct! {
    pub struct InventoryLot {
        sku: Sku,
        lot_id: Id,
        warehouse: Warehouse,
        expires_at: Timestamp,
        quantity: Quantity,
    }
}

#[must_use]
pub fn lot_usable_at(now: Timestamp, lot: &InventoryLot) -> bool {
    now <= lot.expires_at && lot.quantity > 0
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkuSubstitution {
    pub(crate) requested_sku: Sku,
    pub(crate) substitute_sku: Sku,
    pub(crate) substitute_stock: StockState,
    pub(crate) max_substitute_qty: Quantity,
}

impl SkuSubstitution {
    pub fn try_new(
        requested_sku: Sku,
        substitute_sku: Sku,
        substitute_stock: StockState,
        max_substitute_qty: Quantity,
    ) -> DomainResult<Self> {
        if substitute_stock.sku != substitute_sku
            || max_substitute_qty > available_stock(&substitute_stock)
        {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            requested_sku,
            substitute_sku,
            substitute_stock,
            max_substitute_qty,
        })
    }
}

#[must_use]
pub fn allocation_warehouse_ids(allocations: &[Allocation]) -> Vec<Id> {
    allocations
        .iter()
        .map(|allocation| allocation.node.warehouse.id)
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplitFulfillmentPlan {
    pub(crate) plan: DistinctFulfillmentPlan,
    pub(crate) first_warehouse: Warehouse,
    pub(crate) second_warehouse: Warehouse,
}

impl SplitFulfillmentPlan {
    pub fn try_new(
        plan: DistinctFulfillmentPlan,
        first_warehouse: Warehouse,
        second_warehouse: Warehouse,
    ) -> DomainResult<Self> {
        let ids = allocation_warehouse_ids(plan.allocations());
        if !ids.contains(&first_warehouse.id())
            || !ids.contains(&second_warehouse.id())
            || first_warehouse.id() == second_warehouse.id()
        {
            return Err(ValidationError::InventoryInvariantFailed);
        }
        Ok(Self {
            plan,
            first_warehouse,
            second_warehouse,
        })
    }
}

impl_getters!(PickTask {
    sku: Sku,
    requested: Quantity,
    bin: BinStock,
});

impl_getters!(PackTask {
    picked: Quantity,
    packed: Quantity,
});

impl_getters!(WarehouseShipment {
    packed: Quantity,
    shipped: Quantity,
});

impl_getters!(DistinctFulfillmentPlan {
    requested: Quantity,
    allocations: Vec<Allocation>,
});

impl_getters!(ReservationAttempt {
    stock: VersionedStock,
    quantity: Quantity,
    expected_version: Nat,
});

impl_getters!(ConcurrentReservationConflict {
    first: ReservationAttempt,
    second: ReservationAttempt,
});

impl_getters!(TimedReservation {
    stock: StockState,
    quantity: Quantity,
    reserved_at: Timestamp,
    expires_at: Timestamp,
    status: ReservationStatus,
});

impl_getters!(BackorderRequest {
    sku: Sku,
    requested: Quantity,
    available_now: Quantity,
    backordered: Quantity,
});

impl_getters!(PreorderWindow {
    sku: Sku,
    opens_at: Timestamp,
    closes_at: Timestamp,
    capacity: Quantity,
});

impl_getters!(PreorderReservation {
    window: PreorderWindow,
    quantity: Quantity,
    reserved_at: Timestamp,
});

impl_getters!(SerializedInventorySet { units: Vec<SerializedInventoryUnit> });

impl_getters!(SkuSubstitution {
    requested_sku: Sku,
    substitute_sku: Sku,
    substitute_stock: StockState,
    max_substitute_qty: Quantity,
});

impl_getters!(SplitFulfillmentPlan {
    plan: DistinctFulfillmentPlan,
    first_warehouse: Warehouse,
    second_warehouse: Warehouse,
});
