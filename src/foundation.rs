use core::fmt;
use core::marker::PhantomData;
use time::{Date as TimeDate, Duration as TimeDuration, Month, PrimitiveDateTime, Time};

pub type Nat = u128;
pub type MinorUnit = Nat;
pub type NonNegMoney = MinorUnit;
pub type Money = NonNegMoney;
pub type SignedMoney = i128;
pub type Quantity = u128;
pub type Weight = u128;
pub type Date = TimeDate;
pub type Timestamp = PrimitiveDateTime;
pub type Duration = TimeDuration;
pub type Days = Duration;
pub type Id = u128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecimalMoney {
    pub(crate) coefficient: SignedMoney,
    pub(crate) scale: Nat,
}

impl DecimalMoney {
    pub const fn new(coefficient: SignedMoney, scale: Nat) -> Self {
        Self { coefficient, scale }
    }

    pub const fn coefficient(&self) -> SignedMoney {
        self.coefficient
    }

    pub const fn scale(&self) -> Nat {
        self.scale
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidationError {
    Invariant(&'static str),
    Overflow(&'static str),
    DivisionByZero(&'static str),
    LineDiscountExceedsGross,
    CouponExceedsSubtotal,
    ShippingUnavailable,
    OrderTotalMismatch,
    StockReservedExceedsTotal,
    PricePolicyInvalid,
    FeedSkuMismatch,
    FeedPriceOutOfPolicy,
    FeedStockUnavailable,
    LedgerRefundedExceedsCaptured,
    RefundExceedsRemaining,
    BasisPointsOutOfRange,
    CatalogInvariantFailed,
    InventoryInvariantFailed,
    AccountingInvariantFailed,
    MarketplaceInvariantFailed,
    MarketingInvariantFailed,
    B2BInvariantFailed,
    DropshippingInvariantFailed,
    ProfitInvariantFailed,
    CompetitorInvariantFailed,
    MerchandisingInvariantFailed,
    FinanceInvariantFailed,
    AuditPermissionDenied,
    EventStreamInvalid,
    PostPurchaseInvariantFailed,
    SupplierQualityInvalid,
    OpportunityInvariantFailed,
    CrmInvariantFailed,
    LogisticsInvariantFailed,
    ImplicitInvariantFailed,
    TaxInvariantFailed,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invariant(message) => write!(f, "invariant failed: {message}"),
            Self::Overflow(message) => write!(f, "arithmetic overflow: {message}"),
            Self::DivisionByZero(message) => write!(f, "division by zero: {message}"),
            Self::LineDiscountExceedsGross => write!(f, "line discount exceeds gross"),
            Self::CouponExceedsSubtotal => write!(f, "coupon exceeds cart net subtotal"),
            Self::ShippingUnavailable => write!(f, "shipping unavailable"),
            Self::OrderTotalMismatch => write!(f, "order total mismatch"),
            Self::StockReservedExceedsTotal => write!(f, "reserved stock exceeds total"),
            Self::PricePolicyInvalid => write!(f, "price policy invalid"),
            Self::FeedSkuMismatch => write!(f, "feed SKU mismatch"),
            Self::FeedPriceOutOfPolicy => write!(f, "feed price out of policy"),
            Self::FeedStockUnavailable => write!(f, "feed stock unavailable"),
            Self::LedgerRefundedExceedsCaptured => write!(f, "ledger refunded exceeds captured"),
            Self::RefundExceedsRemaining => write!(f, "refund exceeds remaining amount"),
            Self::BasisPointsOutOfRange => write!(f, "basis points out of range"),
            Self::CatalogInvariantFailed => write!(f, "catalog invariant failed"),
            Self::InventoryInvariantFailed => write!(f, "inventory invariant failed"),
            Self::AccountingInvariantFailed => write!(f, "accounting invariant failed"),
            Self::MarketplaceInvariantFailed => write!(f, "marketplace invariant failed"),
            Self::MarketingInvariantFailed => write!(f, "marketing invariant failed"),
            Self::B2BInvariantFailed => write!(f, "B2B invariant failed"),
            Self::DropshippingInvariantFailed => write!(f, "dropshipping invariant failed"),
            Self::ProfitInvariantFailed => write!(f, "profit invariant failed"),
            Self::CompetitorInvariantFailed => write!(f, "competitor invariant failed"),
            Self::MerchandisingInvariantFailed => write!(f, "merchandising invariant failed"),
            Self::FinanceInvariantFailed => write!(f, "finance invariant failed"),
            Self::AuditPermissionDenied => write!(f, "audit permission denied"),
            Self::EventStreamInvalid => write!(f, "event stream invalid"),
            Self::PostPurchaseInvariantFailed => write!(f, "post-purchase invariant failed"),
            Self::SupplierQualityInvalid => write!(f, "supplier quality invalid"),
            Self::OpportunityInvariantFailed => write!(f, "opportunity invariant failed"),
            Self::CrmInvariantFailed => write!(f, "CRM invariant failed"),
            Self::LogisticsInvariantFailed => write!(f, "logistics invariant failed"),
            Self::ImplicitInvariantFailed => write!(f, "implicit invariant failed"),
            Self::TaxInvariantFailed => write!(f, "tax invariant failed"),
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
    a.checked_div(b)
        .ok_or(ValidationError::DivisionByZero(context))
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

pub fn timestamp_from_ymdhms(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Option<Timestamp> {
    let month = Month::try_from(month).ok()?;
    let date = Date::from_calendar_date(year, month, day).ok()?;
    let time = Time::from_hms(hour, minute, second).ok()?;
    Some(PrimitiveDateTime::new(date, time))
}

pub fn unix_epoch_timestamp() -> Timestamp {
    timestamp_from_ymdhms(1970, 1, 1, 0, 0, 0).expect("valid unix epoch timestamp")
}

pub fn timestamp_age(now: Timestamp, observed_at: Timestamp) -> Duration {
    now - observed_at
}

pub fn days(n: Nat) -> Days {
    Duration::days(i64::try_from(n).unwrap_or(i64::MAX))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoundingMode {
    Floor,
    Ceiling,
    HalfUp,
}

pub fn round_div(mode: RoundingMode, numerator: Nat, denominator: Nat) -> DomainResult<Nat> {
    if denominator == 0 {
        return Err(ValidationError::DivisionByZero("round_div denominator"));
    }
    match mode {
        RoundingMode::Floor => Ok(numerator / denominator),
        RoundingMode::Ceiling => {
            let quotient = numerator / denominator;
            if numerator.is_multiple_of(denominator) {
                Ok(quotient)
            } else {
                checked_add(quotient, 1, "round_div ceiling")
            }
        }
        RoundingMode::HalfUp => {
            let half = denominator / 2;
            checked_div(
                checked_add(numerator, half, "round_div half-up")?,
                denominator,
                "round_div half-up",
            )
        }
    }
}

pub fn round_money(mode: RoundingMode, numerator: Nat, denominator: Nat) -> DomainResult<Money> {
    round_div(mode, numerator, denominator)
}

pub fn floor_rounding_remainder(numerator: Nat, denominator: Nat) -> DomainResult<Nat> {
    if denominator == 0 {
        Err(ValidationError::DivisionByZero(
            "floor_rounding_remainder denominator",
        ))
    } else {
        Ok(numerator % denominator)
    }
}

pub fn floor_rounded_lines_remainder_total(
    denominator: Nat,
    numerators: &[Nat],
) -> DomainResult<Nat> {
    checked_sum(
        numerators
            .iter()
            .map(|numerator| floor_rounding_remainder(*numerator, denominator))
            .collect::<DomainResult<Vec<_>>>()?,
        "floor_rounded_lines_remainder_total",
    )
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
id_type!(AccountId);
id_type!(ContactId);
id_type!(LeadId);
id_type!(OpportunityId);
id_type!(InteractionId);
id_type!(SegmentId);
id_type!(SupportCaseId);
id_type!(ShipmentId);
id_type!(TrackingEventId);
id_type!(TransferId);
id_type!(ReturnAuthorizationId);

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

pub fn round_bps_amount(mode: RoundingMode, amount: Money, bp: BasisPoints) -> DomainResult<Money> {
    round_money(
        mode,
        checked_mul(amount, bp.value, "round_bps_amount multiplication")?,
        10_000,
    )
}

pub fn profit_amount(revenue: Money, total_costs: Money) -> Money {
    nat_sub(revenue, total_costs)
}

pub fn profit_loss_amount(revenue: Money, total_costs: Money) -> DomainResult<SignedMoney> {
    let revenue =
        SignedMoney::try_from(revenue).map_err(|_| ValidationError::Overflow("profit revenue"))?;
    let total_costs = SignedMoney::try_from(total_costs)
        .map_err(|_| ValidationError::Overflow("profit costs"))?;
    revenue
        .checked_sub(total_costs)
        .ok_or(ValidationError::Overflow("profit subtraction"))
}
