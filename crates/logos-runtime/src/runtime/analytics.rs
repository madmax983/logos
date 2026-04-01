use crate::error::RuntimeError;
use crate::runtime::{AppRuntime, PARQUET_DIRECTORY, SnapshotPostingRow, current_time_us};
use logos_store_aletheia::model::{StoredAnalyticsArtifactManifest, StoredTransaction};
use polars::prelude::{DataFrame, NamedFrom, ParquetWriter, Series};
use std::fs::File;
use std::path::Path;
use blake3::Hasher;

impl AppRuntime {
    /// Creates an immutable Parquet analytics snapshot and records its manifest in Aletheia.
    ///
    /// # Errors
    ///
    /// Returns an error when snapshot reconstruction, parquet writing, or manifest persistence fails.
    pub fn create_analytics_snapshot(
        &mut self,
        as_of_valid_time_us: Option<i64>,
        as_of_tx_time_us: Option<i64>,
        schema_version: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, RuntimeError> {
        if schema_version <= 0 {
            return Err(RuntimeError::Analytics {
                message: format!("schema_version must be positive, got {schema_version}"),
            });
        }

        let snapshot_now = current_time_us();
        let as_of_valid = as_of_valid_time_us.unwrap_or(snapshot_now);
        let as_of_tx = as_of_tx_time_us.unwrap_or(snapshot_now);
        let rows = snapshot_rows(self.store.transactions_as_of_us(as_of_valid, as_of_tx)?);
        let content_hash = hash_rows(&rows, as_of_valid, as_of_tx, schema_version);

        let parquet_dir = self.artifacts_root.join(PARQUET_DIRECTORY);
        std::fs::create_dir_all(&parquet_dir).map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to create parquet artifact directory '{}': {err}",
                parquet_dir.display()
            ),
        })?;

        let parquet_path = parquet_dir.join(format!("{content_hash}.parquet"));
        if !parquet_path.exists() {
            write_rows_to_parquet(&parquet_path, &rows, as_of_valid, as_of_tx)?;
        }

        let parquet_uri = parquet_path.display().to_string();
        self.store
            .write_analytics_artifact_manifest_us(
                "parquet",
                &parquet_uri,
                &content_hash,
                schema_version,
                i64::try_from(rows.len()).unwrap_or(i64::MAX),
                as_of_valid,
                as_of_tx,
                supersedes_artifact_id,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn list_analytics_snapshots(&self) -> Vec<StoredAnalyticsArtifactManifest> {
        let mut manifests: Vec<_> = self.store.analytics_artifacts().cloned().collect();
        manifests.sort_by(|left, right| {
            right
                .created_at()
                .wallclock()
                .cmp(&left.created_at().wallclock())
                .then_with(|| left.artifact_id().cmp(right.artifact_id()))
        });
        manifests
    }

    #[must_use]
    pub fn get_analytics_snapshot(
        &self,
        artifact_id: &str,
    ) -> Option<StoredAnalyticsArtifactManifest> {
        self.store.analytics_artifact(artifact_id).cloned()
    }

}

    #[must_use]
    pub const fn default_analytics_schema_version() -> i64 {
        1
    }

/// Extracts `SnapshotPostingRow`s from a list of transactions.
///
/// **Performance Optimization**: Pre-calculates the exact number of postings across all
/// transactions to pre-allocate the `rows` vector. This avoids multiple intermediate
/// heap reallocations when processing large ledgers.
pub(crate) fn snapshot_rows(transactions: Vec<StoredTransaction>) -> Vec<SnapshotPostingRow> {
    let capacity: usize = transactions
        .iter()
        .map(|stored| stored.transaction().postings().len())
        .sum();
    let mut rows = Vec::with_capacity(capacity);
    for stored in transactions {
        for (index, posting) in stored.transaction().postings().iter().enumerate() {
            let posting_ordinal = i64::try_from(index).unwrap_or(i64::MAX);
            rows.push(SnapshotPostingRow {
                txn_id: stored.id().as_str().to_owned(),
                description: stored.transaction().description().to_owned(),
                effective_at_us: stored.effective_at().wallclock(),
                posting_ordinal,
                account: posting.account().as_str().to_owned(),
                amount_cents: posting.amount(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.txn_id
            .cmp(&right.txn_id)
            .then_with(|| left.posting_ordinal.cmp(&right.posting_ordinal))
            .then_with(|| left.account.cmp(&right.account))
            .then_with(|| left.amount_cents.cmp(&right.amount_cents))
    });
    rows
}

pub(crate) fn hash_rows(
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
    schema_version: i64,
) -> String {
    let mut hasher = Hasher::new();
    hasher
        .update(format!("schema:{schema_version}|valid:{as_of_valid}|tx:{as_of_tx}\n").as_bytes());
    for row in rows {
        hasher.update(
            format!(
                "{}|{}|{}|{}|{}|{}\n",
                row.txn_id,
                row.description,
                row.effective_at_us,
                row.posting_ordinal,
                row.account,
                row.amount_cents
            )
            .as_bytes(),
        );
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn write_rows_to_parquet(
    parquet_path: &Path,
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
) -> Result<(), RuntimeError> {
    let txn_ids: Vec<&str> = rows.iter().map(|row| row.txn_id.as_str()).collect();
    let descriptions: Vec<&str> = rows.iter().map(|row| row.description.as_str()).collect();
    let effective_at_values: Vec<i64> = rows.iter().map(|row| row.effective_at_us).collect();
    let posting_ordinals: Vec<i64> = rows.iter().map(|row| row.posting_ordinal).collect();
    let accounts: Vec<&str> = rows.iter().map(|row| row.account.as_str()).collect();
    let amounts: Vec<i64> = rows.iter().map(|row| row.amount_cents).collect();
    let snapshot_valid_values = vec![as_of_valid; rows.len()];
    let snapshot_tx_values = vec![as_of_tx; rows.len()];

    let mut frame = DataFrame::new(vec![
        Series::new("txn_id".into(), txn_ids).into(),
        Series::new("description".into(), descriptions).into(),
        Series::new("effective_at_us".into(), effective_at_values).into(),
        Series::new("posting_ordinal".into(), posting_ordinals).into(),
        Series::new("account".into(), accounts).into(),
        Series::new("amount_cents".into(), amounts).into(),
        Series::new("snapshot_valid_at_us".into(), snapshot_valid_values).into(),
        Series::new("snapshot_tx_at_us".into(), snapshot_tx_values).into(),
    ])
    .map_err(|err| RuntimeError::Analytics {
        message: format!("failed to construct analytics frame: {err}"),
    })?;

    let file = File::create(parquet_path).map_err(|err| RuntimeError::Analytics {
        message: format!(
            "failed to create parquet snapshot '{}': {err}",
            parquet_path.display()
        ),
    })?;
    ParquetWriter::new(file)
        .finish(&mut frame)
        .map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to write parquet snapshot '{}': {err}",
                parquet_path.display()
            ),
        })?;
    Ok(())
}
