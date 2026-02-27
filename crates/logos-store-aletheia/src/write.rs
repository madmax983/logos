use aletheiadb::Timestamp;
use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::{AletheiaStore, StoreError, model::StoredBudgetTarget};

impl AletheiaStore {
    /// Validates and persists a transaction in append-only storage.
    ///
    /// # Errors
    ///
    /// Returns an error when domain validation fails.
    pub fn write_transaction(
        &mut self,
        builder: TransactionBuilder,
    ) -> Result<TransactionId, StoreError> {
        self.write_transaction_with_valid_time(builder, None)
    }

    /// Validates and persists a transaction with an optional explicit valid-time start.
    ///
    /// # Errors
    ///
    /// Returns an error when domain validation fails.
    pub fn write_transaction_with_valid_time(
        &mut self,
        builder: TransactionBuilder,
        valid_from: Option<Timestamp>,
    ) -> Result<TransactionId, StoreError> {
        let txn = Self::build_and_validate(builder)?;
        let id = self.next_transaction_id();
        let effective_at = valid_from.unwrap_or_else(aletheiadb::time::now);
        self.persist_transaction_graph(&id, &txn, effective_at)?;
        self.persist_transaction(id.clone(), txn, effective_at);
        Ok(id)
    }

    /// Appends a correction edge to an existing transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when the superseded transaction id does not exist.
    pub fn write_correction(&mut self, correction: Correction) -> Result<(), StoreError> {
        if !self.transactions.contains_key(correction.supersedes_id()) {
            return Err(StoreError::UnknownTransaction {
                transaction_id: correction.supersedes_id().clone(),
            });
        }

        self.persist_correction_graph(&correction)?;
        self.push_correction(correction);
        Ok(())
    }

    /// Appends or replaces the effective budget target for a `(month_key, expense_prefix)` pair.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails.
    pub fn write_budget_target(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), StoreError> {
        let target = StoredBudgetTarget::new(month_key, expense_account_prefix, budget_cents);
        self.persist_budget_target_graph(&target)?;
        self.persist_budget_target(target);
        Ok(())
    }
}
