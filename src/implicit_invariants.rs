use crate::accounting::*;
use crate::b2b::*;
use crate::catalog::*;
use crate::competitor_pricing::*;
use crate::dropshipping::*;
use crate::event_sourcing::*;
use crate::forecasting::*;
use crate::foundation::*;
use crate::fulfillment_finance::*;
use crate::marketplace::*;
use crate::merchandising::*;
use crate::opportunity_portfolio::*;
use crate::orders::*;
use crate::post_purchase::*;
use crate::pricing::*;
use crate::risk_privacy::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundedCouponApplication {
    pub(crate) coupon: Coupon,
    pub(crate) subtotal: Money,
    pub(crate) uses_before: Nat,
}

impl BoundedCouponApplication {
    pub fn try_new(coupon: Coupon, subtotal: Money, uses_before: Nat) -> DomainResult<Self> {
        if !coupon_can_be_applied(&coupon, subtotal, uses_before) {
            return Err(ValidationError::Invariant("coupon cannot be applied"));
        }
        if coupon.amount > subtotal {
            return Err(ValidationError::Invariant("coupon amount exceeds subtotal"));
        }
        Ok(Self {
            coupon,
            subtotal,
            uses_before,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CapturedPaymentMatchesOrder {
    pub(crate) order: Order,
    pub(crate) payment: CapturedPayment,
}

impl CapturedPaymentMatchesOrder {
    pub fn try_new(order: Order, payment: CapturedPayment) -> DomainResult<Self> {
        if payment.order_id != order.id()
            || payment.amount != order.total()
            || payment.currency != order.currency()
        {
            return Err(ValidationError::Invariant(
                "captured payment does not match order",
            ));
        }
        Ok(Self { order, payment })
    }
}

pub fn event_stream_last_sequence_from(last: Nat, events: &[EventEnvelope]) -> Nat {
    events.last().map_or(last, |event| event.sequence)
}

pub fn event_stream_computed_last_sequence(stream: &EventStream) -> Nat {
    event_stream_last_sequence_from(0, &stream.events)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidEventStream {
    pub(crate) stream: EventStream,
}

impl ValidEventStream {
    pub fn try_new(stream: EventStream) -> DomainResult<Self> {
        if !stream_sequences_strictly_increase(&stream) {
            return Err(ValidationError::Invariant(
                "event stream sequences must strictly increase",
            ));
        }
        if stream.last_sequence != event_stream_computed_last_sequence(&stream) {
            return Err(ValidationError::Invariant(
                "event stream cursor does not match events",
            ));
        }
        Ok(Self { stream })
    }
}

pub fn product_active(product: &Product) -> bool {
    product.status == ProductStatus::Active
}

pub fn variant_active(variant: &ProductVariant) -> bool {
    variant.active
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SellableCatalogEntry {
    pub(crate) entry: ProductCatalogEntry,
}

impl SellableCatalogEntry {
    pub fn try_new(entry: ProductCatalogEntry) -> DomainResult<Self> {
        if !product_active(&entry.product) || !variant_active(&entry.variant) {
            return Err(ValidationError::Invariant("catalog entry is not sellable"));
        }
        Ok(Self { entry })
    }
}

pub fn feed_line_has_stock(line: &SafeProductFeedLine) -> bool {
    line.stock > 0
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublishableFeedLine {
    pub(crate) line: SafeProductFeedLine,
}

impl PublishableFeedLine {
    pub fn try_new(line: SafeProductFeedLine) -> DomainResult<Self> {
        if !feed_line_has_stock(&line) {
            return Err(ValidationError::Invariant("feed line has no stock"));
        }
        Ok(Self { line })
    }
}

pub fn distributor_product_active(product: &DistributorProduct) -> bool {
    product.active
}

pub fn distributor_product_can_source(product: &DistributorProduct, units: Quantity) -> bool {
    distributor_product_active(product)
        && product.min_order_qty <= units
        && units <= product.available_qty
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceableDistributorProduct {
    pub(crate) product: DistributorProduct,
    pub(crate) units: Quantity,
}

impl SourceableDistributorProduct {
    pub fn try_new(product: DistributorProduct, units: Quantity) -> DomainResult<Self> {
        if !distributor_product_can_source(&product, units) {
            return Err(ValidationError::Invariant(
                "distributor product cannot source requested units",
            ));
        }
        Ok(Self { product, units })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FraudCheckedCouponApplication {
    pub(crate) application: BoundedCouponApplication,
    pub(crate) policy: FraudPolicy,
}

impl FraudCheckedCouponApplication {
    pub fn try_new(
        application: BoundedCouponApplication,
        policy: FraudPolicy,
    ) -> DomainResult<Self> {
        if !coupon_uses_allowed(&policy, application.uses_before) {
            return Err(ValidationError::Invariant("coupon use fails fraud policy"));
        }
        Ok(Self {
            application,
            policy,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CapturedPaymentJournalProjection {
    pub(crate) accounts: AccountingAccounts,
    pub(crate) payment: CapturedPayment,
    pub(crate) journal: BalancedJournalEntry,
}

impl CapturedPaymentJournalProjection {
    pub fn try_new(
        accounts: AccountingAccounts,
        payment: CapturedPayment,
        journal: BalancedJournalEntry,
    ) -> DomainResult<Self> {
        if journal != payment_captured_journal(&accounts, payment.amount)? {
            return Err(ValidationError::Invariant(
                "payment-capture journal projection is incorrect",
            ));
        }
        Ok(Self {
            accounts,
            payment,
            journal,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefundJournalProjection {
    pub(crate) accounts: AccountingAccounts,
    pub(crate) ledger: PaymentLedger,
    pub(crate) amount: Money,
    pub(crate) journal: BalancedJournalEntry,
}

impl RefundJournalProjection {
    pub fn try_new(
        accounts: AccountingAccounts,
        ledger: PaymentLedger,
        amount: Money,
        journal: BalancedJournalEntry,
    ) -> DomainResult<Self> {
        if !can_refund(&ledger, amount) {
            return Err(ValidationError::Invariant(
                "refund amount is not refundable",
            ));
        }
        if journal != refund_issued_journal(&accounts, amount)? {
            return Err(ValidationError::Invariant(
                "refund journal projection is incorrect",
            ));
        }
        Ok(Self {
            accounts,
            ledger,
            amount,
            journal,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AdvertisableSyncedMarketplaceListing {
    pub(crate) synced: SyncedMarketplaceListing,
}

impl AdvertisableSyncedMarketplaceListing {
    pub fn try_new(synced: SyncedMarketplaceListing) -> DomainResult<Self> {
        if !listing_can_be_advertised(&synced.listing) {
            return Err(ValidationError::Invariant(
                "synced listing cannot be advertised",
            ));
        }
        Ok(Self { synced })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WholesaleCreditCheckout {
    pub(crate) account: WholesaleCreditAccount,
    pub(crate) lines: Vec<WholesaleLine>,
    pub(crate) terms: PaymentTerms,
    pub(crate) order_total: Money,
}

impl WholesaleCreditCheckout {
    pub fn try_new(
        account: WholesaleCreditAccount,
        lines: Vec<WholesaleLine>,
        terms: PaymentTerms,
        order_total: Money,
    ) -> DomainResult<Self> {
        if order_total != wholesale_order_net_total(&lines)? {
            return Err(ValidationError::Invariant(
                "wholesale checkout total is incorrect",
            ));
        }
        if !payment_terms_allowed(TradeMode::Wholesale, terms) {
            return Err(ValidationError::Invariant(
                "payment terms not allowed for wholesale",
            ));
        }
        if !can_place_wholesale_credit_order(&account, order_total) {
            return Err(ValidationError::Invariant(
                "wholesale checkout exceeds credit limit",
            ));
        }
        Ok(Self {
            account,
            lines,
            terms,
            order_total,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrustedFreshCompetitorBenchmark {
    pub(crate) benchmark: CompetitorPriceBenchmark,
    pub(crate) now: Timestamp,
    pub(crate) max_age: Timestamp,
    pub(crate) trust: TrustLevel,
}

impl TrustedFreshCompetitorBenchmark {
    pub fn try_new(
        benchmark: CompetitorPriceBenchmark,
        now: Timestamp,
        max_age: Timestamp,
        trust: TrustLevel,
    ) -> DomainResult<Self> {
        if !price_snapshot_fresh(now, max_age, benchmark.best_offer.observed_at) {
            return Err(ValidationError::Invariant("benchmark best offer is stale"));
        }
        if !trust_allows_auto_repricing(trust) {
            return Err(ValidationError::Invariant(
                "trust level does not allow auto repricing",
            ));
        }
        Ok(Self {
            benchmark,
            now,
            max_age,
            trust,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapCompliantCompetitorAwareOffer {
    pub(crate) offer: CompetitorAwareDropshipOffer,
    pub(crate) policy: BrandPricingPolicy,
}

impl MapCompliantCompetitorAwareOffer {
    pub fn try_new(
        offer: CompetitorAwareDropshipOffer,
        policy: BrandPricingPolicy,
    ) -> DomainResult<Self> {
        if !advertised_price_allowed(&policy, offer.offer.sale_unit_price()) {
            return Err(ValidationError::Invariant("offer violates MAP policy"));
        }
        Ok(Self { offer, policy })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FreshCurrencyConversion {
    pub(crate) source_amount: MoneyAmount,
    pub(crate) rate: ExchangeRate,
    pub(crate) target_amount: MoneyAmount,
    pub(crate) now: Timestamp,
    pub(crate) max_age: Timestamp,
}

impl FreshCurrencyConversion {
    pub fn try_new(
        source_amount: MoneyAmount,
        rate: ExchangeRate,
        target_amount: MoneyAmount,
        now: Timestamp,
        max_age: Timestamp,
    ) -> DomainResult<Self> {
        if source_amount.currency != rate.source || target_amount.currency != rate.target {
            return Err(ValidationError::Invariant(
                "currency conversion currencies do not match rate",
            ));
        }
        if target_amount.amount != convert_money_floor(source_amount.amount, &rate)? {
            return Err(ValidationError::Invariant(
                "currency conversion amount is incorrect",
            ));
        }
        if !fx_quote_fresh(now, max_age, &rate) {
            return Err(ValidationError::Invariant("exchange rate is stale"));
        }
        Ok(Self {
            source_amount,
            rate,
            target_amount,
            now,
            max_age,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidGiftCardRedemptionAt {
    pub(crate) now: Timestamp,
    pub(crate) redemption: GiftCardRedemption,
}

impl ValidGiftCardRedemptionAt {
    pub fn try_new(now: Timestamp, redemption: GiftCardRedemption) -> DomainResult<Self> {
        if !gift_card_valid_at(now, &redemption.card) {
            return Err(ValidationError::Invariant("gift card has expired"));
        }
        Ok(Self { now, redemption })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChargebackForCapturedPayment {
    pub(crate) payment: CapturedPayment,
    pub(crate) chargeback: Chargeback,
}

impl ChargebackForCapturedPayment {
    pub fn try_new(payment: CapturedPayment, chargeback: Chargeback) -> DomainResult<Self> {
        if chargeback.payment_amount != payment.amount {
            return Err(ValidationError::Invariant(
                "chargeback payment amount mismatch",
            ));
        }
        Ok(Self {
            payment,
            chargeback,
        })
    }
}

pub fn demand_forecast_actionable(forecast: &DemandForecast) -> bool {
    confidence_allows_auto_replenish(forecast.confidence)
        && forecast.expected_units > 0
        && forecast.horizon_days > 0
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionableDemandForecast {
    pub(crate) forecast: DemandForecast,
}

impl ActionableDemandForecast {
    pub fn try_new(forecast: DemandForecast) -> DomainResult<Self> {
        if !demand_forecast_actionable(&forecast) {
            return Err(ValidationError::Invariant(
                "demand forecast is not actionable",
            ));
        }
        Ok(Self { forecast })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApprovedOrderableSupplierQuality {
    pub(crate) quality: ApprovedSupplierQuality,
}

impl ApprovedOrderableSupplierQuality {
    pub fn try_new(quality: ApprovedSupplierQuality) -> DomainResult<Self> {
        if !supplier_can_receive_orders(&quality.supplier) {
            return Err(ValidationError::Invariant(
                "approved supplier cannot receive orders",
            ));
        }
        Ok(Self { quality })
    }
}

impl_getters!(BoundedCouponApplication {
    coupon: Coupon,
    subtotal: Money,
    uses_before: Nat,
});

impl_getters!(CapturedPaymentMatchesOrder {
    order: Order,
    payment: CapturedPayment,
});

impl_getters!(ValidEventStream {
    stream: EventStream,
});

impl_getters!(SellableCatalogEntry {
    entry: ProductCatalogEntry,
});

impl_getters!(PublishableFeedLine {
    line: SafeProductFeedLine,
});

impl_getters!(SourceableDistributorProduct {
    product: DistributorProduct,
    units: Quantity,
});

impl_getters!(FraudCheckedCouponApplication {
    application: BoundedCouponApplication,
    policy: FraudPolicy,
});

impl_getters!(CapturedPaymentJournalProjection {
    accounts: AccountingAccounts,
    payment: CapturedPayment,
    journal: BalancedJournalEntry,
});

impl_getters!(RefundJournalProjection {
    accounts: AccountingAccounts,
    ledger: PaymentLedger,
    amount: Money,
    journal: BalancedJournalEntry,
});

impl_getters!(AdvertisableSyncedMarketplaceListing {
    synced: SyncedMarketplaceListing,
});

impl_getters!(WholesaleCreditCheckout {
    account: WholesaleCreditAccount,
    lines: Vec<WholesaleLine>,
    terms: PaymentTerms,
    order_total: Money,
});

impl_getters!(TrustedFreshCompetitorBenchmark {
    benchmark: CompetitorPriceBenchmark,
    now: Timestamp,
    max_age: Timestamp,
    trust: TrustLevel,
});

impl_getters!(MapCompliantCompetitorAwareOffer {
    offer: CompetitorAwareDropshipOffer,
    policy: BrandPricingPolicy,
});

impl_getters!(FreshCurrencyConversion {
    source_amount: MoneyAmount,
    rate: ExchangeRate,
    target_amount: MoneyAmount,
    now: Timestamp,
    max_age: Timestamp,
});

impl_getters!(ValidGiftCardRedemptionAt {
    now: Timestamp,
    redemption: GiftCardRedemption,
});

impl_getters!(ChargebackForCapturedPayment {
    payment: CapturedPayment,
    chargeback: Chargeback,
});

impl_getters!(ActionableDemandForecast {
    forecast: DemandForecast,
});

impl_getters!(ApprovedOrderableSupplierQuality {
    quality: ApprovedSupplierQuality,
});
