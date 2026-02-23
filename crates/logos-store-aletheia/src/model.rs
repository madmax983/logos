use logos_core::{Correction, Transaction, TransactionId};

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
