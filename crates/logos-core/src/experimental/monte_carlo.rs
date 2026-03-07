use crate::planning::fire::FireSimulator;

/// Represents the results of a Monte Carlo simulation for FIRE projections.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResult {
    /// The lowest final net worth across all iterations.
    pub min_net_worth_cents: i64,
    /// The highest final net worth across all iterations.
    pub max_net_worth_cents: i64,
    /// The average final net worth across all iterations.
    pub avg_net_worth_cents: i64,
    /// The probability (0.0 to 1.0) of reaching or exceeding the FIRE number.
    pub success_probability: f64,
}

/// A simulator that uses Monte Carlo methods to project the probability of reaching FIRE.
///
/// It combines a baseline `FireSimulator` with randomized monthly market returns to simulate
/// multiple possible future paths over a given time horizon.
#[derive(Debug, Clone)]
pub struct MonteCarloSimulator {
    base_simulator: FireSimulator,
    monthly_savings_cents: i64,
}

impl MonteCarloSimulator {
    /// Creates a new `MonteCarloSimulator` using a given `FireSimulator` for the baseline.
    #[must_use]
    pub const fn new(base_simulator: FireSimulator, monthly_savings_cents: i64) -> Self {
        Self {
            base_simulator,
            monthly_savings_cents,
        }
    }

    /// Simulates the progress towards the FIRE number over a set number of months.
    ///
    /// For each iteration, a random market return is applied each month.
    ///
    /// * `months`: The number of months to simulate into the future.
    /// * `iterations`: The number of parallel simulation paths to run.
    /// * `mean_monthly_return`: The expected average monthly return (e.g., 0.005 for 0.5%).
    /// * `std_dev_monthly`: The standard deviation of the monthly return (e.g., 0.04 for 4%).
    #[must_use]
    pub fn simulate(
        &self,
        months: u16,
        iterations: u32,
        mean_monthly_return: f64,
        std_dev_monthly: f64,
    ) -> SimulationResult {
        if iterations == 0 {
            return SimulationResult {
                min_net_worth_cents: 0,
                max_net_worth_cents: 0,
                avg_net_worth_cents: 0,
                success_probability: 0.0,
            };
        }

        let start_nw = self.base_simulator.safe_net_worth_cents();
        let target_nw = self.base_simulator.fire_number_cents();

        let mut min_nw = i64::MAX;
        let mut max_nw = i64::MIN;
        let mut sum_nw: f64 = 0.0;
        let mut success_count = 0;

        for i in 0..iterations {
            #[allow(clippy::cast_precision_loss)]
            let mut current_nw = start_nw as f64;

            // Generate a simple pseudo-random sequence for demonstration
            // We'll use a deterministic LCG to avoid adding external rand dependencies
            let mut seed = u64::from(i)
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);

            for _ in 0..months {
                // Box-Muller transform for normal distribution
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                #[allow(clippy::cast_precision_loss)]
                let u1 = (seed as f64) / (u64::MAX as f64);
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                #[allow(clippy::cast_precision_loss)]
                let u2 = (seed as f64) / (u64::MAX as f64);

                // Prevent log(0)
                let u1 = if u1 <= 0.0 { 0.000_000_001 } else { u1 };

                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                let monthly_return = mean_monthly_return + z0 * std_dev_monthly;

                #[allow(clippy::cast_precision_loss)]
                let savings_f64 = self.monthly_savings_cents as f64;
                current_nw = current_nw.mul_add(1.0 + monthly_return, savings_f64);
            }

            #[allow(clippy::cast_possible_truncation)]
            let final_nw = current_nw.round() as i64;

            if final_nw < min_nw {
                min_nw = final_nw;
            }
            if final_nw > max_nw {
                max_nw = final_nw;
            }
            sum_nw += current_nw;

            if final_nw >= target_nw {
                success_count += 1;
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let avg_net_worth_cents = (sum_nw / f64::from(iterations)).round() as i64;

        SimulationResult {
            min_net_worth_cents: min_nw,
            max_net_worth_cents: max_nw,
            avg_net_worth_cents,
            success_probability: f64::from(success_count) / f64::from(iterations),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::FireSimulator;

    #[test]
    fn test_zero_iterations() {
        let base_sim = FireSimulator::new(500_000);
        let mc = MonteCarloSimulator::new(base_sim, 10_000);

        let result = mc.simulate(12, 0, 0.0, 0.0);

        assert_eq!(result.min_net_worth_cents, 0);
        assert_eq!(result.max_net_worth_cents, 0);
        assert_eq!(result.avg_net_worth_cents, 0);
        assert!((result.success_probability - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deterministic_growth() {
        // With 0 std_dev, it's just compound interest + savings
        let mut base_sim = FireSimulator::new(500_000); // 1.5M FIRE number
        base_sim.add_assets_liabilities(100_000_000, 0); // 1M start NW

        let mc = MonteCarloSimulator::new(base_sim, 500_000); // $5k/mo savings

        let result = mc.simulate(12, 1, 0.0, 0.0);

        // 1M + 12 * 5k = 1,060,000 (106M cents)
        assert_eq!(result.min_net_worth_cents, 106_000_000);
        assert_eq!(result.max_net_worth_cents, 106_000_000);
        assert_eq!(result.avg_net_worth_cents, 106_000_000);
        assert!((result.success_probability - 0.0).abs() < f64::EPSILON); // Not hit 1.5M
    }

    #[test]
    fn test_success_probability() {
        let mut base_sim = FireSimulator::new(500_000); // 1.5M FIRE number
        base_sim.add_assets_liabilities(149_000_000, 0); // Start very close to 1.5M

        let mc = MonteCarloSimulator::new(base_sim, 0);

        // 50% chance to go up, 50% chance to go down with mean 0
        let result = mc.simulate(1, 1000, 0.0, 0.1);

        assert!(result.success_probability > 0.0 && result.success_probability < 1.0);
        assert!(result.min_net_worth_cents < 149_000_000);
        assert!(result.max_net_worth_cents > 149_000_000);
    }
}
