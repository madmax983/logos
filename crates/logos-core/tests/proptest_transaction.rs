use logos_core::domain::transaction::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_posting_credit_overflow(a in any::<i64>()) {
        let _ = Posting::credit("income:salary", a);
    }

    #[test]
    fn test_transaction_builder_overflow(a in any::<i64>(), b in any::<i64>()) {
        let _ = TransactionBuilder::new("Test")
            .posting(Posting::debit("assets:checking", a))
            .posting(Posting::credit("income:salary", b))
            .build();
    }
}
