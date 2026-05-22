use crate::competitor_pricing::*;
use crate::foundation::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BrandPricingPolicy {
    pub(crate) map_price: Money,
    pub(crate) msrp: Money,
}

impl BrandPricingPolicy {
    pub const fn try_new(map_price: Money, msrp: Money) -> DomainResult<Self> {
        if map_price > msrp {
            return Err(ValidationError::Invariant("MAP exceeds MSRP"));
        }
        Ok(Self { map_price, msrp })
    }
}

#[must_use]
pub const fn advertised_price_allowed(
    policy: &BrandPricingPolicy,
    advertised_price: Money,
) -> bool {
    policy.map_price <= advertised_price
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleComponent {
    pub(crate) sku: Sku,
    pub(crate) units_per_bundle: Quantity,
    pub(crate) stock_available: Quantity,
}

impl BundleComponent {
    pub const fn try_new(
        sku: Sku,
        units_per_bundle: Quantity,
        stock_available: Quantity,
    ) -> DomainResult<Self> {
        if units_per_bundle == 0 {
            return Err(ValidationError::Invariant(
                "bundle units per component must be positive",
            ));
        }
        Ok(Self {
            sku,
            units_per_bundle,
            stock_available,
        })
    }
}

pub fn component_required_for_bundles(
    bundle_qty: Quantity,
    component: &BundleComponent,
) -> DomainResult<Quantity> {
    checked_mul(
        bundle_qty,
        component.units_per_bundle,
        "component_required_for_bundles",
    )
}

pub fn component_can_fulfill_bundles(
    bundle_qty: Quantity,
    component: &BundleComponent,
) -> DomainResult<bool> {
    Ok(component_required_for_bundles(bundle_qty, component)? <= component.stock_available)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleReservation {
    pub(crate) bundle_qty: Quantity,
    pub(crate) components: Vec<BundleComponent>,
}

impl BundleReservation {
    pub fn try_new(bundle_qty: Quantity, components: Vec<BundleComponent>) -> DomainResult<Self> {
        for component in &components {
            if !component_can_fulfill_bundles(bundle_qty, component)? {
                return Err(ValidationError::Invariant(
                    "bundle component cannot fulfill reservation",
                ));
            }
        }
        Ok(Self {
            bundle_qty,
            components,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PromotionStackingPolicy {
    Exclusive,
    Stackable,
    StackableWithCap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AcceptedPromotionSet {
    pub(crate) resulting_price: Money,
    pub(crate) total_discount: Money,
    pub(crate) discount_cap: Money,
    pub(crate) profit_floor: Money,
}

impl AcceptedPromotionSet {
    pub const fn try_new(
        resulting_price: Money,
        total_discount: Money,
        discount_cap: Money,
        profit_floor: Money,
    ) -> DomainResult<Self> {
        if total_discount > discount_cap {
            return Err(ValidationError::Invariant("promotion discount exceeds cap"));
        }
        if profit_floor > resulting_price {
            return Err(ValidationError::Invariant(
                "promotion price below profit floor",
            ));
        }
        Ok(Self {
            resulting_price,
            total_discount,
            discount_cap,
            profit_floor,
        })
    }
}

#[must_use]
pub const fn promotion_set_allowed_by_policy(
    policy: PromotionStackingPolicy,
    promotion_count: Nat,
    set: &AcceptedPromotionSet,
) -> bool {
    match policy {
        PromotionStackingPolicy::Exclusive => promotion_count <= 1,
        PromotionStackingPolicy::Stackable => true,
        PromotionStackingPolicy::StackableWithCap => set.total_discount <= set.discount_cap,
    }
}

domain_struct! {
    pub struct SearchResultItem {
        sku: Sku,
        archived: bool,
        in_stock: bool,
        margin_safe: bool,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidSearchResultItem {
    pub(crate) item: SearchResultItem,
}

impl ValidSearchResultItem {
    pub const fn try_new(item: SearchResultItem) -> DomainResult<Self> {
        if item.archived || !item.in_stock || !item.margin_safe {
            return Err(ValidationError::Invariant("search result is not safe"));
        }
        Ok(Self { item })
    }
}

pub(crate) const fn _competitor_anchor(_: Option<TrustLevel>) {}

impl_getters!(BrandPricingPolicy {
    map_price: Money,
    msrp: Money,
});

impl_getters!(BundleComponent {
    sku: Sku,
    units_per_bundle: Quantity,
    stock_available: Quantity,
});

impl_getters!(BundleReservation {
    bundle_qty: Quantity,
    components: Vec<BundleComponent>,
});

impl_getters!(AcceptedPromotionSet {
    resulting_price: Money,
    total_discount: Money,
    discount_cap: Money,
    profit_floor: Money,
});

impl_getters!(ValidSearchResultItem {
    item: SearchResultItem,
});
