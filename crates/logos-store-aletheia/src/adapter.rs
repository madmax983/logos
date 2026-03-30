use logos_core::{Correction, TransactionBuilder, TransactionId};
use logos_store::{error as neutral_error, model as neutral_model, traits::LedgerStore};

use crate::model as aletheia_model;
use crate::{AletheiaStore, StoreError};

impl From<StoreError> for neutral_error::StoreError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::Domain(err) => Self::Domain(err),
            StoreError::UnknownTransaction { transaction_id } => {
                Self::UnknownTransaction { transaction_id }
            }
            StoreError::UnknownArtifact { artifact_id } => Self::UnknownArtifact { artifact_id },
            StoreError::LoadFailed { message } => Self::LoadFailed { message },
            StoreError::PersistFailed { message } => Self::PersistFailed { message },
        }
    }
}

impl From<aletheia_model::StoredTransaction> for neutral_model::StoredTransaction {
    fn from(value: aletheia_model::StoredTransaction) -> Self {
        Self::with_effective_at(
            value.id().clone(),
            value.transaction().clone(),
            value.effective_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredBudgetTarget> for neutral_model::StoredBudgetTarget {
    fn from(value: aletheia_model::StoredBudgetTarget) -> Self {
        Self::new(
            value.month_key(),
            value.expense_account_prefix(),
            value.budget_cents(),
        )
    }
}

impl From<aletheia_model::StoredAnalyticsArtifactManifest>
    for neutral_model::StoredAnalyticsArtifactManifest
{
    fn from(value: aletheia_model::StoredAnalyticsArtifactManifest) -> Self {
        Self::new(
            value.artifact_id(),
            value.artifact_kind(),
            value.artifact_uri(),
            value.content_hash(),
            value.schema_version(),
            value.row_count(),
            value.snapshot_valid_at().wallclock(),
            value.snapshot_tx_at().wallclock(),
            value.created_at().wallclock(),
            value.supersedes_artifact_id(),
            value.snapshot_key(),
        )
    }
}

impl From<aletheia_model::StoredImportBatch> for neutral_model::StoredImportBatch {
    fn from(value: aletheia_model::StoredImportBatch) -> Self {
        Self::new(
            value.batch_id(),
            value.import_kind(),
            value.source_uri(),
            value.batch_key(),
            value.record_count(),
            value.duplicate_count(),
            value.dry_run(),
            value.ocr_enabled(),
            value.imported_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredImportRecord> for neutral_model::StoredImportRecord {
    fn from(value: aletheia_model::StoredImportRecord) -> Self {
        Self::new(
            value.content_hash_key(),
            value.batch_id(),
            value.imported_txn_id().cloned(),
            value.imported_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredStatementLine> for neutral_model::StoredStatementLine {
    fn from(value: aletheia_model::StoredStatementLine) -> Self {
        Self::new(
            value.line_id(),
            value.batch_id(),
            value.source_uri(),
            value.statement_timestamp(),
            value.memo(),
            value.amount_cents(),
            value.imported_txn_id().cloned(),
            value.imported_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredReconciliationRun> for neutral_model::StoredReconciliationRun {
    fn from(value: aletheia_model::StoredReconciliationRun) -> Self {
        Self::new(
            value.run_id(),
            value.month_key(),
            value.checking_account(),
            value.opening_balance_cents(),
            value.ledger_delta_cents(),
            value.expected_closing_balance_cents(),
            value.statement_closing_balance_cents(),
            value.variance_cents(),
            value.reconciled(),
            value.matched_postings(),
            value.matched_transaction_count(),
            value.inflow_cents(),
            value.outflow_cents(),
            value.created_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredMonthClose> for neutral_model::StoredMonthClose {
    fn from(value: aletheia_model::StoredMonthClose) -> Self {
        Self::new(
            value.close_id(),
            value.month_key(),
            value.checking_account(),
            value.reconciliation_run_id(),
            value.analytics_artifact_id(),
            value.closed_at().wallclock(),
        )
    }
}

impl From<aletheia_model::StoredFetchRunStatus> for neutral_model::StoredFetchRunStatus {
    fn from(value: aletheia_model::StoredFetchRunStatus) -> Self {
        match value {
            aletheia_model::StoredFetchRunStatus::Downloaded => Self::Downloaded,
            aletheia_model::StoredFetchRunStatus::Imported => Self::Imported,
            aletheia_model::StoredFetchRunStatus::NoNewStatement => Self::NoNewStatement,
            aletheia_model::StoredFetchRunStatus::NeedsAttention => Self::NeedsAttention,
            aletheia_model::StoredFetchRunStatus::Failed => Self::Failed,
        }
    }
}

impl From<neutral_model::StoredFetchRunStatus> for aletheia_model::StoredFetchRunStatus {
    fn from(value: neutral_model::StoredFetchRunStatus) -> Self {
        match value {
            neutral_model::StoredFetchRunStatus::Downloaded => Self::Downloaded,
            neutral_model::StoredFetchRunStatus::Imported => Self::Imported,
            neutral_model::StoredFetchRunStatus::NoNewStatement => Self::NoNewStatement,
            neutral_model::StoredFetchRunStatus::NeedsAttention => Self::NeedsAttention,
            neutral_model::StoredFetchRunStatus::Failed => Self::Failed,
        }
    }
}

impl From<aletheia_model::StoredFetchArtifactFormat> for neutral_model::StoredFetchArtifactFormat {
    fn from(value: aletheia_model::StoredFetchArtifactFormat) -> Self {
        match value {
            aletheia_model::StoredFetchArtifactFormat::Csv => Self::Csv,
            aletheia_model::StoredFetchArtifactFormat::Pdf => Self::Pdf,
        }
    }
}

impl From<neutral_model::StoredFetchArtifactFormat> for aletheia_model::StoredFetchArtifactFormat {
    fn from(value: neutral_model::StoredFetchArtifactFormat) -> Self {
        match value {
            neutral_model::StoredFetchArtifactFormat::Csv => Self::Csv,
            neutral_model::StoredFetchArtifactFormat::Pdf => Self::Pdf,
        }
    }
}

impl From<aletheia_model::StoredFetchRun> for neutral_model::StoredFetchRun {
    fn from(value: aletheia_model::StoredFetchRun) -> Self {
        Self::new(
            value.run_id(),
            value.source_id(),
            value.institution_id(),
            value.ledger_account(),
            value.month_key(),
            value.status().into(),
            value.artifact_path(),
            value.output_format().map(Into::into),
            value.opening_balance_cents(),
            value.closing_balance_cents(),
            value.error_summary(),
            value.created_at().wallclock(),
        )
    }
}

impl From<&neutral_model::NewImportRecord> for aletheia_model::NewImportRecord {
    fn from(value: &neutral_model::NewImportRecord) -> Self {
        match value.statement_line() {
            Some(line) => Self::with_statement_line(
                value.content_hash_key(),
                value.imported_txn_id(),
                line.source_uri(),
                line.statement_timestamp(),
                line.memo(),
                line.amount_cents(),
            ),
            None => Self::new(value.content_hash_key(), value.imported_txn_id()),
        }
    }
}

impl LedgerStore for AletheiaStore {
    fn transaction_count(&self) -> usize {
        AletheiaStore::transaction_count(self)
    }

    fn correction_count(&self) -> usize {
        AletheiaStore::correction_count(self)
    }

    fn has_transaction(&self, id: &TransactionId) -> bool {
        AletheiaStore::has_transaction(self, id)
    }

    fn latest_correction(&self) -> Option<Correction> {
        AletheiaStore::latest_correction(self).cloned()
    }

    fn transactions(&self) -> Vec<neutral_model::StoredTransaction> {
        AletheiaStore::transactions(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn budget_target(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<neutral_model::StoredBudgetTarget> {
        AletheiaStore::budget_target(self, month_key, expense_account_prefix)
            .cloned()
            .map(Into::into)
    }

    fn budget_targets(&self) -> Vec<neutral_model::StoredBudgetTarget> {
        AletheiaStore::budget_targets(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn analytics_artifact(
        &self,
        artifact_id: &str,
    ) -> Option<neutral_model::StoredAnalyticsArtifactManifest> {
        AletheiaStore::analytics_artifact(self, artifact_id)
            .cloned()
            .map(Into::into)
    }

    fn analytics_artifacts(&self) -> Vec<neutral_model::StoredAnalyticsArtifactManifest> {
        AletheiaStore::analytics_artifacts(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn import_record_count(&self) -> usize {
        AletheiaStore::import_record_count(self)
    }

    fn has_import_record_content_hash(&self, content_hash_key: &str) -> bool {
        AletheiaStore::has_import_record_content_hash(self, content_hash_key)
    }

    fn import_records(&self) -> Vec<neutral_model::StoredImportRecord> {
        AletheiaStore::import_records(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn import_batches(&self) -> Vec<neutral_model::StoredImportBatch> {
        AletheiaStore::import_batches(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn statement_line_count(&self) -> usize {
        AletheiaStore::statement_line_count(self)
    }

    fn statement_lines(&self) -> Vec<neutral_model::StoredStatementLine> {
        AletheiaStore::statement_lines(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn fetch_run_count(&self) -> usize {
        AletheiaStore::fetch_run_count(self)
    }

    fn fetch_run(&self, run_id: &str) -> Option<neutral_model::StoredFetchRun> {
        AletheiaStore::fetch_run(self, run_id)
            .cloned()
            .map(Into::into)
    }

    fn fetch_runs(&self) -> Vec<neutral_model::StoredFetchRun> {
        AletheiaStore::fetch_runs(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn statement_lines_for_reconciliation_run(
        &self,
        run_id: &str,
    ) -> Vec<neutral_model::StoredStatementLine> {
        AletheiaStore::statement_lines_for_reconciliation_run(self, run_id)
            .into_iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn reconciliation_run_count(&self) -> usize {
        AletheiaStore::reconciliation_run_count(self)
    }

    fn reconciliation_run(&self, run_id: &str) -> Option<neutral_model::StoredReconciliationRun> {
        AletheiaStore::reconciliation_run(self, run_id)
            .cloned()
            .map(Into::into)
    }

    fn reconciliation_runs(&self) -> Vec<neutral_model::StoredReconciliationRun> {
        AletheiaStore::reconciliation_runs(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn month_close_count(&self) -> usize {
        AletheiaStore::month_close_count(self)
    }

    fn month_close(&self, close_id: &str) -> Option<neutral_model::StoredMonthClose> {
        AletheiaStore::month_close(self, close_id)
            .cloned()
            .map(Into::into)
    }

    fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<neutral_model::StoredMonthClose> {
        AletheiaStore::month_close_for_scope(self, month_key, checking_account)
            .cloned()
            .map(Into::into)
    }

    fn month_closes(&self) -> Vec<neutral_model::StoredMonthClose> {
        AletheiaStore::month_closes(self)
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn transactions_as_of_us(
        &self,
        valid_time_us: i64,
        tx_time_us: i64,
    ) -> Result<Vec<neutral_model::StoredTransaction>, neutral_error::StoreError> {
        AletheiaStore::transactions_as_of_us(self, valid_time_us, tx_time_us)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn write_transaction(
        &mut self,
        builder: TransactionBuilder,
    ) -> Result<TransactionId, neutral_error::StoreError> {
        AletheiaStore::write_transaction(self, builder).map_err(Into::into)
    }

    fn write_transaction_with_valid_time(
        &mut self,
        builder: TransactionBuilder,
        valid_from: Option<i64>,
    ) -> Result<TransactionId, neutral_error::StoreError> {
        AletheiaStore::write_transaction_with_valid_time(self, builder, valid_from.map(Into::into))
            .map_err(Into::into)
    }

    fn write_correction(
        &mut self,
        correction: Correction,
    ) -> Result<(), neutral_error::StoreError> {
        AletheiaStore::write_correction(self, correction).map_err(Into::into)
    }

    fn write_budget_target(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), neutral_error::StoreError> {
        AletheiaStore::write_budget_target(self, month_key, expense_account_prefix, budget_cents)
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_analytics_artifact_manifest(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at_us: i64,
        snapshot_tx_at_us: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<neutral_model::StoredAnalyticsArtifactManifest, neutral_error::StoreError> {
        AletheiaStore::write_analytics_artifact_manifest_us(
            self,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            supersedes_artifact_id,
        )
        .map(Into::into)
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_analytics_artifact_manifest_us(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at_us: i64,
        snapshot_tx_at_us: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<neutral_model::StoredAnalyticsArtifactManifest, neutral_error::StoreError> {
        AletheiaStore::write_analytics_artifact_manifest_us(
            self,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            supersedes_artifact_id,
        )
        .map(Into::into)
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_import_batch(
        &mut self,
        import_kind: &str,
        source_uri: &str,
        batch_key: &str,
        duplicate_count: i64,
        dry_run: bool,
        ocr_enabled: bool,
        records: &[neutral_model::NewImportRecord],
    ) -> Result<neutral_model::StoredImportBatch, neutral_error::StoreError> {
        let converted_records: Vec<aletheia_model::NewImportRecord> =
            records.iter().map(Into::into).collect();
        AletheiaStore::write_import_batch(
            self,
            import_kind,
            source_uri,
            batch_key,
            duplicate_count,
            dry_run,
            ocr_enabled,
            &converted_records,
        )
        .map(Into::into)
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_fetch_run(
        &mut self,
        source_id: &str,
        institution_id: &str,
        ledger_account: &str,
        month_key: &str,
        status: neutral_model::StoredFetchRunStatus,
        artifact_path: Option<&str>,
        output_format: Option<neutral_model::StoredFetchArtifactFormat>,
        opening_balance_cents: Option<i64>,
        closing_balance_cents: Option<i64>,
        error_summary: Option<&str>,
    ) -> Result<neutral_model::StoredFetchRun, neutral_error::StoreError> {
        AletheiaStore::write_fetch_run(
            self,
            source_id,
            institution_id,
            ledger_account,
            month_key,
            status.into(),
            artifact_path,
            output_format.map(Into::into),
            opening_balance_cents,
            closing_balance_cents,
            error_summary,
        )
        .map(Into::into)
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_reconciliation_run(
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
    ) -> Result<neutral_model::StoredReconciliationRun, neutral_error::StoreError> {
        AletheiaStore::write_reconciliation_run(
            self,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            inflow_cents,
            outflow_cents,
            reconciled_txn_ids,
        )
        .map(Into::into)
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_reconciliation_run_and_month_close(
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
    ) -> Result<
        (
            neutral_model::StoredReconciliationRun,
            neutral_model::StoredMonthClose,
        ),
        neutral_error::StoreError,
    > {
        AletheiaStore::write_reconciliation_run_and_month_close(
            self,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            inflow_cents,
            outflow_cents,
            reconciled_txn_ids,
            analytics_artifact_id,
        )
        .map(|(run, close)| (run.into(), close.into()))
        .map_err(Into::into)
    }

    fn write_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<neutral_model::StoredMonthClose, neutral_error::StoreError> {
        AletheiaStore::write_month_close(
            self,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
        )
        .map(Into::into)
        .map_err(Into::into)
    }
}
