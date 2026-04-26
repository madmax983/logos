//! FIRE (Financial Independence, Retire Early) Simulation Module
//!
//! # The Great Escape
//!
//! This module answers the ultimate question: *When can I stop working?*
//!
//! It provides the mathematical crystal ball needed to project your progress toward financial
//! independence. Instead of just looking at current cash, it incorporates your burn rate (monthly expenses),
//! your war chest (base net worth), and the future promises of your unvested RSUs. Because future RSUs
//! are risky, they are adjusted using haircut tiers to ensure your projections stay grounded in reality.
//!
//! Provides primitives to project progress toward financial independence
//! by incorporating current expenses, base net worth, and upcoming RSU
//! vests (adjusted for risk via haircut tiers).

use crate::domain::rsu::{HaircutTierTable, forecast_value_cents};

/// Configuration for the FIRE Simulator.
///
/// Defines the rulebook for your retirement math. The most critical lever here is the
/// Safe Withdrawal Rate, which dictates how large your war chest needs to be to sustain
/// your lifestyle indefinitely without running dry.
///
/// # Examples
/// ```
/// use logos_core::planning::fire::FireConfig;
///
/// let config = FireConfig { safe_withdrawal_rate_pct: 3 };
/// ```
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
/// The [`FireSimulator`] is your financial co-pilot. It computes your target retirement
/// number based on a safe withdrawal rate and your monthly expenses (because you can't manage
/// what you don't measure). It also factors in risk-adjusted upcoming RSU vests to determine a "safe"
/// net worth, and ultimately calculates the percentage of the journey you've completed.
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
    /// Ignites a new [`FireSimulator`] based on your monthly burn rate.
    ///
    /// The simulator needs to know how much cash you bleed each month to calculate your target.
    /// By default, it assumes a 4% safe withdrawal rate—the classic Trinity study baseline—and
    /// standard risk haircuts for any RSUs you add later.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// // Simulator for $5,000 monthly expenses ($60,000/yr)
    /// let sim = FireSimulator::new(500_000);
    /// ```
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

    /// Alters the core assumptions of the simulation.
    ///
    /// Use this if you want to be more conservative (e.g., dropping the safe withdrawal rate
    /// to 3% because you plan to live to 150) or if the economic winds have shifted.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::{FireConfig, FireSimulator};
    ///
    /// let mut sim = FireSimulator::new(500_000);
    /// sim.set_config(FireConfig { safe_withdrawal_rate_pct: 3 });
    /// ```
    pub const fn set_config(&mut self, config: FireConfig) {
        self.config = config;
    }

    /// Returns the current configuration.
    #[must_use]
    pub const fn config(&self) -> FireConfig {
        self.config
    }


    /// Injects your current liquid reality into the simulation.
    ///
    /// Before calculating how far you have left to go, we need to know where you are starting from.
    /// This directly increases your base net worth by subtracting the liabilities from the assets.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// let mut sim = FireSimulator::new(500_000);
    /// // Add $200k in assets, $50k in liabilities
    /// sim.add_assets_liabilities(20_000_000, 5_000_000);
    /// ```
    pub const fn add_assets_liabilities(&mut self, assets_cents: i64, liabilities_cents: i64) {
        self.liquid_assets_cents += assets_cents;
        self.liabilities_cents += liabilities_cents;
    }

    /// Registers an upcoming RSU vest to be included in the safe net worth.
    ///
    /// Vests are risk-adjusted based on the number of days until the vest date
    /// and the configured [`HaircutTierTable`].
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::{FireSimulator, UpcomingVest};
    ///
    /// let mut sim = FireSimulator::new(500_000);
    ///
    /// // Add a vest of 500 units @ $1000 each ($500k gross) in 60 days
    /// sim.add_upcoming_vest(UpcomingVest {
    ///     avg_close_price_cents: 100_000,
    ///     units: 500,
    ///     days_to_vest: 60,
    /// });
    /// ```
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Reveals the mountain peak: your target FIRE number in cents.
    ///
    /// This is the absolute dollar amount required to generate enough passive income
    /// to cover your monthly burn rate indefinitely, according to your safe withdrawal rate.
    ///
    /// Returns `i64::MAX` if the safe withdrawal rate is dangerously configured to 0.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// // $5000/month expenses = $60,000/yr.
    /// // At 4% safe withdrawal rate, target is $1.5M.
    /// let sim = FireSimulator::new(500_000);
    /// assert_eq!(sim.fire_number_cents(), 150_000_000);
    /// ```
    #[must_use]
    pub fn fire_number_cents(&self) -> i64 {
        if self.config.safe_withdrawal_rate_pct == 0 {
            return i64::MAX;
        }
        let yearly_expenses = self.monthly_expenses_cents.saturating_mul(12);
        yearly_expenses.saturating_mul(100) / i64::from(self.config.safe_withdrawal_rate_pct)
    }

    /// Distills your total financial picture into a single, risk-adjusted "safe" net worth.
    ///
    /// It combines your cold, hard liquid reality (assets minus liabilities) with the
    /// risk-discounted value of your upcoming stock awards. This is the number you should
    /// actually trust when deciding if you can quit your job tomorrow.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// let mut sim = FireSimulator::new(500_000);
    /// sim.add_assets_liabilities(20_000_000, 5_000_000); // $150k base NW
    /// assert_eq!(sim.safe_net_worth_cents(), 15_000_000);
    /// ```
    /// Returns the configured monthly expenses in cents.
    #[must_use]
    pub const fn monthly_expenses_cents(&self) -> i64 {
        self.monthly_expenses_cents
    }

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
            .fold(0_i64, i64::saturating_add);
        base_nw.saturating_add(rsu_value)
    }

    /// Returns the progress towards the FIRE number as an integer percentage from 0 to 100.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number
    /// sim.add_assets_liabilities(30_000_000, 0); // $300k NW
    ///
    /// assert_eq!(sim.fire_progress_pct(), 20); // 300k / 1.5M = 20%
    /// ```
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

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn havoc_safe_net_worth_cents_panics_on_overflow(
            units in 100_000..200_000_u32,
        ) {
            let mut sim = FireSimulator::new(500_000);

            // `sum()` over i64 will panic if the accumulated value overflows.
            // But we must be careful: if `forecast_value_cents` overflows internally,
            // it catches it with `checked_mul` and returns 0!

            // forecast_value_cents calculates: gross = avg * units.
            // Then it does: gross.checked_mul(retained_pct)
            // So we need:
            // 1) avg * units < i64::MAX
            // 2) (avg * units) * 75 < i64::MAX
            // So avg * units < i64::MAX / 75.

            // Let's set gross = i64::MAX / 100. (so it's < i64::MAX / 75)
            // Then forecast returns (i64::MAX / 100) * 75 / 100

            let gross = i64::MAX / 100;
            let avg_close_price_cents = gross / i64::from(units);

            // Each forecast gives ~ 0.0075 * i64::MAX
            // We need more than 1 / 0.0075 = 133 vests to overflow `sum()`
            // Let's add 200 vests.

            for _ in 0..200 {
                sim.add_upcoming_vest(UpcomingVest {
                    avg_close_price_cents,
                    units,
                    days_to_vest: 15,
                });
            }

            let _ = sim.safe_net_worth_cents();
        }
    }

    #[test]
    fn should_handle_progress_pct_below_zero_safe_net_worth() {
        let mut sim = FireSimulator::new(500_000);
        sim.add_assets_liabilities(0, 10_000_000);
        assert_eq!(sim.fire_progress_pct(), 0);
    }
}
