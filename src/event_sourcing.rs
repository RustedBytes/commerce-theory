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
    OrderShipped(OrderId),
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

pub fn stream_sequences_strictly_increase(stream: &EventStream) -> bool {
    stream_sequences_strictly_increase_from(0, &stream.events)
}

domain_struct! {
    pub struct WebhookOrderingState {
        last_sequence: Nat,
    }
}

pub fn apply_webhook(s: &WebhookOrderingState, seq: Nat) -> DomainResult<WebhookOrderingState> {
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

pub fn already_processed(key: IdempotencyKey, state: &IdempotencyState) -> bool {
    state.processed.contains(&key)
}

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
    ))
}

pub fn apply_refund_issued_event(
    state: &ValidSystemState,
    amount: Money,
) -> DomainResult<ValidSystemState> {
    Ok(ValidSystemState::new(
        state.stock.clone(),
        issue_refund(&state.ledger, amount)?,
    ))
}

pub(crate) fn _risk_anchor(_: Option<Role>) {}
