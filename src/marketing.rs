use crate::foundation::*;
use crate::marketplace::*;
use crate::orders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AdPlatform {
    GoogleLike,
    MetaLike,
    TikTokLike,
    MarketplaceAds,
    EmailProvider,
    SmsProvider,
    AffiliateNetwork,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AdType {
    Search,
    Shopping,
    Display,
    Social,
    Video,
    Retargeting,
    EmailMarketing,
    SmsMarketing,
    MarketplaceSponsoredProducts,
    Affiliate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AdDestination {
    Website,
    MarketplaceStore(Marketplace),
    MarketplaceListing(Marketplace, Nat),
}

pub fn destination_matches_marketplace(
    destination: AdDestination,
    marketplace: Marketplace,
) -> bool {
    match destination {
        AdDestination::Website => false,
        AdDestination::MarketplaceStore(m) | AdDestination::MarketplaceListing(m, _) => {
            m == marketplace
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketingCampaign {
    pub(crate) id: CampaignId,
    pub(crate) platform: AdPlatform,
    pub(crate) ad_type: AdType,
    pub(crate) destination: AdDestination,
    pub(crate) status: CampaignStatus,
    pub(crate) budget: Money,
    pub(crate) spend: Money,
    pub(crate) impressions: Nat,
    pub(crate) clicks: Nat,
    pub(crate) conversions: Nat,
    pub(crate) attributed_revenue: Money,
}

impl MarketingCampaign {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: CampaignId,
        platform: AdPlatform,
        ad_type: AdType,
        destination: AdDestination,
        status: CampaignStatus,
        budget: Money,
        spend: Money,
        impressions: Nat,
        clicks: Nat,
        conversions: Nat,
        attributed_revenue: Money,
    ) -> DomainResult<Self> {
        if spend > budget {
            return Err(ValidationError::Invariant("campaign spend exceeds budget"));
        }
        if clicks > impressions {
            return Err(ValidationError::Invariant("clicks exceed impressions"));
        }
        Ok(Self {
            id,
            platform,
            ad_type,
            destination,
            status,
            budget,
            spend,
            impressions,
            clicks,
            conversions,
            attributed_revenue,
        })
    }
}

pub fn campaigns_spend_total(campaigns: &[MarketingCampaign]) -> DomainResult<Money> {
    checked_sum(campaigns.iter().map(|c| c.spend), "campaigns_spend_total")
}

pub fn campaigns_budget_total(campaigns: &[MarketingCampaign]) -> DomainResult<Money> {
    checked_sum(campaigns.iter().map(|c| c.budget), "campaigns_budget_total")
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClickAttributedCampaign {
    pub(crate) campaign: MarketingCampaign,
}

impl ClickAttributedCampaign {
    pub fn try_new(campaign: MarketingCampaign) -> DomainResult<Self> {
        if campaign.conversions > campaign.clicks {
            return Err(ValidationError::Invariant("conversions exceed clicks"));
        }
        Ok(Self { campaign })
    }
}

pub fn meets_roas_target(campaign: &MarketingCampaign, num: Nat, den: Nat) -> DomainResult<bool> {
    Ok(
        checked_mul(campaign.attributed_revenue, den, "ROAS revenue")?
            >= checked_mul(campaign.spend, num, "ROAS spend")?,
    )
}

pub fn meets_roi_target(profit: Money, ad_spend: Money, num: Nat, den: Nat) -> DomainResult<bool> {
    Ok(checked_mul(profit, den, "ROI profit")? >= checked_mul(ad_spend, num, "ROI spend")?)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Funnel {
    pub(crate) visitors: Nat,
    pub(crate) add_to_cart: Nat,
    pub(crate) checkout_started: Nat,
    pub(crate) purchases: Nat,
}

impl Funnel {
    pub fn try_new(
        visitors: Nat,
        add_to_cart: Nat,
        checkout_started: Nat,
        purchases: Nat,
    ) -> DomainResult<Self> {
        if add_to_cart > visitors || checkout_started > add_to_cart || purchases > checkout_started
        {
            return Err(ValidationError::Invariant("funnel counts are not monotone"));
        }
        Ok(Self {
            visitors,
            add_to_cart,
            checkout_started,
            purchases,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConsentStatus {
    Granted,
    Denied,
    Unknown,
}

pub fn can_retarget(consent: ConsentStatus) -> bool {
    consent == ConsentStatus::Granted
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubscriptionStatus {
    Subscribed,
    Unsubscribed,
}

pub fn can_send_marketing_message(status: SubscriptionStatus) -> bool {
    status == SubscriptionStatus::Subscribed
}

domain_struct! {
    pub struct AttributionCredit {
        campaign_id: CampaignId,
        order_id: OrderId,
        amount: Money,
    }
}

pub fn attribution_credit_total(credits: &[AttributionCredit]) -> DomainResult<Money> {
    checked_sum(
        credits.iter().map(|credit| credit.amount),
        "attribution_credit_total",
    )
}

pub fn attribution_credits_match_order(order: &Order, credits: &[AttributionCredit]) -> bool {
    credits.iter().all(|credit| credit.order_id == order.id())
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderAttributionLedger {
    pub(crate) order: Order,
    pub(crate) credits: Vec<AttributionCredit>,
}

impl OrderAttributionLedger {
    pub fn try_new(order: Order, credits: Vec<AttributionCredit>) -> DomainResult<Self> {
        if attribution_credit_total(&credits)? > order.total() {
            return Err(ValidationError::Invariant(
                "attribution credits exceed order total",
            ));
        }
        Ok(Self { order, credits })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchedOrderAttributionLedger {
    pub(crate) ledger: OrderAttributionLedger,
}

impl MatchedOrderAttributionLedger {
    pub fn try_new(ledger: OrderAttributionLedger) -> DomainResult<Self> {
        if !attribution_credits_match_order(&ledger.order, &ledger.credits) {
            return Err(ValidationError::Invariant("credit order ids must match"));
        }
        Ok(Self { ledger })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExperimentVariant {
    pub(crate) id: Id,
    pub(crate) traffic_weight: Nat,
    pub(crate) visitors: Nat,
    pub(crate) conversions: Nat,
}

impl ExperimentVariant {
    pub fn try_new(
        id: Id,
        traffic_weight: Nat,
        visitors: Nat,
        conversions: Nat,
    ) -> DomainResult<Self> {
        if conversions > visitors {
            return Err(ValidationError::Invariant(
                "experiment conversions exceed visitors",
            ));
        }
        Ok(Self {
            id,
            traffic_weight,
            visitors,
            conversions,
        })
    }
}

pub fn experiment_traffic_total(variants: &[ExperimentVariant]) -> DomainResult<Nat> {
    checked_sum(
        variants.iter().map(|v| v.traffic_weight),
        "experiment_traffic_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Experiment {
    pub(crate) id: Id,
    pub(crate) variants: Vec<ExperimentVariant>,
}

impl Experiment {
    pub fn try_new(id: Id, variants: Vec<ExperimentVariant>) -> DomainResult<Self> {
        if experiment_traffic_total(&variants)? != 100 {
            return Err(ValidationError::Invariant(
                "experiment traffic must total 100",
            ));
        }
        Ok(Self { id, variants })
    }
}

impl_getters!(MarketingCampaign {
    id: CampaignId,
    platform: AdPlatform,
    ad_type: AdType,
    destination: AdDestination,
    status: CampaignStatus,
    budget: Money,
    spend: Money,
    impressions: Nat,
    clicks: Nat,
    conversions: Nat,
    attributed_revenue: Money,
});

impl_getters!(ClickAttributedCampaign {
    campaign: MarketingCampaign,
});

impl_getters!(Funnel {
    visitors: Nat,
    add_to_cart: Nat,
    checkout_started: Nat,
    purchases: Nat,
});

impl_getters!(OrderAttributionLedger {
    order: Order,
    credits: Vec<AttributionCredit>,
});

impl_getters!(MatchedOrderAttributionLedger {
    ledger: OrderAttributionLedger,
});

impl_getters!(ExperimentVariant {
    id: Id,
    traffic_weight: Nat,
    visitors: Nat,
    conversions: Nat,
});

impl_getters!(Experiment {
    id: Id,
    variants: Vec<ExperimentVariant>,
});
