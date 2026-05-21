use crate::foundation::*;
use crate::inventory::*;
use crate::orders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Marketplace {
    AmazonLike,
    RozetkaLike,
    EtsyLike,
    EbayLike,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SalesChannel {
    OwnWebsite,
    MarketplaceChannel(Marketplace),
    B2BPortal,
    DropshipFeed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ListingStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

domain_struct! {
    pub struct MarketplaceListing {
        sku: Sku,
        marketplace: Marketplace,
        external_id: Nat,
        price: Money,
        currency: Currency,
        published_stock: Quantity,
        status: ListingStatus,
    }
}

pub fn listing_active(listing: &MarketplaceListing) -> bool {
    listing.status == ListingStatus::Active
}

pub fn listing_in_stock(listing: &MarketplaceListing) -> bool {
    listing.published_stock > 0
}

pub fn listing_can_be_advertised(listing: &MarketplaceListing) -> bool {
    listing_active(listing) && listing_in_stock(listing)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncedMarketplaceListing {
    pub(crate) listing: MarketplaceListing,
    pub(crate) stock: StockState,
}

impl SyncedMarketplaceListing {
    pub fn try_new(listing: MarketplaceListing, stock: StockState) -> DomainResult<Self> {
        if listing.sku != stock.sku() {
            return Err(ValidationError::Invariant(
                "listing SKU must match stock SKU",
            ));
        }
        if listing.published_stock > available_stock(&stock) {
            return Err(ValidationError::Invariant(
                "published stock exceeds available stock",
            ));
        }
        Ok(Self { listing, stock })
    }

    pub fn listing(&self) -> &MarketplaceListing {
        &self.listing
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelPricePolicy {
    pub(crate) min_price: Money,
    pub(crate) max_price: Money,
}

impl ChannelPricePolicy {
    pub fn try_new(min_price: Money, max_price: Money) -> DomainResult<Self> {
        if min_price > max_price {
            return Err(ValidationError::Invariant("minimum price exceeds maximum"));
        }
        Ok(Self {
            min_price,
            max_price,
        })
    }
}

pub fn valid_channel_price(policy: &ChannelPricePolicy, price: Money) -> bool {
    policy.min_price <= price && price <= policy.max_price
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SafeProductFeedLine {
    pub(crate) sku: Sku,
    pub(crate) channel: SalesChannel,
    pub(crate) price: Money,
    pub(crate) currency: Currency,
    pub(crate) stock: Quantity,
    pub(crate) stock_state: StockState,
    pub(crate) price_policy: ChannelPricePolicy,
}

impl SafeProductFeedLine {
    pub fn try_new(
        sku: Sku,
        channel: SalesChannel,
        price: Money,
        currency: Currency,
        stock: Quantity,
        stock_state: StockState,
        price_policy: ChannelPricePolicy,
    ) -> DomainResult<Self> {
        if sku != stock_state.sku() {
            return Err(ValidationError::Invariant("feed SKU must match stock SKU"));
        }
        if !valid_channel_price(&price_policy, price) {
            return Err(ValidationError::Invariant("feed price outside policy"));
        }
        if stock > available_stock(&stock_state) {
            return Err(ValidationError::Invariant(
                "feed stock exceeds availability",
            ));
        }
        Ok(Self {
            sku,
            channel,
            price,
            currency,
            stock,
            stock_state,
            price_policy,
        })
    }
}

pub fn marketplace_fee_rounded(
    mode: RoundingMode,
    gross: Money,
    fee_rate: BasisPoints,
) -> DomainResult<Money> {
    round_bps_amount(mode, gross, fee_rate)
}

pub fn marketplace_payout_rounded(
    mode: RoundingMode,
    gross: Money,
    payout_rate: BasisPoints,
) -> DomainResult<Money> {
    round_bps_amount(mode, gross, payout_rate)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketplaceFeeLedger {
    pub(crate) gross: Money,
    pub(crate) fee_rate: BasisPoints,
    pub(crate) fee_rounding_mode: RoundingMode,
    pub(crate) fee: Money,
    pub(crate) payout: Money,
}

impl MarketplaceFeeLedger {
    pub fn try_new(
        gross: Money,
        fee_rate: BasisPoints,
        fee_rounding_mode: RoundingMode,
        fee: Money,
        payout: Money,
    ) -> DomainResult<Self> {
        if fee != marketplace_fee_rounded(fee_rounding_mode, gross, fee_rate)? {
            return Err(ValidationError::Invariant("marketplace fee is incorrect"));
        }
        if fee > gross {
            return Err(ValidationError::Invariant("marketplace fee exceeds gross"));
        }
        if payout != nat_sub(gross, fee) {
            return Err(ValidationError::Invariant(
                "marketplace payout is incorrect",
            ));
        }
        Ok(Self {
            gross,
            fee_rate,
            fee_rounding_mode,
            fee,
            payout,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketplacePayoutCalculation {
    pub(crate) gross: Money,
    pub(crate) payout_rate: BasisPoints,
    pub(crate) payout_rounding_mode: RoundingMode,
    pub(crate) payout: Money,
}

impl MarketplacePayoutCalculation {
    pub fn try_new(
        gross: Money,
        payout_rate: BasisPoints,
        payout_rounding_mode: RoundingMode,
        payout: Money,
    ) -> DomainResult<Self> {
        if payout != marketplace_payout_rounded(payout_rounding_mode, gross, payout_rate)? {
            return Err(ValidationError::Invariant(
                "marketplace payout is incorrect",
            ));
        }
        Ok(Self {
            gross,
            payout_rate,
            payout_rounding_mode,
            payout,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketplaceOrder {
    pub(crate) marketplace: Marketplace,
    pub(crate) external_order_id: MarketplaceOrderId,
    pub(crate) internal_order: Order,
    pub(crate) gross_from_marketplace: Money,
    pub(crate) fee_ledger: MarketplaceFeeLedger,
}

impl MarketplaceOrder {
    pub fn try_new(
        marketplace: Marketplace,
        external_order_id: MarketplaceOrderId,
        internal_order: Order,
        gross_from_marketplace: Money,
        fee_ledger: MarketplaceFeeLedger,
    ) -> DomainResult<Self> {
        if gross_from_marketplace != internal_order.total() {
            return Err(ValidationError::Invariant(
                "marketplace gross must match internal order total",
            ));
        }
        if fee_ledger.gross != gross_from_marketplace {
            return Err(ValidationError::Invariant(
                "fee ledger gross must match marketplace gross",
            ));
        }
        Ok(Self {
            marketplace,
            external_order_id,
            internal_order,
            gross_from_marketplace,
            fee_ledger,
        })
    }
}

impl_getters!(ChannelPricePolicy {
    min_price: Money,
    max_price: Money,
});

impl_getters!(SyncedMarketplaceListing { stock: StockState });

impl_getters!(SafeProductFeedLine {
    sku: Sku,
    channel: SalesChannel,
    price: Money,
    currency: Currency,
    stock: Quantity,
    stock_state: StockState,
    price_policy: ChannelPricePolicy,
});

impl_getters!(MarketplaceFeeLedger {
    gross: Money,
    fee_rate: BasisPoints,
    fee_rounding_mode: RoundingMode,
    fee: Money,
    payout: Money,
});

impl_getters!(MarketplacePayoutCalculation {
    gross: Money,
    payout_rate: BasisPoints,
    payout_rounding_mode: RoundingMode,
    payout: Money,
});

impl_getters!(MarketplaceOrder {
    marketplace: Marketplace,
    external_order_id: MarketplaceOrderId,
    internal_order: Order,
    gross_from_marketplace: Money,
    fee_ledger: MarketplaceFeeLedger,
});
