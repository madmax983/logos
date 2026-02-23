use std::collections::HashSet;
use std::fmt;

use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_import::{CsvMapping, ImportError, deterministic_fingerprint, parse_simple_csv_row};
use logos_reporting::{RegisterEntry, project_register_balance};
use logos_store_aletheia::{AletheiaStore, StoreError};

#[derive(Debug)]
pub enum RuntimeError {
    Store(StoreError),
    Import(ImportError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(err) => write!(f, "{err}"),
            Self::Import(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<StoreError> for RuntimeError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ImportError> for RuntimeError {
    fn from(value: ImportError) -> Self {
        Self::Import(value)
    }
}

#[derive(Debug, Default)]
pub struct CliRuntime {
    store: AletheiaStore,
    seen_fingerprints: HashSet<u64>,
    imported_records: usize,
}

impl CliRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
        let builder = TransactionBuilder::new(description)
            .posting(Posting::debit(debit_account, amount_cents))
            .posting(Posting::credit(credit_account, amount_cents));
        Ok(self.store.write_transaction(builder)?)
    }

    #[must_use]
    pub fn transaction_exists(&self, id: &TransactionId) -> bool {
        self.store.has_transaction(id)
    }

    #[must_use]
    pub fn register_balance_for(&self, account: &str) -> i64 {
        let entries: Vec<RegisterEntry> = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account() == account)
            .map(|posting| RegisterEntry::new(posting.amount()))
            .collect();

        project_register_balance(0, &entries)
    }

    /// Imports one CSV row with deterministic idempotency.
    ///
    /// Returns `true` when newly inserted, `false` when duplicate.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails.
    pub fn import_csv_row(
        &mut self,
        row: &str,
        mapping: &CsvMapping,
    ) -> Result<bool, RuntimeError> {
        let record = parse_simple_csv_row(row, mapping)?;
        let fingerprint = deterministic_fingerprint(&record);
        if !self.seen_fingerprints.insert(fingerprint) {
            return Ok(false);
        }

        self.imported_records = self.imported_records.saturating_add(1);
        Ok(true)
    }

    #[must_use]
    pub const fn imported_record_count(&self) -> usize {
        self.imported_records
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
