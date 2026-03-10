use logos_core::{Correction, TransactionId};
use logos_store_aletheia::{AletheiaStore, StoreError};

#[test]
fn write_correction_fails_when_superseded_transaction_is_unknown() {
    let mut store = AletheiaStore::new();
    let correction = Correction::new(TransactionId::new("txn-unknown").unwrap(), "reason").unwrap();

    let err = store.write_correction(correction).unwrap_err();
    assert!(matches!(err, StoreError::UnknownTransaction { .. }));
}
