//! FIRE (Financial Independence, Retire Early) Simulation Module
//!
//! Provides primitives to project progress toward financial independence
//! by incorporating current expenses, base net worth, and upcoming RSU
//! vests (adjusted for risk via haircut tiers).

use crate::domain::rsu::{HaircutTierTable, forecast_value_cents};

/// Configuration for the FIRE Simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireConfig {
    /// Safe withdrawal rate as a percentage (e.g., 4 for 4%)
    pub safe_withdrawal_rate_pct: u8,
}

impl Default for FireConfig {
    fn default() -> Self {
        Self {
            safe_withdrawal_rate_pct: 4,
        }
    }
}

/// Represents an upcoming RSU vest to be included in the FIRE calculation.
///
/// # Examples
/// ```
/// use logos_core::planning::fire::UpcomingVest;
///
/// let vest = UpcomingVest {
///     avg_close_price_cents: 100_000, // $1,000.00
///     units: 500,                     // 500 shares
///     days_to_vest: 60,               // vesting in 60 days
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpcomingVest {
    /// The average close price per unit in integer cents.
    pub avg_close_price_cents: i64,
    /// The number of units scheduled to vest.
    pub units: u32,
    /// The number of days until the vest date.
    pub days_to_vest: u16,
}

/// A simulator to calculate FIRE (Financial Independence, Retire Early) metrics.
///
/// The `FireSimulator` computes target retirement numbers based on a safe
/// withdrawal rate and monthly expenses. It also factors in risk-adjusted
/// upcoming RSU vests to determine a "safe" net worth and the resulting
/// progress percentage.
///
/// # Examples
/// ```
/// use logos_core::planning::fire::{FireConfig, FireSimulator, UpcomingVest};
///
/// // Create a simulator for $5,000 monthly expenses ($60,000/yr).
/// // The default Safe Withdrawal Rate is 4%, yielding a $1,500,000 FIRE number.
/// let mut sim = FireSimulator::new(500_000);
///
/// // Add current assets and liabilities: $200k assets, $50k liabilities = $150k Base Net Worth
/// sim.add_assets_liabilities(20_000_000, 5_000_000);
///
/// // Add an upcoming RSU vest: 500 units @ $1000 ($500k gross), vesting in 60 days.
/// // At 60 days, the default haircut tier retains 60%, adding $300k safe value.
/// // Total Safe Net Worth = $150k (base) + $300k (RSUs) = $450k.
/// sim.add_upcoming_vest(UpcomingVest {
///     avg_close_price_cents: 100_000,
///     units: 500,
///     days_to_vest: 60,
/// });
///
/// assert_eq!(sim.fire_number_cents(), 150_000_000);
/// assert_eq!(sim.safe_net_worth_cents(), 45_000_000);
/// assert_eq!(sim.fire_progress_pct(), 30); // $450k / $1.5M = 30%
/// ```
#[derive(Debug, Clone)]
pub struct FireSimulator {
    config: FireConfig,
    haircut_tiers: HaircutTierTable,
    monthly_expenses_cents: i64,
    liquid_assets_cents: i64,
    liabilities_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
}

impl FireSimulator {
    /// Creates a new `FireSimulator` with the given monthly expenses.
    ///
    /// By default, this uses a 4% safe withdrawal rate and standard haircut tiers.
    #[must_use]
    pub fn new(monthly_expenses_cents: i64) -> Self {
        Self {
            config: FireConfig::default(),
            haircut_tiers: HaircutTierTable::default(),
            monthly_expenses_cents,
            liquid_assets_cents: 0,
            liabilities_cents: 0,
            upcoming_vests: Vec::new(),
        }
    }

    /// Updates the configuration, such as changing the safe withdrawal rate.
    pub const fn set_config(&mut self, config: FireConfig) {
        self.config = config;
    }

    /// Adds base liquid assets and liabilities to the calculation.
    /// This directly increases the base net worth by `assets_cents - liabilities_cents`.
    pub const fn add_assets_liabilities(&mut self, assets_cents: i64, liabilities_cents: i64) {
        self.liquid_assets_cents += assets_cents;
        self.liabilities_cents += liabilities_cents;
    }

    /// Registers an upcoming RSU vest to be included in the safe net worth.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Calculates the target FIRE number in cents.
    ///
    /// Returns `i64::MAX` if the safe withdrawal rate is configured to 0.
    #[must_use]
    pub fn fire_number_cents(&self) -> i64 {
        if self.config.safe_withdrawal_rate_pct == 0 {
            return i64::MAX;
        }
        let yearly_expenses = self.monthly_expenses_cents.saturating_mul(12);
        yearly_expenses.saturating_mul(100) / i64::from(self.config.safe_withdrawal_rate_pct)
    }

    /// Computes the risk-adjusted "safe" net worth in cents.
    ///
    /// The safe net worth is the sum of liquid assets minus liabilities,
    /// plus the sum of all upcoming vests scaled by their respective haircut tiers.
    #[must_use]
    pub fn safe_net_worth_cents(&self) -> i64 {
        let base_nw = self
            .liquid_assets_cents
            .saturating_sub(self.liabilities_cents);
        let rsu_value: i64 = self
            .upcoming_vests
            .iter()
            .map(|v| {
                forecast_value_cents(
                    v.avg_close_price_cents,
                    v.units,
                    v.days_to_vest,
                    &self.haircut_tiers,
                )
            })
            .sum();
        base_nw.saturating_add(rsu_value)
    }

    /// Returns the progress towards the FIRE number as an integer percentage from 0 to 100.
    #[must_use]
    pub fn fire_progress_pct(&self) -> u8 {
        let fire_num = self.fire_number_cents();
        if fire_num == 0 {
            return 100;
        }
        if fire_num == i64::MAX {
            return 0;
        }
        let nw = self.safe_net_worth_cents();
        if nw <= 0 {
            return 0;
        }
        let pct = (nw.saturating_mul(100)) / fire_num;
        std::cmp::min(100, pct).try_into().unwrap_or(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_number_calculation() {
        // $5000/month expenses = $60,000/year. At 4% SWR, FIRE number is $1,500,000.
        let mut sim = FireSimulator::new(500_000);

        assert_eq!(sim.fire_number_cents(), 150_000_000);

        // At 3% SWR, FIRE number is $2,000,000
        sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 3,
        });
        assert_eq!(sim.fire_number_cents(), 200_000_000);
    }

    #[test]
    fn test_safe_net_worth_with_rsus() {
        let mut sim = FireSimulator::new(500_000);
        sim.add_assets_liabilities(20_000_000, 5_000_000); // 150k base NW

        // Add a vest: 1000 units @ $100 ($100k gross), vesting in 15 days (short tier, 25% haircut by default)
        // Retained = 75%, so $75k safe value.
        // Total Safe NW = 150k + 75k = 225k.
        sim.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 1000,
            days_to_vest: 15,
        });

        assert_eq!(sim.safe_net_worth_cents(), 22_500_000);
    }

    #[test]
    fn test_fire_progress_percentage() {
        // Expenses: $4000/month -> $48,000/year -> $1.2M FIRE number @ 4%
        let mut sim = FireSimulator::new(400_000);

        // Base NW: $300k
        sim.add_assets_liabilities(35_000_000, 5_000_000);

        // Add a vest: 500 units @ $1000 ($500k gross), vesting in 60 days (medium tier, 40% haircut)
        // Retained = 60%, so $300k safe value.
        // Total Safe NW = 300k + 300k = $600k.
        sim.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 100_000,
            units: 500,
            days_to_vest: 60,
        });

        // 600k / 1.2M = 50%
        assert_eq!(sim.fire_progress_pct(), 50);
    }

    #[test]
    fn test_fire_progress_edge_cases() {
        // 1. fire_num == 0
        let sim_zero_expenses = FireSimulator::new(0);
        assert_eq!(sim_zero_expenses.fire_number_cents(), 0);
        assert_eq!(sim_zero_expenses.fire_progress_pct(), 100);

        // 2. safe_withdrawal_rate_pct == 0
        let mut sim_zero_swr = FireSimulator::new(500_000);
        sim_zero_swr.set_config(FireConfig {
            safe_withdrawal_rate_pct: 0,
        });
        assert_eq!(sim_zero_swr.fire_number_cents(), i64::MAX);
        assert_eq!(sim_zero_swr.fire_progress_pct(), 0);

        // 3. nw <= 0
        let mut sim_negative_nw = FireSimulator::new(500_000);
        sim_negative_nw.add_assets_liabilities(0, 10_000_000); // -100k NW
        assert_eq!(sim_negative_nw.fire_progress_pct(), 0);

        // 4. pct exceeds 100
        let mut sim_exceeds = FireSimulator::new(500_000); // fire_num = 1.5M
        sim_exceeds.add_assets_liabilities(200_000_000, 0); // 2M NW
        assert_eq!(sim_exceeds.fire_progress_pct(), 100);
    }
}
