use std::collections::HashMap;
use std::fmt;

use logos_core::{Correction, DomainError, TransactionBuilder, TransactionId};

use crate::model::{StoredCorrection, StoredTransaction};

pub mod model;
pub mod read;
pub mod write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    Domain(DomainError),
    UnknownTransaction { transaction_id: TransactionId },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(err) => write!(f, "{err}"),
            Self::UnknownTransaction { transaction_id } => {
                write!(
                    f,
                    "cannot apply correction: unknown transaction '{}'",
                    transaction_id.as_str()
                )
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl From<DomainError> for StoreError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

#[derive(Debug, Default)]
pub struct AletheiaStore {
    pub(crate) next_id: u64,
    pub(crate) transactions: HashMap<TransactionId, StoredTransaction>,
    pub(crate) corrections: Vec<StoredCorrection>,
}

impl AletheiaStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn next_transaction_id(&mut self) -> TransactionId {
        self.next_id = self.next_id.saturating_add(1);
        TransactionId::new(&format!("txn-{}", self.next_id))
    }

    pub(crate) fn push_correction(&mut self, correction: Correction) {
        self.corrections.push(StoredCorrection::new(correction));
    }

    pub(crate) fn persist_transaction(
        &mut self,
        id: TransactionId,
        txn: logos_core::Transaction,
    ) {
        self.transactions
            .insert(id.clone(), StoredTransaction::new(id, txn));
    }

    pub(crate) fn build_and_validate(
        builder: TransactionBuilder,
    ) -> Result<logos_core::Transaction, StoreError> {
        Ok(builder.build()?)
    }
}
