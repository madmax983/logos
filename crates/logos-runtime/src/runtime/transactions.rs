use crate::error::RuntimeError;
use crate::runtime::AppRuntime;
use logos_core::{Correction, TransactionId};
use logos_store_aletheia::model::StoredTransaction;
use logos_store_aletheia::StoreError;

impl AppRuntime {
    /// Posts a balanced double-entry transaction and persists it.
    ///
    /// # Errors
    ///
    /// Returns an error when transaction validation or persistence fails.
    pub fn post_double_entry(
        &mut self,
        description: &str,
        debit_account: &str,
        credit_account: &str,
        amount_cents: i64,
    ) -> Result<TransactionId, RuntimeError> {
        let builder = crate::runtime::build_double_entry(description, debit_account, credit_account, amount_cents)?;
        Ok(self.store.write_transaction(builder)?)
    }

    #[must_use]
    pub fn transaction_exists(&self, id: &TransactionId) -> bool {
        self.store.has_transaction(id)
    }

    /// Returns transactions visible at a bi-temporal as-of point (valid-time, tx-time).
    ///
    /// # Errors
    ///
    /// Returns an error when underlying as-of query execution fails.
    pub fn transactions_as_of_us(
        &self,
        as_of_valid_time_us: i64,
        as_of_tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, RuntimeError> {
        self.store
            .transactions_as_of_us(as_of_valid_time_us, as_of_tx_time_us)
            .map_err(RuntimeError::from)
    }

    /// Applies an append-only correction to a previously written transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when correction creation or persistence fails.
    pub fn apply_correction(
        &mut self,
        supersedes_id: TransactionId,
        reason: &str,
    ) -> Result<(), RuntimeError> {
        let correction = Correction::new(supersedes_id, reason).map_err(StoreError::from)?;
        self.store.write_correction(correction)?;
        Ok(())
    }

    #[must_use]
    pub fn latest_correction_target(&self) -> Option<TransactionId> {
        self.store
            .latest_correction()
            .map(|correction| correction.supersedes_id().clone())
    }

}
