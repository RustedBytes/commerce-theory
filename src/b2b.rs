use crate::foundation::*;
use crate::marketing::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TradeMode {
    Retail,
    Wholesale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CustomerKind {
    Guest,
    Registered,
    WholesaleAccount,
}

domain_struct! {
    pub struct Customer {
        id: CustomerId,
        kind: CustomerKind,
        wholesale_approved: bool,
    }
}

#[must_use]
pub fn customer_can_buy_wholesale(customer: &Customer) -> bool {
    customer.kind == CustomerKind::WholesaleAccount && customer.wholesale_approved
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentTerms {
    Prepaid,
    NetDays(Nat),
}

#[must_use]
pub const fn payment_terms_allowed(mode: TradeMode, terms: PaymentTerms) -> bool {
    !matches!((mode, terms), (TradeMode::Retail, PaymentTerms::NetDays(_)))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradePriceBookEntry {
    pub(crate) sku: Sku,
    pub(crate) currency: Currency,
    pub(crate) unit_cost: Money,
    pub(crate) retail_unit_price: Money,
    pub(crate) wholesale_unit_price: Money,
    pub(crate) retail_margin: Money,
    pub(crate) wholesale_margin: Money,
    pub(crate) wholesale_min_qty: Quantity,
}

impl TradePriceBookEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        sku: Sku,
        currency: Currency,
        unit_cost: Money,
        retail_unit_price: Money,
        wholesale_unit_price: Money,
        retail_margin: Money,
        wholesale_margin: Money,
        wholesale_min_qty: Quantity,
    ) -> DomainResult<Self> {
        if checked_add(unit_cost, retail_margin, "retail margin")? > retail_unit_price {
            return Err(ValidationError::Invariant("retail margin is unsafe"));
        }
        if checked_add(unit_cost, wholesale_margin, "wholesale margin")? > wholesale_unit_price {
            return Err(ValidationError::Invariant("wholesale margin is unsafe"));
        }
        if wholesale_unit_price > retail_unit_price {
            return Err(ValidationError::Invariant(
                "wholesale price exceeds retail price",
            ));
        }
        if wholesale_min_qty == 0 {
            return Err(ValidationError::Invariant(
                "wholesale minimum quantity must be positive",
            ));
        }
        Ok(Self {
            sku,
            currency,
            unit_cost,
            retail_unit_price,
            wholesale_unit_price,
            retail_margin,
            wholesale_margin,
            wholesale_min_qty,
        })
    }
}

#[must_use]
pub const fn unit_price_for_trade_mode(mode: TradeMode, entry: &TradePriceBookEntry) -> Money {
    match mode {
        TradeMode::Retail => entry.retail_unit_price,
        TradeMode::Wholesale => entry.wholesale_unit_price,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetailLine {
    pub(crate) entry: TradePriceBookEntry,
    pub(crate) quantity: Quantity,
    pub(crate) discount: Money,
}

impl RetailLine {
    pub fn try_new(
        entry: TradePriceBookEntry,
        quantity: Quantity,
        discount: Money,
    ) -> DomainResult<Self> {
        if discount > checked_mul(entry.retail_unit_price, quantity, "retail line gross")? {
            return Err(ValidationError::Invariant("retail discount exceeds gross"));
        }
        Ok(Self {
            entry,
            quantity,
            discount,
        })
    }
}

pub fn retail_line_gross_total(line: &RetailLine) -> DomainResult<Money> {
    checked_mul(
        line.entry.retail_unit_price,
        line.quantity,
        "retail_line_gross_total",
    )
}

pub fn retail_line_net_total(line: &RetailLine) -> DomainResult<Money> {
    Ok(nat_sub(retail_line_gross_total(line)?, line.discount))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WholesaleLine {
    pub(crate) entry: TradePriceBookEntry,
    pub(crate) quantity: Quantity,
    pub(crate) discount: Money,
}

impl WholesaleLine {
    pub fn try_new(
        entry: TradePriceBookEntry,
        quantity: Quantity,
        discount: Money,
    ) -> DomainResult<Self> {
        if quantity < entry.wholesale_min_qty {
            return Err(ValidationError::Invariant(
                "wholesale quantity below minimum",
            ));
        }
        if discount > checked_mul(entry.wholesale_unit_price, quantity, "wholesale line gross")? {
            return Err(ValidationError::Invariant(
                "wholesale discount exceeds gross",
            ));
        }
        Ok(Self {
            entry,
            quantity,
            discount,
        })
    }
}

pub fn wholesale_line_gross_total(line: &WholesaleLine) -> DomainResult<Money> {
    checked_mul(
        line.entry.wholesale_unit_price,
        line.quantity,
        "wholesale_line_gross_total",
    )
}

pub fn wholesale_line_retail_equivalent_total(line: &WholesaleLine) -> DomainResult<Money> {
    checked_mul(
        line.entry.retail_unit_price,
        line.quantity,
        "wholesale_line_retail_equivalent_total",
    )
}

pub fn wholesale_line_net_total(line: &WholesaleLine) -> DomainResult<Money> {
    Ok(nat_sub(wholesale_line_gross_total(line)?, line.discount))
}

pub fn wholesale_order_net_total(lines: &[WholesaleLine]) -> DomainResult<Money> {
    checked_result_sum(
        lines.iter().map(wholesale_line_net_total),
        "wholesale_order_net_total",
    )
}

pub fn wholesale_retail_equivalent_total(lines: &[WholesaleLine]) -> DomainResult<Money> {
    checked_result_sum(
        lines.iter().map(wholesale_line_retail_equivalent_total),
        "wholesale_retail_equivalent_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WholesaleCreditAccount {
    pub(crate) customer: Customer,
    pub(crate) credit_limit: Money,
    pub(crate) outstanding: Money,
}

impl WholesaleCreditAccount {
    pub fn try_new(
        customer: Customer,
        credit_limit: Money,
        outstanding: Money,
    ) -> DomainResult<Self> {
        if !customer_can_buy_wholesale(&customer) {
            return Err(ValidationError::Invariant("customer cannot buy wholesale"));
        }
        if outstanding > credit_limit {
            return Err(ValidationError::Invariant(
                "outstanding exceeds credit limit",
            ));
        }
        Ok(Self {
            customer,
            credit_limit,
            outstanding,
        })
    }
}

#[must_use]
pub fn can_place_wholesale_credit_order(
    account: &WholesaleCreditAccount,
    order_total: Money,
) -> bool {
    account
        .outstanding
        .checked_add(order_total)
        .is_some_and(|total| total <= account.credit_limit)
}

pub(crate) const fn _marketing_anchor(_: Option<ConsentStatus>) {}

impl_getters!(TradePriceBookEntry {
    sku: Sku,
    currency: Currency,
    unit_cost: Money,
    retail_unit_price: Money,
    wholesale_unit_price: Money,
    retail_margin: Money,
    wholesale_margin: Money,
    wholesale_min_qty: Quantity,
});

impl_getters!(RetailLine {
    entry: TradePriceBookEntry,
    quantity: Quantity,
    discount: Money,
});

impl_getters!(WholesaleLine {
    entry: TradePriceBookEntry,
    quantity: Quantity,
    discount: Money,
});

impl_getters!(WholesaleCreditAccount {
    customer: Customer,
    credit_limit: Money,
    outstanding: Money,
});
