//! Portfolio Stress Tester
//!
//! A simulator to stress-test your current portfolio against historical
//! market crash scenarios to determine immediate drawdown and estimated recovery time.

/// Pre-defined historical market crash scenarios.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrashScenario {
    /// 1929 Great Crash (-89%)
    GreatDepression,
    /// 1987 Black Monday (-22.6% in one day, but let's use peak-to-trough -33%)
    BlackMonday,
    /// 2000 Dot-Com Bubble (-49%)
    DotComBubble,
    /// 2008 Great Recession (-57%)
    GreatRecession,
    /// 2020 COVID-19 Crash (-33%)
    Covid19,
    /// Custom drawdown percentage
    Custom(f64),
}

impl CrashScenario {
    /// Returns the peak-to-trough drawdown percentage as a positive float (e.g., 0.57 for 57%).
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn drawdown_pct(&self) -> f64 {
        match self {
            Self::GreatDepression => 0.89,
            Self::BlackMonday => 0.33,
            Self::DotComBubble => 0.49,
            Self::GreatRecession => 0.57,
            Self::Covid19 => 0.33,
            Self::Custom(pct) => *pct,
        }
    }

    /// Returns the name of the scenario.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::GreatDepression => "Great Depression (1929)",
            Self::BlackMonday => "Black Monday (1987)",
            Self::DotComBubble => "Dot-Com Bubble (2000)",
            Self::GreatRecession => "Great Recession (2008)",
            Self::Covid19 => "COVID-19 Crash (2020)",
            Self::Custom(_) => "Custom Scenario",
        }
    }
}

/// The result of a stress test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressTestResult {
    /// The name of the scenario applied.
    pub scenario_name: String,
    /// The portfolio value before the crash.
    pub initial_portfolio_cents: i64,
    /// The absolute lowest point of the portfolio during the crash.
    pub trough_portfolio_cents: i64,
    /// Total amount lost in cents.
    pub loss_cents: i64,
    /// Estimated months to recover to the initial portfolio value, assuming a standard growth rate.
    pub months_to_recover: u32,
}

/// A stress tester for financial portfolios.
#[derive(Debug, Clone)]
pub struct PortfolioStressTester {
    initial_portfolio_cents: i64,
    monthly_contribution_cents: i64,
    annual_recovery_return: f64,
}

impl PortfolioStressTester {
    /// Creates a new `PortfolioStressTester`.
    ///
    /// * `initial_portfolio_cents`: Current value of the portfolio.
    /// * `monthly_contribution_cents`: Expected monthly savings during the recovery period.
    /// * `annual_recovery_return`: Expected annual growth rate during recovery (e.g., 0.07 for 7%).
    #[must_use]
    pub const fn new(
        initial_portfolio_cents: i64,
        monthly_contribution_cents: i64,
        annual_recovery_return: f64,
    ) -> Self {
        Self {
            initial_portfolio_cents,
            monthly_contribution_cents,
            annual_recovery_return,
        }
    }

    /// Applies a specific crash scenario and calculates the drawdown and recovery time.
    #[must_use]
    pub fn apply_scenario(&self, scenario: CrashScenario) -> StressTestResult {
        let drawdown = scenario.drawdown_pct();

        #[allow(clippy::cast_precision_loss)]
        let initial_f64 = self.initial_portfolio_cents as f64;

        let loss_f64 = initial_f64 * drawdown;

        #[allow(clippy::cast_possible_truncation)]
        let loss_cents = loss_f64.round() as i64;

        let trough_portfolio_cents = self
            .initial_portfolio_cents
            .saturating_sub(loss_cents)
            .max(0);
        let actual_loss_cents = self.initial_portfolio_cents - trough_portfolio_cents;

        let months_to_recover = self.calculate_recovery_months(trough_portfolio_cents);

        StressTestResult {
            scenario_name: scenario.name().to_string(),
            initial_portfolio_cents: self.initial_portfolio_cents,
            trough_portfolio_cents,
            loss_cents: actual_loss_cents,
            months_to_recover,
        }
    }

    fn calculate_recovery_months(&self, mut current_portfolio: i64) -> u32 {
        if current_portfolio >= self.initial_portfolio_cents {
            return 0;
        }

        let monthly_rate = self.annual_recovery_return / 12.0;
        let mut months = 0;

        // Cap recovery at 100 years (1200 months) to prevent infinite loops
        // if growth and contributions are 0.
        while current_portfolio < self.initial_portfolio_cents && months < 1200 {
            months += 1;

            #[allow(clippy::cast_precision_loss)]
            let current_f64 = current_portfolio as f64;

            let gain = current_f64 * monthly_rate;

            #[allow(clippy::cast_possible_truncation)]
            let gain_cents = gain.round() as i64;

            current_portfolio = current_portfolio
                .saturating_add(gain_cents)
                .saturating_add(self.monthly_contribution_cents);
        }

        months
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_great_recession_recovery() {
        // $100k portfolio, contributing $1k/mo, 7% recovery return
        let tester = PortfolioStressTester::new(10_000_000, 100_000, 0.07);
        let result = tester.apply_scenario(CrashScenario::GreatRecession);

        assert_eq!(result.scenario_name, "Great Recession (2008)");
        assert_eq!(result.initial_portfolio_cents, 10_000_000);
        // 57% drawdown -> $43k remaining
        assert_eq!(result.trough_portfolio_cents, 4_300_000);
        assert_eq!(result.loss_cents, 5_700_000);

        // At 7% growth and $1k/mo, it should take a few years to get back to $100k
        assert!(result.months_to_recover > 12);
        assert!(result.months_to_recover < 60);
    }

    #[test]
    fn test_zero_recovery_return_and_contributions() {
        // If we lose 50% and have 0 growth/contribution, we never recover.
        let tester = PortfolioStressTester::new(10_000_000, 0, 0.0);
        let result = tester.apply_scenario(CrashScenario::Custom(0.5));

        assert_eq!(result.trough_portfolio_cents, 5_000_000);
        // Should hit the 1200 month cap
        assert_eq!(result.months_to_recover, 1200);
    }

    #[test]
    fn test_covid_crash() {
        let tester = PortfolioStressTester::new(10_000_000, 0, 0.12);
        let result = tester.apply_scenario(CrashScenario::Covid19);

        assert_eq!(result.loss_cents, 3_300_000); // 33% of 10M
    }
}
