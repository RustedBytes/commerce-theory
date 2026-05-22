use crate::event_sourcing::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderEventSymbol {
    OrderPlaced,
    PaymentCaptured,
    RefundIssued,
    StockReserved,
    OrderShipped,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderEventValidationState {
    Start,
    Placed,
    Captured,
    Refunded,
    Shipped,
    Invalid,
}

#[must_use]
pub const fn domain_event_symbol(event: &DomainEvent) -> OrderEventSymbol {
    match event {
        DomainEvent::OrderPlaced(_, _) => OrderEventSymbol::OrderPlaced,
        DomainEvent::PaymentCaptured(_, _) => OrderEventSymbol::PaymentCaptured,
        DomainEvent::RefundIssued(_, _) => OrderEventSymbol::RefundIssued,
        DomainEvent::StockReserved(_, _) => OrderEventSymbol::StockReserved,
        DomainEvent::OrderShipped(_) => OrderEventSymbol::OrderShipped,
        DomainEvent::ReservationReleased(_, _)
        | DomainEvent::ReservedShipmentConfirmed(_, _)
        | DomainEvent::TaxLiabilityRecorded(_, _)
        | DomainEvent::LeadConverted(_, _)
        | DomainEvent::SupportCaseOpened(_, _)
        | DomainEvent::ShipmentPlanned(_, _)
        | DomainEvent::ShipmentDelivered(_)
        | DomainEvent::ReturnApproved(_, _, _) => OrderEventSymbol::Other,
    }
}

pub fn domain_event_symbols(events: &[DomainEvent]) -> Vec<OrderEventSymbol> {
    events.iter().map(domain_event_symbol).collect()
}

#[must_use]
pub const fn order_event_validation_step(
    state: OrderEventValidationState,
    symbol: OrderEventSymbol,
) -> OrderEventValidationState {
    match (state, symbol) {
        (OrderEventValidationState::Start, OrderEventSymbol::OrderPlaced)
        | (OrderEventValidationState::Placed, OrderEventSymbol::StockReserved) => {
            OrderEventValidationState::Placed
        }
        (OrderEventValidationState::Placed, OrderEventSymbol::PaymentCaptured)
        | (OrderEventValidationState::Captured, OrderEventSymbol::StockReserved) => {
            OrderEventValidationState::Captured
        }
        (OrderEventValidationState::Captured, OrderEventSymbol::RefundIssued) => {
            OrderEventValidationState::Refunded
        }
        (OrderEventValidationState::Captured, OrderEventSymbol::OrderShipped) => {
            OrderEventValidationState::Shipped
        }
        (state, OrderEventSymbol::Other) => state,
        _ => OrderEventValidationState::Invalid,
    }
}

pub fn validate_order_event_symbols(symbols: &[OrderEventSymbol]) -> OrderEventValidationState {
    symbols.iter().copied().fold(
        OrderEventValidationState::Start,
        order_event_validation_step,
    )
}

#[must_use]
pub fn order_event_word_accepted(symbols: &[OrderEventSymbol]) -> bool {
    matches!(
        validate_order_event_symbols(symbols),
        OrderEventValidationState::Shipped | OrderEventValidationState::Refunded
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrderEventValidator;

impl OrderEventValidator {
    #[must_use]
    pub const fn start(self) -> OrderEventValidationState {
        OrderEventValidationState::Start
    }

    #[must_use]
    pub const fn step(
        self,
        state: OrderEventValidationState,
        symbol: OrderEventSymbol,
    ) -> OrderEventValidationState {
        order_event_validation_step(state, symbol)
    }

    #[must_use]
    pub fn run(self, symbols: &[OrderEventSymbol]) -> OrderEventValidationState {
        validate_order_event_symbols(symbols)
    }

    #[must_use]
    pub fn accepts(self, symbols: &[OrderEventSymbol]) -> bool {
        order_event_word_accepted(symbols)
    }
}

#[must_use]
pub const fn order_event_validator() -> OrderEventValidator {
    OrderEventValidator
}
