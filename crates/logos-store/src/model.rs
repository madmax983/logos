//! Storage Models
//!
//! Defines the "plain old data" structs that represent how domain concepts are serialized and retrieved from the persistence layer.

use logos_core::{Correction, Transaction, TransactionId};

pub type Timestamp = i64;
pub type BudgetTargetKey = (String, String);

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a transaction as it exists in the persistence layer, including its internal [`TransactionId`] and `effective_at` timestamp.
///
/// Unlike the core [`Transaction`] domain object, this struct explicitly pairs the business data with its storage metadata.
///
/// ## Examples
///
/// ```
/// use logos_core::{TransactionBuilder, TransactionId, AccountId, Posting};
/// use logos_store::StoredTransaction;
///
/// let txn = TransactionBuilder::new("groceries")
///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 100).unwrap())
///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 100).unwrap())
///     .build().unwrap();
/// let id = TransactionId::new("txn-1").unwrap();
/// let stored = StoredTransaction::with_effective_at(id.clone(), txn, 1672531200000000);
/// assert_eq!(stored.effective_at(), 1672531200000000);
/// ```
pub struct StoredTransaction {
    id: TransactionId,
    transaction: Transaction,
    effective_at: Timestamp,
}

impl StoredTransaction {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAnalyticsArtifactManifest {
    artifact_id: String,
    artifact_kind: String,
    artifact_uri: String,
    content_hash: String,
    schema_version: i64,
    row_count: i64,
    snapshot_valid_at: Timestamp,
    snapshot_tx_at: Timestamp,
    created_at: Timestamp,
    supersedes_artifact_id: Option<String>,
    snapshot_key: String,
}

impl StoredAnalyticsArtifactManifest {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        artifact_id: &str,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at: Timestamp,
        snapshot_tx_at: Timestamp,
        created_at: Timestamp,
        supersedes_artifact_id: Option<&str>,
        snapshot_key: &str,
    ) -> Self {
        Self {
            artifact_id: artifact_id.to_owned(),
            artifact_kind: artifact_kind.to_owned(),
            artifact_uri: artifact_uri.to_owned(),
            content_hash: content_hash.to_owned(),
            schema_version,
            row_count,
            snapshot_valid_at,
            snapshot_tx_at,
            created_at,
            supersedes_artifact_id: supersedes_artifact_id.map(str::to_owned),
            snapshot_key: snapshot_key.to_owned(),
        }
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn artifact_kind(&self) -> &str {
        &self.artifact_kind
    }

    #[must_use]
    pub fn artifact_uri(&self) -> &str {
        &self.artifact_uri
    }

    #[must_use]
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    #[must_use]
    pub const fn schema_version(&self) -> i64 {
        self.schema_version
    }

    #[must_use]
    pub const fn row_count(&self) -> i64 {
        self.row_count
    }

    #[must_use]
    pub const fn snapshot_valid_at(&self) -> Timestamp {
        self.snapshot_valid_at
    }

    #[must_use]
    pub const fn snapshot_tx_at(&self) -> Timestamp {
        self.snapshot_tx_at
    }

    #[must_use]
    pub const fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[must_use]
    pub fn supersedes_artifact_id(&self) -> Option<&str> {
        self.supersedes_artifact_id.as_deref()
    }

    #[must_use]
    pub fn snapshot_key(&self) -> &str {
        &self.snapshot_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImportBatch {
    batch_id: String,
    import_kind: String,
    source_uri: String,
    batch_key: String,
    record_count: i64,
    duplicate_count: i64,
    dry_run: bool,
    ocr_enabled: bool,
    imported_at: Timestamp,
}

impl StoredImportBatch {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        batch_id: &str,
        import_kind: &str,
        source_uri: &str,
        batch_key: &str,
        record_count: i64,
        duplicate_count: i64,
        dry_run: bool,
        ocr_enabled: bool,
        imported_at: Timestamp,
    ) -> Self {
        Self {
            batch_id: batch_id.to_owned(),
            import_kind: import_kind.to_owned(),
            source_uri: source_uri.to_owned(),
            batch_key: batch_key.to_owned(),
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at,
        }
    }

    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    #[must_use]
    pub fn import_kind(&self) -> &str {
        &self.import_kind
    }

    #[must_use]
    pub fn source_uri(&self) -> &str {
        &self.source_uri
    }

    #[must_use]
    pub fn batch_key(&self) -> &str {
        &self.batch_key
    }

    #[must_use]
    pub const fn record_count(&self) -> i64 {
        self.record_count
    }

    #[must_use]
    pub const fn duplicate_count(&self) -> i64 {
        self.duplicate_count
    }

    #[must_use]
    pub const fn dry_run(&self) -> bool {
        self.dry_run
    }

    #[must_use]
    pub const fn ocr_enabled(&self) -> bool {
        self.ocr_enabled
    }

    #[must_use]
    pub const fn imported_at(&self) -> Timestamp {
        self.imported_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImportRecord {
    content_hash_key: String,
    batch_id: String,
    imported_txn_id: Option<TransactionId>,
    imported_at: Timestamp,
}

impl StoredImportRecord {
    #[must_use]
    pub fn new(
        content_hash_key: &str,
        batch_id: &str,
        imported_txn_id: Option<TransactionId>,
        imported_at: Timestamp,
    ) -> Self {
        Self {
            content_hash_key: content_hash_key.to_owned(),
            batch_id: batch_id.to_owned(),
            imported_txn_id,
            imported_at,
        }
    }

    #[must_use]
    pub fn content_hash_key(&self) -> &str {
        &self.content_hash_key
    }

    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    #[must_use]
    pub const fn imported_txn_id(&self) -> Option<&TransactionId> {
        self.imported_txn_id.as_ref()
    }

    #[must_use]
    pub const fn imported_at(&self) -> Timestamp {
        self.imported_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewStatementLine {
    source_uri: String,
    statement_timestamp: String,
    memo: String,
    amount_cents: i64,
}

impl NewStatementLine {
    #[must_use]
    pub fn new(source_uri: &str, statement_timestamp: &str, memo: &str, amount_cents: i64) -> Self {
        Self {
            source_uri: source_uri.to_owned(),
            statement_timestamp: statement_timestamp.to_owned(),
            memo: memo.to_owned(),
            amount_cents,
        }
    }

    #[must_use]
    pub fn source_uri(&self) -> &str {
        &self.source_uri
    }

    #[must_use]
    pub fn statement_timestamp(&self) -> &str {
        &self.statement_timestamp
    }

    #[must_use]
    pub fn memo(&self) -> &str {
        &self.memo
    }

    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImportRecord {
    content_hash_key: String,
    imported_txn_id: Option<TransactionId>,
    statement_line: Option<NewStatementLine>,
}

impl NewImportRecord {
    #[must_use]
    pub fn new(content_hash_key: &str, imported_txn_id: Option<&TransactionId>) -> Self {
        Self {
            content_hash_key: content_hash_key.to_owned(),
            imported_txn_id: imported_txn_id.cloned(),
            statement_line: None,
        }
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn with_statement_line(
        content_hash_key: &str,
        imported_txn_id: Option<&TransactionId>,
        source_uri: &str,
        statement_timestamp: &str,
        memo: &str,
        amount_cents: i64,
    ) -> Self {
        Self {
            content_hash_key: content_hash_key.to_owned(),
            imported_txn_id: imported_txn_id.cloned(),
            statement_line: Some(NewStatementLine::new(
                source_uri,
                statement_timestamp,
                memo,
                amount_cents,
            )),
        }
    }

    #[must_use]
    pub fn content_hash_key(&self) -> &str {
        &self.content_hash_key
    }

    #[must_use]
    pub const fn imported_txn_id(&self) -> Option<&TransactionId> {
        self.imported_txn_id.as_ref()
    }

    #[must_use]
    pub const fn statement_line(&self) -> Option<&NewStatementLine> {
        self.statement_line.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredStatementLine {
    line_id: String,
    batch_id: String,
    source_uri: String,
    statement_timestamp: String,
    memo: String,
    amount_cents: i64,
    imported_txn_id: Option<TransactionId>,
    imported_at: Timestamp,
}

impl StoredStatementLine {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        line_id: &str,
        batch_id: &str,
        source_uri: &str,
        statement_timestamp: &str,
        memo: &str,
        amount_cents: i64,
        imported_txn_id: Option<TransactionId>,
        imported_at: Timestamp,
    ) -> Self {
        Self {
            line_id: line_id.to_owned(),
            batch_id: batch_id.to_owned(),
            source_uri: source_uri.to_owned(),
            statement_timestamp: statement_timestamp.to_owned(),
            memo: memo.to_owned(),
            amount_cents,
            imported_txn_id,
            imported_at,
        }
    }

    #[must_use]
    pub fn line_id(&self) -> &str {
        &self.line_id
    }

    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    #[must_use]
    pub fn source_uri(&self) -> &str {
        &self.source_uri
    }

    #[must_use]
    pub fn statement_timestamp(&self) -> &str {
        &self.statement_timestamp
    }

    #[must_use]
    pub fn memo(&self) -> &str {
        &self.memo
    }

    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }

    #[must_use]
    pub const fn imported_txn_id(&self) -> Option<&TransactionId> {
        self.imported_txn_id.as_ref()
    }

    #[must_use]
    pub const fn imported_at(&self) -> Timestamp {
        self.imported_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredReconciliationRun {
    run_id: String,
    month_key: String,
    checking_account: String,
    opening_balance_cents: i64,
    ledger_delta_cents: i64,
    expected_closing_balance_cents: i64,
    statement_closing_balance_cents: i64,
    variance_cents: i64,
    reconciled: bool,
    matched_postings: i64,
    matched_transaction_count: i64,
    inflow_cents: i64,
    outflow_cents: i64,
    created_at: Timestamp,
}

impl StoredReconciliationRun {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        run_id: &str,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        matched_transaction_count: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        created_at: Timestamp,
    ) -> Self {
        Self {
            run_id: run_id.to_owned(),
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
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
        }
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub fn checking_account(&self) -> &str {
        &self.checking_account
    }

    #[must_use]
    pub const fn opening_balance_cents(&self) -> i64 {
        self.opening_balance_cents
    }

    #[must_use]
    pub const fn ledger_delta_cents(&self) -> i64 {
        self.ledger_delta_cents
    }

    #[must_use]
    pub const fn expected_closing_balance_cents(&self) -> i64 {
        self.expected_closing_balance_cents
    }

    #[must_use]
    pub const fn statement_closing_balance_cents(&self) -> i64 {
        self.statement_closing_balance_cents
    }

    #[must_use]
    pub const fn variance_cents(&self) -> i64 {
        self.variance_cents
    }

    #[must_use]
    pub const fn reconciled(&self) -> bool {
        self.reconciled
    }

    #[must_use]
    pub const fn matched_postings(&self) -> i64 {
        self.matched_postings
    }

    #[must_use]
    pub const fn matched_transaction_count(&self) -> i64 {
        self.matched_transaction_count
    }

    #[must_use]
    pub const fn inflow_cents(&self) -> i64 {
        self.inflow_cents
    }

    #[must_use]
    pub const fn outflow_cents(&self) -> i64 {
        self.outflow_cents
    }

    #[must_use]
    pub const fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_fetch_run_status_behavior() {
        assert_eq!(StoredFetchRunStatus::Downloaded.as_str(), "downloaded");
        assert_eq!(StoredFetchRunStatus::Imported.as_str(), "imported");
        assert_eq!(
            StoredFetchRunStatus::NoNewStatement.as_str(),
            "no_new_statement"
        );
        assert_eq!(
            StoredFetchRunStatus::NeedsAttention.as_str(),
            "needs_attention"
        );
        assert_eq!(StoredFetchRunStatus::Failed.as_str(), "failed");

        assert_eq!(
            StoredFetchRunStatus::parse("downloaded"),
            Some(StoredFetchRunStatus::Downloaded)
        );
        assert_eq!(
            StoredFetchRunStatus::parse("imported"),
            Some(StoredFetchRunStatus::Imported)
        );
        assert_eq!(
            StoredFetchRunStatus::parse("no_new_statement"),
            Some(StoredFetchRunStatus::NoNewStatement)
        );
        assert_eq!(
            StoredFetchRunStatus::parse("needs_attention"),
            Some(StoredFetchRunStatus::NeedsAttention)
        );
        assert_eq!(
            StoredFetchRunStatus::parse("failed"),
            Some(StoredFetchRunStatus::Failed)
        );
        assert_eq!(StoredFetchRunStatus::parse("unknown"), None);

        assert!(StoredFetchRunStatus::Downloaded.is_success());
        assert!(StoredFetchRunStatus::Imported.is_success());
        assert!(StoredFetchRunStatus::NoNewStatement.is_success());
        assert!(!StoredFetchRunStatus::NeedsAttention.is_success());
        assert!(!StoredFetchRunStatus::Failed.is_success());
    }

    #[test]
    fn test_stored_fetch_artifact_format_behavior() {
        assert_eq!(StoredFetchArtifactFormat::Csv.as_str(), "csv");
        assert_eq!(StoredFetchArtifactFormat::Pdf.as_str(), "pdf");

        assert_eq!(
            StoredFetchArtifactFormat::parse("csv"),
            Some(StoredFetchArtifactFormat::Csv)
        );
        assert_eq!(
            StoredFetchArtifactFormat::parse("pdf"),
            Some(StoredFetchArtifactFormat::Pdf)
        );
        assert_eq!(StoredFetchArtifactFormat::parse("unknown"), None);
    }

    #[test]
    fn test_stored_fetch_run_getters() {
        let ts: Timestamp = 1_696_118_400;
        let run = StoredFetchRun::new(
            "run_123",
            "src_456",
            "inst_789",
            "acct_abc",
            "2023-10",
            StoredFetchRunStatus::Downloaded,
            Some("/path/to/artifact"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(1000),
            Some(2000),
            Some("error msg"),
            ts,
        );

        assert_eq!(run.run_id(), "run_123");
        assert_eq!(run.source_id(), "src_456");
        assert_eq!(run.institution_id(), "inst_789");
        assert_eq!(run.ledger_account(), "acct_abc");
        assert_eq!(run.month_key(), "2023-10");
        assert_eq!(run.status(), StoredFetchRunStatus::Downloaded);
        assert_eq!(run.artifact_path(), Some("/path/to/artifact"));
        assert_eq!(run.output_format(), Some(StoredFetchArtifactFormat::Csv));
        assert_eq!(run.opening_balance_cents(), Some(1000));
        assert_eq!(run.closing_balance_cents(), Some(2000));
        assert_eq!(run.error_summary(), Some("error msg"));
        assert_eq!(run.created_at(), ts);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMonthClose {
    close_id: String,
    month_key: String,
    checking_account: String,
    reconciliation_run_id: String,
    analytics_artifact_id: Option<String>,
    closed_at: Timestamp,
}

impl StoredMonthClose {
    #[must_use]
    pub fn new(
        close_id: &str,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
        closed_at: Timestamp,
    ) -> Self {
        Self {
            close_id: close_id.to_owned(),
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
            reconciliation_run_id: reconciliation_run_id.to_owned(),
            analytics_artifact_id: analytics_artifact_id.map(str::to_owned),
            closed_at,
        }
    }

    #[must_use]
    pub fn close_id(&self) -> &str {
        &self.close_id
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub fn checking_account(&self) -> &str {
        &self.checking_account
    }

    #[must_use]
    pub fn reconciliation_run_id(&self) -> &str {
        &self.reconciliation_run_id
    }

    #[must_use]
    pub fn analytics_artifact_id(&self) -> Option<&str> {
        self.analytics_artifact_id.as_deref()
    }

    #[must_use]
    pub const fn closed_at(&self) -> Timestamp {
        self.closed_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredFetchRunStatus {
    Downloaded,
    Imported,
    NoNewStatement,
    NeedsAttention,
    Failed,
}

impl StoredFetchRunStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Downloaded => "downloaded",
            Self::Imported => "imported",
            Self::NoNewStatement => "no_new_statement",
            Self::NeedsAttention => "needs_attention",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "downloaded" => Some(Self::Downloaded),
            "imported" => Some(Self::Imported),
            "no_new_statement" => Some(Self::NoNewStatement),
            "needs_attention" => Some(Self::NeedsAttention),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(
            self,
            Self::Downloaded | Self::Imported | Self::NoNewStatement
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredFetchArtifactFormat {
    Csv,
    Pdf,
}

impl StoredFetchArtifactFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Pdf => "pdf",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "csv" => Some(Self::Csv),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFetchRun {
    run_id: String,
    source_id: String,
    institution_id: String,
    ledger_account: String,
    month_key: String,
    status: StoredFetchRunStatus,
    artifact_path: Option<String>,
    output_format: Option<StoredFetchArtifactFormat>,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    error_summary: Option<String>,
    created_at: Timestamp,
}

impl StoredFetchRun {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        run_id: &str,
        source_id: &str,
        institution_id: &str,
        ledger_account: &str,
        month_key: &str,
        status: StoredFetchRunStatus,
        artifact_path: Option<&str>,
        output_format: Option<StoredFetchArtifactFormat>,
        opening_balance_cents: Option<i64>,
        closing_balance_cents: Option<i64>,
        error_summary: Option<&str>,
        created_at: Timestamp,
    ) -> Self {
        Self {
            run_id: run_id.to_owned(),
            source_id: source_id.to_owned(),
            institution_id: institution_id.to_owned(),
            ledger_account: ledger_account.to_owned(),
            month_key: month_key.to_owned(),
            status,
            artifact_path: artifact_path.map(str::to_owned),
            output_format,
            opening_balance_cents,
            closing_balance_cents,
            error_summary: error_summary.map(str::to_owned),
            created_at,
        }
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    #[must_use]
    pub fn institution_id(&self) -> &str {
        &self.institution_id
    }

    #[must_use]
    pub fn ledger_account(&self) -> &str {
        &self.ledger_account
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub const fn status(&self) -> StoredFetchRunStatus {
        self.status
    }

    #[must_use]
    pub fn artifact_path(&self) -> Option<&str> {
        self.artifact_path.as_deref()
    }

    #[must_use]
    pub const fn output_format(&self) -> Option<StoredFetchArtifactFormat> {
        self.output_format
    }

    #[must_use]
    pub const fn opening_balance_cents(&self) -> Option<i64> {
        self.opening_balance_cents
    }

    #[must_use]
    pub const fn closing_balance_cents(&self) -> Option<i64> {
        self.closing_balance_cents
    }

    #[must_use]
    pub fn error_summary(&self) -> Option<&str> {
        self.error_summary.as_deref()
    }

    #[must_use]
    pub const fn created_at(&self) -> Timestamp {
        self.created_at
    }
}
