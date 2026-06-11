use logos_core::{TransactionBuilder, Posting, AccountId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_transaction_fuzz(
        amounts in prop::collection::vec(any::<i64>(), 1..100),
    ) {
        let account = AccountId::new("assets:checking").unwrap();
        let mut builder = TransactionBuilder::new("Test");
        for amount in amounts {
            if amount > 0 {
                if let Ok(posting) = Posting::debit(account.clone(), amount) {
                    builder = builder.posting(posting);
                }
            } else if amount < 0 {
                // credit takes positive amount and negates it
                if let Ok(posting) = Posting::credit(account.clone(), -amount) {
                    builder = builder.posting(posting);
                }
            }
        }
        let _ = builder.build();
    }
}
