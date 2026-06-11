use logos_core::{TransactionBuilder, Posting, AccountId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_transaction_credit_overflow(
        amount in i64::MIN..=0,
    ) {
        let account = AccountId::new("assets:checking").unwrap();
        let _ = Posting::credit(account, amount);
    }
}
