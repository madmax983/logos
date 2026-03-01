use proptest::prelude::*;
use logos_core::domain::transaction::{Posting, TransactionBuilder};

proptest! {
    #[test]
    #[should_panic]
    fn test_transaction_builder_max_amount_prop(
        amount_1 in any::<i64>(),
        amount_2 in any::<i64>(),
    ) {
        let builder = TransactionBuilder::new("Test Transaction")
            .posting(Posting::debit("account1", amount_1))
            .posting(Posting::credit("account2", amount_2));

        // This will panic when total sum overflows i64
        let _ = builder.build();
    }
}

#[test]
#[should_panic]
fn test_posting_credit_min_amount() {
    // -i64::MIN overflows i64, this should panic
    let _ = Posting::credit("account", i64::MIN);
}
