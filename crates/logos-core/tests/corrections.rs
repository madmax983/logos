use logos_core::{Correction, TransactionId};

#[test]
fn correction_links_to_prior_transaction() {
    let original = TransactionId::new("txn-1");
    let correction = Correction::new(original.clone(), "fix memo").expect("correction");

    assert_eq!(correction.supersedes_id(), &original);
}

#[test]
fn correction_cannot_supersede_itself() {
    let id = TransactionId::new("txn-1");
    let err = Correction::new(id.clone(), "self")
        .and_then(|correction| correction.validate_not_self(&id));

    assert!(err.is_err());
}

#[test]
fn correction_requires_non_empty_reason() {
    let err = Correction::new(TransactionId::new("txn-1"), "   ").expect_err("must fail");
    assert_eq!(err.to_string(), "correction reason cannot be empty");
}

#[test]
fn correction_validate_not_self_allows_different_transaction() {
    let correction = Correction::new(TransactionId::new("txn-1"), "fix memo")
        .expect("create")
        .validate_not_self(&TransactionId::new("txn-2"))
        .expect("different id");

    assert_eq!(correction.supersedes_id().as_str(), "txn-1");
}
