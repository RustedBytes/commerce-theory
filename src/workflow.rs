use crate::dropshipping::*;
use crate::orders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderTransitionLabel {
    CapturePayment,
    CancelBeforePayment,
    MarkBackordered,
    PackPaidOrder,
    RefundPaidOrder,
    ShipPackedOrder,
    ConfirmDelivery,
    RefundDeliveredOrder,
    ReceiveBackorderPayment,
    CancelBackorder,
}

#[must_use]
pub const fn order_transition_target(
    source: OrderStatus,
    label: OrderTransitionLabel,
) -> Option<OrderStatus> {
    match (source, label) {
        (OrderStatus::New, OrderTransitionLabel::CapturePayment) => Some(OrderStatus::Paid),
        (OrderStatus::New, OrderTransitionLabel::CancelBeforePayment)
        | (OrderStatus::Backordered, OrderTransitionLabel::CancelBackorder) => {
            Some(OrderStatus::Cancelled)
        }
        (OrderStatus::New, OrderTransitionLabel::MarkBackordered) => Some(OrderStatus::Backordered),
        (OrderStatus::Paid, OrderTransitionLabel::PackPaidOrder) => Some(OrderStatus::Packed),
        (OrderStatus::Paid, OrderTransitionLabel::RefundPaidOrder) => Some(OrderStatus::Refunded),
        (OrderStatus::Packed, OrderTransitionLabel::ShipPackedOrder) => Some(OrderStatus::Shipped),
        (OrderStatus::Shipped, OrderTransitionLabel::ConfirmDelivery) => {
            Some(OrderStatus::Delivered)
        }
        (OrderStatus::Delivered, OrderTransitionLabel::RefundDeliveredOrder) => {
            Some(OrderStatus::Refunded)
        }
        (OrderStatus::Backordered, OrderTransitionLabel::ReceiveBackorderPayment) => {
            Some(OrderStatus::Paid)
        }
        _ => None,
    }
}

#[must_use]
pub fn execute_order_trace(
    start: OrderStatus,
    trace: &[OrderTransitionLabel],
) -> Option<Vec<OrderStatus>> {
    let mut states = vec![start];
    let mut current = start;
    for label in trace {
        current = order_transition_target(current, *label)?;
        states.push(current);
    }
    Some(states)
}

#[must_use]
pub fn paid_fulfillment_trace() -> Vec<OrderTransitionLabel> {
    vec![
        OrderTransitionLabel::CapturePayment,
        OrderTransitionLabel::PackPaidOrder,
        OrderTransitionLabel::ShipPackedOrder,
        OrderTransitionLabel::ConfirmDelivery,
    ]
}

#[must_use]
pub fn unpaid_cancellation_trace() -> Vec<OrderTransitionLabel> {
    vec![OrderTransitionLabel::CancelBeforePayment]
}

#[must_use]
pub const fn terminal_order_status(status: OrderStatus) -> bool {
    matches!(
        status,
        OrderStatus::Delivered | OrderStatus::Cancelled | OrderStatus::Refunded
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrderStatusLts;

impl OrderStatusLts {
    #[must_use]
    pub const fn transition(
        self,
        source: OrderStatus,
        label: OrderTransitionLabel,
    ) -> Option<OrderStatus> {
        order_transition_target(source, label)
    }

    #[must_use]
    pub fn execute(
        self,
        start: OrderStatus,
        trace: &[OrderTransitionLabel],
    ) -> Option<Vec<OrderStatus>> {
        execute_order_trace(start, trace)
    }
}

#[must_use]
pub const fn order_status_lts() -> OrderStatusLts {
    OrderStatusLts
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DropshipPOTransitionLabel {
    Submit,
    CancelBeforeSubmit,
    Accept,
    Reject,
    CancelSubmitted,
    ShipAccepted,
    CancelAccepted,
    ConfirmDelivery,
}

#[must_use]
pub const fn dropship_po_transition_target(
    source: DropshipPOStatus,
    label: DropshipPOTransitionLabel,
) -> Option<DropshipPOStatus> {
    match (source, label) {
        (DropshipPOStatus::Created, DropshipPOTransitionLabel::Submit) => {
            Some(DropshipPOStatus::Submitted)
        }
        (DropshipPOStatus::Created, DropshipPOTransitionLabel::CancelBeforeSubmit)
        | (DropshipPOStatus::Submitted, DropshipPOTransitionLabel::CancelSubmitted)
        | (DropshipPOStatus::Accepted, DropshipPOTransitionLabel::CancelAccepted) => {
            Some(DropshipPOStatus::Cancelled)
        }
        (DropshipPOStatus::Submitted, DropshipPOTransitionLabel::Accept) => {
            Some(DropshipPOStatus::Accepted)
        }
        (DropshipPOStatus::Submitted, DropshipPOTransitionLabel::Reject) => {
            Some(DropshipPOStatus::Rejected)
        }
        (DropshipPOStatus::Accepted, DropshipPOTransitionLabel::ShipAccepted) => {
            Some(DropshipPOStatus::Shipped)
        }
        (DropshipPOStatus::Shipped, DropshipPOTransitionLabel::ConfirmDelivery) => {
            Some(DropshipPOStatus::Delivered)
        }
        _ => None,
    }
}

#[must_use]
pub fn execute_dropship_po_trace(
    start: DropshipPOStatus,
    trace: &[DropshipPOTransitionLabel],
) -> Option<Vec<DropshipPOStatus>> {
    let mut states = vec![start];
    let mut current = start;
    for label in trace {
        current = dropship_po_transition_target(current, *label)?;
        states.push(current);
    }
    Some(states)
}

#[must_use]
pub fn dropship_po_delivery_trace() -> Vec<DropshipPOTransitionLabel> {
    vec![
        DropshipPOTransitionLabel::Submit,
        DropshipPOTransitionLabel::Accept,
        DropshipPOTransitionLabel::ShipAccepted,
        DropshipPOTransitionLabel::ConfirmDelivery,
    ]
}

#[must_use]
pub const fn terminal_dropship_po_status(status: DropshipPOStatus) -> bool {
    matches!(
        status,
        DropshipPOStatus::Delivered | DropshipPOStatus::Cancelled | DropshipPOStatus::Rejected
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropshipPoLts;

impl DropshipPoLts {
    #[must_use]
    pub const fn transition(
        self,
        source: DropshipPOStatus,
        label: DropshipPOTransitionLabel,
    ) -> Option<DropshipPOStatus> {
        dropship_po_transition_target(source, label)
    }

    #[must_use]
    pub fn execute(
        self,
        start: DropshipPOStatus,
        trace: &[DropshipPOTransitionLabel],
    ) -> Option<Vec<DropshipPOStatus>> {
        execute_dropship_po_trace(start, trace)
    }
}

#[must_use]
pub const fn dropship_po_lts() -> DropshipPoLts {
    DropshipPoLts
}

#[must_use]
pub const fn dropship_polts() -> DropshipPoLts {
    DropshipPoLts
}
