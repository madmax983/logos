use logos_core::income_router::{IncomeRouter, RouteRule};
use logos_core::AccountId;

#[test]
fn havoc_income_router_sweep_overflow_4() {
    let amount = i64::MAX;
    let source = AccountId::new("income:salary").unwrap();
    let dest1 = AccountId::new("assets:checking").unwrap();
    let dest2 = AccountId::new("assets:savings").unwrap();

    let router = IncomeRouter::new(source, vec![
        RouteRule { destination: dest1, percentage: 50 },
        RouteRule { destination: dest2, percentage: 50 }
    ]).unwrap();

    // amount = 9223372036854775807
    // allocations[0] = amount * 50 / 100
    // amount * 50 overflows i64::MAX and gets saturated at i64::MAX.
    // i64::MAX / 100 = 92233720368547758.
    // remaining_cents = i64::MAX - 92233720368547758 = 9131138316486228049.
    // allocations[1] = 92233720368547758.
    // remaining_cents = 9131138316486228049 - 92233720368547758 = 9038904596117680291.
    // allocations[0] += remaining_cents (92233720368547758 + 9038904596117680291 = 9131138316486228049).
    //
    // builder debit:
    // allocations[0] (checking) debit: 9131138316486228049
    // allocations[1] (savings) debit: 92233720368547758
    // total debit = 9131138316486228049 + 92233720368547758 = 9223372036854775807 (i64::MAX)
    // credit: i64::MAX
    //
    // So the transaction BALANCES! But the routing ratio is COMPLETELY wrong because of saturation.
    // 50/50 is routed as 99% / 1%.
    let tx = router.route_income("Paycheck", amount).unwrap();
    let postings = tx.postings();
    println!("{:?}", postings);

    // Assert the ratio is completely broken
    assert_ne!(postings[1].amount(), postings[2].amount());
}
