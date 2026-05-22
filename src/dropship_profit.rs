use crate::dropshipping::*;
use crate::foundation::*;

domain_struct! {
    pub struct DropshipProfitCosts {
        supplier_goods: Money,
        supplier_shipping: Money,
        marketplace_fee: Money,
        payment_fee: Money,
        ad_spend: Money,
        return_reserve: Money,
        tax: Money,
        other_costs: Money,
    }
}

pub fn dropship_profit_costs_total(c: &DropshipProfitCosts) -> DomainResult<Money> {
    checked_sum(
        [
            c.supplier_goods,
            c.supplier_shipping,
            c.marketplace_fee,
            c.payment_fee,
            c.ad_spend,
            c.return_reserve,
            c.tax,
            c.other_costs,
        ],
        "dropship_profit_costs_total",
    )
}

#[must_use]
pub const fn revenue_after_discount(gross: Money, discount: Money) -> Money {
    nat_sub(gross, discount)
}

pub fn required_revenue_for_profit(total_costs: Money, min_profit: Money) -> DomainResult<Money> {
    checked_add(total_costs, min_profit, "required_revenue_for_profit")
}

pub fn required_gross_for_profit(
    total_costs: Money,
    min_profit: Money,
    discount: Money,
) -> DomainResult<Money> {
    checked_add(
        required_revenue_for_profit(total_costs, min_profit)?,
        discount,
        "required_gross_for_profit",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuaranteedDropshipProfitQuote {
    pub(crate) revenue: Money,
    pub(crate) costs: DropshipProfitCosts,
    pub(crate) min_profit: Money,
    pub(crate) profit: Money,
    pub(crate) signed_profit: SignedMoney,
}

impl GuaranteedDropshipProfitQuote {
    pub fn try_new(
        revenue: Money,
        costs: DropshipProfitCosts,
        min_profit: Money,
        profit: Money,
        signed_profit: SignedMoney,
    ) -> DomainResult<Self> {
        let costs_total = dropship_profit_costs_total(&costs)?;
        if profit != profit_amount(revenue, costs_total) {
            return Err(ValidationError::Invariant("profit is incorrect"));
        }
        if signed_profit != profit_loss_amount(revenue, costs_total)? {
            return Err(ValidationError::Invariant("signed profit is incorrect"));
        }
        if checked_add(costs_total, min_profit, "guaranteed quote")? > revenue {
            return Err(ValidationError::Invariant(
                "revenue does not cover costs plus minimum profit",
            ));
        }
        Ok(Self {
            revenue,
            costs,
            min_profit,
            profit,
            signed_profit,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropshipCostUpperBounds {
    pub(crate) actual: DropshipProfitCosts,
    pub(crate) upper: DropshipProfitCosts,
}

impl DropshipCostUpperBounds {
    pub fn try_new(actual: DropshipProfitCosts, upper: DropshipProfitCosts) -> DomainResult<Self> {
        let pairs = [
            (actual.supplier_goods, upper.supplier_goods),
            (actual.supplier_shipping, upper.supplier_shipping),
            (actual.marketplace_fee, upper.marketplace_fee),
            (actual.payment_fee, upper.payment_fee),
            (actual.ad_spend, upper.ad_spend),
            (actual.return_reserve, upper.return_reserve),
            (actual.tax, upper.tax),
            (actual.other_costs, upper.other_costs),
        ];
        if pairs.iter().any(|(a, u)| a > u) {
            return Err(ValidationError::Invariant(
                "actual dropship cost exceeds upper bound",
            ));
        }
        Ok(Self { actual, upper })
    }
}

#[must_use]
pub fn ad_spend_safe_for_min_profit(
    revenue: Money,
    non_ad_costs: Money,
    ad_spend: Money,
    min_profit: Money,
) -> bool {
    non_ad_costs
        .checked_add(ad_spend)
        .and_then(|x| x.checked_add(min_profit))
        .is_some_and(|required| required <= revenue)
}

pub fn profit_after_ad_spend(
    revenue: Money,
    non_ad_costs: Money,
    ad_spend: Money,
) -> DomainResult<Money> {
    Ok(profit_amount(
        revenue,
        checked_add(non_ad_costs, ad_spend, "profit_after_ad_spend")?,
    ))
}

pub fn profit_loss_int(revenue: Money, total_costs: Money) -> DomainResult<i128> {
    profit_loss_amount(revenue, total_costs)
}

pub(crate) const fn _dropshipping_anchor(_: Option<DropshipPOStatus>) {}

impl_getters!(GuaranteedDropshipProfitQuote {
    revenue: Money,
    costs: DropshipProfitCosts,
    min_profit: Money,
    profit: Money,
    signed_profit: SignedMoney,
});

impl_getters!(DropshipCostUpperBounds {
    actual: DropshipProfitCosts,
    upper: DropshipProfitCosts,
});
