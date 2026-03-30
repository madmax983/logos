use logos_store::model::StoredFetchRun;
use logos_store::model::{StoredMonthClose, StoredReconciliationRun};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthReport {
    checking_balance: i64,
    income: i64,
    expense: i64,
    cashflow: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthReconciliation {
    ledger_delta_cents: i64,
    expected_closing_balance_cents: i64,
    statement_closing_balance_cents: i64,
    variance_cents: i64,
    reconciled: bool,
    matched_postings: usize,
    inflow_cents: i64,
    outflow_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportSummary {
    imported_count: usize,
    duplicate_count: usize,
    dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthAutopilotRequest {
    month_key: String,
    checking_account: String,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    statement_pdf_path: Option<PathBuf>,
    enable_ocr: bool,
    allow_variance: bool,
    analytics_artifact_id: Option<String>,
    confirm_close: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthAutopilotSummary {
    month_key: String,
    checking_account: String,
    imported_count: usize,
    duplicate_count: usize,
    fetch_runs: Vec<StoredFetchRun>,
    reconciliation_run: StoredReconciliationRun,
    report: MonthReport,
    close: StoredMonthClose,
}

pub type PdfImportSummary = ImportSummary;
pub type CsvImportSummary = ImportSummary;

impl ImportSummary {
    #[must_use]
    pub const fn new(imported_count: usize, duplicate_count: usize, dry_run: bool) -> Self {
        Self {
            imported_count,
            duplicate_count,
            dry_run,
        }
    }

    #[must_use]
    pub const fn imported_count(&self) -> usize {
        self.imported_count
    }

    #[must_use]
    pub const fn duplicate_count(&self) -> usize {
        self.duplicate_count
    }

    #[must_use]
    pub const fn dry_run(&self) -> bool {
        self.dry_run
    }
}

impl MonthAutopilotRequest {
    #[must_use]
    pub fn new(month_key: &str, checking_account: &str) -> Self {
        Self {
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
            opening_balance_cents: None,
            closing_balance_cents: None,
            statement_pdf_path: None,
            enable_ocr: false,
            allow_variance: false,
            analytics_artifact_id: None,
            confirm_close: false,
        }
    }

    #[must_use]
    pub fn with_statement_pdf(mut self, path: impl AsRef<Path>) -> Self {
        self.statement_pdf_path = Some(path.as_ref().to_path_buf());
        self
    }

    #[must_use]
    pub const fn with_balances(
        mut self,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> Self {
        self.opening_balance_cents = Some(opening_balance_cents);
        self.closing_balance_cents = Some(closing_balance_cents);
        self
    }

    #[must_use]
    pub const fn with_ocr(mut self, enable_ocr: bool) -> Self {
        self.enable_ocr = enable_ocr;
        self
    }

    #[must_use]
    pub const fn with_allow_variance(mut self, allow_variance: bool) -> Self {
        self.allow_variance = allow_variance;
        self
    }

    #[must_use]
    pub fn with_analytics_artifact_id(mut self, artifact_id: &str) -> Self {
        self.analytics_artifact_id = Some(artifact_id.to_owned());
        self
    }

    #[must_use]
    pub const fn with_confirm_close(mut self, confirm_close: bool) -> Self {
        self.confirm_close = confirm_close;
        self
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
    pub const fn opening_balance_cents(&self) -> Option<i64> {
        self.opening_balance_cents
    }

    #[must_use]
    pub const fn closing_balance_cents(&self) -> Option<i64> {
        self.closing_balance_cents
    }

    #[must_use]
    pub const fn statement_pdf_path(&self) -> Option<&PathBuf> {
        self.statement_pdf_path.as_ref()
    }

    #[must_use]
    pub const fn enable_ocr(&self) -> bool {
        self.enable_ocr
    }

    #[must_use]
    pub const fn allow_variance(&self) -> bool {
        self.allow_variance
    }

    #[must_use]
    pub fn analytics_artifact_id(&self) -> Option<&str> {
        self.analytics_artifact_id.as_deref()
    }

    #[must_use]
    pub const fn confirm_close(&self) -> bool {
        self.confirm_close
    }
}

impl MonthAutopilotSummary {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        month_key: &str,
        checking_account: &str,
        imported_count: usize,
        duplicate_count: usize,
        fetch_runs: Vec<StoredFetchRun>,
        reconciliation_run: StoredReconciliationRun,
        report: MonthReport,
        close: StoredMonthClose,
    ) -> Self {
        Self {
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
            imported_count,
            duplicate_count,
            fetch_runs,
            reconciliation_run,
            report,
            close,
        }
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
    pub const fn imported_count(&self) -> usize {
        self.imported_count
    }

    #[must_use]
    pub const fn duplicate_count(&self) -> usize {
        self.duplicate_count
    }

    #[must_use]
    pub fn fetch_runs(&self) -> &[StoredFetchRun] {
        &self.fetch_runs
    }

    #[must_use]
    pub const fn reconciliation_run(&self) -> &StoredReconciliationRun {
        &self.reconciliation_run
    }

    #[must_use]
    pub const fn report(&self) -> &MonthReport {
        &self.report
    }

    #[must_use]
    pub const fn close(&self) -> &StoredMonthClose {
        &self.close
    }
}

impl MonthReport {
    #[must_use]
    pub const fn new(
        checking_balance_cents: i64,
        income_cents: i64,
        expense_cents: i64,
        cashflow_cents: i64,
    ) -> Self {
        Self {
            checking_balance: checking_balance_cents,
            income: income_cents,
            expense: expense_cents,
            cashflow: cashflow_cents,
        }
    }

    #[must_use]
    pub const fn checking_balance_cents(&self) -> i64 {
        self.checking_balance
    }

    #[must_use]
    pub const fn income_cents(&self) -> i64 {
        self.income
    }

    #[must_use]
    pub const fn expense_cents(&self) -> i64 {
        self.expense
    }

    #[must_use]
    pub const fn cashflow_cents(&self) -> i64 {
        self.cashflow
    }
}

impl MonthReconciliation {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: usize,
        inflow_cents: i64,
        outflow_cents: i64,
    ) -> Self {
        Self {
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            inflow_cents,
            outflow_cents,
        }
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
    pub const fn is_reconciled(&self) -> bool {
        self.reconciled
    }

    #[must_use]
    pub const fn matched_postings(&self) -> usize {
        self.matched_postings
    }

    #[must_use]
    pub const fn inflow_cents(&self) -> i64 {
        self.inflow_cents
    }

    #[must_use]
    pub const fn outflow_cents(&self) -> i64 {
        self.outflow_cents
    }
}
