#![allow(clippy::should_panic_without_expect)]

use logos_core::domain::account::AccountId;
use logos_core::experimental::income_router::{IncomeRouter, RouteRule};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn income_router_panics_on_overflow(
        amount in (i64::MAX / 100 + 1)..=i64::MAX,
    ) {
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

        let _ = router.route_income("Paycheck", amount);
    }
}
