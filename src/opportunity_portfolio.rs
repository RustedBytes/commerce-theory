use crate::competitor_pricing::*;
use crate::dropship_profit::*;
use crate::forecasting::*;
use crate::foundation::*;

domain_struct! {
    pub struct DistributorProduct {
        distributor_id: SupplierId,
        sku: Sku,
        unit_cost: Money,
        supplier_shipping_per_unit: Money,
        available_qty: Quantity,
        min_order_qty: Quantity,
        currency: Currency,
        active: bool,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipOpportunityCandidate {
    pub(crate) sku: Sku,
    pub(crate) units: Quantity,
    pub(crate) target_price: Money,
    pub(crate) required_capital: Money,
    pub(crate) expected_profit: Money,
    pub(crate) min_profit: Money,
    pub(crate) competitor_price: Money,
    pub(crate) costs: DropshipProfitCosts,
}

impl DropshipOpportunityCandidate {
    pub fn try_new(
        sku: Sku,
        units: Quantity,
        target_price: Money,
        required_capital: Money,
        expected_profit: Money,
        min_profit: Money,
        competitor_price: Money,
        costs: DropshipProfitCosts,
    ) -> DomainResult<Self> {
        if required_capital == 0 {
            return Err(ValidationError::Invariant(
                "opportunity required capital must be positive",
            ));
        }
        if expected_profit < min_profit {
            return Err(ValidationError::Invariant(
                "expected profit below minimum profit",
            ));
        }
        if !price_profitable_for_min_profit(target_price, 0, &costs, min_profit)? {
            return Err(ValidationError::Invariant(
                "target price below profit floor",
            ));
        }
        if target_price > competitor_price {
            return Err(ValidationError::Invariant(
                "target price exceeds competitor price",
            ));
        }
        Ok(Self {
            sku,
            units,
            target_price,
            required_capital,
            expected_profit,
            min_profit,
            competitor_price,
            costs,
        })
    }

    #[must_use]
    pub const fn expected_profit(&self) -> Money {
        self.expected_profit
    }

    #[must_use]
    pub const fn min_profit(&self) -> Money {
        self.min_profit
    }
}

pub fn candidates_capital_total(
    candidates: &[DropshipOpportunityCandidate],
) -> DomainResult<Money> {
    checked_sum(
        candidates
            .iter()
            .map(|candidate| candidate.required_capital),
        "candidates_capital_total",
    )
}

pub fn candidates_profit_total(candidates: &[DropshipOpportunityCandidate]) -> DomainResult<Money> {
    checked_sum(
        candidates.iter().map(|candidate| candidate.expected_profit),
        "candidates_profit_total",
    )
}

pub fn candidates_min_profit_total(
    candidates: &[DropshipOpportunityCandidate],
) -> DomainResult<Money> {
    checked_sum(
        candidates.iter().map(|candidate| candidate.min_profit),
        "candidates_min_profit_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipOpportunityPortfolio {
    pub(crate) selected: Vec<DropshipOpportunityCandidate>,
    pub(crate) investment_fund: Money,
}

impl DropshipOpportunityPortfolio {
    pub fn try_new(
        selected: Vec<DropshipOpportunityCandidate>,
        investment_fund: Money,
    ) -> DomainResult<Self> {
        if candidates_capital_total(&selected)? > investment_fund {
            return Err(ValidationError::Invariant(
                "opportunity portfolio exceeds investment fund",
            ));
        }
        Ok(Self {
            selected,
            investment_fund,
        })
    }
}

pub(crate) const fn _forecasting_anchor(_: Option<Confidence>) {}

impl_getters!(DropshipOpportunityCandidate {
    sku: Sku,
    units: Quantity,
    target_price: Money,
    required_capital: Money,
    competitor_price: Money,
    costs: DropshipProfitCosts,
});

impl_getters!(DropshipOpportunityPortfolio {
    selected: Vec<DropshipOpportunityCandidate>,
    investment_fund: Money,
});
