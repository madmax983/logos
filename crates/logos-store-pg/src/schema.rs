#![allow(missing_docs)]
diesel::table! {
    transactions (id) {
        id -> Text,
        description -> Text,
        effective_at_us -> BigInt,
        recorded_at_us -> BigInt,
        source_kind -> Nullable<Text>,
        external_ref -> Nullable<Text>,
    }
}

diesel::table! {
    postings (transaction_id, ordinal) {
        transaction_id -> Text,
        ordinal -> Int4,
        account -> Text,
        amount_cents -> BigInt,
    }
}

diesel::table! {
    corrections (correction_id) {
        correction_id -> Int8,
        supersedes_txn_id -> Text,
        reason -> Text,
        recorded_at_us -> BigInt,
    }
}

diesel::table! {
    budget_targets (month_key, expense_account_prefix) {
        month_key -> Text,
        expense_account_prefix -> Text,
        budget_cents -> BigInt,
    }
}

diesel::table! {
    analytics_artifact_manifests (artifact_id) {
        artifact_id -> Text,
        artifact_kind -> Text,
        artifact_uri -> Text,
        content_hash -> Text,
        schema_version -> BigInt,
        row_count -> BigInt,
        snapshot_valid_at_us -> BigInt,
        snapshot_tx_at_us -> BigInt,
        created_at_us -> BigInt,
        supersedes_artifact_id -> Nullable<Text>,
        snapshot_key -> Text,
    }
}

diesel::table! {
    import_batches (batch_id) {
        batch_id -> Text,
        import_kind -> Text,
        source_uri -> Text,
        batch_key -> Text,
        record_count -> BigInt,
        duplicate_count -> BigInt,
        dry_run -> Bool,
        ocr_enabled -> Bool,
        imported_at_us -> BigInt,
    }
}

diesel::table! {
    import_records (content_hash_key) {
        content_hash_key -> Text,
        batch_id -> Text,
        imported_txn_id -> Nullable<Text>,
        imported_at_us -> BigInt,
    }
}

diesel::table! {
    statement_lines (line_id) {
        line_id -> Text,
        batch_id -> Text,
        source_uri -> Text,
        statement_timestamp -> Text,
        memo -> Text,
        amount_cents -> BigInt,
        imported_txn_id -> Nullable<Text>,
        imported_at_us -> BigInt,
    }
}

diesel::table! {
    fetch_runs (run_id) {
        run_id -> Text,
        source_id -> Text,
        institution_id -> Text,
        ledger_account -> Text,
        month_key -> Text,
        status -> Text,
        artifact_path -> Nullable<Text>,
        output_format -> Nullable<Text>,
        opening_balance_cents -> Nullable<BigInt>,
        closing_balance_cents -> Nullable<BigInt>,
        error_summary -> Nullable<Text>,
        created_at_us -> BigInt,
    }
}

diesel::table! {
    reconciliation_runs (run_id) {
        run_id -> Text,
        month_key -> Text,
        checking_account -> Text,
        opening_balance_cents -> BigInt,
        ledger_delta_cents -> BigInt,
        expected_closing_balance_cents -> BigInt,
        statement_closing_balance_cents -> BigInt,
        variance_cents -> BigInt,
        reconciled -> Bool,
        matched_postings -> BigInt,
        matched_transaction_count -> BigInt,
        inflow_cents -> BigInt,
        outflow_cents -> BigInt,
        created_at_us -> BigInt,
    }
}

diesel::table! {
    reconciliation_run_transactions (run_id, transaction_id) {
        run_id -> Text,
        transaction_id -> Text,
    }
}

diesel::table! {
    reconciliation_run_statement_lines (run_id, statement_line_id) {
        run_id -> Text,
        statement_line_id -> Text,
    }
}

diesel::table! {
    month_closes (close_id) {
        close_id -> Text,
        month_key -> Text,
        checking_account -> Text,
        reconciliation_run_id -> Text,
        analytics_artifact_id -> Nullable<Text>,
        closed_at_us -> BigInt,
    }
}

diesel::joinable!(postings -> transactions (transaction_id));
diesel::joinable!(corrections -> transactions (supersedes_txn_id));
diesel::joinable!(import_records -> import_batches (batch_id));
diesel::joinable!(import_records -> transactions (imported_txn_id));
diesel::joinable!(statement_lines -> import_batches (batch_id));
diesel::joinable!(statement_lines -> transactions (imported_txn_id));
diesel::joinable!(reconciliation_run_transactions -> reconciliation_runs (run_id));
diesel::joinable!(reconciliation_run_transactions -> transactions (transaction_id));
diesel::joinable!(reconciliation_run_statement_lines -> reconciliation_runs (run_id));
diesel::joinable!(reconciliation_run_statement_lines -> statement_lines (statement_line_id));
diesel::joinable!(month_closes -> reconciliation_runs (reconciliation_run_id));
diesel::joinable!(month_closes -> analytics_artifact_manifests (analytics_artifact_id));

diesel::allow_tables_to_appear_in_same_query!(
    analytics_artifact_manifests,
    budget_targets,
    corrections,
    fetch_runs,
    import_batches,
    import_records,
    month_closes,
    postings,
    reconciliation_run_statement_lines,
    reconciliation_run_transactions,
    reconciliation_runs,
    statement_lines,
    transactions,
);
