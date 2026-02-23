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
    let err = Correction::new(id.clone(), "self").and_then(|correction| correction.validate_not_self(&id));

    assert!(err.is_err());
}
