use logos_core::{Correction, TransactionId};

#[test]
fn test_correction_reason() {
    let id = TransactionId::new("txn-1").expect("id");
    let correction = Correction::new(id, "fix typo").unwrap();
    assert_eq!(correction.reason(), "fix typo");
}
