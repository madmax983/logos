#![allow(clippy::should_panic_without_expect)]
#![cfg(feature = "nova")]
use logos_core::domain::account::AccountId;
use logos_core::experimental::income_router::{IncomeRouter, RouteRule};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn income_router_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();

        // Use a rule with 100% to sweep remainder
        let router = IncomeRouter::new(source, vec![RouteRule { destination: dest1, percentage: 100 }]).unwrap();
        let _ = router.route_income("Paycheck", amount);
    }
}
