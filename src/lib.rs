//! Runtime Rust mirror of the `CommerceTheory` Lean package.
//!
//! The Lean package stores proof fields in validated records. This crate mirrors
//! those records with private fields, smart constructors, executable predicates,
//! and tests that exercise the same safety guarantees at runtime.

#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

#[doc(hidden)]
pub trait FieldAccess {
    type Output<'a>
    where
        Self: 'a;

    fn access(&self) -> Self::Output<'_>;
}

#[doc(hidden)]
pub trait RefFieldAccess {}

impl<T: RefFieldAccess> FieldAccess for T {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn access(&self) -> Self::Output<'_> {
        self
    }
}

impl FieldAccess for String {
    type Output<'a> = &'a str;

    fn access(&self) -> Self::Output<'_> {
        self
    }
}

impl<T> FieldAccess for Vec<T> {
    type Output<'a>
        = &'a [T]
    where
        T: 'a;

    fn access(&self) -> Self::Output<'_> {
        self
    }
}

impl<T: Copy> FieldAccess for Option<T> {
    type Output<'a>
        = Self
    where
        T: 'a;

    fn access(&self) -> Self::Output<'_> {
        *self
    }
}

macro_rules! impl_copy_field_access {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::FieldAccess for $ty {
                type Output<'a> = Self;

                fn access(&self) -> Self::Output<'_> {
                    *self
                }
            }
        )*
    };
}

macro_rules! impl_ref_field_access {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::RefFieldAccess for $ty {}
        )*
    };
}

impl_copy_field_access!(
    bool,
    u128,
    i128,
    time::Date,
    time::Duration,
    time::PrimitiveDateTime,
);

macro_rules! field_getter {
    ($field:ident : $ty:ty) => {
        #[must_use]
        pub fn $field(&self) -> <$ty as $crate::FieldAccess>::Output<'_> {
            <$ty as $crate::FieldAccess>::access(&self.$field)
        }
    };
}

macro_rules! domain_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $vis struct $name {
            $(pub(crate) $field: $ty),*
        }

        impl $name {
            #[must_use]
            pub const fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            pub const fn try_new($($field: $ty),*) -> Result<Self, $crate::foundation::ValidationError> {
                Ok(Self::new($($field),*))
            }

            $(
                field_getter!($field: $ty);
            )*
        }

        impl $crate::RefFieldAccess for $name {}
    };
}

macro_rules! impl_getters {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        impl $name {
            $(
                field_getter!($field: $ty);
            )*
        }
    };
}

pub mod accounting;
pub mod b2b;
pub mod basic;
pub mod catalog;
pub mod competitor_pricing;
pub mod crm;
pub mod dropship_profit;
pub mod dropshipping;
pub mod event_language;
pub mod event_replay;
pub mod event_sourcing;
pub mod forecasting;
pub mod foundation;
pub mod fulfillment_finance;
pub mod implicit_invariants;
pub mod inventory;
pub mod inventory_algorithms;
pub mod keyed_totals;
pub mod logistics;
pub mod marketing;
pub mod marketplace;
pub mod merchandising;
pub mod opportunity_portfolio;
pub mod opportunity_ranking;
pub mod orders;
pub mod post_purchase;
pub mod pricing;
pub mod risk_privacy;
pub mod summary;
pub mod tax;
pub mod validation;
pub mod workflow;

pub use accounting::*;
pub use b2b::*;
pub use basic::*;
pub use catalog::*;
pub use competitor_pricing::*;
pub use crm::*;
pub use dropship_profit::*;
pub use dropshipping::*;
pub use event_language::*;
pub use event_replay::*;
pub use event_sourcing::*;
pub use forecasting::*;
pub use foundation::*;
pub use fulfillment_finance::*;
pub use implicit_invariants::*;
pub use inventory::*;
pub use inventory_algorithms::*;
pub use keyed_totals::*;
pub use logistics::*;
pub use marketing::*;
pub use marketplace::*;
pub use merchandising::*;
pub use opportunity_portfolio::*;
pub use opportunity_ranking::*;
pub use orders::*;
pub use post_purchase::*;
pub use pricing::*;
pub use risk_privacy::*;
pub use tax::*;
pub use validation::*;
pub use workflow::*;

impl_copy_field_access!(
    AccountTier,
    Action,
    AccessPurpose,
    AdDestination,
    AdPlatform,
    AdType,
    CampaignStatus,
    CanOrderTransition,
    CompetitivePricingStrategy,
    Confidence,
    ConsentPurpose,
    ConsentStatus,
    ContactKind,
    Currency,
    CustomerKind,
    CRMAccountStatus,
    DataCategory,
    DropshipPOStatus,
    DropshipPOTransitionLabel,
    ErasureStatus,
    InteractionKind,
    LeadStatus,
    ListingStatus,
    LogisticsExceptionKind,
    Marketplace,
    OpportunityStage,
    OrderEventSymbol,
    OrderEventValidationState,
    OrderStatus,
    OrderTransitionLabel,
    PaymentState,
    PaymentTerms,
    PostingSide,
    ProcessingBasis,
    ProductStatus,
    PromotionStackingPolicy,
    ReservationStatus,
    ReturnAuthorizationStatus,
    Role,
    RoundingMode,
    SalesChannel,
    SerialNumber,
    ShipmentStatus,
    SubscriptionLifecycleStatus,
    SubscriptionStatus,
    SupplierReservationStatus,
    SupportCaseStatus,
    SupportPriority,
    TaxPriceMode,
    TaxRegime,
    TaxTreatment,
    TrackingEventKind,
    TradeMode,
    TrustLevel,
    ValidationError,
    Lead,
    SalesOpportunity,
    StockState,
    SupportCase,
    TimedReservation,
    VersionedStock,
);

impl<T> RefFieldAccess for Timed<T> {}

impl<S: OrderStatusMarker> FieldAccess for TypedOrder<S> {
    type Output<'a>
        = Self
    where
        S: 'a;

    fn access(&self) -> Self::Output<'_> {
        *self
    }
}

impl<S: PaymentStateMarker> FieldAccess for TypedPayment<S> {
    type Output<'a>
        = Self
    where
        S: 'a;

    fn access(&self) -> Self::Output<'_> {
        *self
    }
}

impl_ref_field_access!(
    AcceptedPromotionSet,
    ActiveCRMAccount,
    ApprovedOrderableSupplierQuality,
    ApprovedSupplierQuality,
    AuditedCommand,
    AuditedDataAccess,
    AuditedEntityCommand,
    BackorderRequest,
    B2BTaxExemption,
    BalancedJournalEntry,
    BrandPricingPolicy,
    BoundedCouponApplication,
    BundleComponent,
    BundleReservation,
    CapturedPaymentJournalProjection,
    CapturedPaymentMatchesOrder,
    CarrierHandoff,
    CarrierQuote,
    CartLine,
    CashflowPlan,
    ChannelPricePolicy,
    Chargeback,
    ChargebackForCapturedPayment,
    ClickAttributedCampaign,
    CompetitorAwareDropshipOffer,
    CompetitorPriceBenchmark,
    ConcurrentReservationConflict,
    ConvertedLeadOpportunity,
    CRMAccount,
    CRMAccountContact,
    CRMApprovedReturnHandling,
    CRMInteraction,
    CRMInteractionForContact,
    CRMOrderContact,
    CustomerSegment,
    DeliveredShipment,
    DeliveryPromise,
    DistinctFulfillmentPlan,
    DomainEvent,
    DropshipCostUpperBounds,
    DropshipFulfillment,
    DropshipLine,
    DropshipOpportunityCandidate,
    DropshipOpportunityPortfolio,
    DropshipOffer,
    DropshipPurchaseOrder,
    DropshipReturnRequest,
    EventBackedCashflowPlan,
    Experiment,
    ExperimentVariant,
    ExchangeRate,
    FreshCurrencyConversion,
    FraudCheckedCouponApplication,
    Funnel,
    GiftCardRedemption,
    GuaranteedDropshipProfitQuote,
    LeadForContact,
    LogisticsExceptionSupportCase,
    LogisticsShipment,
    LogisticsShipmentPlan,
    MapCompliantCompetitorAwareOffer,
    MarketplaceFacilitatorTax,
    MarketplaceFeeLedger,
    MarketplaceOrder,
    MarketplacePayoutCalculation,
    MarketingCampaign,
    MatchedOrderAttributionLedger,
    Order,
    OrderAttributionLedger,
    OrderTaxInvoiceLink,
    OpportunityForContact,
    PaymentLedger,
    PermittedAccountMessage,
    PermittedCustomerMessage,
    PickTask,
    PreorderReservation,
    PreorderWindow,
    ProductCatalogEntry,
    PublishableFeedLine,
    ReconciliationWithinTolerance,
    RecurringSubscription,
    RefundJournalProjection,
    ReservedDropshipLine,
    ReservationAttempt,
    ResolvedSupportCase,
    RetainedPersonalData,
    RetentionOffer,
    RetailLine,
    ReturnAuthorization,
    ReturnReceipt,
    SalesPipeline,
    SafeProductFeedLine,
    SellableCatalogEntry,
    SegmentMembership,
    SerializedInventorySet,
    ShipmentForCRMOrder,
    SkuSubstitution,
    SourceableDistributorProduct,
    SplitFulfillmentPlan,
    SubscriptionPlan,
    SupplierDailyCapacity,
    SupplierReservation,
    SupportCaseForContact,
    SyncedMarketplaceListing,
    TaxCalculation,
    TaxExclusivePrice,
    TaxExemptionCertificate,
    TaxInclusivePrice,
    TaxInvoice,
    TaxInvoiceLine,
    TrackingHistory,
    TradePriceBookEntry,
    TrustedFreshCompetitorBenchmark,
    ValidEventStream,
    ValidGiftCardRedemptionAt,
    ValidListingContent,
    ValidRefund,
    ValidSearchResultItem,
    WarehouseShipment,
    WarehouseTransfer,
    WholesaleCreditAccount,
    WholesaleCreditCheckout,
    WholesaleLine,
);
