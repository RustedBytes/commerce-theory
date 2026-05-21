use crate::foundation::*;
use crate::marketing::*;

domain_struct! {
    pub struct FraudPolicy {
        max_coupon_uses: Nat,
        max_orders_per_hour: Nat,
        max_zero_total_items: Nat,
    }
}

pub fn coupon_uses_allowed(policy: &FraudPolicy, uses: Nat) -> bool {
    uses <= policy.max_coupon_uses
}

pub fn orders_per_hour_allowed(policy: &FraudPolicy, orders_per_hour: Nat) -> bool {
    orders_per_hour <= policy.max_orders_per_hour
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Role {
    Customer,
    Support,
    Warehouse,
    Manager,
    Finance,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    ViewOrder,
    PackOrder,
    ShipOrder,
    IssueRefund,
    OverridePrice,
    AdjustStock,
    DeleteOrder,
    ManageCRM,
    CreateSupportCase,
    ResolveSupportCase,
    ManageShipment,
    ApproveReturn,
}

pub fn can_perform(role: Role, action: Action) -> bool {
    matches!(
        (role, action),
        (Role::Admin, _)
            | (Role::Support, Action::ViewOrder)
            | (Role::Warehouse, Action::PackOrder)
            | (Role::Warehouse, Action::ShipOrder)
            | (Role::Warehouse, Action::AdjustStock)
            | (Role::Warehouse, Action::ManageShipment)
            | (Role::Manager, Action::ViewOrder)
            | (Role::Manager, Action::OverridePrice)
            | (Role::Manager, Action::ManageCRM)
            | (Role::Manager, Action::ResolveSupportCase)
            | (Role::Manager, Action::ApproveReturn)
            | (Role::Support, Action::CreateSupportCase)
            | (Role::Support, Action::ResolveSupportCase)
            | (Role::Support, Action::ManageCRM)
            | (Role::Finance, Action::ViewOrder)
            | (Role::Finance, Action::IssueRefund)
            | (Role::Finance, Action::ApproveReturn)
    )
}

domain_struct! {
    pub struct AuditEvent {
        actor: Role,
        action: Action,
        order_id: OrderId,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuditedCommand {
    pub(crate) actor: Role,
    pub(crate) action: Action,
    pub(crate) order_id: OrderId,
    pub(crate) event: AuditEvent,
}

domain_struct! {
    pub struct EntityAuditEvent {
        actor: Role,
        action: Action,
        subject_id: Id,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuditedEntityCommand {
    pub(crate) actor: Role,
    pub(crate) action: Action,
    pub(crate) subject_id: Id,
    pub(crate) event: EntityAuditEvent,
}

impl AuditedEntityCommand {
    pub fn try_new(
        actor: Role,
        action: Action,
        subject_id: Id,
        event: EntityAuditEvent,
    ) -> DomainResult<Self> {
        if !can_perform(actor, action) {
            return Err(ValidationError::AuditPermissionDenied);
        }
        if event.actor != actor || event.action != action || event.subject_id != subject_id {
            return Err(ValidationError::Invariant(
                "entity audit event does not match command",
            ));
        }
        Ok(Self {
            actor,
            action,
            subject_id,
            event,
        })
    }
}

impl AuditedCommand {
    pub fn try_new(
        actor: Role,
        action: Action,
        order_id: OrderId,
        event: AuditEvent,
    ) -> DomainResult<Self> {
        if !can_perform(actor, action) {
            return Err(ValidationError::Invariant("actor cannot perform action"));
        }
        if event.actor != actor || event.action != action || event.order_id != order_id {
            return Err(ValidationError::Invariant(
                "audit event does not match command",
            ));
        }
        Ok(Self {
            actor,
            action,
            order_id,
            event,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConsentPurpose {
    Marketing,
    Analytics,
    Personalization,
    FraudPrevention,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProcessingBasis {
    Consent,
    Contract,
    LegalObligation,
    LegitimateInterest,
}

domain_struct! {
    pub struct DataProcessingPermission {
        purpose: ConsentPurpose,
        basis: ProcessingBasis,
        allowed: bool,
    }
}

pub fn data_processing_allowed(permission: &DataProcessingPermission) -> bool {
    permission.allowed
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataCategory {
    CustomerProfile,
    ContactData,
    OrderData,
    PaymentToken,
    MarketingProfile,
    SupportNotes,
    AnalyticsEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccessPurpose {
    CustomerSupport,
    Fulfillment,
    RefundProcessing,
    MarketingOperations,
    FraudReview,
    Analytics,
    Administration,
}

pub fn role_can_access_data(role: Role, purpose: AccessPurpose, category: DataCategory) -> bool {
    matches!(
        (role, purpose, category),
        (Role::Admin, _, _)
            | (
                Role::Support,
                AccessPurpose::CustomerSupport,
                DataCategory::OrderData
            )
            | (
                Role::Support,
                AccessPurpose::CustomerSupport,
                DataCategory::ContactData
            )
            | (
                Role::Support,
                AccessPurpose::CustomerSupport,
                DataCategory::SupportNotes
            )
            | (
                Role::Warehouse,
                AccessPurpose::Fulfillment,
                DataCategory::OrderData
            )
            | (
                Role::Warehouse,
                AccessPurpose::Fulfillment,
                DataCategory::ContactData
            )
            | (
                Role::Finance,
                AccessPurpose::RefundProcessing,
                DataCategory::OrderData
            )
            | (
                Role::Finance,
                AccessPurpose::RefundProcessing,
                DataCategory::PaymentToken
            )
            | (
                Role::Manager,
                AccessPurpose::MarketingOperations,
                DataCategory::MarketingProfile
            )
            | (
                Role::Manager,
                AccessPurpose::MarketingOperations,
                DataCategory::ContactData
            )
            | (
                Role::Manager,
                AccessPurpose::Administration,
                DataCategory::CustomerProfile
            )
            | (
                Role::Manager,
                AccessPurpose::Administration,
                DataCategory::MarketingProfile
            )
    )
}

pub fn processing_allowed_for(
    permission: &DataProcessingPermission,
    purpose: ConsentPurpose,
    basis: ProcessingBasis,
) -> bool {
    data_processing_allowed(permission)
        && permission.purpose == purpose
        && permission.basis == basis
}

domain_struct! {
    pub struct MarketingConsentState {
        subscription: SubscriptionStatus,
        retargeting_consent: ConsentStatus,
        data_permission: DataProcessingPermission,
    }
}

pub fn marketing_allowed(state: &MarketingConsentState) -> bool {
    can_send_marketing_message(state.subscription)
        && can_retarget(state.retargeting_consent)
        && processing_allowed_for(
            &state.data_permission,
            ConsentPurpose::Marketing,
            ProcessingBasis::Consent,
        )
}

pub fn withdraw_marketing_consent(state: &MarketingConsentState) -> MarketingConsentState {
    MarketingConsentState::new(
        SubscriptionStatus::Unsubscribed,
        ConsentStatus::Denied,
        DataProcessingPermission::new(
            *state.data_permission.purpose(),
            *state.data_permission.basis(),
            false,
        ),
    )
}

domain_struct! {
    pub struct DataRetentionPolicy {
        category: DataCategory,
        retention_window: Duration,
    }
}

pub fn within_retention_window(
    policy: &DataRetentionPolicy,
    now: Timestamp,
    collected_at: Timestamp,
) -> bool {
    collected_at <= now && timestamp_age(now, collected_at) <= policy.retention_window
}

pub fn retention_expired(
    policy: &DataRetentionPolicy,
    now: Timestamp,
    collected_at: Timestamp,
) -> bool {
    collected_at <= now && policy.retention_window < timestamp_age(now, collected_at)
}

pub fn can_retain_personal_data(
    policy: &DataRetentionPolicy,
    now: Timestamp,
    collected_at: Timestamp,
) -> bool {
    within_retention_window(policy, now, collected_at)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetainedPersonalData {
    pub(crate) subject_id: CustomerId,
    pub(crate) category: DataCategory,
    pub(crate) collected_at: Timestamp,
    pub(crate) checked_at: Timestamp,
    pub(crate) policy: DataRetentionPolicy,
}

impl RetainedPersonalData {
    pub fn try_new(
        subject_id: CustomerId,
        category: DataCategory,
        collected_at: Timestamp,
        checked_at: Timestamp,
        policy: DataRetentionPolicy,
    ) -> DomainResult<Self> {
        if policy.category != category
            || !can_retain_personal_data(&policy, checked_at, collected_at)
        {
            return Err(ValidationError::Invariant(
                "personal data cannot be retained",
            ));
        }
        Ok(Self {
            subject_id,
            category,
            collected_at,
            checked_at,
            policy,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErasureStatus {
    Active,
    Requested,
    Completed,
    BlockedByLegalHold,
}

pub fn personal_data_usable(status: ErasureStatus) -> bool {
    status == ErasureStatus::Active
}

pub fn can_process_personal_data(
    status: ErasureStatus,
    permission: &DataProcessingPermission,
    purpose: ConsentPurpose,
    basis: ProcessingBasis,
) -> bool {
    personal_data_usable(status) && processing_allowed_for(permission, purpose, basis)
}

pub fn can_complete_erasure(status: ErasureStatus, legal_hold: bool) -> bool {
    status == ErasureStatus::Requested && !legal_hold
}

pub fn audit_log_appended(
    before: &[EntityAuditEvent],
    after: &[EntityAuditEvent],
    new_events: &[EntityAuditEvent],
) -> bool {
    after.len() == before.len() + new_events.len()
        && after.starts_with(before)
        && after[before.len()..] == new_events[..]
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuditedDataAccess {
    pub(crate) actor: Role,
    pub(crate) action: Action,
    pub(crate) purpose: AccessPurpose,
    pub(crate) category: DataCategory,
    pub(crate) subject_id: Id,
    pub(crate) event: EntityAuditEvent,
}

impl AuditedDataAccess {
    pub fn try_new(
        actor: Role,
        action: Action,
        purpose: AccessPurpose,
        category: DataCategory,
        subject_id: Id,
        event: EntityAuditEvent,
    ) -> DomainResult<Self> {
        if !can_perform(actor, action) || !role_can_access_data(actor, purpose, category) {
            return Err(ValidationError::AuditPermissionDenied);
        }
        if event.actor != actor || event.action != action || event.subject_id != subject_id {
            return Err(ValidationError::Invariant(
                "data-access audit event does not match access",
            ));
        }
        Ok(Self {
            actor,
            action,
            purpose,
            category,
            subject_id,
            event,
        })
    }
}

impl_getters!(AuditedCommand {
    actor: Role,
    action: Action,
    order_id: OrderId,
    event: AuditEvent,
});

impl_getters!(AuditedEntityCommand {
    actor: Role,
    action: Action,
    subject_id: Id,
    event: EntityAuditEvent,
});

impl_getters!(RetainedPersonalData {
    subject_id: CustomerId,
    category: DataCategory,
    collected_at: Timestamp,
    checked_at: Timestamp,
    policy: DataRetentionPolicy,
});

impl_getters!(AuditedDataAccess {
    actor: Role,
    action: Action,
    purpose: AccessPurpose,
    category: DataCategory,
    subject_id: Id,
    event: EntityAuditEvent,
});
