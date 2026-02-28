use std::collections::HashSet;

use aletheiadb::Timestamp;
use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::{
    AletheiaStore, StoreError,
    model::{
        NewImportRecord, StoredAnalyticsArtifactManifest, StoredBudgetTarget, StoredImportBatch,
        StoredImportRecord, StoredMonthClose, StoredReconciliationRun, StoredStatementLine,
    },
};

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

    /// Writes an immutable analytics artifact manifest with optional lineage.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails or `supersedes_artifact_id` is unknown.
    #[allow(clippy::too_many_arguments)]
    pub fn write_analytics_artifact_manifest(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at: Timestamp,
        snapshot_tx_at: Timestamp,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        if let Some(supersedes) = supersedes_artifact_id {
            if !self.analytics_artifacts.contains_key(supersedes) {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: supersedes.to_owned(),
                });
            }
        }

        let artifact_id = self.next_artifact_id();
        let created_at = aletheiadb::time::now();
        let snapshot_key = format!(
            "valid:{}|tx:{}",
            snapshot_valid_at.wallclock(),
            snapshot_tx_at.wallclock()
        );
        let manifest = StoredAnalyticsArtifactManifest::new(
            &artifact_id,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at,
            snapshot_tx_at,
            created_at,
            supersedes_artifact_id,
            &snapshot_key,
        );

        self.persist_analytics_artifact_graph(&manifest)?;
        self.persist_analytics_artifact(manifest.clone());
        Ok(manifest)
    }

    /// Writes an immutable analytics artifact manifest using microsecond timestamps.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails or lineage targets are invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn write_analytics_artifact_manifest_us(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at_us: i64,
        snapshot_tx_at_us: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        self.write_analytics_artifact_manifest(
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us.into(),
            snapshot_tx_at_us.into(),
            supersedes_artifact_id,
        )
    }

    /// Writes an import batch and all newly accepted import records.
    ///
    /// # Errors
    ///
    /// Returns an error when required metadata is missing or record keys already exist.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub fn write_import_batch(
        &mut self,
        import_kind: &str,
        source_uri: &str,
        batch_key: &str,
        duplicate_count: i64,
        dry_run: bool,
        ocr_enabled: bool,
        records: &[NewImportRecord],
    ) -> Result<StoredImportBatch, StoreError> {
        if import_kind.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "import_kind must not be empty".to_owned(),
            });
        }
        if source_uri.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "source_uri must not be empty".to_owned(),
            });
        }
        if batch_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "batch_key must not be empty".to_owned(),
            });
        }
        if duplicate_count < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("duplicate_count must be non-negative, got {duplicate_count}"),
            });
        }

        let mut seen_input_keys = HashSet::new();
        for record in records {
            let key = record.content_hash_key();
            if key.is_empty() {
                return Err(StoreError::PersistFailed {
                    message: "import record content_hash_key must not be empty".to_owned(),
                });
            }
            if !seen_input_keys.insert(key.to_owned()) {
                return Err(StoreError::PersistFailed {
                    message: format!(
                        "duplicate import record content_hash_key '{key}' in write payload"
                    ),
                });
            }
            if self.import_records.contains_key(key) {
                return Err(StoreError::PersistFailed {
                    message: format!("import record content_hash_key '{key}' already exists"),
                });
            }
        }

        let batch_id = self.next_import_batch_id();
        let imported_at = aletheiadb::time::now();
        let record_count = i64::try_from(records.len()).unwrap_or(i64::MAX);
        let batch = StoredImportBatch::new(
            &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at,
        );

        let statement_lines = records
            .iter()
            .map(|record| {
                record.statement_line().map(|line| {
                    StoredStatementLine::new(
                        &self.next_statement_line_id(),
                        batch.batch_id(),
                        line.source_uri(),
                        line.statement_timestamp(),
                        line.memo(),
                        line.amount_cents(),
                        record.imported_txn_id().cloned(),
                        imported_at,
                    )
                })
            })
            .collect::<Vec<_>>();

        for line in statement_lines.iter().flatten() {
            if line.source_uri().is_empty() {
                return Err(StoreError::PersistFailed {
                    message: "statement line source_uri must not be empty".to_owned(),
                });
            }
            if line.statement_timestamp().is_empty() {
                return Err(StoreError::PersistFailed {
                    message: "statement line timestamp must not be empty".to_owned(),
                });
            }
            if line.memo().is_empty() {
                return Err(StoreError::PersistFailed {
                    message: "statement line memo must not be empty".to_owned(),
                });
            }
        }

        self.persist_import_batch_graph(&batch, records, &statement_lines)?;
        self.persist_import_batch(batch.clone());

        for record in records {
            let stored = StoredImportRecord::new(
                record.content_hash_key(),
                batch.batch_id(),
                record.imported_txn_id().cloned(),
                imported_at,
            );
            self.persist_import_record(stored);
        }
        for line in statement_lines.into_iter().flatten() {
            self.persist_statement_line(line);
        }

        Ok(batch)
    }

    /// Writes an immutable reconciliation run with linked transaction evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata is invalid, linked transactions are unknown, or persistence fails.
    #[allow(clippy::too_many_arguments)]
    pub fn write_reconciliation_run(
        &mut self,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        reconciled_txn_ids: &[TransactionId],
    ) -> Result<StoredReconciliationRun, StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if matched_postings < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("matched_postings must be non-negative, got {matched_postings}"),
            });
        }
        if inflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("inflow_cents must be non-negative, got {inflow_cents}"),
            });
        }
        if outflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("outflow_cents must be non-negative, got {outflow_cents}"),
            });
        }
        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);

        let run_id = self.next_reconciliation_run_id();
        let created_at = aletheiadb::time::now();
        let run = StoredReconciliationRun::new(
            &run_id,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            matched_transaction_count,
            inflow_cents,
            outflow_cents,
            created_at,
        );
        let statement_line_ids = self.persist_reconciliation_run_graph(&run, reconciled_txn_ids)?;
        self.persist_reconciliation_run(run.clone());
        self.persist_reconciliation_statement_line_ids(run.run_id(), statement_line_ids);
        Ok(run)
    }

    /// Atomically writes reconciliation and month close evidence in one append-only unit.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata is invalid, references are unknown, scope is already closed,
    /// or a single embedded transaction cannot be committed.
    #[allow(clippy::too_many_arguments)]
    pub fn write_reconciliation_run_and_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        reconciled_txn_ids: &[TransactionId],
        analytics_artifact_id: Option<&str>,
    ) -> Result<(StoredReconciliationRun, StoredMonthClose), StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if matched_postings < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("matched_postings must be non-negative, got {matched_postings}"),
            });
        }
        if inflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("inflow_cents must be non-negative, got {inflow_cents}"),
            });
        }
        if outflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("outflow_cents must be non-negative, got {outflow_cents}"),
            });
        }
        if let Some(artifact_id) = analytics_artifact_id {
            if !self.analytics_artifacts.contains_key(artifact_id) {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                });
            }
        }
        let scope_key = (month_key.to_owned(), checking_account.to_owned());
        if let Some(existing_close_id) = self.month_close_by_scope.get(&scope_key) {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month '{month_key}' for account '{checking_account}' is already closed by '{existing_close_id}'"
                ),
            });
        }

        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);
        let run_id = self.next_reconciliation_run_id();
        let created_at = aletheiadb::time::now();
        let run = StoredReconciliationRun::new(
            &run_id,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            matched_transaction_count,
            inflow_cents,
            outflow_cents,
            created_at,
        );

        let close_id = self.next_month_close_id();
        let closed_at = aletheiadb::time::now();
        let close = StoredMonthClose::new(
            &close_id,
            month_key,
            checking_account,
            run.run_id(),
            analytics_artifact_id,
            closed_at,
        );

        let statement_line_ids = self.persist_reconciliation_run_and_month_close_graph(
            &run,
            &close,
            reconciled_txn_ids,
        )?;
        self.persist_reconciliation_run(run.clone());
        self.persist_reconciliation_statement_line_ids(run.run_id(), statement_line_ids);
        self.persist_month_close(close.clone());
        Ok((run, close))
    }

    /// Writes an immutable month-close record that links to reconciliation and optional analytics evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata is invalid, references are unknown, or the scope is already closed.
    pub fn write_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if reconciliation_run_id.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "reconciliation_run_id must not be empty".to_owned(),
            });
        }

        let run = self
            .reconciliation_runs
            .get(reconciliation_run_id)
            .ok_or_else(|| StoreError::PersistFailed {
                message: format!("unknown reconciliation run '{reconciliation_run_id}'"),
            })?;
        if run.month_key() != month_key {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month close month '{month_key}' does not match reconciliation run month '{}'",
                    run.month_key()
                ),
            });
        }
        if run.checking_account() != checking_account {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month close checking_account '{checking_account}' does not match reconciliation run account '{}'",
                    run.checking_account()
                ),
            });
        }

        if let Some(artifact_id) = analytics_artifact_id {
            if !self.analytics_artifacts.contains_key(artifact_id) {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                });
            }
        }

        let scope_key = (month_key.to_owned(), checking_account.to_owned());
        if let Some(existing_close_id) = self.month_close_by_scope.get(&scope_key) {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month '{month_key}' for account '{checking_account}' is already closed by '{existing_close_id}'"
                ),
            });
        }

        let close_id = self.next_month_close_id();
        let closed_at = aletheiadb::time::now();
        let close = StoredMonthClose::new(
            &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at,
        );
        self.persist_month_close_graph(&close)?;
        self.persist_month_close(close.clone());
        Ok(close)
    }
}
