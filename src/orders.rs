use core::marker::PhantomData;

use crate::foundation::*;
use crate::pricing::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderStatus {
    New,
    Paid,
    Packed,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
    Backordered,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    pub(crate) id: OrderId,
    pub(crate) items: Vec<CartLine>,
    pub(crate) coupon_amount: Money,
    pub(crate) shipping_method: ShippingMethod,
    pub(crate) tax: Money,
    pub(crate) currency: Currency,
    pub(crate) status: OrderStatus,
    pub(crate) total: Money,
}

impl Order {
    pub fn try_new(
        id: OrderId,
        items: Vec<CartLine>,
        coupon_amount: Money,
        shipping_method: ShippingMethod,
        tax: Money,
        currency: Currency,
        status: OrderStatus,
        total: Money,
    ) -> DomainResult<Self> {
        if !shipping_available(&shipping_method, cart_weight_total(&items)?) {
            return Err(ValidationError::Invariant(
                "shipping method cannot carry cart",
            ));
        }
        let expected_total = order_total(&shipping_method, coupon_amount, tax, &items)?;
        if total != expected_total {
            return Err(ValidationError::Invariant(
                "stored order total is incorrect",
            ));
        }
        Ok(Self {
            id,
            items,
            coupon_amount,
            shipping_method,
            tax,
            currency,
            status,
            total,
        })
    }

    pub fn id(&self) -> OrderId {
        self.id
    }

    pub fn items(&self) -> &[CartLine] {
        &self.items
    }

    pub fn total(&self) -> Money {
        self.total
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn shipping_method(&self) -> &ShippingMethod {
        &self.shipping_method
    }

    pub fn tax(&self) -> Money {
        self.tax
    }
}

pub fn can_order_transition(source: OrderStatus, target: OrderStatus) -> bool {
    matches!(
        (source, target),
        (OrderStatus::New, OrderStatus::Paid)
            | (OrderStatus::New, OrderStatus::Cancelled)
            | (OrderStatus::New, OrderStatus::Backordered)
            | (OrderStatus::Paid, OrderStatus::Packed)
            | (OrderStatus::Paid, OrderStatus::Refunded)
            | (OrderStatus::Packed, OrderStatus::Shipped)
            | (OrderStatus::Shipped, OrderStatus::Delivered)
            | (OrderStatus::Delivered, OrderStatus::Refunded)
            | (OrderStatus::Backordered, OrderStatus::Paid)
            | (OrderStatus::Backordered, OrderStatus::Cancelled)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CanOrderTransition {
    NewPaid,
    NewCancelled,
    NewBackordered,
    PaidPacked,
    PaidRefunded,
    PackedShipped,
    ShippedDelivered,
    DeliveredRefunded,
    BackorderedPaid,
    BackorderedCancelled,
}

impl CanOrderTransition {
    pub const fn source(self) -> OrderStatus {
        match self {
            Self::NewPaid | Self::NewCancelled | Self::NewBackordered => OrderStatus::New,
            Self::PaidPacked | Self::PaidRefunded => OrderStatus::Paid,
            Self::PackedShipped => OrderStatus::Packed,
            Self::ShippedDelivered => OrderStatus::Shipped,
            Self::DeliveredRefunded => OrderStatus::Delivered,
            Self::BackorderedPaid | Self::BackorderedCancelled => OrderStatus::Backordered,
        }
    }

    pub const fn target(self) -> OrderStatus {
        match self {
            Self::NewPaid => OrderStatus::Paid,
            Self::NewCancelled => OrderStatus::Cancelled,
            Self::NewBackordered => OrderStatus::Backordered,
            Self::PaidPacked => OrderStatus::Packed,
            Self::PaidRefunded => OrderStatus::Refunded,
            Self::PackedShipped => OrderStatus::Shipped,
            Self::ShippedDelivered => OrderStatus::Delivered,
            Self::DeliveredRefunded => OrderStatus::Refunded,
            Self::BackorderedPaid => OrderStatus::Paid,
            Self::BackorderedCancelled => OrderStatus::Cancelled,
        }
    }

    pub fn from_statuses(source: OrderStatus, target: OrderStatus) -> Option<Self> {
        match (source, target) {
            (OrderStatus::New, OrderStatus::Paid) => Some(Self::NewPaid),
            (OrderStatus::New, OrderStatus::Cancelled) => Some(Self::NewCancelled),
            (OrderStatus::New, OrderStatus::Backordered) => Some(Self::NewBackordered),
            (OrderStatus::Paid, OrderStatus::Packed) => Some(Self::PaidPacked),
            (OrderStatus::Paid, OrderStatus::Refunded) => Some(Self::PaidRefunded),
            (OrderStatus::Packed, OrderStatus::Shipped) => Some(Self::PackedShipped),
            (OrderStatus::Shipped, OrderStatus::Delivered) => Some(Self::ShippedDelivered),
            (OrderStatus::Delivered, OrderStatus::Refunded) => Some(Self::DeliveredRefunded),
            (OrderStatus::Backordered, OrderStatus::Paid) => Some(Self::BackorderedPaid),
            (OrderStatus::Backordered, OrderStatus::Cancelled) => Some(Self::BackorderedCancelled),
            _ => None,
        }
    }
}

pub trait OrderStatusMarker: Clone + Copy + core::fmt::Debug + PartialEq + Eq {
    const STATUS: OrderStatus;
}

macro_rules! order_marker {
    ($name:ident, $status:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name;
        impl OrderStatusMarker for $name {
            const STATUS: OrderStatus = OrderStatus::$status;
        }
    };
}

order_marker!(NewOrder, New);
order_marker!(PaidOrder, Paid);
order_marker!(PackedOrder, Packed);
order_marker!(ShippedOrder, Shipped);
order_marker!(DeliveredOrder, Delivered);
order_marker!(CancelledOrder, Cancelled);
order_marker!(RefundedOrder, Refunded);
order_marker!(BackorderedOrder, Backordered);

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedOrder<S: OrderStatusMarker> {
    pub(crate) id: OrderId,
    pub(crate) total: Money,
    pub(crate) currency: Currency,
    _state: PhantomData<S>,
}

impl<S: OrderStatusMarker> TypedOrder<S> {
    pub fn try_new(id: OrderId, total: Money, currency: Currency) -> DomainResult<Self> {
        if total == 0 {
            return Err(ValidationError::Invariant(
                "typed order total must be positive",
            ));
        }
        Ok(Self {
            id,
            total,
            currency,
            _state: PhantomData,
        })
    }

    pub fn id(&self) -> OrderId {
        self.id
    }

    pub fn total(&self) -> Money {
        self.total
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }
}

domain_struct! {
    pub struct CapturedPayment {
        order_id: OrderId,
        amount: Money,
        currency: Currency,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentState {
    Created,
    Authorized,
    Captured,
    Failed,
    Voided,
    Refunded,
}

pub trait PaymentStateMarker: Clone + Copy + core::fmt::Debug + PartialEq + Eq {
    const STATE: PaymentState;
}

macro_rules! payment_marker {
    ($name:ident, $state:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name;
        impl PaymentStateMarker for $name {
            const STATE: PaymentState = PaymentState::$state;
        }
    };
}

payment_marker!(CreatedPayment, Created);
payment_marker!(AuthorizedPayment, Authorized);
payment_marker!(CapturedPaymentState, Captured);
payment_marker!(FailedPayment, Failed);
payment_marker!(VoidedPayment, Voided);
payment_marker!(RefundedPayment, Refunded);

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedPayment<S: PaymentStateMarker> {
    pub(crate) id: PaymentId,
    pub(crate) order_id: OrderId,
    pub(crate) amount: Money,
    pub(crate) currency: Currency,
    _state: PhantomData<S>,
}

impl<S: PaymentStateMarker> TypedPayment<S> {
    pub fn try_new(
        id: PaymentId,
        order_id: OrderId,
        amount: Money,
        currency: Currency,
    ) -> DomainResult<Self> {
        if amount == 0 {
            return Err(ValidationError::Invariant(
                "payment amount must be positive",
            ));
        }
        Ok(Self {
            id,
            order_id,
            amount,
            currency,
            _state: PhantomData,
        })
    }
}

pub fn authorize_payment(p: TypedPayment<CreatedPayment>) -> TypedPayment<AuthorizedPayment> {
    TypedPayment {
        id: p.id,
        order_id: p.order_id,
        amount: p.amount,
        currency: p.currency,
        _state: PhantomData,
    }
}

pub fn capture_payment(
    p: TypedPayment<AuthorizedPayment>,
) -> (TypedPayment<CapturedPaymentState>, CapturedPayment) {
    let receipt = CapturedPayment::new(p.order_id, p.amount, p.currency);
    (
        TypedPayment {
            id: p.id,
            order_id: p.order_id,
            amount: p.amount,
            currency: p.currency,
            _state: PhantomData,
        },
        receipt,
    )
}

pub fn mark_paid(
    order: TypedOrder<NewOrder>,
    payment: &CapturedPayment,
) -> DomainResult<TypedOrder<PaidOrder>> {
    if payment.order_id != order.id {
        return Err(ValidationError::Invariant("payment order id mismatch"));
    }
    if payment.amount != order.total {
        return Err(ValidationError::Invariant("payment amount mismatch"));
    }
    if payment.currency != order.currency {
        return Err(ValidationError::Invariant("payment currency mismatch"));
    }
    Ok(TypedOrder {
        id: order.id,
        total: order.total,
        currency: order.currency,
        _state: PhantomData,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaymentLedger {
    pub(crate) captured: Money,
    pub(crate) refunded: Money,
}

impl PaymentLedger {
    pub fn try_new(captured: Money, refunded: Money) -> DomainResult<Self> {
        if refunded > captured {
            return Err(ValidationError::Invariant("refunded exceeds captured"));
        }
        Ok(Self { captured, refunded })
    }

    pub fn captured(&self) -> Money {
        self.captured
    }

    pub fn refunded(&self) -> Money {
        self.refunded
    }
}

pub fn remaining_refund_amount(ledger: &PaymentLedger) -> Money {
    nat_sub(ledger.captured, ledger.refunded)
}

pub fn can_refund(ledger: &PaymentLedger, amount: Money) -> bool {
    ledger
        .refunded
        .checked_add(amount)
        .is_some_and(|total| total <= ledger.captured)
}

pub fn issue_refund(ledger: &PaymentLedger, amount: Money) -> DomainResult<PaymentLedger> {
    if !can_refund(ledger, amount) {
        return Err(ValidationError::Invariant("refund exceeds captured amount"));
    }
    PaymentLedger::try_new(
        ledger.captured,
        checked_add(ledger.refunded, amount, "issue_refund")?,
    )
}

impl_getters!(Order {
    coupon_amount: Money,
    status: OrderStatus,
});
