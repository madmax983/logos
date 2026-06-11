use logos_core::{TransactionBuilder, Posting, AccountId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_transaction_builder_underflow(
        amount1 in i64::MAX-1000..=i64::MAX,
        amount2 in i64::MAX-1000..=i64::MAX,
    ) {
        let account = AccountId::new("assets:checking").unwrap();
        let mut builder = TransactionBuilder::new("Test");
        if let Ok(posting1) = Posting::credit(account.clone(), amount1) {
            builder = builder.posting(posting1);
        }
        if let Ok(posting2) = Posting::credit(account.clone(), amount2) {
            builder = builder.posting(posting2);
        }
        let _ = builder.build();
    }
}
