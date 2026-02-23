use logos_core::{Correction, TransactionId};

use crate::{
    AletheiaStore,
    model::{StoredCorrection, StoredTransaction},
};

impl AletheiaStore {
    #[must_use]
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    #[must_use]
    pub fn correction_count(&self) -> usize {
        self.corrections.len()
    }

    #[must_use]
    pub fn has_transaction(&self, id: &TransactionId) -> bool {
        self.transactions.contains_key(id)
    }

    #[must_use]
    pub fn latest_correction(&self) -> Option<&Correction> {
        self.corrections.last().map(StoredCorrection::correction)
    }

    pub fn transactions(&self) -> impl Iterator<Item = &StoredTransaction> + '_ {
        self.transactions.values()
    }
}
