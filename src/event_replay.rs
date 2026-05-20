use crate::event_sourcing::*;
use crate::foundation::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timed<T> {
    pub ret: T,
    pub time: Nat,
}

impl<T> Timed<T> {
    pub fn new(ret: T, time: Nat) -> Self {
        Self { ret, time }
    }
}

pub fn webhook_replay_in_steps(
    state: WebhookOrderingState,
    events: &[EventEnvelope],
) -> DomainResult<Timed<WebhookOrderingState>> {
    let next = replay_webhook_stream(state, events)?;
    Ok(Timed::new(next, events.len() as Nat))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WebhookOrderingStep {
    Accept {
        before: WebhookOrderingState,
        sequence: Nat,
        after: WebhookOrderingState,
    },
}

impl WebhookOrderingStep {
    pub fn accept(before: WebhookOrderingState, sequence: Nat) -> DomainResult<Self> {
        let after = apply_webhook(&before, sequence)?;
        Ok(Self::Accept {
            before,
            sequence,
            after,
        })
    }

    pub fn before(&self) -> &WebhookOrderingState {
        match self {
            Self::Accept { before, .. } => before,
        }
    }

    pub fn after(&self) -> &WebhookOrderingState {
        match self {
            Self::Accept { after, .. } => after,
        }
    }
}

pub fn webhook_replay_within_steps(
    state: WebhookOrderingState,
    events: &[EventEnvelope],
    bound: Nat,
) -> DomainResult<Option<Timed<WebhookOrderingState>>> {
    let replay = webhook_replay_in_steps(state, events)?;
    Ok((replay.time <= bound).then_some(replay))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidSystemEvent {
    StockReserved(Sku, Quantity),
    RefundIssued(Money),
}

pub fn valid_system_replay_in_steps(
    mut state: ValidSystemState,
    events: &[ValidSystemEvent],
) -> DomainResult<Timed<ValidSystemState>> {
    for event in events {
        state = match *event {
            ValidSystemEvent::StockReserved(sku, quantity) => {
                apply_stock_reserved_event(&state, sku, quantity)?
            }
            ValidSystemEvent::RefundIssued(amount) => apply_refund_issued_event(&state, amount)?,
        };
    }
    Ok(Timed::new(state, events.len() as Nat))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidSystemEventStep {
    StockReserved {
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
        after: ValidSystemState,
    },
    RefundIssued {
        before: ValidSystemState,
        amount: Money,
        after: ValidSystemState,
    },
}

impl ValidSystemEventStep {
    pub fn stock_reserved(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_stock_reserved_event(&before, sku, quantity)?;
        Ok(Self::StockReserved {
            before,
            sku,
            quantity,
            after,
        })
    }

    pub fn refund_issued(before: ValidSystemState, amount: Money) -> DomainResult<Self> {
        let after = apply_refund_issued_event(&before, amount)?;
        Ok(Self::RefundIssued {
            before,
            amount,
            after,
        })
    }
}

pub fn valid_system_replay_within_steps(
    state: ValidSystemState,
    events: &[ValidSystemEvent],
    bound: Nat,
) -> DomainResult<Option<Timed<ValidSystemState>>> {
    let replay = valid_system_replay_in_steps(state, events)?;
    Ok((replay.time <= bound).then_some(replay))
}
