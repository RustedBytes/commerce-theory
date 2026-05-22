use crate::foundation::*;
use crate::merchandising::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExchangeRate {
    pub(crate) source: Currency,
    pub(crate) target: Currency,
    pub(crate) numerator: Nat,
    pub(crate) denominator: Nat,
    pub(crate) observed_at: Timestamp,
}

impl ExchangeRate {
    pub const fn try_new(
        source: Currency,
        target: Currency,
        numerator: Nat,
        denominator: Nat,
        observed_at: Timestamp,
    ) -> DomainResult<Self> {
        if denominator == 0 {
            return Err(ValidationError::Invariant(
                "exchange-rate denominator must be positive",
            ));
        }
        Ok(Self {
            source,
            target,
            numerator,
            denominator,
            observed_at,
        })
    }
}

#[must_use]
pub fn fx_quote_fresh(now: Timestamp, max_age: Duration, rate: &ExchangeRate) -> bool {
    rate.observed_at <= now && timestamp_age(now, rate.observed_at) <= max_age
}

pub fn convert_money_rounded(
    mode: RoundingMode,
    amount: Money,
    rate: &ExchangeRate,
) -> DomainResult<Money> {
    round_money(
        mode,
        checked_mul(amount, rate.numerator, "convert_money_floor multiply")?,
        rate.denominator,
    )
}

pub fn convert_money_floor(amount: Money, rate: &ExchangeRate) -> DomainResult<Money> {
    convert_money_rounded(RoundingMode::Floor, amount, rate)
}

domain_struct! {
    pub struct TaxRate {
        bps: BasisPoints,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxCalculation {
    pub(crate) taxable_amount: Money,
    pub(crate) rate: TaxRate,
    pub(crate) rounding_mode: RoundingMode,
    pub(crate) tax: Money,
    pub(crate) total: Money,
}

impl TaxCalculation {
    pub fn try_new(
        taxable_amount: Money,
        rate: TaxRate,
        rounding_mode: RoundingMode,
        tax: Money,
        total: Money,
    ) -> DomainResult<Self> {
        if tax != tax_amount_rounded(rounding_mode, &rate, taxable_amount)? {
            return Err(ValidationError::Invariant("tax amount is incorrect"));
        }
        if total != checked_add(taxable_amount, tax, "tax calculation total")? {
            return Err(ValidationError::Invariant("tax total is incorrect"));
        }
        Ok(Self {
            taxable_amount,
            rate,
            rounding_mode,
            tax,
            total,
        })
    }
}

pub fn tax_amount_rounded(
    mode: RoundingMode,
    rate: &TaxRate,
    taxable_amount: Money,
) -> DomainResult<Money> {
    round_bps_amount(mode, taxable_amount, rate.bps())
}

domain_struct! {
    pub struct ShippingZone {
        id: Id,
        name: String,
    }
}

domain_struct! {
    pub struct CarrierService {
        carrier_id: Id,
        zone: ShippingZone,
        max_weight: Weight,
        base_cost: Money,
        promised_days: Days,
    }
}

domain_struct! {
    pub struct Package {
        weight: Weight,
        volume: Nat,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CarrierQuote {
    pub(crate) service: CarrierService,
    pub(crate) package: Package,
    pub(crate) price: Money,
}

impl CarrierQuote {
    pub fn try_new(service: CarrierService, package: Package, price: Money) -> DomainResult<Self> {
        if package.weight > service.max_weight {
            return Err(ValidationError::Invariant(
                "package exceeds service max weight",
            ));
        }
        if price < service.base_cost {
            return Err(ValidationError::Invariant("quote price below base cost"));
        }
        Ok(Self {
            service,
            package,
            price,
        })
    }
}

#[must_use]
pub const fn abs_diff_nat(a: Nat, b: Nat) -> Nat {
    a.abs_diff(b)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReconciliationWithinTolerance {
    pub(crate) expected: Money,
    pub(crate) actual: Money,
    pub(crate) tolerance: Money,
}

impl ReconciliationWithinTolerance {
    pub const fn try_new(expected: Money, actual: Money, tolerance: Money) -> DomainResult<Self> {
        if abs_diff_nat(expected, actual) > tolerance {
            return Err(ValidationError::Invariant(
                "reconciliation diff exceeds tolerance",
            ));
        }
        Ok(Self {
            expected,
            actual,
            tolerance,
        })
    }
}

pub(crate) const fn _merchandising_anchor(_: Option<PromotionStackingPolicy>) {}

impl_getters!(ExchangeRate {
    source: Currency,
    target: Currency,
    numerator: Nat,
    denominator: Nat,
    observed_at: Timestamp,
});

impl_getters!(TaxCalculation {
    taxable_amount: Money,
    rate: TaxRate,
    rounding_mode: RoundingMode,
    tax: Money,
    total: Money,
});

impl_getters!(CarrierQuote {
    service: CarrierService,
    package: Package,
    price: Money,
});

impl_getters!(ReconciliationWithinTolerance {
    expected: Money,
    actual: Money,
    tolerance: Money,
});
