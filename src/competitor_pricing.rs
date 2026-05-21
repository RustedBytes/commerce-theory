use crate::dropship_profit::*;
use crate::dropshipping::*;
use crate::foundation::*;

domain_struct! {
    pub struct CompetitorOffer {
        competitor_id: CompetitorId,
        sku: Sku,
        price: Money,
        currency: Currency,
        active: bool,
        in_stock: bool,
        observed_at: Timestamp,
    }
}

pub fn competitor_offer_relevant(offer: &CompetitorOffer, sku: Sku, currency: Currency) -> bool {
    offer.sku == sku && offer.currency == currency && offer.active && offer.in_stock
}

pub fn price_snapshot_fresh(now: Timestamp, max_age: Duration, observed_at: Timestamp) -> bool {
    observed_at <= now && timestamp_age(now, observed_at) <= max_age
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrustLevel {
    Low,
    Medium,
    High,
}

pub fn trust_allows_auto_repricing(trust: TrustLevel) -> bool {
    matches!(trust, TrustLevel::Medium | TrustLevel::High)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompetitorPriceBenchmark {
    pub(crate) sku: Sku,
    pub(crate) currency: Currency,
    pub(crate) offers: Vec<CompetitorOffer>,
    pub(crate) best_offer: CompetitorOffer,
}

impl CompetitorPriceBenchmark {
    pub fn try_new(
        sku: Sku,
        currency: Currency,
        offers: Vec<CompetitorOffer>,
        best_offer: CompetitorOffer,
    ) -> DomainResult<Self> {
        if !offers.contains(&best_offer) {
            return Err(ValidationError::Invariant("best offer must be in offers"));
        }
        if !competitor_offer_relevant(&best_offer, sku, currency) {
            return Err(ValidationError::Invariant("best offer must be relevant"));
        }
        if offers
            .iter()
            .filter(|offer| competitor_offer_relevant(offer, sku, currency))
            .any(|offer| best_offer.price > offer.price)
        {
            return Err(ValidationError::Invariant(
                "best offer must be the lowest relevant offer",
            ));
        }
        Ok(Self {
            sku,
            currency,
            offers,
            best_offer,
        })
    }

    pub fn best_offer(&self) -> &CompetitorOffer {
        &self.best_offer
    }
}

pub fn customer_net_at_offer_price(price: Money, discount: Money) -> Money {
    nat_sub(price, discount)
}

pub fn profit_at_offer_price(
    price: Money,
    discount: Money,
    costs: &DropshipProfitCosts,
) -> DomainResult<Money> {
    Ok(profit_amount(
        customer_net_at_offer_price(price, discount),
        dropship_profit_costs_total(costs)?,
    ))
}

pub fn profitable_price_floor(
    costs: &DropshipProfitCosts,
    min_profit: Money,
    discount: Money,
) -> DomainResult<Money> {
    checked_add(
        checked_add(
            dropship_profit_costs_total(costs)?,
            min_profit,
            "profitable_price_floor profit",
        )?,
        discount,
        "profitable_price_floor discount",
    )
}

pub fn price_profitable_for_min_profit(
    price: Money,
    discount: Money,
    costs: &DropshipProfitCosts,
    min_profit: Money,
) -> DomainResult<bool> {
    Ok(profitable_price_floor(costs, min_profit, discount)? <= price)
}

pub fn price_at_or_below_competitor(own_price: Money, competitor_price: Money) -> bool {
    own_price <= competitor_price
}

pub fn undercut_price(competitor_price: Money, delta: Money) -> Money {
    nat_sub(competitor_price, delta)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompetitivePricingStrategy {
    Match,
    Undercut(Money),
    Premium(Money),
}

pub fn target_price_from_strategy(
    strategy: CompetitivePricingStrategy,
    reference_price: Money,
) -> DomainResult<Money> {
    match strategy {
        CompetitivePricingStrategy::Match => Ok(reference_price),
        CompetitivePricingStrategy::Undercut(delta) => Ok(undercut_price(reference_price, delta)),
        CompetitivePricingStrategy::Premium(premium) => {
            checked_add(reference_price, premium, "target_price_from_strategy")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompetitorAwareDropshipOffer {
    pub(crate) offer: DropshipOffer,
    pub(crate) benchmark: CompetitorPriceBenchmark,
    pub(crate) discount: Money,
    pub(crate) costs: DropshipProfitCosts,
    pub(crate) min_profit: Money,
}

impl CompetitorAwareDropshipOffer {
    pub fn try_new(
        offer: DropshipOffer,
        benchmark: CompetitorPriceBenchmark,
        discount: Money,
        costs: DropshipProfitCosts,
        min_profit: Money,
    ) -> DomainResult<Self> {
        if benchmark.sku != offer.sku() {
            return Err(ValidationError::Invariant("benchmark SKU mismatch"));
        }
        if benchmark.currency != offer.currency() {
            return Err(ValidationError::Invariant("benchmark currency mismatch"));
        }
        if !price_profitable_for_min_profit(offer.sale_unit_price(), discount, &costs, min_profit)?
        {
            return Err(ValidationError::Invariant("offer price below profit floor"));
        }
        if offer.sale_unit_price() > benchmark.best_offer.price {
            return Err(ValidationError::Invariant(
                "offer price exceeds best competitor price",
            ));
        }
        Ok(Self {
            offer,
            benchmark,
            discount,
            costs,
            min_profit,
        })
    }
}

impl_getters!(CompetitorPriceBenchmark {
    sku: Sku,
    currency: Currency,
    offers: Vec<CompetitorOffer>,
});

impl_getters!(CompetitorAwareDropshipOffer {
    offer: DropshipOffer,
    benchmark: CompetitorPriceBenchmark,
    discount: Money,
    costs: DropshipProfitCosts,
    min_profit: Money,
});
