fn main() {
    let content = std::fs::read_to_string("crates/logos-core/src/experimental/income_router.rs").unwrap();
    let fixed = content.replace("destination: dest2,", "destination: dest2.clone(),").replace("fn test_zero_amount_allocation_is_skipped() {
        let source = AccountId::new(\"income:salary\").unwrap();
        let dest1 = AccountId::new(\"assets:checking\").unwrap();
        let dest2 = AccountId::new(\"assets:savings\").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1.clone(),
                    percentage: 99,
                },
                RouteRule {
                    destination: dest2.clone(),
                    percentage: 1,
                },", "fn test_zero_amount_allocation_is_skipped() {
        let source = AccountId::new(\"income:salary\").unwrap();
        let dest1 = AccountId::new(\"assets:checking\").unwrap();
        let dest2 = AccountId::new(\"assets:savings\").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1.clone(),
                    percentage: 99,
                },
                RouteRule {
                    destination: dest2,
                    percentage: 1,
                },");
    std::fs::write("crates/logos-core/src/experimental/income_router.rs", fixed).unwrap();
}
