use logos_core::{TransactionBuilder, Posting, AccountId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_transaction_balance(
        description in "\\PC*",
        amount1 in any::<i64>(),
        amount2 in any::<i64>(),
    ) {
        if amount1 <= 0 || amount2 <= 0 {
            return Ok(());
        }

        let account1 = AccountId::new("assets:checking").unwrap();
        let account2 = AccountId::new("expenses:food").unwrap();

        let mut builder = TransactionBuilder::new(&description);
        if let Ok(posting1) = Posting::debit(account1.clone(), amount1) {
            builder = builder.posting(posting1);
        }
        if let Ok(posting2) = Posting::credit(account2.clone(), amount2) {
            builder = builder.posting(posting2);
        }

        let _ = builder.build();
    }
}
