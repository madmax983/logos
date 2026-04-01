pub mod transactions;
pub mod budgets;
pub mod reconcile;
pub mod fetch;
pub mod import;
pub mod analytics;

pub(crate) struct SnapshotPostingRow {
    txn_id: String,
    description: String,
    effective_at_us: i64,
    posting_ordinal: i64,
    account: String,
    amount_cents: i64,
}

pub(crate) struct ResolvedAutopilotBalances {
    opening_balance_cents: i64,
    closing_balance_cents: i64,
}

use chrono::Local;
use logos_core::{Posting, TransactionBuilder};
use logos_fetch::FetchedStatementArtifact;
use logos_reporting::{
    RegisterEntry, project_register_balance,
};
use logos_store_aletheia::{
    AletheiaStore,
    model::StoredTransaction,
};

use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::RuntimeError;

const LOGOS_DB_PATH_ENV: &str = "LOGOS_DB_PATH";
const LOGOS_ARTIFACTS_PATH_ENV: &str = "LOGOS_ARTIFACTS_PATH";
const LOGOS_FETCH_CONFIG_PATH_ENV: &str = "LOGOS_FETCH_CONFIG_PATH";
const DEFAULT_DB_DIRECTORY: &str = ".logos";
const DEFAULT_DB_NAME: &str = "ledger";
const DEFAULT_FETCH_CONFIG_NAME: &str = "statement-sources.toml";
const ARTIFACTS_DIRECTORY: &str = "artifacts";
const PARQUET_DIRECTORY: &str = "parquet";
pub const DEFAULT_ANALYTICS_SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub struct AppRuntime {
    store: AletheiaStore,
    imported_records: usize,
    artifacts_root: PathBuf,
    fetch_config_path: Option<PathBuf>,
    fetched_statement_artifacts: Vec<FetchedStatementArtifact>,
}

impl Default for AppRuntime {
    fn default() -> Self {
        let default_store_path = Self::default_store_path();
        Self {
            store: AletheiaStore::new_in_memory(),
            imported_records: 0,
            artifacts_root: default_artifacts_root(&default_store_path),
            fetch_config_path: None,
            fetched_statement_artifacts: Vec::new(),
        }
    }
}

impl AppRuntime {
    /// Creates a runtime backed by the default durable store path.
    /// `LOGOS_DB_PATH` overrides the location. Otherwise, the path defaults to:
    /// - `${HOME}/.logos/ledger` on Unix-like systems
    /// - `%USERPROFILE%\\.logos\\ledger` on Windows
    /// - `./.logos/ledger` when no home directory is available
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::open(Self::default_store_path())
    }

    /// Creates a runtime pinned to a specific durable store path.
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let store_path = path.as_ref().to_path_buf();
        Ok(Self {
            store: AletheiaStore::open(&store_path)?,
            imported_records: 0,
            artifacts_root: default_artifacts_root(&store_path),
            fetch_config_path: Some(default_fetch_config_path(&store_path)),
            fetched_statement_artifacts: Vec::new(),
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

    #[must_use]
    pub fn register_balance_for(&self, account: &str) -> i64 {
        let entries: Vec<RegisterEntry> = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().as_str() == account)
            .map(|posting| RegisterEntry::new(posting.amount()))
            .collect();

        project_register_balance(0, &entries)
    }




    #[must_use]
    pub fn current_month_key_local() -> String {
        Local::now().format("%Y-%m").to_string()
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

}

fn default_artifacts_root(store_path: &Path) -> PathBuf {
    if let Some(path) = env::var_os(LOGOS_ARTIFACTS_PATH_ENV) {
        return PathBuf::from(path);
    }

    store_path.parent().map_or_else(
        || store_path.join(ARTIFACTS_DIRECTORY),
        |parent| parent.join(ARTIFACTS_DIRECTORY),
    )
}

fn default_fetch_config_path(store_path: &Path) -> PathBuf {
    if let Some(path) = env::var_os(LOGOS_FETCH_CONFIG_PATH_ENV) {
        return PathBuf::from(path);
    }

    store_path.parent().map_or_else(
        || store_path.join(DEFAULT_FETCH_CONFIG_NAME),
        |parent| parent.join(DEFAULT_FETCH_CONFIG_NAME),
    )
}

fn current_time_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros())
        .ok()
        .and_then(|micros| i64::try_from(micros).ok())
        .unwrap_or(0)
}

fn build_double_entry(
    description: &str,
    debit_account: &str,
    credit_account: &str,
    amount_cents: i64,
) -> Result<TransactionBuilder, RuntimeError> {
    use logos_core::AccountId;
    Ok(TransactionBuilder::new(description)
        .posting(Posting::debit(
            AccountId::new(debit_account)?,
            amount_cents,
        )?)
        .posting(Posting::credit(
            AccountId::new(credit_account)?,
            amount_cents,
        )?))
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

#[cfg(test)]
mod tests {
    use super::AppRuntime;
    use logos_fetch::{FetchError, OutputFormat, SecretBundle, SecretResolver, StatementSource};

    #[derive(Debug, Clone)]
    struct StubSecretResolver {
        bundle: SecretBundle,
    }

    impl SecretResolver for StubSecretResolver {
        fn resolve(&self, _source: &StatementSource) -> Result<SecretBundle, FetchError> {
            Ok(self.bundle.clone())
        }
    }

    #[test]
    fn fake_fetch_sources_bypass_external_secret_resolution() {
        let _runtime = AppRuntime::new_in_memory();
        let source = StatementSource::new(
            "fixture:checking",
            "fake-fixture",
            "assets:checking",
            vec![OutputFormat::Pdf],
        )
        .expect("source");
        let resolver = StubSecretResolver {
            bundle: SecretBundle::new("wrong", "wrong", Some("999999")).expect("bundle"),
        };

        let bundle = AppRuntime::secret_bundle_for_fetch_source_with_resolver(&source, &resolver)
            .expect("bundle");

        assert_eq!(
            bundle,
            SecretBundle::new("fixture-user", "fixture-pass", Some("000000")).expect("fixture")
        );
    }

    #[test]
    fn real_fetch_sources_use_external_secret_resolution() {
        let _runtime = AppRuntime::new_in_memory();
        let source = StatementSource::new(
            "pcu:checking",
            "provident-credit-union",
            "assets:checking",
            vec![OutputFormat::Pdf],
        )
        .expect("source")
        .with_secret_refs(
            "op://logos/provident/username",
            "op://logos/provident/password",
            Some("op://logos/provident/totp"),
        )
        .expect("secret refs");
        let resolver = StubSecretResolver {
            bundle: SecretBundle::new("markm", "s3cr3t", Some("123456")).expect("bundle"),
        };

        let bundle = AppRuntime::secret_bundle_for_fetch_source_with_resolver(&source, &resolver)
            .expect("bundle");

        assert_eq!(
            bundle,
            SecretBundle::new("markm", "s3cr3t", Some("123456")).expect("expected")
        );
    }
}
