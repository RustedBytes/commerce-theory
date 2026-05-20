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

pub fn debit(account: LedgerAccount, amount: Money) -> Posting {
    Posting::new(account, PostingSide::Debit, amount)
}

pub fn credit(account: LedgerAccount, amount: Money) -> Posting {
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
