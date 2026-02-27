use aletheiadb::{Timestamp, time};
use logos_core::{Correction, Transaction, TransactionId};

pub(crate) const LABEL_LEDGER_TRANSACTION: &str = "LedgerTransaction";
pub(crate) const LABEL_LEDGER_POSTING: &str = "LedgerPosting";
pub(crate) const LABEL_LEDGER_CORRECTION: &str = "LedgerCorrection";
pub(crate) const LABEL_LEDGER_BUDGET_TARGET: &str = "LedgerBudgetTarget";

pub(crate) const EDGE_HAS_POSTING: &str = "HAS_POSTING";
pub(crate) const EDGE_SUPERSEDES: &str = "SUPERSEDES";

pub(crate) const PROP_TXN_ID: &str = "txn_id";
pub(crate) const PROP_DESCRIPTION: &str = "description";
pub(crate) const PROP_ACCOUNT: &str = "account";
pub(crate) const PROP_AMOUNT_CENTS: &str = "amount_cents";
pub(crate) const PROP_ORDINAL: &str = "ordinal";
pub(crate) const PROP_SUPERSEDES_TXN_ID: &str = "supersedes_txn_id";
pub(crate) const PROP_REASON: &str = "reason";
pub(crate) const PROP_EFFECTIVE_AT_US: &str = "effective_at_us";
pub(crate) const PROP_MONTH_KEY: &str = "month_key";
pub(crate) const PROP_EXPENSE_ACCOUNT_PREFIX: &str = "expense_account_prefix";
pub(crate) const PROP_BUDGET_CENTS: &str = "budget_cents";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsOf {
    valid_time: Timestamp,
    tx_time: Timestamp,
}

impl AsOf {
    #[must_use]
    pub const fn new(valid_time: Timestamp, tx_time: Timestamp) -> Self {
        Self {
            valid_time,
            tx_time,
        }
    }

    #[must_use]
    pub const fn valid_time(&self) -> Timestamp {
        self.valid_time
    }

    #[must_use]
    pub const fn tx_time(&self) -> Timestamp {
        self.tx_time
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredTransaction {
    id: TransactionId,
    transaction: Transaction,
    effective_at: Timestamp,
}

impl StoredTransaction {
    #[must_use]
    pub fn new(id: TransactionId, transaction: Transaction) -> Self {
        Self::with_effective_at(id, transaction, time::now())
    }

    #[must_use]
    pub const fn with_effective_at(
        id: TransactionId,
        transaction: Transaction,
        effective_at: Timestamp,
    ) -> Self {
        Self {
            id,
            transaction,
            effective_at,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &TransactionId {
        &self.id
    }

    #[must_use]
    pub const fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    #[must_use]
    pub const fn effective_at(&self) -> Timestamp {
        self.effective_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCorrection {
    correction: Correction,
}

impl StoredCorrection {
    #[must_use]
    pub const fn new(correction: Correction) -> Self {
        Self { correction }
    }

    #[must_use]
    pub const fn correction(&self) -> &Correction {
        &self.correction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBudgetTarget {
    month_key: String,
    expense_account_prefix: String,
    budget_cents: i64,
}

impl StoredBudgetTarget {
    #[must_use]
    pub fn new(month_key: &str, expense_account_prefix: &str, budget_cents: i64) -> Self {
        Self {
            month_key: month_key.to_owned(),
            expense_account_prefix: expense_account_prefix.to_owned(),
            budget_cents,
        }
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub fn expense_account_prefix(&self) -> &str {
        &self.expense_account_prefix
    }

    #[must_use]
    pub const fn budget_cents(&self) -> i64 {
        self.budget_cents
    }
}
