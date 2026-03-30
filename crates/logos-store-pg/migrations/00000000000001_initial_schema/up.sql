CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    effective_at_us BIGINT NOT NULL,
    recorded_at_us BIGINT NOT NULL,
    source_kind TEXT,
    external_ref TEXT
);

CREATE TABLE postings (
    transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL,
    account TEXT NOT NULL,
    amount_cents BIGINT NOT NULL,
    PRIMARY KEY (transaction_id, ordinal)
);

CREATE TABLE corrections (
    correction_id BIGSERIAL PRIMARY KEY,
    supersedes_txn_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE RESTRICT,
    reason TEXT NOT NULL,
    recorded_at_us BIGINT NOT NULL
);

CREATE TABLE budget_targets (
    month_key TEXT NOT NULL,
    expense_account_prefix TEXT NOT NULL,
    budget_cents BIGINT NOT NULL,
    PRIMARY KEY (month_key, expense_account_prefix)
);

CREATE TABLE analytics_artifact_manifests (
    artifact_id TEXT PRIMARY KEY,
    artifact_kind TEXT NOT NULL,
    artifact_uri TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    schema_version BIGINT NOT NULL,
    row_count BIGINT NOT NULL,
    snapshot_valid_at_us BIGINT NOT NULL,
    snapshot_tx_at_us BIGINT NOT NULL,
    created_at_us BIGINT NOT NULL,
    supersedes_artifact_id TEXT REFERENCES analytics_artifact_manifests(artifact_id) ON DELETE RESTRICT,
    snapshot_key TEXT NOT NULL UNIQUE
);

CREATE TABLE import_batches (
    batch_id TEXT PRIMARY KEY,
    import_kind TEXT NOT NULL,
    source_uri TEXT NOT NULL,
    batch_key TEXT NOT NULL UNIQUE,
    record_count BIGINT NOT NULL,
    duplicate_count BIGINT NOT NULL,
    dry_run BOOLEAN NOT NULL,
    ocr_enabled BOOLEAN NOT NULL,
    imported_at_us BIGINT NOT NULL
);

CREATE TABLE import_records (
    content_hash_key TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL REFERENCES import_batches(batch_id) ON DELETE RESTRICT,
    imported_txn_id TEXT REFERENCES transactions(id) ON DELETE RESTRICT,
    imported_at_us BIGINT NOT NULL
);

CREATE TABLE statement_lines (
    line_id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL REFERENCES import_batches(batch_id) ON DELETE RESTRICT,
    source_uri TEXT NOT NULL,
    statement_timestamp TEXT NOT NULL,
    memo TEXT NOT NULL,
    amount_cents BIGINT NOT NULL,
    imported_txn_id TEXT REFERENCES transactions(id) ON DELETE RESTRICT,
    imported_at_us BIGINT NOT NULL
);

CREATE TABLE fetch_runs (
    run_id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    institution_id TEXT NOT NULL,
    ledger_account TEXT NOT NULL,
    month_key TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('downloaded', 'imported', 'no_new_statement', 'needs_attention', 'failed')),
    artifact_path TEXT,
    output_format TEXT CHECK (output_format IN ('csv', 'pdf')),
    opening_balance_cents BIGINT,
    closing_balance_cents BIGINT,
    error_summary TEXT,
    created_at_us BIGINT NOT NULL
);

CREATE TABLE reconciliation_runs (
    run_id TEXT PRIMARY KEY,
    month_key TEXT NOT NULL,
    checking_account TEXT NOT NULL,
    opening_balance_cents BIGINT NOT NULL,
    ledger_delta_cents BIGINT NOT NULL,
    expected_closing_balance_cents BIGINT NOT NULL,
    statement_closing_balance_cents BIGINT NOT NULL,
    variance_cents BIGINT NOT NULL,
    reconciled BOOLEAN NOT NULL,
    matched_postings BIGINT NOT NULL,
    matched_transaction_count BIGINT NOT NULL,
    inflow_cents BIGINT NOT NULL,
    outflow_cents BIGINT NOT NULL,
    created_at_us BIGINT NOT NULL
);

CREATE TABLE reconciliation_run_transactions (
    run_id TEXT NOT NULL REFERENCES reconciliation_runs(run_id) ON DELETE CASCADE,
    transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE RESTRICT,
    PRIMARY KEY (run_id, transaction_id)
);

CREATE TABLE reconciliation_run_statement_lines (
    run_id TEXT NOT NULL REFERENCES reconciliation_runs(run_id) ON DELETE CASCADE,
    statement_line_id TEXT NOT NULL REFERENCES statement_lines(line_id) ON DELETE RESTRICT,
    PRIMARY KEY (run_id, statement_line_id)
);

CREATE TABLE month_closes (
    close_id TEXT PRIMARY KEY,
    month_key TEXT NOT NULL,
    checking_account TEXT NOT NULL,
    reconciliation_run_id TEXT NOT NULL REFERENCES reconciliation_runs(run_id) ON DELETE RESTRICT,
    analytics_artifact_id TEXT REFERENCES analytics_artifact_manifests(artifact_id) ON DELETE RESTRICT,
    closed_at_us BIGINT NOT NULL,
    UNIQUE (month_key, checking_account)
);

CREATE INDEX idx_transactions_effective_recorded
    ON transactions (effective_at_us, recorded_at_us);

CREATE INDEX idx_postings_account_transaction
    ON postings (account, transaction_id);

CREATE INDEX idx_corrections_supersedes
    ON corrections (supersedes_txn_id);

CREATE INDEX idx_import_records_batch
    ON import_records (batch_id);

CREATE INDEX idx_statement_lines_batch
    ON statement_lines (batch_id);

CREATE INDEX idx_fetch_runs_scope
    ON fetch_runs (month_key, ledger_account, created_at_us);

CREATE INDEX idx_reconciliation_runs_scope
    ON reconciliation_runs (month_key, checking_account, created_at_us);

CREATE INDEX idx_month_closes_scope
    ON month_closes (month_key, checking_account);
