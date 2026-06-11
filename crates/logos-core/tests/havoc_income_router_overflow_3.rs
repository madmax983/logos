use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::AccountId;

#[test]
fn havoc_income_router_sweep_overflow_3() {
    let amount = i64::MAX;
    let source = AccountId::new("income:salary").unwrap();
    let dest1 = AccountId::new("assets:checking").unwrap();
    let dest2 = AccountId::new("assets:savings").unwrap();

    let router = IncomeRouter::new(source, vec![
        RouteRule { destination: dest1, percentage: 50 },
        RouteRule { destination: dest2, percentage: 50 }
    ]).unwrap();

    let tx = router.route_income("Paycheck", amount).unwrap();
    println!("{:?}", tx);
}
