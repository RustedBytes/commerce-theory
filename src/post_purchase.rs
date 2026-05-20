use crate::event_sourcing::*;
use crate::foundation::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubscriptionLifecycleStatus {
    Active,
    Paused,
    PastDue,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubscriptionPlan {
    pub(crate) price: Money,
    pub(crate) period_days: Days,
}

impl SubscriptionPlan {
    pub fn try_new(price: Money, period_days: Days) -> DomainResult<Self> {
        if period_days == 0 {
            return Err(ValidationError::Invariant(
                "subscription period must be positive",
            ));
        }
        Ok(Self { price, period_days })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecurringSubscription {
    pub(crate) customer: CustomerId,
    pub(crate) plan: SubscriptionPlan,
    pub(crate) status: SubscriptionLifecycleStatus,
    pub(crate) current_billing_date: Timestamp,
    pub(crate) next_billing_date: Timestamp,
}

impl RecurringSubscription {
    pub fn try_new(
        customer: CustomerId,
        plan: SubscriptionPlan,
        status: SubscriptionLifecycleStatus,
        current_billing_date: Timestamp,
        next_billing_date: Timestamp,
    ) -> DomainResult<Self> {
        if current_billing_date >= next_billing_date {
            return Err(ValidationError::Invariant(
                "next billing date must be after current date",
            ));
        }
        Ok(Self {
            customer,
            plan,
            status,
            current_billing_date,
            next_billing_date,
        })
    }
}

domain_struct! {
    pub struct GiftCard {
        balance: Money,
        expires_at: Timestamp,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GiftCardRedemption {
    pub(crate) card: GiftCard,
    pub(crate) amount: Money,
}

impl GiftCardRedemption {
    pub fn try_new(card: GiftCard, amount: Money) -> DomainResult<Self> {
        if amount > card.balance {
            return Err(ValidationError::Invariant(
                "gift-card redemption exceeds balance",
            ));
        }
        Ok(Self { card, amount })
    }
}

pub fn gift_card_balance_after_redeem(redemption: &GiftCardRedemption) -> Money {
    nat_sub(redemption.card.balance, redemption.amount)
}

pub fn gift_card_valid_at(now: Timestamp, card: &GiftCard) -> bool {
    now <= card.expires_at
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chargeback {
    pub(crate) payment_amount: Money,
    pub(crate) chargeback_amount: Money,
}

impl Chargeback {
    pub fn try_new(payment_amount: Money, chargeback_amount: Money) -> DomainResult<Self> {
        if chargeback_amount > payment_amount {
            return Err(ValidationError::Invariant(
                "chargeback exceeds payment amount",
            ));
        }
        Ok(Self {
            payment_amount,
            chargeback_amount,
        })
    }
}

domain_struct! {
    pub struct CashflowEvent {
        inflow: Money,
        outflow: Money,
    }
}

pub fn cashflow_inflows_total(events: &[CashflowEvent]) -> DomainResult<Money> {
    checked_sum(
        events.iter().map(|event| event.inflow),
        "cashflow_inflows_total",
    )
}

pub fn cashflow_outflows_total(events: &[CashflowEvent]) -> DomainResult<Money> {
    checked_sum(
        events.iter().map(|event| event.outflow),
        "cashflow_outflows_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CashflowPlan {
    pub(crate) starting_cash: Money,
    pub(crate) required_reserve: Money,
    pub(crate) expected_inflows: Money,
    pub(crate) expected_outflows: Money,
}

impl CashflowPlan {
    pub fn try_new(
        starting_cash: Money,
        required_reserve: Money,
        expected_inflows: Money,
        expected_outflows: Money,
    ) -> DomainResult<Self> {
        if checked_add(required_reserve, expected_outflows, "cashflow reserve")?
            > checked_add(starting_cash, expected_inflows, "cashflow available")?
        {
            return Err(ValidationError::Invariant("cashflow reserve is unsafe"));
        }
        Ok(Self {
            starting_cash,
            required_reserve,
            expected_inflows,
            expected_outflows,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventBackedCashflowPlan {
    pub(crate) starting_cash: Money,
    pub(crate) required_reserve: Money,
    pub(crate) events: Vec<CashflowEvent>,
}

impl EventBackedCashflowPlan {
    pub fn try_new(
        starting_cash: Money,
        required_reserve: Money,
        events: Vec<CashflowEvent>,
    ) -> DomainResult<Self> {
        if checked_add(
            required_reserve,
            cashflow_outflows_total(&events)?,
            "event-backed outflows",
        )? > checked_add(
            starting_cash,
            cashflow_inflows_total(&events)?,
            "event-backed inflows",
        )? {
            return Err(ValidationError::Invariant(
                "event-backed cashflow reserve is unsafe",
            ));
        }
        Ok(Self {
            starting_cash,
            required_reserve,
            events,
        })
    }
}

pub(crate) fn _event_anchor(_: Option<DomainEvent>) {}

impl_getters!(SubscriptionPlan {
    price: Money,
    period_days: Days,
});

impl_getters!(RecurringSubscription {
    customer: CustomerId,
    plan: SubscriptionPlan,
    status: SubscriptionLifecycleStatus,
    current_billing_date: Timestamp,
    next_billing_date: Timestamp,
});

impl_getters!(GiftCardRedemption {
    card: GiftCard,
    amount: Money,
});

impl_getters!(Chargeback {
    payment_amount: Money,
    chargeback_amount: Money,
});

impl_getters!(CashflowPlan {
    starting_cash: Money,
    required_reserve: Money,
    expected_inflows: Money,
    expected_outflows: Money,
});

impl_getters!(EventBackedCashflowPlan {
    starting_cash: Money,
    required_reserve: Money,
    events: Vec<CashflowEvent>,
});
