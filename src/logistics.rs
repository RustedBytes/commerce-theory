use std::collections::HashSet;

use crate::crm::*;
use crate::foundation::*;
use crate::fulfillment_finance::*;
use crate::inventory::*;
use crate::orders::*;
use crate::pricing::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ShipmentStatus {
    Planned,
    Allocated,
    Packed,
    InTransit,
    OutForDelivery,
    Delivered,
    Exception,
    Returned,
    Cancelled,
}

pub fn can_shipment_transition(source: ShipmentStatus, target: ShipmentStatus) -> bool {
    matches!(
        (source, target),
        (ShipmentStatus::Planned, ShipmentStatus::Allocated)
            | (ShipmentStatus::Planned, ShipmentStatus::Cancelled)
            | (ShipmentStatus::Allocated, ShipmentStatus::Packed)
            | (ShipmentStatus::Allocated, ShipmentStatus::Cancelled)
            | (ShipmentStatus::Packed, ShipmentStatus::InTransit)
            | (ShipmentStatus::InTransit, ShipmentStatus::OutForDelivery)
            | (ShipmentStatus::InTransit, ShipmentStatus::Exception)
            | (ShipmentStatus::OutForDelivery, ShipmentStatus::Delivered)
            | (ShipmentStatus::OutForDelivery, ShipmentStatus::Exception)
            | (ShipmentStatus::Exception, ShipmentStatus::InTransit)
            | (ShipmentStatus::Exception, ShipmentStatus::Returned)
    )
}

pub fn order_eligible_for_logistics(order: &Order) -> bool {
    matches!(*order.status(), OrderStatus::Paid | OrderStatus::Packed)
}

domain_struct! {
    pub struct ShippingDestination {
        id: Id,
        zone: ShippingZone,
        postal_code: Nat,
    }
}

pub fn cart_contains_sku(sku: Sku, items: &[CartLine]) -> bool {
    items.iter().any(|line| *line.sku() == sku)
}

pub fn allocations_match_cart_skus(items: &[CartLine], allocations: &[Allocation]) -> bool {
    allocations
        .iter()
        .all(|allocation| cart_contains_sku(allocation.node.stock.sku, items))
}

pub fn allocations_use_warehouse(warehouse: &Warehouse, allocations: &[Allocation]) -> bool {
    allocations
        .iter()
        .all(|allocation| allocation.node.warehouse.id == *warehouse.id())
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogisticsShipmentPlan {
    pub(crate) id: ShipmentId,
    pub(crate) order: Order,
    pub(crate) fulfillment: DistinctFulfillmentPlan,
    pub(crate) package: Package,
    pub(crate) quote: CarrierQuote,
    pub(crate) warehouse: Warehouse,
    pub(crate) destination: ShippingDestination,
    pub(crate) planned_ship_at: Timestamp,
    pub(crate) promised_delivery_at: Timestamp,
}

impl LogisticsShipmentPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: ShipmentId,
        order: Order,
        fulfillment: DistinctFulfillmentPlan,
        package: Package,
        quote: CarrierQuote,
        warehouse: Warehouse,
        destination: ShippingDestination,
        planned_ship_at: Timestamp,
        promised_delivery_at: Timestamp,
    ) -> DomainResult<Self> {
        if !order_eligible_for_logistics(&order) {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if *fulfillment.requested() != cart_quantity_total(order.items())? {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if !allocations_match_cart_skus(order.items(), fulfillment.allocations()) {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if !allocations_use_warehouse(&warehouse, fulfillment.allocations()) {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if quote.package != package || quote.service.zone != destination.zone {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if cart_weight_total(order.items())? > package.weight {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        if promised_delivery_at < planned_ship_at {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            id,
            order,
            fulfillment,
            package,
            quote,
            warehouse,
            destination,
            planned_ship_at,
            promised_delivery_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogisticsShipment {
    pub(crate) id: ShipmentId,
    pub(crate) plan: LogisticsShipmentPlan,
    pub(crate) status: ShipmentStatus,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,
}

impl LogisticsShipment {
    pub fn try_new(
        id: ShipmentId,
        plan: LogisticsShipmentPlan,
        status: ShipmentStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> DomainResult<Self> {
        if id != plan.id || updated_at < created_at {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            id,
            plan,
            status,
            created_at,
            updated_at,
        })
    }
}

pub fn transition_shipment(
    shipment: LogisticsShipment,
    next: ShipmentStatus,
    updated_at: Timestamp,
) -> DomainResult<LogisticsShipment> {
    if !can_shipment_transition(shipment.status, next) || updated_at < shipment.created_at {
        return Err(ValidationError::LogisticsInvariantFailed);
    }
    Ok(LogisticsShipment {
        status: next,
        updated_at,
        ..shipment
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CarrierHandoff {
    pub(crate) plan: LogisticsShipmentPlan,
    pub(crate) service: CarrierService,
    pub(crate) tracking_number: Id,
    pub(crate) handed_off_at: Timestamp,
    pub(crate) acceptance_scan_at: Timestamp,
}

impl CarrierHandoff {
    pub fn try_new(
        plan: LogisticsShipmentPlan,
        service: CarrierService,
        tracking_number: Id,
        handed_off_at: Timestamp,
        acceptance_scan_at: Timestamp,
    ) -> DomainResult<Self> {
        if service != plan.quote.service
            || handed_off_at < plan.planned_ship_at
            || acceptance_scan_at < handed_off_at
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            plan,
            service,
            tracking_number,
            handed_off_at,
            acceptance_scan_at,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrackingEventKind {
    LabelCreated,
    PickupScan,
    InTransitScan,
    OutForDeliveryScan,
    DeliveredScan,
    ExceptionScan,
    ReturnScan,
}

pub fn can_tracking_progress(source: TrackingEventKind, target: TrackingEventKind) -> bool {
    matches!(
        (source, target),
        (
            TrackingEventKind::LabelCreated,
            TrackingEventKind::LabelCreated
        ) | (
            TrackingEventKind::LabelCreated,
            TrackingEventKind::PickupScan
        ) | (
            TrackingEventKind::PickupScan,
            TrackingEventKind::InTransitScan
        ) | (
            TrackingEventKind::InTransitScan,
            TrackingEventKind::InTransitScan
        ) | (
            TrackingEventKind::InTransitScan,
            TrackingEventKind::OutForDeliveryScan
        ) | (
            TrackingEventKind::InTransitScan,
            TrackingEventKind::ExceptionScan
        ) | (
            TrackingEventKind::OutForDeliveryScan,
            TrackingEventKind::DeliveredScan
        ) | (
            TrackingEventKind::OutForDeliveryScan,
            TrackingEventKind::ExceptionScan
        ) | (
            TrackingEventKind::ExceptionScan,
            TrackingEventKind::InTransitScan
        ) | (
            TrackingEventKind::ExceptionScan,
            TrackingEventKind::ReturnScan
        )
    )
}

domain_struct! {
    pub struct TrackingEvent {
        id: TrackingEventId,
        shipment_id: ShipmentId,
        carrier_id: Id,
        tracking_number: Id,
        kind: TrackingEventKind,
        occurred_at: Timestamp,
    }
}

pub fn tracking_events_monotone_from(last: Timestamp, events: &[TrackingEvent]) -> bool {
    let mut cursor = last;
    for event in events {
        if event.occurred_at < cursor {
            return false;
        }
        cursor = event.occurred_at;
    }
    true
}

pub fn tracking_events_for_shipment(shipment_id: ShipmentId, events: &[TrackingEvent]) -> bool {
    events.iter().all(|event| event.shipment_id == shipment_id)
}

pub fn tracking_events_for_carrier(
    carrier_id: Id,
    tracking_number: Id,
    events: &[TrackingEvent],
) -> bool {
    events
        .iter()
        .all(|event| event.carrier_id == carrier_id && event.tracking_number == tracking_number)
}

pub fn tracking_last_observed_from(last: Timestamp, events: &[TrackingEvent]) -> Timestamp {
    events.last().map_or(last, |event| event.occurred_at)
}

pub fn tracking_event_ids_distinct(events: &[TrackingEvent]) -> bool {
    let mut seen = HashSet::new();
    events.iter().all(|event| seen.insert(event.id.value()))
}

pub fn tracking_events_progress_from(
    last_kind: TrackingEventKind,
    events: &[TrackingEvent],
) -> bool {
    let mut cursor = last_kind;
    for event in events {
        if !can_tracking_progress(cursor, event.kind) {
            return false;
        }
        cursor = event.kind;
    }
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackingHistory {
    pub(crate) shipment_id: ShipmentId,
    pub(crate) carrier_id: Id,
    pub(crate) tracking_number: Id,
    pub(crate) events: Vec<TrackingEvent>,
    pub(crate) last_observed_at: Timestamp,
}

impl TrackingHistory {
    pub fn try_new(
        shipment_id: ShipmentId,
        carrier_id: Id,
        tracking_number: Id,
        events: Vec<TrackingEvent>,
        last_observed_at: Timestamp,
    ) -> DomainResult<Self> {
        if !tracking_events_monotone_from(unix_epoch_timestamp(), &events)
            || !tracking_events_for_shipment(shipment_id, &events)
            || !tracking_events_for_carrier(carrier_id, tracking_number, &events)
            || !tracking_event_ids_distinct(&events)
            || !tracking_events_progress_from(TrackingEventKind::LabelCreated, &events)
            || last_observed_at != tracking_last_observed_from(unix_epoch_timestamp(), &events)
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            shipment_id,
            carrier_id,
            tracking_number,
            events,
            last_observed_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeliveryPromise {
    pub(crate) plan: LogisticsShipmentPlan,
    pub(crate) promised_by: Timestamp,
}

impl DeliveryPromise {
    pub fn try_new(plan: LogisticsShipmentPlan, promised_by: Timestamp) -> DomainResult<Self> {
        if promised_by != plan.promised_delivery_at {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self { plan, promised_by })
    }
}

pub fn delivered_by_promise(promise: &DeliveryPromise, delivered_at: Timestamp) -> bool {
    delivered_at <= promise.promised_by
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeliveredShipment {
    pub(crate) promise: DeliveryPromise,
    pub(crate) history: TrackingHistory,
    pub(crate) delivery_event: TrackingEvent,
    pub(crate) delivered_at: Timestamp,
}

impl DeliveredShipment {
    pub fn try_new(
        promise: DeliveryPromise,
        history: TrackingHistory,
        delivery_event: TrackingEvent,
        delivered_at: Timestamp,
    ) -> DomainResult<Self> {
        if history.shipment_id != promise.plan.id
            || history.carrier_id != promise.plan.quote.service.carrier_id
            || !history.events.contains(&delivery_event)
            || delivery_event.kind != TrackingEventKind::DeliveredScan
            || delivery_event.occurred_at != delivered_at
            || delivery_event.shipment_id != promise.plan.id
            || delivery_event.carrier_id != history.carrier_id
            || delivery_event.tracking_number != history.tracking_number
            || delivered_at < promise.plan.planned_ship_at
            || !delivered_by_promise(&promise, delivered_at)
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            promise,
            history,
            delivery_event,
            delivered_at,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogisticsExceptionKind {
    CarrierDelay,
    WeatherDelay,
    AddressIssue,
    LostPackage,
    DamagedPackage,
    CustomerUnavailable,
}

domain_struct! {
    pub struct LogisticsException {
        shipment_id: ShipmentId,
        kind: LogisticsExceptionKind,
        raised_at: Timestamp,
        customer_visible: bool,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarehouseTransfer {
    pub(crate) id: TransferId,
    pub(crate) sku: Sku,
    pub(crate) from_warehouse: Warehouse,
    pub(crate) to_warehouse: Warehouse,
    pub(crate) source_stock: StockState,
    pub(crate) requested: Quantity,
    pub(crate) in_transit: Quantity,
    pub(crate) received: Quantity,
}

impl WarehouseTransfer {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: TransferId,
        sku: Sku,
        from_warehouse: Warehouse,
        to_warehouse: Warehouse,
        source_stock: StockState,
        requested: Quantity,
        in_transit: Quantity,
        received: Quantity,
    ) -> DomainResult<Self> {
        if source_stock.sku() != sku
            || from_warehouse.id() == to_warehouse.id()
            || requested > available_stock(&source_stock)
            || in_transit > requested
            || received > in_transit
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            id,
            sku,
            from_warehouse,
            to_warehouse,
            source_stock,
            requested,
            in_transit,
            received,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReturnAuthorizationStatus {
    Requested,
    Approved,
    Rejected,
    Received,
    Refunded,
    Closed,
}

pub fn can_return_authorization_transition(
    source: ReturnAuthorizationStatus,
    target: ReturnAuthorizationStatus,
) -> bool {
    matches!(
        (source, target),
        (
            ReturnAuthorizationStatus::Requested,
            ReturnAuthorizationStatus::Approved
        ) | (
            ReturnAuthorizationStatus::Requested,
            ReturnAuthorizationStatus::Rejected
        ) | (
            ReturnAuthorizationStatus::Approved,
            ReturnAuthorizationStatus::Received
        ) | (
            ReturnAuthorizationStatus::Received,
            ReturnAuthorizationStatus::Refunded
        ) | (
            ReturnAuthorizationStatus::Refunded,
            ReturnAuthorizationStatus::Closed
        )
    )
}

domain_struct! {
    pub struct ReturnLine {
        sku: Sku,
        quantity: Quantity,
        refund_amount: Money,
    }
}

pub fn return_lines_quantity_total(lines: &[ReturnLine]) -> DomainResult<Quantity> {
    checked_sum(
        lines.iter().map(|line| line.quantity),
        "return_lines_quantity_total",
    )
}

pub fn return_lines_refund_total(lines: &[ReturnLine]) -> DomainResult<Money> {
    checked_sum(
        lines.iter().map(|line| line.refund_amount),
        "return_lines_refund_total",
    )
}

pub fn return_lines_match_order_skus(items: &[CartLine], lines: &[ReturnLine]) -> bool {
    lines.iter().all(|line| cart_contains_sku(line.sku, items))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturnAuthorization {
    pub(crate) id: ReturnAuthorizationId,
    pub(crate) support_case: SupportCase,
    pub(crate) order: Order,
    pub(crate) ledger: PaymentLedger,
    pub(crate) status: ReturnAuthorizationStatus,
    pub(crate) lines: Vec<ReturnLine>,
    pub(crate) quantity: Quantity,
    pub(crate) refund_amount: Money,
    pub(crate) requested_at: Timestamp,
    pub(crate) decided_at: Timestamp,
}

impl ReturnAuthorization {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: ReturnAuthorizationId,
        support_case: SupportCase,
        order: Order,
        ledger: PaymentLedger,
        status: ReturnAuthorizationStatus,
        lines: Vec<ReturnLine>,
        quantity: Quantity,
        refund_amount: Money,
        requested_at: Timestamp,
        decided_at: Timestamp,
    ) -> DomainResult<Self> {
        if support_case.order_id != Some(order.id())
            || !return_lines_match_order_skus(order.items(), &lines)
            || return_lines_quantity_total(&lines)? != quantity
            || return_lines_refund_total(&lines)? != refund_amount
            || quantity > cart_quantity_total(order.items())?
            || !can_refund(&ledger, refund_amount)
            || ledger.captured() != order.total()
            || decided_at < requested_at
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            id,
            support_case,
            order,
            ledger,
            status,
            lines,
            quantity,
            refund_amount,
            requested_at,
            decided_at,
        })
    }
}

pub fn return_authorization_approved(authorization: &ReturnAuthorization) -> bool {
    authorization.status == ReturnAuthorizationStatus::Approved
}

pub fn transition_return_authorization(
    authorization: ReturnAuthorization,
    next: ReturnAuthorizationStatus,
    decided_at: Timestamp,
) -> DomainResult<ReturnAuthorization> {
    if !can_return_authorization_transition(authorization.status, next)
        || decided_at < authorization.requested_at
    {
        return Err(ValidationError::LogisticsInvariantFailed);
    }
    Ok(ReturnAuthorization {
        status: next,
        decided_at,
        ..authorization
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturnReceipt {
    pub(crate) authorization: ReturnAuthorization,
    pub(crate) received_quantity: Quantity,
    pub(crate) refund_issued: Money,
    pub(crate) received_at: Timestamp,
}

impl ReturnReceipt {
    pub fn try_new(
        authorization: ReturnAuthorization,
        received_quantity: Quantity,
        refund_issued: Money,
        received_at: Timestamp,
    ) -> DomainResult<Self> {
        if !return_authorization_approved(&authorization)
            || received_quantity > authorization.quantity
            || refund_issued > authorization.refund_amount
            || received_at < authorization.decided_at
        {
            return Err(ValidationError::LogisticsInvariantFailed);
        }
        Ok(Self {
            authorization,
            received_quantity,
            refund_issued,
            received_at,
        })
    }
}

impl_getters!(LogisticsShipmentPlan {
    id: ShipmentId,
    order: Order,
    fulfillment: DistinctFulfillmentPlan,
    package: Package,
    quote: CarrierQuote,
    warehouse: Warehouse,
    destination: ShippingDestination,
    planned_ship_at: Timestamp,
    promised_delivery_at: Timestamp,
});

impl_getters!(LogisticsShipment {
    id: ShipmentId,
    plan: LogisticsShipmentPlan,
    status: ShipmentStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
});

impl_getters!(CarrierHandoff {
    plan: LogisticsShipmentPlan,
    service: CarrierService,
    tracking_number: Id,
    handed_off_at: Timestamp,
    acceptance_scan_at: Timestamp,
});

impl_getters!(TrackingHistory {
    shipment_id: ShipmentId,
    carrier_id: Id,
    tracking_number: Id,
    events: Vec<TrackingEvent>,
    last_observed_at: Timestamp,
});

impl_getters!(DeliveryPromise {
    plan: LogisticsShipmentPlan,
    promised_by: Timestamp,
});

impl_getters!(DeliveredShipment {
    promise: DeliveryPromise,
    history: TrackingHistory,
    delivery_event: TrackingEvent,
    delivered_at: Timestamp,
});

impl_getters!(WarehouseTransfer {
    id: TransferId,
    sku: Sku,
    from_warehouse: Warehouse,
    to_warehouse: Warehouse,
    source_stock: StockState,
    requested: Quantity,
    in_transit: Quantity,
    received: Quantity,
});

impl_getters!(ReturnAuthorization {
    id: ReturnAuthorizationId,
    support_case: SupportCase,
    order: Order,
    ledger: PaymentLedger,
    status: ReturnAuthorizationStatus,
    lines: Vec<ReturnLine>,
    quantity: Quantity,
    refund_amount: Money,
    requested_at: Timestamp,
    decided_at: Timestamp,
});

impl_getters!(ReturnReceipt {
    authorization: ReturnAuthorization,
    received_quantity: Quantity,
    refund_issued: Money,
    received_at: Timestamp,
});
