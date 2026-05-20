use crate::foundation::*;
use crate::inventory::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CartLine {
    pub(crate) sku: Sku,
    pub(crate) price: Money,
    pub(crate) cost: Money,
    pub(crate) quantity: Quantity,
    pub(crate) discount: Money,
    pub(crate) weight: Weight,
}

impl CartLine {
    pub fn try_new(
        sku: Sku,
        price: Money,
        cost: Money,
        quantity: Quantity,
        discount: Money,
        weight: Weight,
    ) -> DomainResult<Self> {
        let gross = checked_mul(price, quantity, "CartLine gross")?;
        if discount > gross {
            return Err(ValidationError::Invariant("line discount exceeds gross"));
        }
        Ok(Self {
            sku,
            price,
            cost,
            quantity,
            discount,
            weight,
        })
    }

    pub fn quantity(&self) -> Quantity {
        self.quantity
    }
}

pub fn line_gross_total(line: &CartLine) -> DomainResult<Money> {
    checked_mul(line.price, line.quantity, "line_gross_total")
}

pub fn line_cost_total(line: &CartLine) -> DomainResult<Money> {
    checked_mul(line.cost, line.quantity, "line_cost_total")
}

pub fn line_net_total(line: &CartLine) -> DomainResult<Money> {
    Ok(nat_sub(line_gross_total(line)?, line.discount))
}

pub fn line_weight_total(line: &CartLine) -> DomainResult<Weight> {
    checked_mul(line.weight, line.quantity, "line_weight_total")
}

pub fn cart_gross_total(items: &[CartLine]) -> DomainResult<Money> {
    checked_sum(
        items
            .iter()
            .map(line_gross_total)
            .collect::<DomainResult<Vec<_>>>()?,
        "cart_gross_total",
    )
}

pub fn cart_net_total(items: &[CartLine]) -> DomainResult<Money> {
    checked_sum(
        items
            .iter()
            .map(line_net_total)
            .collect::<DomainResult<Vec<_>>>()?,
        "cart_net_total",
    )
}

pub fn cart_discount_total(items: &[CartLine]) -> DomainResult<Money> {
    checked_sum(
        items.iter().map(|line| line.discount),
        "cart_discount_total",
    )
}

pub fn cart_weight_total(items: &[CartLine]) -> DomainResult<Weight> {
    checked_sum(
        items
            .iter()
            .map(line_weight_total)
            .collect::<DomainResult<Vec<_>>>()?,
        "cart_weight_total",
    )
}

pub fn cart_quantity_total(items: &[CartLine]) -> DomainResult<Quantity> {
    checked_sum(
        items.iter().map(|line| line.quantity),
        "cart_quantity_total",
    )
}

domain_struct! {
    pub struct Coupon {
        amount: Money,
        min_subtotal: Money,
        max_uses: Nat,
    }
}

pub fn coupon_can_be_applied(coupon: &Coupon, subtotal: Money, uses_before: Nat) -> bool {
    coupon.min_subtotal <= subtotal && uses_before < coupon.max_uses
}

pub fn subtotal_after_coupon_amount(subtotal: Money, coupon_amount: Money) -> Money {
    nat_sub(subtotal, coupon_amount)
}

pub fn order_subtotal(items: &[CartLine], coupon_amount: Money) -> DomainResult<Money> {
    Ok(subtotal_after_coupon_amount(
        cart_net_total(items)?,
        coupon_amount,
    ))
}

domain_struct! {
    pub struct ShippingMethod {
        price: Money,
        free_threshold: Money,
        max_weight: Weight,
    }
}

pub fn shipping_available(method: &ShippingMethod, weight: Weight) -> bool {
    weight <= method.max_weight
}

pub fn shipping_charge(method: &ShippingMethod, subtotal: Money) -> Money {
    if method.free_threshold <= subtotal {
        0
    } else {
        method.price
    }
}

pub fn order_total(
    method: &ShippingMethod,
    coupon_amount: Money,
    tax: Money,
    items: &[CartLine],
) -> DomainResult<Money> {
    let subtotal = order_subtotal(items, coupon_amount)?;
    checked_add(
        checked_add(
            subtotal,
            shipping_charge(method, subtotal),
            "order_total shipping",
        )?,
        tax,
        "order_total tax",
    )
}

pub(crate) fn _inventory_anchor(_: &StockState) {}

impl_getters!(CartLine {
    sku: Sku,
    price: Money,
    cost: Money,
    discount: Money,
    weight: Weight,
});
