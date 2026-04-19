with open('crates/logos-store-pg/src/store.rs', 'r') as f:
    content = f.read()

structs = """
struct ImportBatchPayload<'a> {
    batch_id: &'a str,
    import_kind: &'a str,
    source_uri: &'a str,
    batch_key: &'a str,
    record_count: i64,
    duplicate_count: i64,
    dry_run: bool,
    ocr_enabled: bool,
    imported_at_us: i64,
}

impl<'a> From<&ImportBatchPayload<'a>> for NewImportBatchRow<'a> {
    fn from(payload: &ImportBatchPayload<'a>) -> Self {
        Self {
            batch_id: payload.batch_id,
            import_kind: payload.import_kind,
            source_uri: payload.source_uri,
            batch_key: payload.batch_key,
            record_count: payload.record_count,
            duplicate_count: payload.duplicate_count,
            dry_run: payload.dry_run,
            ocr_enabled: payload.ocr_enabled,
            imported_at_us: payload.imported_at_us,
        }
    }
}

struct FetchRunPayload<'a> {
    run_id: &'a str,
    source_id: &'a str,
    institution_id: &'a str,
    ledger_account: &'a str,
    month_key: &'a str,
    status: &'a str,
    artifact_path: Option<&'a str>,
    output_format: Option<&'a str>,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    error_summary: Option<&'a str>,
    created_at_us: i64,
}

impl<'a> From<&FetchRunPayload<'a>> for NewFetchRunRow<'a> {
    fn from(payload: &FetchRunPayload<'a>) -> Self {
        Self {
            run_id: payload.run_id,
            source_id: payload.source_id,
            institution_id: payload.institution_id,
            ledger_account: payload.ledger_account,
            month_key: payload.month_key,
            status: payload.status,
            artifact_path: payload.artifact_path,
            output_format: payload.output_format,
            opening_balance_cents: payload.opening_balance_cents,
            closing_balance_cents: payload.closing_balance_cents,
            error_summary: payload.error_summary,
            created_at_us: payload.created_at_us,
        }
    }
}

struct ReconciliationRunPayload<'a> {
    run_id: &'a str,
    month_key: &'a str,
    checking_account: &'a str,
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
    created_at_us: i64,
}

impl<'a> From<&ReconciliationRunPayload<'a>> for NewReconciliationRunRow<'a> {
    fn from(payload: &ReconciliationRunPayload<'a>) -> Self {
        Self {
            run_id: payload.run_id,
            month_key: payload.month_key,
            checking_account: payload.checking_account,
            opening_balance_cents: payload.opening_balance_cents,
            ledger_delta_cents: payload.ledger_delta_cents,
            expected_closing_balance_cents: payload.expected_closing_balance_cents,
            statement_closing_balance_cents: payload.statement_closing_balance_cents,
            variance_cents: payload.variance_cents,
            reconciled: payload.reconciled,
            matched_postings: payload.matched_postings,
            matched_transaction_count: payload.matched_transaction_count,
            inflow_cents: payload.inflow_cents,
            outflow_cents: payload.outflow_cents,
            created_at_us: payload.created_at_us,
        }
    }
}

struct MonthClosePayload<'a> {
    close_id: &'a str,
    month_key: &'a str,
    checking_account: &'a str,
    reconciliation_run_id: &'a str,
    analytics_artifact_id: Option<&'a str>,
    closed_at_us: i64,
}

impl<'a> From<&MonthClosePayload<'a>> for NewMonthCloseRow<'a> {
    fn from(payload: &MonthClosePayload<'a>) -> Self {
        Self {
            close_id: payload.close_id,
            month_key: payload.month_key,
            checking_account: payload.checking_account,
            reconciliation_run_id: payload.reconciliation_run_id,
            analytics_artifact_id: payload.analytics_artifact_id,
            closed_at_us: payload.closed_at_us,
        }
    }
}
"""

import re
# Insert the structs before the impl LedgerStore block
content = re.sub(r'(impl LedgerStore for PostgresStore \{)', structs + r'\n\1', content, 1)

import_batch_old = """        let batch_row = NewImportBatchRow {
            batch_id: &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at_us,
        };"""
import_batch_new = """        let batch_payload = ImportBatchPayload {
            batch_id: &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at_us,
        };
        let batch_row = NewImportBatchRow::from(&batch_payload);"""
content = content.replace(import_batch_old, import_batch_new)

fetch_run_old = """        let row = NewFetchRunRow {
            run_id: &run_id,
            source_id,
            institution_id,
            ledger_account,
            month_key,
            status: status.as_str(),
            artifact_path,
            output_format: output_format.map(StoredFetchArtifactFormat::as_str),
            opening_balance_cents,
            closing_balance_cents,
            error_summary: normalized_error_summary,
            created_at_us,
        };"""
fetch_run_new = """        let status_str = status.as_str();
        let output_format_str = output_format.map(StoredFetchArtifactFormat::as_str);
        let payload = FetchRunPayload {
            run_id: &run_id,
            source_id,
            institution_id,
            ledger_account,
            month_key,
            status: status_str,
            artifact_path,
            output_format: output_format_str,
            opening_balance_cents,
            closing_balance_cents,
            error_summary: normalized_error_summary,
            created_at_us,
        };
        let row = NewFetchRunRow::from(&payload);"""
content = content.replace(fetch_run_old, fetch_run_new)


recon_run_old = """        let run_row = NewReconciliationRunRow {
            run_id: &run_id,
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
            created_at_us,
        };"""
recon_run_new = """        let run_payload = ReconciliationRunPayload {
            run_id: &run_id,
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
            created_at_us,
        };
        let run_row = NewReconciliationRunRow::from(&run_payload);"""
content = content.replace(recon_run_old, recon_run_new)

month_close_old = """        let close_row = NewMonthCloseRow {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id: &run_id,
            analytics_artifact_id,
            closed_at_us,
        };"""
month_close_new = """        let close_payload = MonthClosePayload {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id: &run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        let close_row = NewMonthCloseRow::from(&close_payload);"""
content = content.replace(month_close_old, month_close_new)

month_close_old2 = """        let row = NewMonthCloseRow {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at_us,
        };"""
month_close_new2 = """        let payload = MonthClosePayload {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        let row = NewMonthCloseRow::from(&payload);"""
content = content.replace(month_close_old2, month_close_new2)

content = content.replace('#[allow(clippy::too_many_lines)]\n    fn write_import_batch', 'fn write_import_batch')
content = content.replace('#[allow(clippy::too_many_lines)]\n    fn write_reconciliation_run_and_month_close', 'fn write_reconciliation_run_and_month_close')


with open('crates/logos-store-pg/src/store.rs', 'w') as f:
    f.write(content)
