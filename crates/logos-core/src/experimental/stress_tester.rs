use crate::planning::fire::FireSimulator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressTestResult {
    pub survived: bool,
    pub remaining_assets_cents: i64,
    pub runway_after_shock_months: u32,
}

/// A simulator that stress-tests a financial FIRE plan against "Black Swan" events.
#[derive(Debug, Clone)]
pub struct FinancialStressTester {
    pub market_drop_pct: u8,
    pub unemployment_months: u32,
}

impl FinancialStressTester {
    #[must_use]
    pub const fn new(market_drop_pct: u8, unemployment_months: u32) -> Self {
        Self {
            market_drop_pct,
            unemployment_months,
        }
    }

    #[must_use]
    pub fn test(&self, fire_sim: &FireSimulator) -> StressTestResult {
        let initial_assets = std::cmp::max(0, fire_sim.safe_net_worth_cents());
        let expenses = fire_sim.monthly_expenses_cents();

        let drop_pct = std::cmp::min(100, self.market_drop_pct);
        let retained_pct = 100_i64.saturating_sub(i64::from(drop_pct));
        let assets_after_drop = (initial_assets.saturating_mul(retained_pct)) / 100;

        let unemployment_cost = expenses.saturating_mul(i64::from(self.unemployment_months));
        let remaining_assets = assets_after_drop.saturating_sub(unemployment_cost);
        let survived = remaining_assets > 0;

        let runway_after_shock_months = if expenses > 0 && remaining_assets > 0 {
            u32::try_from(remaining_assets / expenses).unwrap_or(u32::MAX)
        } else if remaining_assets > 0 {
            u32::MAX
        } else {
            0
        };

        StressTestResult {
            survived,
            remaining_assets_cents: remaining_assets,
            runway_after_shock_months,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::FireSimulator;

    #[test]
    fn test_financial_stress() {
        let mut fire_sim = FireSimulator::new(500_000); // $5k / mo expenses
        fire_sim.add_assets_liabilities(20_000_000, 0); // $200k assets

        // 20% market drop, 6 months unemployment
        let tester = FinancialStressTester::new(20, 6);
        let result = tester.test(&fire_sim);

        // Assets after 20% drop = $160k
        // 6 months expenses @ $5k = $30k
        // Remaining = $130k
        // Runway = $130k / $5k = 26 months

        assert!(result.survived);
        assert_eq!(result.remaining_assets_cents, 13_000_000);
        assert_eq!(result.runway_after_shock_months, 26);
    }
}
