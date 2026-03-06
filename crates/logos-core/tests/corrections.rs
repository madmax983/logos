use logos_core::{Correction, TransactionId};

#[test]
fn correction_links_to_prior_transaction() {
    let original = TransactionId::new("txn-1").expect("id");
    let correction = Correction::new(original.clone(), "fix memo").expect("correction");

    assert_eq!(correction.supersedes_id(), &original);
}

#[test]
fn correction_cannot_supersede_itself() {
    let id = TransactionId::new("txn-1").expect("id");
    let err = Correction::new_for_candidate(id.clone(), &id, "self");

    assert!(err.is_err());
}

#[test]
fn correction_requires_non_empty_reason() {
    let err =
        Correction::new(TransactionId::new("txn-1").expect("id"), "   ").expect_err("must fail");
    assert_eq!(err.to_string(), "correction reason cannot be empty");
}

#[test]
fn correction_validate_not_self_allows_different_transaction() {
    let supersedes = TransactionId::new("txn-1").expect("id");
    let candidate = TransactionId::new("txn-2").expect("id");
    let correction =
        Correction::new_for_candidate(supersedes, &candidate, "fix memo").expect("different id");

    assert_eq!(correction.supersedes_id().as_str(), "txn-1");
}

#[test]
fn correction_reason_is_trimmed() {
    let correction = Correction::new(
        TransactionId::new("txn-1").expect("id"),
        "   fixed payee casing   ",
    )
    .expect("create");

    assert_eq!(correction.reason(), "fixed payee casing");
}

#[test]
fn transaction_id_rejects_empty_after_trim() {
    let err = TransactionId::new("   ").expect_err("empty transaction id");
    assert_eq!(err.to_string(), "transaction id cannot be empty");
}

#[test]
fn transaction_id_is_trimmed() {
    let id = TransactionId::new("  txn-123  ").expect("id");
    assert_eq!(id.as_str(), "txn-123");
}
