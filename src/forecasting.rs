use crate::dropshipping::*;
use crate::foundation::*;
use crate::post_purchase::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Confidence {
    Low,
    Medium,
    High,
}

pub fn confidence_allows_auto_replenish(confidence: Confidence) -> bool {
    matches!(confidence, Confidence::Medium | Confidence::High)
}

domain_struct! {
    pub struct DemandForecast {
        sku: Sku,
        expected_units: Quantity,
        confidence: Confidence,
        horizon_days: Days,
    }
}

domain_struct! {
    pub struct SupplierQualityMetrics {
        defect_rate_bps: Nat,
        late_shipment_rate_bps: Nat,
        cancellation_rate_bps: Nat,
    }
}

domain_struct! {
    pub struct SupplierRiskPolicy {
        max_defect_rate_bps: Nat,
        max_late_shipment_rate_bps: Nat,
        max_cancellation_rate_bps: Nat,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApprovedSupplierQuality {
    pub(crate) supplier: DropshipSupplier,
    pub(crate) metrics: SupplierQualityMetrics,
    pub(crate) policy: SupplierRiskPolicy,
}

impl ApprovedSupplierQuality {
    pub fn try_new(
        supplier: DropshipSupplier,
        metrics: SupplierQualityMetrics,
        policy: SupplierRiskPolicy,
    ) -> DomainResult<Self> {
        if metrics.defect_rate_bps > policy.max_defect_rate_bps
            || metrics.late_shipment_rate_bps > policy.max_late_shipment_rate_bps
            || metrics.cancellation_rate_bps > policy.max_cancellation_rate_bps
        {
            return Err(ValidationError::Invariant(
                "supplier quality metrics exceed policy",
            ));
        }
        Ok(Self {
            supplier,
            metrics,
            policy,
        })
    }
}

pub(crate) fn _post_purchase_anchor(_: Option<SubscriptionLifecycleStatus>) {}

impl_getters!(ApprovedSupplierQuality {
    supplier: DropshipSupplier,
    metrics: SupplierQualityMetrics,
    policy: SupplierRiskPolicy,
});
