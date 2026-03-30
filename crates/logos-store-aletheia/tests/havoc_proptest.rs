#![allow(clippy::should_panic_without_expect)]
use logos_store_aletheia::AletheiaStore;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn write_reconciliation_run_panics_on_overflow_or_invalid(
        month_key in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        checking_account in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        matched_postings in 0..10_i64,
        outflow_cents in -100..0_i64,
    ) {
        let mut store = AletheiaStore::new_in_memory();
        store.write_reconciliation_run(
            &month_key,
            &checking_account,
            0,
            0,
            0,
            0,
            0,
            true,
            matched_postings,
            0,
            outflow_cents,
            &[],
        ).unwrap();
    }
}

proptest! {
    #[test]
    #[should_panic]
    fn write_reconciliation_run_and_month_close_panics_on_invalid(
        month_key in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        checking_account in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        matched_postings in 0..10_i64,
        inflow_cents in -100..0_i64,
    ) {
        let mut store = AletheiaStore::new_in_memory();
        store.write_reconciliation_run_and_month_close(
            &month_key,
            &checking_account,
            0,
            0,
            0,
            0,
            0,
            true,
            matched_postings,
            inflow_cents,
            0,
            &[],
            None,
        ).unwrap();
    }
}

proptest! {
    #[test]
    #[should_panic]
    fn write_fetch_run_panics_on_invalid(
        source_id in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        institution_id in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        ledger_account in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        month_key in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
    ) {
        let mut store = AletheiaStore::new_in_memory();
        store.write_fetch_run(
            &source_id,
            &institution_id,
            &ledger_account,
            &month_key,
            logos_store_aletheia::StoredFetchRunStatus::Downloaded,
            None,
            None,
            None,
            None,
            None,
        ).unwrap();
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "duplicate_count must be non-negative")]
    fn write_import_batch_panics_on_invalid_count(
        import_kind in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        source_uri in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        batch_key in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        duplicate_count in -100..0_i64,
    ) {
        let mut store = AletheiaStore::new_in_memory();
        store.write_import_batch(
            &import_kind,
            &source_uri,
            &batch_key,
            duplicate_count,
            true,
            true,
            &[],
        ).unwrap();
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "unknown reconciliation run")]
    fn write_month_close_panics_on_unknown_reconciliation_run(
        month_key in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        checking_account in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
        reconciliation_run_id in any::<String>().prop_filter("Must not be empty", |s| !s.is_empty()),
    ) {
        let mut store = AletheiaStore::new_in_memory();
        store.write_month_close(
            &month_key,
            &checking_account,
            &reconciliation_run_id,
            None,
        ).unwrap();
    }
}
