use logos_core::AccountId;
use logos_core::{Posting, TransactionBuilder};
use logos_store_aletheia::{AletheiaStore, StoreError};

#[allow(dead_code)]
fn temp_store_path(prefix: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-store-{prefix}-{nanos}.db"))
}

#[test]
fn write_month_close_fails_when_month_does_not_match() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap();

    let err = store
        .write_month_close("2026-04", "assets:checking", run.run_id(), None)
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("does not match reconciliation run month")
    );
}

#[test]
fn write_month_close_fails_when_checking_account_does_not_match() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap();

    let err = store
        .write_month_close("2026-03", "assets:savings", run.run_id(), None)
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("does not match reconciliation run account")
    );
}

#[test]
fn write_month_close_fails_when_artifact_is_unknown() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap();

    let err = store
        .write_month_close(
            "2026-03",
            "assets:checking",
            run.run_id(),
            Some("unknown-artifact"),
        )
        .unwrap_err();

    assert!(matches!(err, StoreError::UnknownArtifact { .. }));
}

#[test]
fn write_month_close_fails_when_reconciliation_run_is_unknown() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_month_close("2026-03", "assets:checking", "unknown-run", None)
        .unwrap_err();

    assert!(matches!(err, StoreError::PersistFailed { .. }));
    assert!(err.to_string().contains("unknown reconciliation run"));
}

#[test]
fn write_month_close_fails_when_month_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_month_close("", "assets:checking", "run-1", None)
        .unwrap_err();

    assert!(err.to_string().contains("month_key must not be empty"));
}

#[test]
fn write_month_close_fails_when_checking_account_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_month_close("2026-03", "", "run-1", None)
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("checking_account must not be empty")
    );
}

#[test]
fn write_month_close_fails_when_reconciliation_run_id_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_month_close("2026-03", "assets:checking", "", None)
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("reconciliation_run_id must not be empty")
    );
}

#[test]
fn write_month_close_fails_when_already_closed() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap();

    store
        .write_month_close("2026-03", "assets:checking", run.run_id(), None)
        .unwrap();

    let err = store
        .write_month_close("2026-03", "assets:checking", run.run_id(), None)
        .unwrap_err();

    assert!(err.to_string().contains("is already closed by"));
}

#[test]
fn write_reconciliation_run_and_month_close_fails_when_month_is_empty() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let err = store
        .write_reconciliation_run_and_month_close(
            "",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
            None,
        )
        .unwrap_err();

    assert!(err.to_string().contains("month_key must not be empty"));
}

#[test]
fn write_reconciliation_run_and_month_close_fails_when_checking_account_is_empty() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-03",
            "",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
            None,
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("checking_account must not be empty")
    );
}

#[test]
fn write_reconciliation_run_and_month_close_fails_when_already_closed() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    store
        .write_reconciliation_run_and_month_close(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
            None,
        )
        .unwrap();

    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-03",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
            None,
        )
        .unwrap_err();

    assert!(err.to_string().contains("is already closed by"));
}

#[test]
fn write_reconciliation_run_fails_when_month_is_empty() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let err = store
        .write_reconciliation_run(
            "",
            "assets:checking",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap_err();

    assert!(err.to_string().contains("month_key must not be empty"));
}

#[test]
fn write_reconciliation_run_fails_when_checking_account_is_empty() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let err = store
        .write_reconciliation_run(
            "2026-03",
            "",
            100_000,
            10_000,
            110_000,
            110_000,
            0,
            true,
            1,
            10_000,
            0,
            std::slice::from_ref(&txn_id),
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("checking_account must not be empty")
    );
}
