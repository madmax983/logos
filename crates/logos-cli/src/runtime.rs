use std::collections::HashSet;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_import::{CsvMapping, ImportError, deterministic_fingerprint, parse_simple_csv_row};
use logos_reporting::{
    RegisterEntry, project_budget_variance, project_cashflow, project_register_balance,
};
use logos_store_aletheia::{AletheiaStore, StoreError, model::StoredTransaction};

const LOGOS_DB_PATH_ENV: &str = "LOGOS_DB_PATH";
const DEFAULT_DB_DIRECTORY: &str = ".logos";
const DEFAULT_DB_NAME: &str = "ledger";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthReport {
    checking_balance: i64,
    income: i64,
    expense: i64,
    cashflow: i64,
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

#[derive(Debug, Default)]
pub struct CliRuntime {
    store: AletheiaStore,
    seen_fingerprints: HashSet<u64>,
    imported_records: usize,
}

impl CliRuntime {
    /// Creates a runtime backed by the default durable store path.
    ///
    /// `LOGOS_DB_PATH` overrides the location. Otherwise, the path defaults to:
    /// - `${HOME}/.logos/ledger` on Unix-like systems
    /// - `%USERPROFILE%\\.logos\\ledger` on Windows
    /// - `./.logos/ledger` when no home directory is available
    ///
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::open(Self::default_store_path())
    }

    /// Creates a runtime pinned to a specific durable store path.
    ///
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        Ok(Self {
            store: AletheiaStore::open(path)?,
            seen_fingerprints: HashSet::new(),
            imported_records: 0,
        })
    }

    #[must_use]
    pub fn new_in_memory() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn default_store_path() -> PathBuf {
        if let Some(path) = env::var_os(LOGOS_DB_PATH_ENV) {
            return PathBuf::from(path);
        }

        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            return PathBuf::from(home)
                .join(DEFAULT_DB_DIRECTORY)
                .join(DEFAULT_DB_NAME);
        }

        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(DEFAULT_DB_DIRECTORY)
            .join(DEFAULT_DB_NAME)
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

    #[must_use]
    pub fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64 {
        let actual_expense_cents: i64 = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with(expense_account_prefix))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum();
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    /// Persists a budget target for a month and account-prefix scope.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails.
    pub fn set_budget_target_for_month(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), RuntimeError> {
        self.store
            .write_budget_target(month_key, expense_account_prefix, budget_cents)?;
        Ok(())
    }

    #[must_use]
    pub fn budget_target_for_month(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<i64> {
        self.store
            .budget_target(month_key, expense_account_prefix)
            .map(logos_store_aletheia::model::StoredBudgetTarget::budget_cents)
    }

    #[must_use]
    pub fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64 {
        let actual_expense_cents = self.expense_total_for_month(month_key, expense_account_prefix);
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    #[must_use]
    pub fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport {
        let checking_balance_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account() == checking_account)
            .map(Posting::amount)
            .sum();

        let income_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with("income:"))
            .map(Posting::amount)
            .filter(|amount| *amount < 0)
            .map(i64::abs)
            .sum();

        let expense_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with("expenses:"))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum();

        let cashflow_cents = project_cashflow(income_cents, expense_cents);
        MonthReport::new(
            checking_balance_cents,
            income_cents,
            expense_cents,
            cashflow_cents,
        )
    }

    #[must_use]
    pub fn current_month_key_utc() -> String {
        let wallclock_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_micros())
            .ok()
            .and_then(|micros| i64::try_from(micros).ok())
            .unwrap_or(0);
        month_key_from_wallclock_utc(wallclock_us)
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

    fn expense_total_for_month(&self, month_key: &str, expense_account_prefix: &str) -> i64 {
        self.store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with(expense_account_prefix))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum()
    }
}

fn transaction_in_month(stored: &StoredTransaction, month_key: &str) -> bool {
    month_key_from_wallclock_utc(stored.effective_at().wallclock()) == month_key
}

fn month_key_from_wallclock_utc(wallclock_us: i64) -> String {
    let secs = wallclock_us.div_euclid(1_000_000);
    let days = secs.div_euclid(86_400);
    let (year, month, _) = civil_from_days(days);
    format!("{year:04}-{month:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    let month_u32 = u32::try_from(month).unwrap_or(1);
    let day_u32 = u32::try_from(day).unwrap_or(1);
    (year, month_u32, day_u32)
}
