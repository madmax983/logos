use logos_core::{TransactionBuilder, Posting, AccountId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_transaction_amount_sum(
        amounts in prop::collection::vec(-1000..1000i64, 1..100),
    ) {
        let account = AccountId::new("assets:checking").unwrap();
        let mut builder = TransactionBuilder::new("Test");
        for amount in amounts {
            if amount > 0 {
                builder = builder.posting(Posting::debit(account.clone(), amount).unwrap());
            } else if amount < 0 {
                builder = builder.posting(Posting::credit(account.clone(), -amount).unwrap());
            }
        }
        let _ = builder.build();
    }
}
