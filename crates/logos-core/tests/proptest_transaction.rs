use logos_core::domain::transaction::TransactionBuilder;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_posting_credit_overflow(a in any::<i64>()) {
        let _ = logos_core::domain::transaction::Posting::credit("income:salary", a);
    }

    #[test]
    fn test_transaction_builder_overflow(a in any::<i64>(), b in any::<i64>()) {
        let debit_res = logos_core::domain::transaction::Posting::debit("assets:checking", a);
        let credit_res = logos_core::domain::transaction::Posting::credit("income:salary", b);

        let debit_is_ok = debit_res.is_ok();
        let credit_is_ok = credit_res.is_ok();

        if debit_is_ok && credit_is_ok {
            let debit = debit_res.unwrap();
            let credit = credit_res.unwrap();
            let res = TransactionBuilder::new("Test")
                .posting(debit)
                .posting(credit)
                .build();
            assert!(res.is_ok() || res.is_err());
        }
    }
}
