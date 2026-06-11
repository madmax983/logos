use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::AccountId;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_income_router_sweep_overflow_2(
        amount in i64::MAX/100..i64::MAX,
    ) {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();
        let dest2 = AccountId::new("assets:savings").unwrap();

        let router = IncomeRouter::new(source, vec![
            RouteRule { destination: dest1, percentage: 50 },
            RouteRule { destination: dest2, percentage: 50 }
        ]).unwrap();

        // This will saturate the multiplication: `amount_cents * 50` will be capped at i64::MAX.
        // `allocated` will be `i64::MAX / 100`.
        // `remaining_cents` will start at `amount` (e.g. `i64::MAX`).
        // `remaining_cents -= allocated` will leave a huge remainder (`i64::MAX - i64::MAX/100`).
        // Then `allocations[0].1 += remaining_cents` will add `i64::MAX/100 + i64::MAX - i64::MAX/100` = `i64::MAX` to the first allocation.
        // It'll probably not panic, just silently incorrectly allocate because of saturating_mul.
        // Let's actually test it.
        let _ = router.route_income("Paycheck", amount);
    }
}
