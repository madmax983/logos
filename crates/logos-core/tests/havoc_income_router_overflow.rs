#![allow(clippy::should_panic_without_expect)]
use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::AccountId;

#[test]
#[should_panic]
fn havoc_income_router_ratio_breaks_on_large_inputs() {
    let amount = i64::MAX / 20;
    let source = AccountId::new("income:salary").unwrap();
    let dest1 = AccountId::new("assets:checking").unwrap();
    let dest2 = AccountId::new("assets:savings").unwrap();

    let router = IncomeRouter::new(source, vec![
        RouteRule { destination: dest1, percentage: 60 },
        RouteRule { destination: dest2, percentage: 40 }
    ]).unwrap();

    let tx = router.route_income("Paycheck", amount).unwrap();
    let postings = tx.postings();

    let dest1_amount = postings.iter().find(|p| p.account().as_str() == "assets:checking").unwrap().amount();

    let expected = (amount as i128 * 60 / 100) as i64;
    assert_eq!(dest1_amount, expected, "Ratio broke because of internal overflow!");
}
