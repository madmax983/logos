use logos_core::domain::transaction::{TransactionBuilder, Posting};
use logos_core::AccountId;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "AmountOverflow")]
    fn havoc_transaction_builder_sum_overflow(
        debit1 in (i64::MAX / 2 + 1)..=i64::MAX,
        debit2 in (i64::MAX / 2 + 1)..=i64::MAX,
        credit1 in (i64::MAX / 2 + 1)..=i64::MAX,
        credit2 in (i64::MAX / 2 + 1)..=i64::MAX
    ) {
        let builder = TransactionBuilder::new("overflow")
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), debit1).unwrap())
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), debit2).unwrap())
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), credit1).unwrap())
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), credit2).unwrap());

        builder.build().unwrap();
    }
}
