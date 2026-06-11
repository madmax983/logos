use logos_core::{TransactionId, Correction};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_correction_validate_not_self(
        id in "\\PC*",
        reason in "\\PC*",
    ) {
        if let Ok(tx_id) = TransactionId::new(&id) {
            if let Ok(correction) = Correction::new(tx_id.clone(), &reason) {
                let _ = correction.validate_not_self(&tx_id);
            }
        }
    }
}
