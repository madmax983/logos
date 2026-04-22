#[cfg(feature = "nova")]
#[cfg(test)]
mod tests {
    use logos_core::experimental::debt_optimizer::{Debt, DebtOptimizer, PayoffStrategy};

    #[test]
    fn test_sentinel_debt_simulate_math_mutations() {
        let mut optimizer = DebtOptimizer::new(100_000); // 1000 a month
        optimizer.add_debt(Debt {
            name: "High Interest".to_string(),
            balance_cents: 1_000_000, // 10k
            interest_rate_pct: 12, // 1% a month
            min_payment_cents: 10_000, // 100 a month minimum
        });

        // Let's add a second debt so remaining cash routing logic gets executed
        // meaning `remaining_cash -= extra_payment` will actually impact the second debt.
        optimizer.add_debt(Debt {
            name: "Second Debt".to_string(),
            balance_cents: 10_000, // 100
            interest_rate_pct: 12, // 1% a month
            min_payment_cents: 10_000, // 100 a month minimum
        });

        let result = optimizer.simulate(PayoffStrategy::Avalanche);
        // Assert exact duration so if math operators are mutated, it fails.
        // It takes exactly 11 months to pay 10k with 1k/month + 1% interest a year = .01 a month
        assert_eq!(result.total_months, 11);

        // Ensure interest exactly matches expected calculation so we catch `*` -> `+` mutations.
        // And `-=` mutations.
        assert_eq!(result.total_interest_paid_cents, 60138);
    }
}
