use logos_core::AccountId;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_account_id(id in "\\PC*") {
        let _ = AccountId::new(&id);
    }
}
