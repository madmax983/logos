#![cfg(feature = "nova")]
//! Geo Arbitrage Simulator
//!
//! Evaluates the financial impact of relocating to a different geographic location.
//! Compares the "stay" scenario vs the "move" scenario, incorporating cost of living differences,
//! salary adjustments, and moving costs.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationProfile {
    pub name: String,
    pub monthly_salary_cents: i64,
    pub monthly_expenses_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelocationCost {
    pub total_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoArbitrageResult {
    pub breakeven_months: Option<u32>,
    pub five_year_net_benefit_cents: i64,
}

#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator;

impl GeoArbitrageSimulator {
    #[must_use]
    pub const fn evaluate(
        current: &LocationProfile,
        target: &LocationProfile,
        moving_cost: &RelocationCost,
    ) -> GeoArbitrageResult {
        let current_monthly_savings = current
            .monthly_salary_cents
            .saturating_sub(current.monthly_expenses_cents);
        let target_monthly_savings = target
            .monthly_salary_cents
            .saturating_sub(target.monthly_expenses_cents);

        let monthly_delta = target_monthly_savings.saturating_sub(current_monthly_savings);

        let breakeven_months = if monthly_delta > 0 {
            let cost = moving_cost.total_cents;
            if cost <= 0 {
                Some(0)
            } else {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some(((cost + monthly_delta - 1) / monthly_delta) as u32) // Ceiling division
            }
        } else {
            None
        };

        let current_5yr = current_monthly_savings.saturating_mul(60);
        let target_5yr = target_monthly_savings
            .saturating_mul(60)
            .saturating_sub(moving_cost.total_cents);
        let net_benefit = target_5yr.saturating_sub(current_5yr);

        GeoArbitrageResult {
            breakeven_months,
            five_year_net_benefit_cents: net_benefit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profitable_move() {
        let current = LocationProfile {
            name: "San Francisco".to_string(),
            monthly_salary_cents: 1_500_000,
            monthly_expenses_cents: 800_000, // $7k savings/mo
        };
        let target = LocationProfile {
            name: "Austin".to_string(),
            monthly_salary_cents: 1_300_000,
            monthly_expenses_cents: 400_000, // $9k savings/mo
        };
        let cost = RelocationCost {
            total_cents: 1_000_000,
        };

        let result = GeoArbitrageSimulator::evaluate(&current, &target, &cost);

        // Extra savings = $2k/mo. Moving cost = $10k. Breakeven = 5 months.
        assert_eq!(result.breakeven_months, Some(5));

        // 5 years = 60 months.
        // Current savings in 60 mo = 60 * 7k = 420k
        // Target savings in 60 mo = 60 * 9k - 10k = 540k - 10k = 530k
        // Net benefit = 530k - 420k = 110k
        assert_eq!(result.five_year_net_benefit_cents, 11_000_000);
    }

    #[test]
    fn test_unprofitable_move() {
        let current = LocationProfile {
            name: "Remote".to_string(),
            monthly_salary_cents: 1_000_000,
            monthly_expenses_cents: 300_000, // $7k savings/mo
        };
        let target = LocationProfile {
            name: "NYC".to_string(),
            monthly_salary_cents: 1_200_000,
            monthly_expenses_cents: 600_000, // $6k savings/mo
        };
        let cost = RelocationCost {
            total_cents: 500_000,
        };

        let result = GeoArbitrageSimulator::evaluate(&current, &target, &cost);

        assert_eq!(result.breakeven_months, None);

        // 60 months. Current savings: 60 * 7k = 420k
        // Target savings: 60 * 6k - 5k = 355k
        // Net benefit = 355k - 420k = -65k
        assert_eq!(result.five_year_net_benefit_cents, -6_500_000);
    }

    #[test]
    fn test_zero_moving_cost_profitable() {
        let current = LocationProfile {
            name: "A".to_string(),
            monthly_salary_cents: 1000,
            monthly_expenses_cents: 500, // 500 savings
        };
        let target = LocationProfile {
            name: "B".to_string(),
            monthly_salary_cents: 1200,
            monthly_expenses_cents: 600, // 600 savings
        };
        let cost = RelocationCost { total_cents: 0 };
        let result = GeoArbitrageSimulator::evaluate(&current, &target, &cost);
        assert_eq!(result.breakeven_months, Some(0));
        assert_eq!(result.five_year_net_benefit_cents, 6000); // 100 * 60
    }
}
