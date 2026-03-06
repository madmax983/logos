use logos_core::AccountId;
use logos_core::domain::budget::rollover_end_balance;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_posting_credit_does_not_panic(amount in any::<i64>()) {
        let _ = Posting::credit(AccountId::new("test").unwrap(), amount);
    }

    #[test]
    fn havoc_transaction_builder_sum_does_not_panic(
        amounts in prop::collection::vec(any::<i64>(), 1..100)
    ) {
        let mut builder = TransactionBuilder::new("Test");
        for amt in amounts {
            builder = builder.posting(Posting::debit(AccountId::new("test").unwrap(), amt));
        }
        let _ = builder.build();
    }

    #[test]
    fn havoc_rollover_end_balance_does_not_panic(
        start in any::<i64>(),
        assigned in any::<i64>(),
        spent in any::<i64>()
    ) {
        let _ = rollover_end_balance(start, assigned, spent);
    }
}

#[test]
fn havoc_posting_credit_min_does_not_panic() {
    let _ =
        logos_core::domain::transaction::Posting::credit(AccountId::new("test").unwrap(), i64::MIN);
}
