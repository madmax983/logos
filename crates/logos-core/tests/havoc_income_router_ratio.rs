use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::AccountId;

#[test]
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
    let dest2_amount = postings.iter().find(|p| p.account().as_str() == "assets:savings").unwrap().amount();

    // Check if the ratio is completely off
    let ratio1 = (dest1_amount as f64) / (amount as f64);
    let ratio2 = (dest2_amount as f64) / (amount as f64);

    println!("Ratio 1: {}, Ratio 2: {}", ratio1, ratio2);
}
