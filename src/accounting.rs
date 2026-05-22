use crate::foundation::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PostingSide {
    Debit,
    Credit,
}

domain_struct! {
    pub struct LedgerAccount {
        id: Id,
        name: String,
    }
}

domain_struct! {
    pub struct Posting {
        account: LedgerAccount,
        side: PostingSide,
        amount: Money,
    }
}

#[must_use]
pub const fn debit(account: LedgerAccount, amount: Money) -> Posting {
    Posting::new(account, PostingSide::Debit, amount)
}

#[must_use]
pub const fn credit(account: LedgerAccount, amount: Money) -> Posting {
    Posting::new(account, PostingSide::Credit, amount)
}

pub fn debit_total(postings: &[Posting]) -> DomainResult<Money> {
    checked_sum(
        postings.iter().map(|posting| {
            if posting.side == PostingSide::Debit {
                posting.amount
            } else {
                0
            }
        }),
        "debit_total",
    )
}

pub fn credit_total(postings: &[Posting]) -> DomainResult<Money> {
    checked_sum(
        postings.iter().map(|posting| {
            if posting.side == PostingSide::Credit {
                posting.amount
            } else {
                0
            }
        }),
        "credit_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalancedJournalEntry {
    pub(crate) postings: Vec<Posting>,
}

impl BalancedJournalEntry {
    pub fn try_new(postings: Vec<Posting>) -> DomainResult<Self> {
        if debit_total(&postings)? != credit_total(&postings)? {
            return Err(ValidationError::Invariant("journal entry is not balanced"));
        }
        Ok(Self { postings })
    }

    #[must_use]
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
}

domain_struct! {
    pub struct AccountingAccounts {
        cash: LedgerAccount,
        deferred_revenue: LedgerAccount,
        revenue: LedgerAccount,
        refunds: LedgerAccount,
        inventory: LedgerAccount,
        cogs: LedgerAccount,
    }
}

pub fn payment_captured_journal(
    accounts: &AccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.cash.clone(), amount),
        credit(accounts.deferred_revenue.clone(), amount),
    ])
}

pub fn refund_issued_journal(
    accounts: &AccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.refunds.clone(), amount),
        credit(accounts.cash.clone(), amount),
    ])
}

domain_struct! {
    pub struct AdvancedAccountingAccounts {
        operating: AccountingAccounts,
        accounts_receivable: LedgerAccount,
        accounts_payable: LedgerAccount,
        tax_liability: LedgerAccount,
        marketplace_clearing: LedgerAccount,
        marketplace_fees: LedgerAccount,
        chargeback_reserve: LedgerAccount,
        chargeback_expense: LedgerAccount,
        realized_fx_gain: LedgerAccount,
        realized_fx_loss: LedgerAccount,
        unrealized_fx_gain: LedgerAccount,
        unrealized_fx_loss: LedgerAccount,
    }
}

pub fn invoice_accrual_journal(
    accounts: &AdvancedAccountingAccounts,
    subtotal: Money,
    tax: Money,
    total: Money,
) -> DomainResult<BalancedJournalEntry> {
    if total != checked_add(subtotal, tax, "invoice accrual total")? {
        return Err(ValidationError::AccountingInvariantFailed);
    }
    BalancedJournalEntry::try_new(vec![
        debit(accounts.accounts_receivable.clone(), total),
        credit(accounts.operating.revenue.clone(), subtotal),
        credit(accounts.tax_liability.clone(), tax),
    ])
}

pub fn cash_sale_journal(
    accounts: &AdvancedAccountingAccounts,
    subtotal: Money,
    tax: Money,
    total: Money,
) -> DomainResult<BalancedJournalEntry> {
    if total != checked_add(subtotal, tax, "cash sale total")? {
        return Err(ValidationError::AccountingInvariantFailed);
    }
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.cash.clone(), total),
        credit(accounts.operating.revenue.clone(), subtotal),
        credit(accounts.tax_liability.clone(), tax),
    ])
}

pub fn receivable_collection_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.cash.clone(), amount),
        credit(accounts.accounts_receivable.clone(), amount),
    ])
}

pub fn supplier_bill_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.inventory.clone(), amount),
        credit(accounts.accounts_payable.clone(), amount),
    ])
}

pub fn supplier_payment_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.accounts_payable.clone(), amount),
        credit(accounts.operating.cash.clone(), amount),
    ])
}

pub fn marketplace_sale_clearing_journal(
    accounts: &AdvancedAccountingAccounts,
    gross: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.marketplace_clearing.clone(), gross),
        credit(accounts.operating.revenue.clone(), gross),
    ])
}

pub fn marketplace_settlement_journal(
    accounts: &AdvancedAccountingAccounts,
    gross: Money,
    fee: Money,
    payout: Money,
) -> DomainResult<BalancedJournalEntry> {
    if checked_add(payout, fee, "marketplace settlement")? != gross {
        return Err(ValidationError::AccountingInvariantFailed);
    }
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.cash.clone(), payout),
        debit(accounts.marketplace_fees.clone(), fee),
        credit(accounts.marketplace_clearing.clone(), gross),
    ])
}

pub fn marketplace_payout_reconciliation_journal(
    accounts: &AdvancedAccountingAccounts,
    gross: Money,
    fee: Money,
    refund: Money,
    reserve: Money,
    tax: Money,
    payout: Money,
) -> DomainResult<BalancedJournalEntry> {
    let debits = checked_sum(
        [payout, fee, refund, reserve, tax],
        "marketplace reconciliation",
    )?;
    if debits != gross {
        return Err(ValidationError::AccountingInvariantFailed);
    }
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.cash.clone(), payout),
        debit(accounts.marketplace_fees.clone(), fee),
        debit(accounts.operating.refunds.clone(), refund),
        debit(accounts.chargeback_reserve.clone(), reserve),
        debit(accounts.tax_liability.clone(), tax),
        credit(accounts.marketplace_clearing.clone(), gross),
    ])
}

pub fn chargeback_reserve_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.chargeback_expense.clone(), amount),
        credit(accounts.chargeback_reserve.clone(), amount),
    ])
}

pub fn chargeback_settlement_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.chargeback_reserve.clone(), amount),
        credit(accounts.operating.cash.clone(), amount),
    ])
}

pub fn unrealized_fx_gain_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.accounts_receivable.clone(), amount),
        credit(accounts.unrealized_fx_gain.clone(), amount),
    ])
}

pub fn unrealized_fx_loss_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.unrealized_fx_loss.clone(), amount),
        credit(accounts.accounts_receivable.clone(), amount),
    ])
}

pub fn realized_fx_gain_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.operating.cash.clone(), amount),
        credit(accounts.realized_fx_gain.clone(), amount),
    ])
}

pub fn realized_fx_loss_journal(
    accounts: &AdvancedAccountingAccounts,
    amount: Money,
) -> DomainResult<BalancedJournalEntry> {
    BalancedJournalEntry::try_new(vec![
        debit(accounts.realized_fx_loss.clone(), amount),
        credit(accounts.operating.cash.clone(), amount),
    ])
}
