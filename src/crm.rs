use crate::b2b::*;
use crate::foundation::*;
use crate::marketing::*;
use crate::pricing::*;
use crate::risk_privacy::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccountTier {
    Standard,
    Preferred,
    Strategic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CRMAccountStatus {
    Prospect,
    Active,
    Paused,
    Closed,
}

pub fn can_crm_account_transition(source: CRMAccountStatus, target: CRMAccountStatus) -> bool {
    matches!(
        (source, target),
        (CRMAccountStatus::Prospect, CRMAccountStatus::Active)
            | (CRMAccountStatus::Prospect, CRMAccountStatus::Closed)
            | (CRMAccountStatus::Active, CRMAccountStatus::Paused)
            | (CRMAccountStatus::Active, CRMAccountStatus::Closed)
            | (CRMAccountStatus::Paused, CRMAccountStatus::Active)
            | (CRMAccountStatus::Paused, CRMAccountStatus::Closed)
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CRMAccount {
    pub(crate) id: AccountId,
    pub(crate) customer: Customer,
    pub(crate) tier: AccountTier,
    pub(crate) status: CRMAccountStatus,
    pub(crate) lifetime_value: Money,
    pub(crate) open_balance: Money,
}

impl CRMAccount {
    pub fn try_new(
        id: AccountId,
        customer: Customer,
        tier: AccountTier,
        status: CRMAccountStatus,
        lifetime_value: Money,
        open_balance: Money,
    ) -> DomainResult<Self> {
        if open_balance > lifetime_value {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            customer,
            tier,
            status,
            lifetime_value,
            open_balance,
        })
    }
}

pub fn crm_account_active(account: &CRMAccount) -> bool {
    account.status == CRMAccountStatus::Active
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveCRMAccount {
    pub(crate) account: CRMAccount,
}

impl ActiveCRMAccount {
    pub fn try_new(account: CRMAccount) -> DomainResult<Self> {
        if !crm_account_active(&account) {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self { account })
    }
}

pub fn transition_crm_account(
    account: CRMAccount,
    next: CRMAccountStatus,
) -> DomainResult<CRMAccount> {
    if !can_crm_account_transition(account.status, next) {
        return Err(ValidationError::CrmInvariantFailed);
    }
    Ok(CRMAccount {
        status: next,
        ..account
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContactKind {
    Primary,
    Billing,
    Shipping,
    Buyer,
    Support,
}

domain_struct! {
    pub struct CRMContact {
        id: ContactId,
        account_id: AccountId,
        customer_id: CustomerId,
        kind: ContactKind,
        owner_role: Role,
        subscription: SubscriptionStatus,
        retargeting_consent: ConsentStatus,
        data_permission: DataProcessingPermission,
    }
}

pub fn contact_can_receive_marketing(contact: &CRMContact) -> bool {
    can_send_marketing_message(contact.subscription)
        && can_retarget(contact.retargeting_consent)
        && data_processing_allowed(&contact.data_permission)
        && contact.data_permission.purpose() == &ConsentPurpose::Marketing
        && contact.data_permission.basis() == &ProcessingBasis::Consent
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CRMAccountContact {
    pub(crate) account: CRMAccount,
    pub(crate) contact: CRMContact,
}

impl CRMAccountContact {
    pub fn try_new(account: CRMAccount, contact: CRMContact) -> DomainResult<Self> {
        if contact.account_id != account.id || contact.customer_id != account.customer.id {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self { account, contact })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PermittedCustomerMessage {
    pub(crate) interaction_id: InteractionId,
    pub(crate) contact: CRMContact,
    pub(crate) sent_at: Timestamp,
}

impl PermittedCustomerMessage {
    pub fn try_new(
        interaction_id: InteractionId,
        contact: CRMContact,
        sent_at: Timestamp,
    ) -> DomainResult<Self> {
        if !contact_can_receive_marketing(&contact) {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            interaction_id,
            contact,
            sent_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PermittedAccountMessage {
    pub(crate) account_contact: CRMAccountContact,
    pub(crate) message: PermittedCustomerMessage,
}

impl PermittedAccountMessage {
    pub fn try_new(
        account_contact: CRMAccountContact,
        message: PermittedCustomerMessage,
    ) -> DomainResult<Self> {
        if message.contact != account_contact.contact {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account_contact,
            message,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InteractionKind {
    Email,
    Call,
    Meeting,
    Chat,
    SupportNote,
    OrderNote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CRMInteraction {
    pub(crate) id: InteractionId,
    pub(crate) account_id: AccountId,
    pub(crate) contact_id: ContactId,
    pub(crate) kind: InteractionKind,
    pub(crate) occurred_at: Timestamp,
    pub(crate) follow_up_due_at: Timestamp,
}

impl CRMInteraction {
    pub fn try_new(
        id: InteractionId,
        account_id: AccountId,
        contact_id: ContactId,
        kind: InteractionKind,
        occurred_at: Timestamp,
        follow_up_due_at: Timestamp,
    ) -> DomainResult<Self> {
        if follow_up_due_at < occurred_at {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            account_id,
            contact_id,
            kind,
            occurred_at,
            follow_up_due_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CRMInteractionForContact {
    pub(crate) account_contact: CRMAccountContact,
    pub(crate) interaction: CRMInteraction,
}

impl CRMInteractionForContact {
    pub fn try_new(
        account_contact: CRMAccountContact,
        interaction: CRMInteraction,
    ) -> DomainResult<Self> {
        if interaction.account_id != account_contact.account.id
            || interaction.contact_id != account_contact.contact.id
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account_contact,
            interaction,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LeadStatus {
    New,
    Working,
    Qualified,
    Disqualified,
    Converted,
}

pub fn can_lead_transition(source: LeadStatus, target: LeadStatus) -> bool {
    matches!(
        (source, target),
        (LeadStatus::New, LeadStatus::Working)
            | (LeadStatus::New, LeadStatus::Disqualified)
            | (LeadStatus::Working, LeadStatus::Qualified)
            | (LeadStatus::Working, LeadStatus::Disqualified)
            | (LeadStatus::Qualified, LeadStatus::Converted)
            | (LeadStatus::Qualified, LeadStatus::Disqualified)
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lead {
    pub(crate) id: LeadId,
    pub(crate) account_id: AccountId,
    pub(crate) contact_id: ContactId,
    pub(crate) source_campaign: Option<CampaignId>,
    pub(crate) status: LeadStatus,
    pub(crate) estimated_value: Money,
    pub(crate) currency: Currency,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,
}

impl Lead {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: LeadId,
        account_id: AccountId,
        contact_id: ContactId,
        source_campaign: Option<CampaignId>,
        status: LeadStatus,
        estimated_value: Money,
        currency: Currency,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> DomainResult<Self> {
        if updated_at < created_at {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            account_id,
            contact_id,
            source_campaign,
            status,
            estimated_value,
            currency,
            created_at,
            updated_at,
        })
    }
}

pub fn transition_lead(lead: Lead, next: LeadStatus, updated_at: Timestamp) -> DomainResult<Lead> {
    if !can_lead_transition(lead.status, next) || updated_at < lead.created_at {
        return Err(ValidationError::CrmInvariantFailed);
    }
    Ok(Lead {
        status: next,
        updated_at,
        ..lead
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LeadForContact {
    pub(crate) account_contact: CRMAccountContact,
    pub(crate) lead: Lead,
}

impl LeadForContact {
    pub fn try_new(account_contact: CRMAccountContact, lead: Lead) -> DomainResult<Self> {
        if lead.account_id != account_contact.account.id
            || lead.contact_id != account_contact.contact.id
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account_contact,
            lead,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpportunityStage {
    Prospecting,
    Qualified,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

pub fn can_opportunity_transition(source: OpportunityStage, target: OpportunityStage) -> bool {
    matches!(
        (source, target),
        (OpportunityStage::Prospecting, OpportunityStage::Qualified)
            | (OpportunityStage::Prospecting, OpportunityStage::Lost)
            | (OpportunityStage::Qualified, OpportunityStage::Proposal)
            | (OpportunityStage::Qualified, OpportunityStage::Lost)
            | (OpportunityStage::Proposal, OpportunityStage::Negotiation)
            | (OpportunityStage::Proposal, OpportunityStage::Lost)
            | (OpportunityStage::Negotiation, OpportunityStage::Won)
            | (OpportunityStage::Negotiation, OpportunityStage::Lost)
    )
}

pub fn opportunity_stage_probability_allowed(
    stage: OpportunityStage,
    probability: BasisPoints,
) -> bool {
    match stage {
        OpportunityStage::Won => probability.value() == 10_000,
        OpportunityStage::Lost => probability.value() == 0,
        _ => true,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SalesOpportunity {
    pub(crate) id: OpportunityId,
    pub(crate) account_id: AccountId,
    pub(crate) contact_id: ContactId,
    pub(crate) source_lead: Option<LeadId>,
    pub(crate) stage: OpportunityStage,
    pub(crate) amount: Money,
    pub(crate) currency: Currency,
    pub(crate) probability: BasisPoints,
    pub(crate) opened_at: Timestamp,
    pub(crate) updated_at: Timestamp,
    pub(crate) expected_close_at: Timestamp,
}

impl SalesOpportunity {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: OpportunityId,
        account_id: AccountId,
        contact_id: ContactId,
        source_lead: Option<LeadId>,
        stage: OpportunityStage,
        amount: Money,
        currency: Currency,
        probability: BasisPoints,
        opened_at: Timestamp,
        updated_at: Timestamp,
        expected_close_at: Timestamp,
    ) -> DomainResult<Self> {
        if updated_at < opened_at
            || expected_close_at < opened_at
            || !opportunity_stage_probability_allowed(stage, probability)
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            account_id,
            contact_id,
            source_lead,
            stage,
            amount,
            currency,
            probability,
            opened_at,
            updated_at,
            expected_close_at,
        })
    }
}

pub fn opportunity_weighted_value(opportunity: &SalesOpportunity) -> DomainResult<Money> {
    apply_bps(opportunity.probability, opportunity.amount)
}

pub fn transition_opportunity(
    opportunity: SalesOpportunity,
    next: OpportunityStage,
    probability: BasisPoints,
    updated_at: Timestamp,
    expected_close_at: Timestamp,
) -> DomainResult<SalesOpportunity> {
    if !can_opportunity_transition(opportunity.stage, next)
        || !opportunity_stage_probability_allowed(next, probability)
        || updated_at < opportunity.opened_at
        || expected_close_at < opportunity.opened_at
    {
        return Err(ValidationError::CrmInvariantFailed);
    }
    Ok(SalesOpportunity {
        stage: next,
        probability,
        updated_at,
        expected_close_at,
        ..opportunity
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpportunityForContact {
    pub(crate) account_contact: CRMAccountContact,
    pub(crate) opportunity: SalesOpportunity,
}

impl OpportunityForContact {
    pub fn try_new(
        account_contact: CRMAccountContact,
        opportunity: SalesOpportunity,
    ) -> DomainResult<Self> {
        if opportunity.account_id != account_contact.account.id
            || opportunity.contact_id != account_contact.contact.id
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account_contact,
            opportunity,
        })
    }
}

pub fn opportunity_gross_value(opportunities: &[SalesOpportunity]) -> DomainResult<Money> {
    checked_sum(
        opportunities.iter().map(|opportunity| opportunity.amount),
        "opportunity_gross_value",
    )
}

pub fn opportunity_weighted_value_total(opportunities: &[SalesOpportunity]) -> DomainResult<Money> {
    checked_sum(
        opportunities
            .iter()
            .map(opportunity_weighted_value)
            .collect::<DomainResult<Vec<_>>>()?,
        "opportunity_weighted_value_total",
    )
}

pub fn opportunities_use_currency(currency: Currency, opportunities: &[SalesOpportunity]) -> bool {
    opportunities
        .iter()
        .all(|opportunity| opportunity.currency == currency)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SalesPipeline {
    pub(crate) currency: Currency,
    pub(crate) opportunities: Vec<SalesOpportunity>,
}

impl SalesPipeline {
    pub fn try_new(currency: Currency, opportunities: Vec<SalesOpportunity>) -> DomainResult<Self> {
        if !opportunities_use_currency(currency, &opportunities) {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            currency,
            opportunities,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomerSegment {
    pub(crate) id: SegmentId,
    pub(crate) name: String,
    pub(crate) member_count: Nat,
    pub(crate) min_lifetime_value: Money,
    pub(crate) max_retention_discount: Money,
}

impl CustomerSegment {
    pub fn try_new(
        id: SegmentId,
        name: String,
        member_count: Nat,
        min_lifetime_value: Money,
        max_retention_discount: Money,
    ) -> DomainResult<Self> {
        if max_retention_discount > min_lifetime_value {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            name,
            member_count,
            min_lifetime_value,
            max_retention_discount,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentMembership {
    pub(crate) account: CRMAccount,
    pub(crate) segment: CustomerSegment,
}

impl SegmentMembership {
    pub fn try_new(account: CRMAccount, segment: CustomerSegment) -> DomainResult<Self> {
        if account.lifetime_value < segment.min_lifetime_value {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self { account, segment })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupportPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupportCaseStatus {
    Opened,
    WaitingOnCustomer,
    WaitingOnInternal,
    Escalated,
    Resolved,
    Closed,
}

pub fn can_support_case_transition(source: SupportCaseStatus, target: SupportCaseStatus) -> bool {
    matches!(
        (source, target),
        (
            SupportCaseStatus::Opened,
            SupportCaseStatus::WaitingOnCustomer
        ) | (
            SupportCaseStatus::Opened,
            SupportCaseStatus::WaitingOnInternal
        ) | (SupportCaseStatus::Opened, SupportCaseStatus::Escalated)
            | (SupportCaseStatus::Opened, SupportCaseStatus::Resolved)
            | (
                SupportCaseStatus::WaitingOnCustomer,
                SupportCaseStatus::Resolved
            )
            | (
                SupportCaseStatus::WaitingOnCustomer,
                SupportCaseStatus::Escalated
            )
            | (
                SupportCaseStatus::WaitingOnInternal,
                SupportCaseStatus::Resolved
            )
            | (
                SupportCaseStatus::WaitingOnInternal,
                SupportCaseStatus::Escalated
            )
            | (SupportCaseStatus::Escalated, SupportCaseStatus::Resolved)
            | (SupportCaseStatus::Resolved, SupportCaseStatus::Closed)
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SupportCase {
    pub(crate) id: SupportCaseId,
    pub(crate) account_id: AccountId,
    pub(crate) contact_id: ContactId,
    pub(crate) order_id: Option<OrderId>,
    pub(crate) status: SupportCaseStatus,
    pub(crate) priority: SupportPriority,
    pub(crate) opened_at: Timestamp,
    pub(crate) last_updated_at: Timestamp,
    pub(crate) sla_due_at: Timestamp,
}

impl SupportCase {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: SupportCaseId,
        account_id: AccountId,
        contact_id: ContactId,
        order_id: Option<OrderId>,
        status: SupportCaseStatus,
        priority: SupportPriority,
        opened_at: Timestamp,
        last_updated_at: Timestamp,
        sla_due_at: Timestamp,
    ) -> DomainResult<Self> {
        if last_updated_at < opened_at || sla_due_at < opened_at {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            id,
            account_id,
            contact_id,
            order_id,
            status,
            priority,
            opened_at,
            last_updated_at,
            sla_due_at,
        })
    }
}

pub fn transition_support_case(
    case_: SupportCase,
    next: SupportCaseStatus,
    updated_at: Timestamp,
) -> DomainResult<SupportCase> {
    if !can_support_case_transition(case_.status, next) || updated_at < case_.opened_at {
        return Err(ValidationError::CrmInvariantFailed);
    }
    Ok(SupportCase {
        status: next,
        last_updated_at: updated_at,
        ..case_
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SupportCaseForContact {
    pub(crate) account_contact: CRMAccountContact,
    pub(crate) case_: SupportCase,
}

impl SupportCaseForContact {
    pub fn try_new(account_contact: CRMAccountContact, case_: SupportCase) -> DomainResult<Self> {
        if case_.account_id != account_contact.account.id
            || case_.contact_id != account_contact.contact.id
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account_contact,
            case_,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResolvedSupportCase {
    pub(crate) case_: SupportCase,
    pub(crate) resolved_at: Timestamp,
}

impl ResolvedSupportCase {
    pub fn try_new(case_: SupportCase, resolved_at: Timestamp) -> DomainResult<Self> {
        if case_.status != SupportCaseStatus::Resolved
            || resolved_at < case_.opened_at
            || resolved_at < case_.last_updated_at
            || resolved_at > case_.sla_due_at
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self { case_, resolved_at })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetentionOffer {
    pub(crate) account: CRMAccount,
    pub(crate) segment: CustomerSegment,
    pub(crate) coupon: Coupon,
    pub(crate) uses_before: Nat,
    pub(crate) discount: Money,
}

impl RetentionOffer {
    pub fn try_new(
        account: CRMAccount,
        segment: CustomerSegment,
        coupon: Coupon,
        uses_before: Nat,
        discount: Money,
    ) -> DomainResult<Self> {
        if !crm_account_active(&account)
            || !coupon_can_be_applied(&coupon, account.lifetime_value, uses_before)
            || account.lifetime_value < segment.min_lifetime_value
            || discount > *coupon.amount()
            || *coupon.amount() > account.lifetime_value
            || discount > segment.max_retention_discount
        {
            return Err(ValidationError::CrmInvariantFailed);
        }
        Ok(Self {
            account,
            segment,
            coupon,
            uses_before,
            discount,
        })
    }
}

impl_getters!(CRMAccount {
    id: AccountId,
    customer: Customer,
    tier: AccountTier,
    status: CRMAccountStatus,
    lifetime_value: Money,
    open_balance: Money,
});

impl_getters!(ActiveCRMAccount {
    account: CRMAccount
});

impl_getters!(CRMAccountContact {
    account: CRMAccount,
    contact: CRMContact,
});

impl_getters!(PermittedCustomerMessage {
    interaction_id: InteractionId,
    contact: CRMContact,
    sent_at: Timestamp,
});

impl_getters!(PermittedAccountMessage {
    account_contact: CRMAccountContact,
    message: PermittedCustomerMessage,
});

impl_getters!(CRMInteraction {
    id: InteractionId,
    account_id: AccountId,
    contact_id: ContactId,
    kind: InteractionKind,
    occurred_at: Timestamp,
    follow_up_due_at: Timestamp,
});

impl_getters!(CRMInteractionForContact {
    account_contact: CRMAccountContact,
    interaction: CRMInteraction,
});

impl_getters!(Lead {
    id: LeadId,
    account_id: AccountId,
    contact_id: ContactId,
    source_campaign: Option<CampaignId>,
    status: LeadStatus,
    estimated_value: Money,
    currency: Currency,
    created_at: Timestamp,
    updated_at: Timestamp,
});

impl_getters!(LeadForContact {
    account_contact: CRMAccountContact,
    lead: Lead,
});

impl_getters!(SalesOpportunity {
    id: OpportunityId,
    account_id: AccountId,
    contact_id: ContactId,
    source_lead: Option<LeadId>,
    stage: OpportunityStage,
    amount: Money,
    currency: Currency,
    probability: BasisPoints,
    opened_at: Timestamp,
    updated_at: Timestamp,
    expected_close_at: Timestamp,
});

impl_getters!(OpportunityForContact {
    account_contact: CRMAccountContact,
    opportunity: SalesOpportunity,
});

impl_getters!(SalesPipeline {
    currency: Currency,
    opportunities: Vec<SalesOpportunity>,
});

impl_getters!(CustomerSegment {
    id: SegmentId,
    name: String,
    member_count: Nat,
    min_lifetime_value: Money,
    max_retention_discount: Money,
});

impl_getters!(SegmentMembership {
    account: CRMAccount,
    segment: CustomerSegment,
});

impl_getters!(SupportCase {
    id: SupportCaseId,
    account_id: AccountId,
    contact_id: ContactId,
    order_id: Option<OrderId>,
    status: SupportCaseStatus,
    priority: SupportPriority,
    opened_at: Timestamp,
    last_updated_at: Timestamp,
    sla_due_at: Timestamp,
});

impl_getters!(SupportCaseForContact {
    account_contact: CRMAccountContact,
    case_: SupportCase,
});

impl_getters!(ResolvedSupportCase {
    case_: SupportCase,
    resolved_at: Timestamp,
});

impl_getters!(RetentionOffer {
    account: CRMAccount,
    segment: CustomerSegment,
    coupon: Coupon,
    uses_before: Nat,
    discount: Money,
});
