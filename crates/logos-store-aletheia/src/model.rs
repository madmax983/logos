use aletheiadb::Timestamp;
use logos_core::{Correction, Transaction, TransactionId};

pub(crate) const LABEL_LEDGER_TRANSACTION: &str = "LedgerTransaction";
pub(crate) const LABEL_LEDGER_POSTING: &str = "LedgerPosting";
pub(crate) const LABEL_LEDGER_CORRECTION: &str = "LedgerCorrection";

pub(crate) const EDGE_HAS_POSTING: &str = "HAS_POSTING";
pub(crate) const EDGE_SUPERSEDES: &str = "SUPERSEDES";

pub(crate) const PROP_TXN_ID: &str = "txn_id";
pub(crate) const PROP_DESCRIPTION: &str = "description";
pub(crate) const PROP_ACCOUNT: &str = "account";
pub(crate) const PROP_AMOUNT_CENTS: &str = "amount_cents";
pub(crate) const PROP_ORDINAL: &str = "ordinal";
pub(crate) const PROP_SUPERSEDES_TXN_ID: &str = "supersedes_txn_id";
pub(crate) const PROP_REASON: &str = "reason";

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
}

impl StoredTransaction {
    #[must_use]
    pub const fn new(id: TransactionId, transaction: Transaction) -> Self {
        Self { id, transaction }
    }

    #[must_use]
    pub const fn id(&self) -> &TransactionId {
        &self.id
    }

    #[must_use]
    pub const fn transaction(&self) -> &Transaction {
        &self.transaction
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
