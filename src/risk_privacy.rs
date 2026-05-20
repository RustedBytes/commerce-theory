use crate::foundation::*;

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
}

pub fn can_perform(role: Role, action: Action) -> bool {
    matches!(
        (role, action),
        (Role::Admin, _)
            | (Role::Support, Action::ViewOrder)
            | (Role::Warehouse, Action::PackOrder)
            | (Role::Warehouse, Action::ShipOrder)
            | (Role::Warehouse, Action::AdjustStock)
            | (Role::Manager, Action::ViewOrder)
            | (Role::Manager, Action::OverridePrice)
            | (Role::Finance, Action::ViewOrder)
            | (Role::Finance, Action::IssueRefund)
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

impl_getters!(AuditedCommand {
    actor: Role,
    action: Action,
    order_id: OrderId,
    event: AuditEvent,
});
