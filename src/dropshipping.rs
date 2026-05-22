use crate::b2b::*;
use crate::foundation::*;
use crate::orders::*;

domain_struct! {
    pub struct DropshipSupplier {
        id: SupplierId,
        name: String,
        currency: Currency,
        active: bool,
        suspended: bool,
        processing_days: Days,
        accepts_returns: bool,
        max_daily_orders: Nat,
    }
}

#[must_use]
pub const fn supplier_can_receive_orders(supplier: &DropshipSupplier) -> bool {
    supplier.active && !supplier.suspended
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SupplierDailyCapacity {
    pub(crate) supplier: DropshipSupplier,
    pub(crate) daily_order_capacity: Nat,
    pub(crate) orders_accepted_today: Nat,
}

impl SupplierDailyCapacity {
    pub fn try_new(
        supplier: DropshipSupplier,
        daily_order_capacity: Nat,
        orders_accepted_today: Nat,
    ) -> DomainResult<Self> {
        if orders_accepted_today > daily_order_capacity {
            return Err(ValidationError::Invariant(
                "accepted orders exceed capacity",
            ));
        }
        if daily_order_capacity > supplier.max_daily_orders {
            return Err(ValidationError::Invariant(
                "daily capacity exceeds supplier maximum",
            ));
        }
        Ok(Self {
            supplier,
            daily_order_capacity,
            orders_accepted_today,
        })
    }
}

#[must_use]
pub fn can_add_supplier_orders(capacity: &SupplierDailyCapacity, new_orders: Nat) -> bool {
    capacity
        .orders_accepted_today
        .checked_add(new_orders)
        .is_some_and(|total| total <= capacity.daily_order_capacity)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipOffer {
    pub(crate) sku: Sku,
    pub(crate) supplier: DropshipSupplier,
    pub(crate) supplier_unit_cost: Money,
    pub(crate) sale_unit_price: Money,
    pub(crate) unit_weight: Weight,
    pub(crate) available_qty: Quantity,
    pub(crate) currency: Currency,
    pub(crate) active: bool,
}

impl DropshipOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        sku: Sku,
        supplier: DropshipSupplier,
        supplier_unit_cost: Money,
        sale_unit_price: Money,
        unit_weight: Weight,
        available_qty: Quantity,
        currency: Currency,
        active: bool,
    ) -> DomainResult<Self> {
        if supplier_unit_cost > sale_unit_price {
            return Err(ValidationError::Invariant(
                "supplier unit cost exceeds sale price",
            ));
        }
        if currency != supplier.currency {
            return Err(ValidationError::Invariant(
                "offer currency must match supplier currency",
            ));
        }
        Ok(Self {
            sku,
            supplier,
            supplier_unit_cost,
            sale_unit_price,
            unit_weight,
            available_qty,
            currency,
            active,
        })
    }

    #[must_use]
    pub const fn sku(&self) -> Sku {
        self.sku
    }

    #[must_use]
    pub const fn sale_unit_price(&self) -> Money {
        self.sale_unit_price
    }

    #[must_use]
    pub const fn currency(&self) -> Currency {
        self.currency
    }
}

#[must_use]
pub const fn dropship_offer_can_be_sold(offer: &DropshipOffer) -> bool {
    supplier_can_receive_orders(&offer.supplier) && offer.active && offer.available_qty > 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupplierReservationStatus {
    Requested,
    Confirmed,
    Rejected,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SupplierReservation {
    pub(crate) offer: DropshipOffer,
    pub(crate) supplier: DropshipSupplier,
    pub(crate) quantity: Quantity,
    pub(crate) status: SupplierReservationStatus,
}

impl SupplierReservation {
    pub fn try_new(
        offer: DropshipOffer,
        supplier: DropshipSupplier,
        quantity: Quantity,
        status: SupplierReservationStatus,
    ) -> DomainResult<Self> {
        if offer.supplier.id != supplier.id {
            return Err(ValidationError::Invariant("reservation supplier mismatch"));
        }
        if quantity > offer.available_qty {
            return Err(ValidationError::Invariant(
                "reservation quantity exceeds availability",
            ));
        }
        Ok(Self {
            offer,
            supplier,
            quantity,
            status,
        })
    }
}

#[must_use]
pub fn reservation_confirmed(r: &SupplierReservation) -> bool {
    r.status == SupplierReservationStatus::Confirmed
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipLine {
    pub(crate) offer: DropshipOffer,
    pub(crate) quantity: Quantity,
    pub(crate) discount: Money,
}

impl DropshipLine {
    pub fn try_new(
        offer: DropshipOffer,
        quantity: Quantity,
        discount: Money,
    ) -> DomainResult<Self> {
        if !supplier_can_receive_orders(&offer.supplier) {
            return Err(ValidationError::Invariant("supplier cannot receive orders"));
        }
        if !offer.active {
            return Err(ValidationError::Invariant("dropship offer is inactive"));
        }
        if quantity > offer.available_qty {
            return Err(ValidationError::Invariant(
                "line quantity exceeds offer availability",
            ));
        }
        let sale_gross = checked_mul(offer.sale_unit_price, quantity, "dropship sale gross")?;
        if discount > sale_gross {
            return Err(ValidationError::Invariant(
                "dropship discount exceeds gross",
            ));
        }
        let supplier_cost =
            checked_mul(offer.supplier_unit_cost, quantity, "dropship supplier cost")?;
        if checked_add(supplier_cost, discount, "dropship margin")? > sale_gross {
            return Err(ValidationError::Invariant(
                "dropship margin after discount is unsafe",
            ));
        }
        Ok(Self {
            offer,
            quantity,
            discount,
        })
    }

    #[must_use]
    pub const fn offer(&self) -> &DropshipOffer {
        &self.offer
    }

    #[must_use]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }
}

pub fn dropship_line_sale_gross(line: &DropshipLine) -> DomainResult<Money> {
    checked_mul(
        line.offer.sale_unit_price,
        line.quantity,
        "dropship_line_sale_gross",
    )
}

pub fn dropship_line_customer_net(line: &DropshipLine) -> DomainResult<Money> {
    Ok(nat_sub(dropship_line_sale_gross(line)?, line.discount))
}

pub fn dropship_line_supplier_cost(line: &DropshipLine) -> DomainResult<Money> {
    checked_mul(
        line.offer.supplier_unit_cost,
        line.quantity,
        "dropship_line_supplier_cost",
    )
}

pub fn dropship_line_weight(line: &DropshipLine) -> DomainResult<Weight> {
    checked_mul(
        line.offer.unit_weight,
        line.quantity,
        "dropship_line_weight",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReservedDropshipLine {
    pub(crate) line: DropshipLine,
    pub(crate) reservation: SupplierReservation,
}

impl ReservedDropshipLine {
    pub fn try_new(line: DropshipLine, reservation: SupplierReservation) -> DomainResult<Self> {
        if reservation.offer != line.offer {
            return Err(ValidationError::Invariant("reservation offer mismatch"));
        }
        if reservation.quantity != line.quantity {
            return Err(ValidationError::Invariant("reservation quantity mismatch"));
        }
        if reservation.status != SupplierReservationStatus::Confirmed {
            return Err(ValidationError::Invariant("reservation must be confirmed"));
        }
        Ok(Self { line, reservation })
    }
}

pub fn dropship_sale_net_total(lines: &[DropshipLine]) -> DomainResult<Money> {
    checked_result_sum(
        lines.iter().map(dropship_line_customer_net),
        "dropship_sale_net_total",
    )
}

pub fn dropship_supplier_cost_total(lines: &[DropshipLine]) -> DomainResult<Money> {
    checked_result_sum(
        lines.iter().map(dropship_line_supplier_cost),
        "dropship_supplier_cost_total",
    )
}

pub fn dropship_weight_total(lines: &[DropshipLine]) -> DomainResult<Weight> {
    checked_result_sum(
        lines.iter().map(dropship_line_weight),
        "dropship_weight_total",
    )
}

domain_struct! {
    pub struct DropshipShippingQuote {
        supplier_id: SupplierId,
        price: Money,
        max_weight: Weight,
        carrier_days: Days,
    }
}

#[must_use]
pub const fn dropship_shipping_quote_can_ship(
    quote: &DropshipShippingQuote,
    weight: Weight,
) -> bool {
    weight <= quote.max_weight
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DropshipPOStatus {
    Created,
    Submitted,
    Accepted,
    Rejected,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipPurchaseOrder {
    pub(crate) supplier: DropshipSupplier,
    pub(crate) lines: Vec<DropshipLine>,
    pub(crate) quote: DropshipShippingQuote,
    pub(crate) status: DropshipPOStatus,
    pub(crate) total: Money,
}

impl DropshipPurchaseOrder {
    pub fn try_new(
        supplier: DropshipSupplier,
        lines: Vec<DropshipLine>,
        quote: DropshipShippingQuote,
        status: DropshipPOStatus,
        total: Money,
    ) -> DomainResult<Self> {
        if quote.supplier_id != supplier.id {
            return Err(ValidationError::Invariant("quote supplier mismatch"));
        }
        if dropship_weight_total(&lines)? > quote.max_weight {
            return Err(ValidationError::Invariant(
                "dropship PO weight exceeds quote",
            ));
        }
        let expected = checked_add(
            dropship_supplier_cost_total(&lines)?,
            quote.price,
            "dropship purchase order total",
        )?;
        if total != expected {
            return Err(ValidationError::Invariant("dropship PO total is incorrect"));
        }
        Ok(Self {
            supplier,
            lines,
            quote,
            status,
            total,
        })
    }

    #[must_use]
    pub fn lines(&self) -> &[DropshipLine] {
        &self.lines
    }
}

#[must_use]
pub const fn can_dropship_po_transition(
    source: DropshipPOStatus,
    target: DropshipPOStatus,
) -> bool {
    matches!(
        (source, target),
        (
            DropshipPOStatus::Created,
            DropshipPOStatus::Submitted | DropshipPOStatus::Cancelled
        ) | (
            DropshipPOStatus::Submitted,
            DropshipPOStatus::Accepted | DropshipPOStatus::Rejected | DropshipPOStatus::Cancelled
        ) | (
            DropshipPOStatus::Accepted,
            DropshipPOStatus::Shipped | DropshipPOStatus::Cancelled
        ) | (DropshipPOStatus::Shipped, DropshipPOStatus::Delivered)
    )
}

#[must_use]
pub fn dropship_sla_safe(
    supplier: &DropshipSupplier,
    quote: &DropshipShippingQuote,
    promised_days: Days,
) -> bool {
    supplier
        .processing_days
        .checked_add(quote.carrier_days)
        .is_some_and(|days| days <= promised_days)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipFulfillment {
    pub(crate) customer_order: Order,
    pub(crate) purchase_order: DropshipPurchaseOrder,
    pub(crate) segment_revenue: Money,
}

impl DropshipFulfillment {
    pub fn try_new(
        customer_order: Order,
        purchase_order: DropshipPurchaseOrder,
        segment_revenue: Money,
    ) -> DomainResult<Self> {
        if segment_revenue != dropship_sale_net_total(&purchase_order.lines)? {
            return Err(ValidationError::Invariant(
                "segment revenue must match dropship sale net total",
            ));
        }
        if segment_revenue > customer_order.total() {
            return Err(ValidationError::Invariant(
                "segment revenue exceeds customer order total",
            ));
        }
        Ok(Self {
            customer_order,
            purchase_order,
            segment_revenue,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipReturnRequest {
    pub(crate) line: DropshipLine,
    pub(crate) return_qty: Quantity,
    pub(crate) customer_refund: Money,
    pub(crate) supplier_credit: Money,
}

impl DropshipReturnRequest {
    pub fn try_new(
        line: DropshipLine,
        return_qty: Quantity,
        customer_refund: Money,
        supplier_credit: Money,
    ) -> DomainResult<Self> {
        if !line.offer.supplier.accepts_returns {
            return Err(ValidationError::Invariant("supplier must accept returns"));
        }
        if return_qty > line.quantity {
            return Err(ValidationError::Invariant(
                "return quantity exceeds sold quantity",
            ));
        }
        if customer_refund > dropship_line_customer_net(&line)? {
            return Err(ValidationError::Invariant(
                "customer refund exceeds customer net",
            ));
        }
        if supplier_credit > dropship_line_supplier_cost(&line)? {
            return Err(ValidationError::Invariant(
                "supplier credit exceeds supplier cost",
            ));
        }
        Ok(Self {
            line,
            return_qty,
            customer_refund,
            supplier_credit,
        })
    }
}

pub(crate) const fn _b2b_anchor(_: Option<TradeMode>) {}

impl_getters!(DropshipOffer {
    supplier: DropshipSupplier,
    supplier_unit_cost: Money,
    unit_weight: Weight,
    available_qty: Quantity,
    active: bool,
});

impl_getters!(SupplierDailyCapacity {
    supplier: DropshipSupplier,
    daily_order_capacity: Nat,
    orders_accepted_today: Nat,
});

impl_getters!(SupplierReservation {
    offer: DropshipOffer,
    supplier: DropshipSupplier,
    quantity: Quantity,
    status: SupplierReservationStatus,
});

impl_getters!(DropshipLine { discount: Money });

impl_getters!(ReservedDropshipLine {
    line: DropshipLine,
    reservation: SupplierReservation,
});

impl_getters!(DropshipPurchaseOrder {
    supplier: DropshipSupplier,
    quote: DropshipShippingQuote,
    status: DropshipPOStatus,
    total: Money,
});

impl_getters!(DropshipFulfillment {
    customer_order: Order,
    purchase_order: DropshipPurchaseOrder,
    segment_revenue: Money,
});

impl_getters!(DropshipReturnRequest {
    line: DropshipLine,
    return_qty: Quantity,
    customer_refund: Money,
    supplier_credit: Money,
});
