use std::collections::HashSet;

use crate::foundation::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StockState {
    pub(crate) sku: Sku,
    pub(crate) total: Quantity,
    pub(crate) reserved: Quantity,
}

impl StockState {
    pub fn try_new(sku: Sku, total: Quantity, reserved: Quantity) -> DomainResult<Self> {
        if reserved > total {
            return Err(ValidationError::Invariant("reserved stock exceeds total"));
        }
        Ok(Self {
            sku,
            total,
            reserved,
        })
    }

    pub fn sku(&self) -> Sku {
        self.sku
    }

    pub fn total(&self) -> Quantity {
        self.total
    }

    pub fn reserved(&self) -> Quantity {
        self.reserved
    }
}

pub fn available_stock(s: &StockState) -> Quantity {
    nat_sub(s.total, s.reserved)
}

pub fn can_reserve(s: &StockState, q: Quantity) -> bool {
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn from_stock(stock: StockState, version: Nat) -> Self {
        Self { stock, version }
    }

    pub fn stock(&self) -> &StockState {
        &self.stock
    }

    pub fn version(&self) -> Nat {
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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PackTask {
    pub(crate) picked: Quantity,
    pub(crate) packed: Quantity,
}

impl PackTask {
    pub fn try_new(picked: Quantity, packed: Quantity) -> DomainResult<Self> {
        if packed > picked {
            return Err(ValidationError::Invariant("packed exceeds picked"));
        }
        Ok(Self { picked, packed })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarehouseShipment {
    pub(crate) packed: Quantity,
    pub(crate) shipped: Quantity,
}

impl WarehouseShipment {
    pub fn try_new(packed: Quantity, shipped: Quantity) -> DomainResult<Self> {
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

    pub fn node(&self) -> &InventoryNode {
        &self.node
    }

    pub fn quantity(&self) -> Quantity {
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

pub fn allocation_key(a: &Allocation) -> (Nat, Nat) {
    (a.node.warehouse.id, a.node.stock.sku.value())
}

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

    pub fn requested(&self) -> Quantity {
        self.requested
    }

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
        FulfillmentPlan::try_new(requested, allocations.clone())?;
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
