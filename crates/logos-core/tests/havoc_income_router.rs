use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::{AccountId, Posting};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_income_router_math_does_not_break(amount in 1..=i64::MAX) {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();
        let dest2 = AccountId::new("assets:savings").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1,
                    percentage: 50,
                },
                RouteRule {
                    destination: dest2,
                    percentage: 50,
                },
            ],
        )
        .unwrap();

        let tx = router.route_income("Paycheck", amount).unwrap();
        let postings = tx.postings();

        let debit1 = postings.iter().find(|p: &&Posting| p.account().as_str() == "assets:checking").unwrap().amount();
        let debit2 = postings.iter().find(|p: &&Posting| p.account().as_str() == "assets:savings").unwrap().amount();

        let diff = (debit1 - debit2).abs();
        assert!(diff <= 1, "Diff is {}, expected <= 1 for 50/50 split on amount {}", diff, amount);
    }
}
