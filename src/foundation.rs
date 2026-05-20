use core::fmt;
use core::marker::PhantomData;

pub type Nat = u128;
pub type Money = u128;
pub type Quantity = u128;
pub type Weight = u128;
pub type Timestamp = u128;
pub type Days = u128;
pub type Id = u128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidationError {
    Invariant(&'static str),
    Overflow(&'static str),
    DivisionByZero(&'static str),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invariant(message) => write!(f, "invariant failed: {message}"),
            Self::Overflow(message) => write!(f, "arithmetic overflow: {message}"),
            Self::DivisionByZero(message) => write!(f, "division by zero: {message}"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub type DomainResult<T> = Result<T, ValidationError>;

pub fn checked_add(a: Nat, b: Nat, context: &'static str) -> DomainResult<Nat> {
    a.checked_add(b).ok_or(ValidationError::Overflow(context))
}

pub fn checked_mul(a: Nat, b: Nat, context: &'static str) -> DomainResult<Nat> {
    a.checked_mul(b).ok_or(ValidationError::Overflow(context))
}

pub fn checked_div(a: Nat, b: Nat, context: &'static str) -> DomainResult<Nat> {
    if b == 0 {
        Err(ValidationError::DivisionByZero(context))
    } else {
        Ok(a / b)
    }
}

pub fn checked_sum<I>(items: I, context: &'static str) -> DomainResult<Nat>
where
    I: IntoIterator<Item = Nat>,
{
    items
        .into_iter()
        .try_fold(0, |acc, item| checked_add(acc, item, context))
}

pub const fn nat_sub(a: Nat, b: Nat) -> Nat {
    a.saturating_sub(b)
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name {
            value: Nat,
        }

        impl $name {
            pub const fn new(value: Nat) -> Self {
                Self { value }
            }

            pub fn try_new(value: Nat) -> DomainResult<Self> {
                Ok(Self::new(value))
            }

            pub const fn value(self) -> Nat {
                self.value
            }
        }
    };
}

id_type!(Sku);
id_type!(ProductId);
id_type!(VariantId);
id_type!(CustomerId);
id_type!(OrderId);
id_type!(PaymentId);
id_type!(SupplierId);
id_type!(MarketplaceOrderId);
id_type!(CampaignId);
id_type!(CompetitorId);
id_type!(IdempotencyKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Currency {
    UAH,
    USD,
    EUR,
    GBP,
    PLN,
}

pub trait CurrencyMarker: Clone + Copy + fmt::Debug + PartialEq + Eq {
    const CURRENCY: Currency;
}

macro_rules! currency_marker {
    ($name:ident, $currency:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;
        impl CurrencyMarker for $name {
            const CURRENCY: Currency = Currency::$currency;
        }
    };
}

currency_marker!(Uah, UAH);
currency_marker!(Usd, USD);
currency_marker!(Eur, EUR);
currency_marker!(Gbp, GBP);
currency_marker!(Pln, PLN);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoneyIn<C: CurrencyMarker> {
    amount: Money,
    _currency: PhantomData<C>,
}

impl<C: CurrencyMarker> MoneyIn<C> {
    pub const fn new(amount: Money) -> Self {
        Self {
            amount,
            _currency: PhantomData,
        }
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub const fn amount(self) -> Money {
        self.amount
    }

    pub const fn currency(self) -> Currency {
        C::CURRENCY
    }

    pub fn checked_add(self, other: Self) -> DomainResult<Self> {
        Ok(Self::new(checked_add(
            self.amount,
            other.amount,
            "MoneyIn::add",
        )?))
    }

    pub const fn saturating_sub(self, other: Self) -> Self {
        Self::new(nat_sub(self.amount, other.amount))
    }
}

domain_struct! {
    pub struct MoneyAmount {
        amount: Money,
        currency: Currency,
    }
}

pub fn same_currency(a: &MoneyAmount, b: &MoneyAmount) -> bool {
    a.currency == b.currency
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BasisPoints {
    value: Nat,
}

impl BasisPoints {
    pub fn try_new(value: Nat) -> DomainResult<Self> {
        if value <= 10_000 {
            Ok(Self { value })
        } else {
            Err(ValidationError::Invariant("basis points must be <= 10000"))
        }
    }

    pub const fn value(self) -> Nat {
        self.value
    }
}

pub fn apply_bps(bp: BasisPoints, amount: Money) -> DomainResult<Money> {
    checked_div(
        checked_mul(amount, bp.value, "apply_bps multiplication")?,
        10_000,
        "apply_bps division",
    )
}

pub fn profit_amount(revenue: Money, total_costs: Money) -> Money {
    nat_sub(revenue, total_costs)
}
