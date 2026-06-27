//! Die With Zero Simulator
//!
//! Calculates how much you can spend per month to exactly draw down your net worth
//! to zero by the time you reach your life expectancy.

use crate::planning::fire::FireSimulator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DieWithZeroResult {
    pub max_monthly_spend_cents: i64,
    pub additional_spend_available_cents: i64,
    pub months_to_live: u32,
}

#[derive(Debug, Clone)]
pub struct DieWithZeroSimulator {
    fire_sim: FireSimulator,
    current_age_years: u8,
    life_expectancy_years: u8,
    annual_growth_rate_pct: f64,
}

impl DieWithZeroSimulator {
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        current_age_years: u8,
        life_expectancy_years: u8,
        annual_growth_rate_pct: f64,
    ) -> Self {
        Self {
            fire_sim,
            current_age_years,
            life_expectancy_years,
            annual_growth_rate_pct,
        }
    }

    #[must_use]
    pub fn calculate(&self) -> DieWithZeroResult {
        let years_to_live = self
            .life_expectancy_years
            .saturating_sub(self.current_age_years);
        let months_to_live = u32::from(years_to_live) * 12;

        if months_to_live == 0 {
            return DieWithZeroResult {
                max_monthly_spend_cents: 0,
                additional_spend_available_cents: 0,
                months_to_live: 0,
            };
        }

        let net_worth = self.fire_sim.safe_net_worth_cents();
        if net_worth <= 0 {
            return DieWithZeroResult {
                max_monthly_spend_cents: 0,
                additional_spend_available_cents: 0,
                months_to_live,
            };
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let max_monthly_spend_cents = if self.annual_growth_rate_pct <= 0.0 {
            net_worth / i64::from(months_to_live)
        } else {
            let monthly_rate = self.annual_growth_rate_pct / 100.0 / 12.0;
            #[allow(
                clippy::suboptimal_flops,
                clippy::cast_possible_truncation,
                clippy::cast_precision_loss
            )]
            let pmt = (net_worth as f64 * monthly_rate)
                / (1.0 - (1.0 + monthly_rate).powf(-(f64::from(months_to_live))));
            pmt as i64
        };

        let current_spend = self.fire_sim.monthly_expenses_cents().max(0);
        let additional_spend = max_monthly_spend_cents.saturating_sub(current_spend);

        DieWithZeroResult {
            max_monthly_spend_cents,
            additional_spend_available_cents: additional_spend,
            months_to_live,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::FireSimulator;

    #[test]
    fn test_basic_die_with_zero() {
        let mut fire_sim = FireSimulator::new(500_000); // 5k monthly spend
        fire_sim.add_assets_liabilities(120_000_000, 0); // 1.2M net worth

        let sim = DieWithZeroSimulator::new(
            fire_sim, 40,  // current age
            90,  // life expectancy
            5.0, // 5% real return
        );

        let result = sim.calculate();

        // 50 years to live = 600 months
        assert_eq!(result.months_to_live, 600);

        // 1.2M over 600 months @ 5% return is approx $5,445/mo
        // Exact formula output: 544966
        // We currently spend $5,000, so we have an extra $449.66/mo
        assert_eq!(result.max_monthly_spend_cents, 544_966);
        assert_eq!(result.additional_spend_available_cents, 44_966);
    }

    #[test]
    fn test_zero_growth() {
        let mut fire_sim = FireSimulator::new(10_000);
        fire_sim.add_assets_liabilities(1_200_000, 0); // 12k
        let sim = DieWithZeroSimulator::new(fire_sim, 50, 60, 0.0);
        let result = sim.calculate();
        // 12k / 120 months = 100/mo
        assert_eq!(result.max_monthly_spend_cents, 10_000);
        assert_eq!(result.additional_spend_available_cents, 0);
    }

    #[test]
    fn test_already_dead_or_dying_soon() {
        let mut fire_sim = FireSimulator::new(10_000);
        fire_sim.add_assets_liabilities(1_000_000, 0);
        let sim = DieWithZeroSimulator::new(fire_sim, 90, 80, 5.0);
        let result = sim.calculate();
        assert_eq!(result.months_to_live, 0);
        assert_eq!(result.max_monthly_spend_cents, 0);
    }
}
