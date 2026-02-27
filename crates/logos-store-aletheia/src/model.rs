use aletheiadb::{Timestamp, time};
use logos_core::{Correction, Transaction, TransactionId};

pub(crate) const LABEL_LEDGER_TRANSACTION: &str = "LedgerTransaction";
pub(crate) const LABEL_LEDGER_POSTING: &str = "LedgerPosting";
pub(crate) const LABEL_LEDGER_CORRECTION: &str = "LedgerCorrection";
pub(crate) const LABEL_LEDGER_BUDGET_TARGET: &str = "LedgerBudgetTarget";
pub(crate) const LABEL_ANALYTICS_ARTIFACT_MANIFEST: &str = "AnalyticsArtifactManifest";
pub(crate) const LABEL_LEDGER_IMPORT_BATCH: &str = "LedgerImportBatch";
pub(crate) const LABEL_LEDGER_IMPORT_RECORD: &str = "LedgerImportRecord";
pub(crate) const LABEL_LEDGER_RECONCILIATION_RUN: &str = "LedgerReconciliationRun";

pub(crate) const EDGE_HAS_POSTING: &str = "HAS_POSTING";
pub(crate) const EDGE_SUPERSEDES: &str = "SUPERSEDES";
pub(crate) const EDGE_DERIVED_FROM: &str = "DERIVED_FROM";
pub(crate) const EDGE_HAS_IMPORT_RECORD: &str = "HAS_IMPORT_RECORD";
pub(crate) const EDGE_RECONCILES_TXN: &str = "RECONCILES_TXN";

pub(crate) const PROP_TXN_ID: &str = "txn_id";
pub(crate) const PROP_DESCRIPTION: &str = "description";
pub(crate) const PROP_ACCOUNT: &str = "account";
pub(crate) const PROP_AMOUNT_CENTS: &str = "amount_cents";
pub(crate) const PROP_ORDINAL: &str = "ordinal";
pub(crate) const PROP_SUPERSEDES_TXN_ID: &str = "supersedes_txn_id";
pub(crate) const PROP_REASON: &str = "reason";
pub(crate) const PROP_EFFECTIVE_AT_US: &str = "effective_at_us";
pub(crate) const PROP_MONTH_KEY: &str = "month_key";
pub(crate) const PROP_EXPENSE_ACCOUNT_PREFIX: &str = "expense_account_prefix";
pub(crate) const PROP_BUDGET_CENTS: &str = "budget_cents";
pub(crate) const PROP_ARTIFACT_ID: &str = "artifact_id";
pub(crate) const PROP_ARTIFACT_KIND: &str = "artifact_kind";
pub(crate) const PROP_ARTIFACT_URI: &str = "artifact_uri";
pub(crate) const PROP_CONTENT_HASH: &str = "content_hash";
pub(crate) const PROP_SCHEMA_VERSION: &str = "schema_version";
pub(crate) const PROP_ROW_COUNT: &str = "row_count";
pub(crate) const PROP_SNAPSHOT_VALID_AT_US: &str = "snapshot_valid_at_us";
pub(crate) const PROP_SNAPSHOT_TX_AT_US: &str = "snapshot_tx_at_us";
pub(crate) const PROP_CREATED_AT_US: &str = "created_at_us";
pub(crate) const PROP_SUPERSEDES_ARTIFACT_ID: &str = "supersedes_artifact_id";
pub(crate) const PROP_SNAPSHOT_KEY: &str = "snapshot_key";
pub(crate) const PROP_IMPORT_BATCH_ID: &str = "batch_id";
pub(crate) const PROP_IMPORT_KIND: &str = "import_kind";
pub(crate) const PROP_IMPORT_SOURCE_URI: &str = "source_uri";
pub(crate) const PROP_IMPORT_BATCH_KEY: &str = "batch_key";
pub(crate) const PROP_IMPORT_RECORD_COUNT: &str = "record_count";
pub(crate) const PROP_IMPORT_DUPLICATE_COUNT: &str = "duplicate_count";
pub(crate) const PROP_IMPORT_DRY_RUN: &str = "dry_run";
pub(crate) const PROP_IMPORT_OCR_ENABLED: &str = "ocr_enabled";
pub(crate) const PROP_IMPORT_CONTENT_HASH_KEY: &str = "content_hash_key";
pub(crate) const PROP_IMPORT_IMPORTED_TXN_ID: &str = "imported_txn_id";
pub(crate) const PROP_IMPORT_IMPORTED_AT_US: &str = "imported_at_us";
pub(crate) const PROP_RECONCILIATION_RUN_ID: &str = "reconciliation_run_id";
pub(crate) const PROP_RECONCILIATION_CHECKING_ACCOUNT: &str = "reconciliation_checking_account";
pub(crate) const PROP_RECONCILIATION_OPENING_BALANCE_CENTS: &str =
    "reconciliation_opening_balance_cents";
pub(crate) const PROP_RECONCILIATION_LEDGER_DELTA_CENTS: &str = "reconciliation_ledger_delta_cents";
pub(crate) const PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS: &str =
    "reconciliation_expected_closing_balance_cents";
pub(crate) const PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS: &str =
    "reconciliation_statement_closing_balance_cents";
pub(crate) const PROP_RECONCILIATION_VARIANCE_CENTS: &str = "reconciliation_variance_cents";
pub(crate) const PROP_RECONCILIATION_RECONCILED: &str = "reconciliation_reconciled";
pub(crate) const PROP_RECONCILIATION_MATCHED_POSTINGS: &str = "reconciliation_matched_postings";
pub(crate) const PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT: &str =
    "reconciliation_matched_transaction_count";
pub(crate) const PROP_RECONCILIATION_INFLOW_CENTS: &str = "reconciliation_inflow_cents";
pub(crate) const PROP_RECONCILIATION_OUTFLOW_CENTS: &str = "reconciliation_outflow_cents";
pub(crate) const PROP_RECONCILIATION_CREATED_AT_US: &str = "reconciliation_created_at_us";

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
    effective_at: Timestamp,
}

impl StoredTransaction {
    #[must_use]
    pub fn new(id: TransactionId, transaction: Transaction) -> Self {
        Self::with_effective_at(id, transaction, time::now())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImportRecord {
    content_hash_key: String,
    batch_id: String,
    imported_txn_id: Option<TransactionId>,
    imported_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImportRecord {
    content_hash_key: String,
    imported_txn_id: Option<TransactionId>,
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

impl NewImportRecord {
    #[must_use]
    pub fn new(content_hash_key: &str, imported_txn_id: Option<&TransactionId>) -> Self {
        Self {
            content_hash_key: content_hash_key.to_owned(),
            imported_txn_id: imported_txn_id.cloned(),
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
}

impl StoredReconciliationRun {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
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
