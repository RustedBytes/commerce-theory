use crate::foundation::*;
use crate::inventory::*;
use crate::orders::*;
use crate::risk_privacy::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DomainEvent {
    OrderPlaced(OrderId, Money),
    PaymentCaptured(OrderId, Money),
    RefundIssued(OrderId, Money),
    StockReserved(Sku, Quantity),
    ReservationReleased(Sku, Quantity),
    ReservedShipmentConfirmed(Sku, Quantity),
    TaxLiabilityRecorded(Id, Money),
    OrderShipped(OrderId),
    LeadConverted(LeadId, OpportunityId),
    SupportCaseOpened(SupportCaseId, Option<OrderId>),
    ShipmentPlanned(ShipmentId, OrderId),
    ShipmentDelivered(ShipmentId),
    ReturnApproved(ReturnAuthorizationId, OrderId, Money),
}

#[must_use]
pub const fn domain_event_is_crm(event: &DomainEvent) -> bool {
    matches!(
        event,
        DomainEvent::LeadConverted(_, _) | DomainEvent::SupportCaseOpened(_, _)
    )
}

#[must_use]
pub const fn domain_event_is_logistics(event: &DomainEvent) -> bool {
    matches!(
        event,
        DomainEvent::ShipmentPlanned(_, _)
            | DomainEvent::ShipmentDelivered(_)
            | DomainEvent::ReturnApproved(_, _, _)
    )
}

domain_struct! {
    pub struct EventEnvelope {
        sequence: Nat,
        event: DomainEvent,
    }
}

domain_struct! {
    pub struct EventStream {
        events: Vec<EventEnvelope>,
        last_sequence: Nat,
    }
}

#[must_use]
pub fn stream_sequences_strictly_increase_from(last: Nat, events: &[EventEnvelope]) -> bool {
    let mut cursor = last;
    for event in events {
        if cursor >= event.sequence {
            return false;
        }
        cursor = event.sequence;
    }
    true
}

#[must_use]
pub fn stream_sequences_strictly_increase(stream: &EventStream) -> bool {
    stream_sequences_strictly_increase_from(0, &stream.events)
}

domain_struct! {
    pub struct WebhookOrderingState {
        last_sequence: Nat,
    }
}

pub const fn apply_webhook(
    s: &WebhookOrderingState,
    seq: Nat,
) -> DomainResult<WebhookOrderingState> {
    if s.last_sequence >= seq {
        return Err(ValidationError::Invariant(
            "webhook sequence must be newer than cursor",
        ));
    }
    Ok(WebhookOrderingState::new(seq))
}

pub fn replay_webhook_stream(
    mut state: WebhookOrderingState,
    events: &[EventEnvelope],
) -> DomainResult<WebhookOrderingState> {
    for event in events {
        state = apply_webhook(&state, event.sequence)?;
    }
    Ok(state)
}

domain_struct! {
    pub struct IdempotencyState {
        processed: Vec<IdempotencyKey>,
    }
}

#[must_use]
pub fn already_processed(key: IdempotencyKey, state: &IdempotencyState) -> bool {
    state.processed.contains(&key)
}

#[must_use]
pub fn mark_processed(key: IdempotencyKey, state: &IdempotencyState) -> IdempotencyState {
    let mut processed = Vec::with_capacity(state.processed.len() + 1);
    processed.push(key);
    processed.extend(state.processed.iter().copied());
    IdempotencyState::new(processed)
}

domain_struct! {
    pub struct ValidSystemState {
        stock: StockState,
        ledger: PaymentLedger,
        tax_liability: Money,
        crm_event_count: Nat,
        logistics_event_count: Nat,
    }
}

pub fn apply_stock_reserved_event(
    state: &ValidSystemState,
    sku: Sku,
    quantity: Quantity,
) -> DomainResult<ValidSystemState> {
    if state.stock.sku() != sku {
        return Err(ValidationError::Invariant("stock-reserved SKU mismatch"));
    }
    Ok(ValidSystemState::new(
        reserve_stock(&state.stock, quantity)?,
        state.ledger.clone(),
        state.tax_liability,
        state.crm_event_count,
        state.logistics_event_count,
    ))
}

pub fn apply_refund_issued_event(
    state: &ValidSystemState,
    amount: Money,
) -> DomainResult<ValidSystemState> {
    Ok(ValidSystemState::new(
        state.stock,
        issue_refund(&state.ledger, amount)?,
        state.tax_liability,
        state.crm_event_count,
        state.logistics_event_count,
    ))
}

pub fn apply_reservation_released_event(
    state: &ValidSystemState,
    sku: Sku,
    quantity: Quantity,
) -> DomainResult<ValidSystemState> {
    if state.stock.sku() != sku {
        return Err(ValidationError::Invariant(
            "reservation-released SKU mismatch",
        ));
    }
    Ok(ValidSystemState::new(
        release_reserved_stock(&state.stock, quantity)?,
        state.ledger.clone(),
        state.tax_liability,
        state.crm_event_count,
        state.logistics_event_count,
    ))
}

pub fn apply_reserved_shipment_confirmed_event(
    state: &ValidSystemState,
    sku: Sku,
    quantity: Quantity,
) -> DomainResult<ValidSystemState> {
    if state.stock.sku() != sku {
        return Err(ValidationError::Invariant(
            "reserved-shipment-confirmed SKU mismatch",
        ));
    }
    Ok(ValidSystemState::new(
        confirm_reserved_shipment(&state.stock, quantity)?,
        state.ledger.clone(),
        state.tax_liability,
        state.crm_event_count,
        state.logistics_event_count,
    ))
}

pub fn apply_tax_liability_recorded_event(
    state: &ValidSystemState,
    amount: Money,
) -> DomainResult<ValidSystemState> {
    Ok(ValidSystemState::new(
        state.stock,
        state.ledger.clone(),
        checked_add(
            state.tax_liability,
            amount,
            "apply_tax_liability_recorded_event",
        )?,
        state.crm_event_count,
        state.logistics_event_count,
    ))
}

pub fn apply_crm_projected_event(state: &ValidSystemState) -> DomainResult<ValidSystemState> {
    Ok(ValidSystemState::new(
        state.stock,
        state.ledger.clone(),
        state.tax_liability,
        checked_add(state.crm_event_count, 1, "apply_crm_projected_event")?,
        state.logistics_event_count,
    ))
}

pub fn apply_logistics_projected_event(state: &ValidSystemState) -> DomainResult<ValidSystemState> {
    Ok(ValidSystemState::new(
        state.stock,
        state.ledger.clone(),
        state.tax_liability,
        state.crm_event_count,
        checked_add(
            state.logistics_event_count,
            1,
            "apply_logistics_projected_event",
        )?,
    ))
}

pub fn record_captured_payment(
    ledger: &PaymentLedger,
    amount: Money,
) -> DomainResult<PaymentLedger> {
    PaymentLedger::try_new(
        checked_add(ledger.captured(), amount, "record_captured_payment")?,
        ledger.refunded(),
    )
}

pub fn apply_domain_event(
    state: &ValidSystemState,
    event: &DomainEvent,
) -> DomainResult<ValidSystemState> {
    match event {
        DomainEvent::OrderPlaced(_, _) | DomainEvent::OrderShipped(_) => Ok(state.clone()),
        DomainEvent::PaymentCaptured(_, amount) => Ok(ValidSystemState::new(
            state.stock,
            record_captured_payment(&state.ledger, *amount)?,
            state.tax_liability,
            state.crm_event_count,
            state.logistics_event_count,
        )),
        DomainEvent::RefundIssued(_, amount) => apply_refund_issued_event(state, *amount),
        DomainEvent::StockReserved(sku, quantity) => {
            apply_stock_reserved_event(state, *sku, *quantity)
        }
        DomainEvent::ReservationReleased(sku, quantity) => {
            apply_reservation_released_event(state, *sku, *quantity)
        }
        DomainEvent::ReservedShipmentConfirmed(sku, quantity) => {
            apply_reserved_shipment_confirmed_event(state, *sku, *quantity)
        }
        DomainEvent::TaxLiabilityRecorded(_, amount) => {
            apply_tax_liability_recorded_event(state, *amount)
        }
        event if domain_event_is_crm(event) => apply_crm_projected_event(state),
        event if domain_event_is_logistics(event) => apply_logistics_projected_event(state),
        _ => Ok(state.clone()),
    }
}

pub fn replay_domain_events(
    mut state: ValidSystemState,
    events: &[DomainEvent],
) -> DomainResult<ValidSystemState> {
    for event in events {
        state = apply_domain_event(&state, event)?;
    }
    Ok(state)
}

pub fn apply_idempotent_domain_event(
    key: IdempotencyKey,
    event: &DomainEvent,
    state: ValidSystemState,
    idempotency: IdempotencyState,
) -> DomainResult<(ValidSystemState, IdempotencyState)> {
    if already_processed(key, &idempotency) {
        Ok((state, idempotency))
    } else {
        Ok((
            apply_domain_event(&state, event)?,
            mark_processed(key, &idempotency),
        ))
    }
}

domain_struct! {
    pub struct EventSnapshot {
        state: ValidSystemState,
        last_sequence: Nat,
    }
}

pub fn replay_from_snapshot(
    snapshot: &EventSnapshot,
    events: &[DomainEvent],
) -> DomainResult<ValidSystemState> {
    replay_domain_events(snapshot.state.clone(), events)
}

pub fn ledger_captured_fold(captured: Money, events: &[DomainEvent]) -> DomainResult<Money> {
    events.iter().try_fold(captured, |acc, event| match event {
        DomainEvent::PaymentCaptured(_, amount) => {
            checked_add(acc, *amount, "ledger_captured_fold")
        }
        _ => Ok(acc),
    })
}

pub fn ledger_refunded_fold(refunded: Money, events: &[DomainEvent]) -> DomainResult<Money> {
    events.iter().try_fold(refunded, |acc, event| match event {
        DomainEvent::RefundIssued(_, amount) => checked_add(acc, *amount, "ledger_refunded_fold"),
        _ => Ok(acc),
    })
}

pub fn tax_liability_fold(liability: Money, events: &[DomainEvent]) -> DomainResult<Money> {
    events.iter().try_fold(liability, |acc, event| match event {
        DomainEvent::TaxLiabilityRecorded(_, amount) => {
            checked_add(acc, *amount, "tax_liability_fold")
        }
        _ => Ok(acc),
    })
}

pub fn project_tax_liability(
    opening_liability: Money,
    events: &[DomainEvent],
) -> DomainResult<Money> {
    tax_liability_fold(opening_liability, events)
}

pub fn project_ledger(
    mut ledger: PaymentLedger,
    events: &[DomainEvent],
) -> DomainResult<PaymentLedger> {
    for event in events {
        ledger = match event {
            DomainEvent::PaymentCaptured(_, amount) => record_captured_payment(&ledger, *amount)?,
            DomainEvent::RefundIssued(_, amount) => issue_refund(&ledger, *amount)?,
            _ => ledger,
        };
    }
    Ok(ledger)
}

pub(crate) const fn _risk_anchor(_: Option<Role>) {}
