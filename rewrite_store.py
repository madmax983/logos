import re

def rewrite():
    with open('crates/logos-store-pg/src/store.rs', 'r') as f:
        code = f.read()

    # Define the structs
    struct_code = '''
pub(crate) struct WriteReconciliationRunPayload<'a> {
    pub run_id: &'a str,
    pub month_key: &'a str,
    pub checking_account: &'a str,
    pub opening_balance_cents: i64,
    pub ledger_delta_cents: i64,
    pub expected_closing_balance_cents: i64,
    pub statement_closing_balance_cents: i64,
    pub variance_cents: i64,
    pub reconciled: bool,
    pub matched_postings: i64,
    pub matched_transaction_count: i64,
    pub inflow_cents: i64,
    pub outflow_cents: i64,
    pub created_at_us: i64,
}

impl<'a> From<&WriteReconciliationRunPayload<'a>> for NewReconciliationRunRow<'a> {
    fn from(payload: &WriteReconciliationRunPayload<'a>) -> Self {
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

pub(crate) struct WriteMonthClosePayload<'a> {
    pub close_id: &'a str,
    pub month_key: &'a str,
    pub checking_account: &'a str,
    pub reconciliation_run_id: &'a str,
    pub analytics_artifact_id: Option<&'a str>,
    pub closed_at_us: i64,
}

impl<'a> From<&WriteMonthClosePayload<'a>> for NewMonthCloseRow<'a> {
    fn from(payload: &WriteMonthClosePayload<'a>) -> Self {
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

pub(crate) struct WriteImportBatchPayload<'a> {
    pub batch_id: &'a str,
    pub import_kind: &'a str,
    pub source_uri: &'a str,
    pub batch_key: &'a str,
    pub record_count: i64,
    pub duplicate_count: i64,
    pub dry_run: bool,
    pub ocr_enabled: bool,
    pub imported_at_us: i64,
}

impl<'a> From<&WriteImportBatchPayload<'a>> for NewImportBatchRow<'a> {
    fn from(payload: &WriteImportBatchPayload<'a>) -> Self {
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
'''
    code = code.replace('pub struct PostgresStore {', struct_code + '\npub struct PostgresStore {')

    # Replace run_row in write_reconciliation_run
    code = re.sub(
        r'let run_row = NewReconciliationRunRow \{\s*run_id: &run_id,\s*month_key,\s*checking_account,\s*opening_balance_cents,\s*ledger_delta_cents,\s*expected_closing_balance_cents,\s*statement_closing_balance_cents,\s*variance_cents,\s*reconciled,\s*matched_postings,\s*matched_transaction_count,\s*inflow_cents,\s*outflow_cents,\s*created_at_us,\s*\};',
        '''let payload = WriteReconciliationRunPayload {
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
        let run_row = NewReconciliationRunRow::from(&payload);''',
        code
    )

    # Replace run_row and close_row in write_reconciliation_run_and_month_close
    code = re.sub(
        r'let run_row = NewReconciliationRunRow \{\s*run_id: &run_id,\s*month_key,\s*checking_account,\s*opening_balance_cents,\s*ledger_delta_cents,\s*expected_closing_balance_cents,\s*statement_closing_balance_cents,\s*variance_cents,\s*reconciled,\s*matched_postings,\s*matched_transaction_count,\s*inflow_cents,\s*outflow_cents,\s*created_at_us,\s*\};',
        '''let payload = WriteReconciliationRunPayload {
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
        let run_row = NewReconciliationRunRow::from(&payload);''',
        code
    )

    code = re.sub(
        r'let close_row = NewMonthCloseRow \{\s*close_id: &close_id,\s*month_key,\s*checking_account,\s*reconciliation_run_id: &run_id,\s*analytics_artifact_id,\s*closed_at_us,\s*\};',
        '''let close_payload = WriteMonthClosePayload {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id: &run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        let close_row = NewMonthCloseRow::from(&close_payload);''',
        code
    )

    # Replace row in write_month_close
    code = re.sub(
        r'let row = NewMonthCloseRow \{\s*close_id: &close_id,\s*month_key,\s*checking_account,\s*reconciliation_run_id,\s*analytics_artifact_id,\s*closed_at_us,\s*\};',
        '''let payload = WriteMonthClosePayload {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        let row = NewMonthCloseRow::from(&payload);''',
        code
    )

    # Replace batch_row in write_import_batch
    code = re.sub(
        r'let batch_row = NewImportBatchRow \{\s*batch_id: &batch_id,\s*import_kind,\s*source_uri,\s*batch_key,\s*record_count,\s*duplicate_count,\s*dry_run,\s*ocr_enabled,\s*imported_at_us,\s*\};',
        '''let payload = WriteImportBatchPayload {
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
        let batch_row = NewImportBatchRow::from(&payload);''',
        code
    )

    # Remove #[allow(clippy::too_many_lines)]
    code = code.replace('#[allow(clippy::too_many_lines)]\n    fn write_reconciliation_run_and_month_close(', 'fn write_reconciliation_run_and_month_close(')
    code = code.replace('#[allow(clippy::too_many_lines)]\n    fn write_import_batch(', 'fn write_import_batch(')

    with open('crates/logos-store-pg/src/store.rs', 'w') as f:
        f.write(code)

rewrite()
