use crate::event_sourcing::*;
use crate::foundation::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timed<T> {
    pub ret: T,
    pub time: Nat,
}

impl<T> Timed<T> {
    pub const fn new(ret: T, time: Nat) -> Self {
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

    #[must_use]
    pub const fn before(&self) -> &WebhookOrderingState {
        match self {
            Self::Accept { before, .. } => before,
        }
    }

    #[must_use]
    pub const fn after(&self) -> &WebhookOrderingState {
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
    ReservationReleased(Sku, Quantity),
    ReservedShipmentConfirmed(Sku, Quantity),
    TaxLiabilityRecorded(Money),
    CrmProjected,
    LogisticsProjected,
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
            ValidSystemEvent::ReservationReleased(sku, quantity) => {
                apply_reservation_released_event(&state, sku, quantity)?
            }
            ValidSystemEvent::ReservedShipmentConfirmed(sku, quantity) => {
                apply_reserved_shipment_confirmed_event(&state, sku, quantity)?
            }
            ValidSystemEvent::TaxLiabilityRecorded(amount) => {
                apply_tax_liability_recorded_event(&state, amount)?
            }
            ValidSystemEvent::CrmProjected => apply_crm_projected_event(&state)?,
            ValidSystemEvent::LogisticsProjected => apply_logistics_projected_event(&state)?,
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
    ReservationReleased {
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
        after: ValidSystemState,
    },
    ReservedShipmentConfirmed {
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
        after: ValidSystemState,
    },
    TaxLiabilityRecorded {
        before: ValidSystemState,
        amount: Money,
        after: ValidSystemState,
    },
    CrmProjected {
        before: ValidSystemState,
        after: ValidSystemState,
    },
    LogisticsProjected {
        before: ValidSystemState,
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

    pub fn reservation_released(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_reservation_released_event(&before, sku, quantity)?;
        Ok(Self::ReservationReleased {
            before,
            sku,
            quantity,
            after,
        })
    }

    pub fn reserved_shipment_confirmed(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_reserved_shipment_confirmed_event(&before, sku, quantity)?;
        Ok(Self::ReservedShipmentConfirmed {
            before,
            sku,
            quantity,
            after,
        })
    }

    pub fn tax_liability_recorded(before: ValidSystemState, amount: Money) -> DomainResult<Self> {
        let after = apply_tax_liability_recorded_event(&before, amount)?;
        Ok(Self::TaxLiabilityRecorded {
            before,
            amount,
            after,
        })
    }

    pub fn crm_projected(before: ValidSystemState) -> DomainResult<Self> {
        let after = apply_crm_projected_event(&before)?;
        Ok(Self::CrmProjected { before, after })
    }

    pub fn logistics_projected(before: ValidSystemState) -> DomainResult<Self> {
        let after = apply_logistics_projected_event(&before)?;
        Ok(Self::LogisticsProjected { before, after })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidDomainEventStep {
    StockReserved {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    RefundIssued {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    ReservationReleased {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    ReservedShipmentConfirmed {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    TaxLiabilityRecorded {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    CrmProjected {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
    LogisticsProjected {
        event: DomainEvent,
        before: ValidSystemState,
        after: ValidSystemState,
    },
}

impl ValidDomainEventStep {
    pub fn stock_reserved(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_stock_reserved_event(&before, sku, quantity)?;
        Ok(Self::StockReserved {
            event: DomainEvent::StockReserved(sku, quantity),
            before,
            after,
        })
    }

    pub fn refund_issued(
        before: ValidSystemState,
        order_id: OrderId,
        amount: Money,
    ) -> DomainResult<Self> {
        let after = apply_refund_issued_event(&before, amount)?;
        Ok(Self::RefundIssued {
            event: DomainEvent::RefundIssued(order_id, amount),
            before,
            after,
        })
    }

    pub fn reservation_released(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_reservation_released_event(&before, sku, quantity)?;
        Ok(Self::ReservationReleased {
            event: DomainEvent::ReservationReleased(sku, quantity),
            before,
            after,
        })
    }

    pub fn reserved_shipment_confirmed(
        before: ValidSystemState,
        sku: Sku,
        quantity: Quantity,
    ) -> DomainResult<Self> {
        let after = apply_reserved_shipment_confirmed_event(&before, sku, quantity)?;
        Ok(Self::ReservedShipmentConfirmed {
            event: DomainEvent::ReservedShipmentConfirmed(sku, quantity),
            before,
            after,
        })
    }

    pub fn tax_liability_recorded(
        before: ValidSystemState,
        id: Id,
        amount: Money,
    ) -> DomainResult<Self> {
        let after = apply_tax_liability_recorded_event(&before, amount)?;
        Ok(Self::TaxLiabilityRecorded {
            event: DomainEvent::TaxLiabilityRecorded(id, amount),
            before,
            after,
        })
    }

    pub fn crm_projected(before: ValidSystemState, event: DomainEvent) -> DomainResult<Self> {
        if !domain_event_is_crm(&event) {
            return Err(ValidationError::EventStreamInvalid);
        }
        let after = apply_crm_projected_event(&before)?;
        Ok(Self::CrmProjected {
            event,
            before,
            after,
        })
    }

    pub fn logistics_projected(before: ValidSystemState, event: DomainEvent) -> DomainResult<Self> {
        if !domain_event_is_logistics(&event) {
            return Err(ValidationError::EventStreamInvalid);
        }
        let after = apply_logistics_projected_event(&before)?;
        Ok(Self::LogisticsProjected {
            event,
            before,
            after,
        })
    }

    pub fn from_event(before: ValidSystemState, event: DomainEvent) -> DomainResult<Self> {
        match event {
            DomainEvent::StockReserved(sku, quantity) => {
                Self::stock_reserved(before, sku, quantity)
            }
            DomainEvent::RefundIssued(order_id, amount) => {
                Self::refund_issued(before, order_id, amount)
            }
            DomainEvent::ReservationReleased(sku, quantity) => {
                Self::reservation_released(before, sku, quantity)
            }
            DomainEvent::ReservedShipmentConfirmed(sku, quantity) => {
                Self::reserved_shipment_confirmed(before, sku, quantity)
            }
            DomainEvent::TaxLiabilityRecorded(id, amount) => {
                Self::tax_liability_recorded(before, id, amount)
            }
            event if domain_event_is_crm(&event) => Self::crm_projected(before, event),
            event if domain_event_is_logistics(&event) => Self::logistics_projected(before, event),
            _ => Err(ValidationError::EventStreamInvalid),
        }
    }

    #[must_use]
    pub const fn event(&self) -> &DomainEvent {
        match self {
            Self::StockReserved { event, .. }
            | Self::RefundIssued { event, .. }
            | Self::ReservationReleased { event, .. }
            | Self::ReservedShipmentConfirmed { event, .. }
            | Self::TaxLiabilityRecorded { event, .. }
            | Self::CrmProjected { event, .. }
            | Self::LogisticsProjected { event, .. } => event,
        }
    }

    #[must_use]
    pub const fn before(&self) -> &ValidSystemState {
        match self {
            Self::StockReserved { before, .. }
            | Self::RefundIssued { before, .. }
            | Self::ReservationReleased { before, .. }
            | Self::ReservedShipmentConfirmed { before, .. }
            | Self::TaxLiabilityRecorded { before, .. }
            | Self::CrmProjected { before, .. }
            | Self::LogisticsProjected { before, .. } => before,
        }
    }

    #[must_use]
    pub const fn after(&self) -> &ValidSystemState {
        match self {
            Self::StockReserved { after, .. }
            | Self::RefundIssued { after, .. }
            | Self::ReservationReleased { after, .. }
            | Self::ReservedShipmentConfirmed { after, .. }
            | Self::TaxLiabilityRecorded { after, .. }
            | Self::CrmProjected { after, .. }
            | Self::LogisticsProjected { after, .. } => after,
        }
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
