use crate::error::RuntimeError;
use crate::models::{CsvImportSummary, PdfImportSummary};
use crate::runtime::{AppRuntime, build_double_entry};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use logos_core::TransactionId;
use logos_import::{CsvMapping, ImportError, ImportRecord, deterministic_fingerprint, deterministic_fingerprint_legacy_v1, parse_pdf_statement_file, parse_simple_csv_row};
use logos_store_aletheia::model::NewImportRecord;
use std::path::Path;
use std::fs;
use std::collections::HashSet;
use blake3::Hasher;

impl AppRuntime {
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
        let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
        if self.store.has_import_record_content_hash(&content_hash_key)
            || self
                .store
                .has_import_record_content_hash(&legacy_content_hash_key)
        {
            return Ok(false);
        }

        let batch_key = import_batch_key(
            "csv-row",
            "inline:csv",
            false,
            false,
            std::slice::from_ref(&content_hash_key),
        );
        self.store.write_import_batch(
            "csv-row",
            "inline:csv",
            &batch_key,
            0,
            false,
            false,
            &[NewImportRecord::new(&content_hash_key, None)],
        )?;
        self.imported_records = self.imported_records.saturating_add(1);
        Ok(true)
    }

    /// Imports transaction rows from a CSV statement file.
    ///
    /// When `dry_run` is true, rows are parsed and deduplicated but not posted.
    ///
    /// # Errors
    ///
    /// Returns an error when file IO, row parsing, or posting fails.
    pub fn import_csv_statement(
        &mut self,
        path: impl AsRef<Path>,
        mapping: &CsvMapping,
        dry_run: bool,
        skip_header: bool,
    ) -> Result<CsvImportSummary, RuntimeError> {
        let source_uri = path.as_ref().display().to_string();
        let csv_text = fs::read_to_string(&path).map_err(|err| ImportError::FileReadFailed {
            path: source_uri.clone(),
            message: err.to_string(),
        })?;

        let mut imported_count = 0_usize;
        let mut duplicate_count = 0_usize;
        let mut seen_in_call: HashSet<String> = HashSet::new();
        let mut imported_records = Vec::new();
        let mut imported_keys = Vec::new();

        for (line_idx, row) in csv_text.lines().enumerate() {
            if skip_header && line_idx == 0 {
                continue;
            }
            if row.trim().is_empty() {
                continue;
            }

            let record = parse_simple_csv_row(row, mapping)?;
            let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
            let seen_previously = self.store.has_import_record_content_hash(&content_hash_key)
                || self
                    .store
                    .has_import_record_content_hash(&legacy_content_hash_key);
            // Avoid allocating strings for duplicate records by checking existence first.
            let seen_in_batch = seen_in_call.contains(&content_hash_key);
            if seen_previously || seen_in_batch {
                duplicate_count = duplicate_count.saturating_add(1);
                continue;
            }
            seen_in_call.insert(content_hash_key.clone());

            imported_count = imported_count.saturating_add(1);
            if dry_run {
                continue;
            }

            let txn_id = self.post_import_record(&record)?;
            imported_records.push(NewImportRecord::with_statement_line(
                &content_hash_key,
                Some(&txn_id),
                &source_uri,
                record.timestamp(),
                record.memo(),
                record.amount_cents(),
            ));
            imported_keys.push(content_hash_key);
            self.imported_records = self.imported_records.saturating_add(1);
        }

        if !dry_run {
            let batch_key =
                import_batch_key("csv-statement", &source_uri, dry_run, false, &imported_keys);
            let duplicate_count = i64::try_from(duplicate_count).unwrap_or(i64::MAX);
            self.store.write_import_batch(
                "csv-statement",
                &source_uri,
                &batch_key,
                duplicate_count,
                dry_run,
                false,
                &imported_records,
            )?;
        }

        Ok(CsvImportSummary::new(
            imported_count,
            duplicate_count,
            dry_run,
        ))
    }

    /// Imports transaction rows from a PDF statement.
    ///
    /// When `dry_run` is true, records are parsed and deduplicated but not posted.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails or posting a derived transaction fails.
    pub fn import_pdf_statement(
        &mut self,
        path: impl AsRef<Path>,
        account: &str,
        dry_run: bool,
        enable_ocr: bool,
    ) -> Result<PdfImportSummary, RuntimeError> {
        let source_uri = path.as_ref().display().to_string();
        let records = parse_pdf_statement_file(&path, account, enable_ocr)?;
        let mut imported_count = 0_usize;
        let mut duplicate_count = 0_usize;
        let mut seen_in_call: HashSet<String> = HashSet::new();
        let mut imported_records = Vec::new();
        let mut imported_keys = Vec::new();

        for record in records {
            let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
            let seen_previously = self.store.has_import_record_content_hash(&content_hash_key)
                || self
                    .store
                    .has_import_record_content_hash(&legacy_content_hash_key);
            // Avoid allocating strings for duplicate records by checking existence first.
            let seen_in_batch = seen_in_call.contains(&content_hash_key);
            if seen_previously || seen_in_batch {
                duplicate_count = duplicate_count.saturating_add(1);
                continue;
            }
            seen_in_call.insert(content_hash_key.clone());

            imported_count = imported_count.saturating_add(1);
            if dry_run {
                continue;
            }

            let txn_id = self.post_import_record(&record)?;
            imported_records.push(NewImportRecord::with_statement_line(
                &content_hash_key,
                Some(&txn_id),
                &source_uri,
                record.timestamp(),
                record.memo(),
                record.amount_cents(),
            ));
            imported_keys.push(content_hash_key);
            self.imported_records = self.imported_records.saturating_add(1);
        }

        if !dry_run {
            let batch_key = import_batch_key(
                "pdf-statement",
                &source_uri,
                dry_run,
                enable_ocr,
                &imported_keys,
            );
            let duplicate_count = i64::try_from(duplicate_count).unwrap_or(i64::MAX);
            self.store.write_import_batch(
                "pdf-statement",
                &source_uri,
                &batch_key,
                duplicate_count,
                dry_run,
                enable_ocr,
                &imported_records,
            )?;
        }

        Ok(PdfImportSummary::new(
            imported_count,
            duplicate_count,
            dry_run,
        ))
    }

    #[must_use]
    pub const fn imported_record_count(&self) -> usize {
        self.imported_records
    }

    pub(crate) fn post_import_record(&mut self, record: &ImportRecord) -> Result<TransactionId, RuntimeError> {
        let amount = record.amount_cents();
        let valid_from = parse_import_timestamp(record.timestamp()).map(Into::into);
        if amount > 0 {
            let builder =
                build_double_entry(record.memo(), record.account(), record.category(), amount)?;
            return self
                .store
                .write_transaction_with_valid_time(builder, valid_from)
                .map_err(RuntimeError::from);
        }

        let debit_amount = amount.checked_abs().ok_or(ImportError::InvalidAmount)?;
        let builder = build_double_entry(
            record.memo(),
            record.category(),
            record.account(),
            debit_amount,
        )?;
        self.store
            .write_transaction_with_valid_time(builder, valid_from)
            .map_err(RuntimeError::from)
    }

}

pub(crate) fn import_content_hash_key_legacy_v1(fingerprint: u64) -> String {
    format!("{fingerprint:016x}")
}

pub(crate) fn import_content_hash_keys(record: &ImportRecord) -> (String, String) {
    let content_hash_key = deterministic_fingerprint(record);
    // Backward compatibility: detect duplicates imported before the v2 fingerprint rollout.
    let legacy_content_hash_key =
        import_content_hash_key_legacy_v1(deterministic_fingerprint_legacy_v1(record));
    (content_hash_key, legacy_content_hash_key)
}

pub(crate) fn import_batch_key(
    import_kind: &str,
    source_uri: &str,
    dry_run: bool,
    ocr_enabled: bool,
    imported_keys: &[String],
) -> String {
    let mut sorted_keys = imported_keys.to_vec();
    sorted_keys.sort();

    let mut hasher = Hasher::new();
    hasher.update(
        format!("kind:{import_kind}|source:{source_uri}|dry_run:{dry_run}|ocr:{ocr_enabled}\n")
            .as_bytes(),
    );
    for key in sorted_keys {
        hasher.update(key.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn parse_import_timestamp(timestamp: &str) -> Option<i64> {
    let trimmed = timestamp.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp_micros());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc).timestamp_micros());
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let date_time = date.and_hms_opt(0, 0, 0)?;
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(date_time, Utc).timestamp_micros());
    }
    if let Ok(micros) = trimmed.parse::<i64>() {
        return Some(micros);
    }

    None
}
