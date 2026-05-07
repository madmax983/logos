#![cfg(feature = "nova")]

//! Emergency Fund Resilience Simulator
//!
//! A mashup of the `AnomalyDetector` and Monte Carlo simulation techniques.
//! It tests if your emergency fund can withstand your historical rate of unexpected
//! financial shocks (anomalies) over a given timeframe.

use crate::domain::transaction::Transaction;
use crate::experimental::anomaly_detector::AnomalyDetector;

/// A simple Linear Congruential Generator for deterministic randomness.
#[derive(Debug, Clone)]
struct Lcg {
    state: u64,
}

impl Lcg {
    const A: u64 = 6_364_136_223_846_793_005;
    const C: u64 = 1_442_695_040_888_963_407;

    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        #[allow(clippy::cast_precision_loss)]
        let result = value as f64 * (1.0 / (1u64 << 53) as f64);
        result
    }
}

/// The result of an emergency fund simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct EmergencyFundResult {
    /// Probability of the emergency fund surviving the full duration without hitting zero (0.0 to 1.0).
    pub survival_probability: f64,
    /// Average number of anomalies encountered across all paths.
    pub avg_anomalies_encountered: f64,
    /// Average cost of the historical anomalies.
    pub avg_historical_anomaly_cost_cents: i64,
}

/// Simulates the resilience of an emergency fund against unexpected expenses.
#[derive(Debug, Clone)]
pub struct EmergencyFundSimulator {
    detector: AnomalyDetector,
    months_of_history: u16,
    emergency_fund_cents: i64,
    monthly_net_cashflow_cents: i64,
    seed: u64,
}

impl EmergencyFundSimulator {
    /// Creates a new `EmergencyFundSimulator`.
    ///
    /// # Arguments
    /// * `detector` - The `AnomalyDetector` to use for identifying historical shocks.
    /// * `months_of_history` - How many months of history the provided transactions cover.
    /// * `emergency_fund_cents` - The current balance of the emergency fund.
    /// * `monthly_net_cashflow_cents` - Baseline monthly cashflow (positive if saving, negative if burning).
    /// * `seed` - Seed for the random number generator.
    #[must_use]
    pub const fn new(
        detector: AnomalyDetector,
        months_of_history: u16,
        emergency_fund_cents: i64,
        monthly_net_cashflow_cents: i64,
        seed: u64,
    ) -> Self {
        Self {
            detector,
            months_of_history,
            emergency_fund_cents,
            monthly_net_cashflow_cents,
            seed,
        }
    }

    /// Simulates the emergency fund's survival over a given number of months and paths.
    /// Uses historical transactions to calibrate the anomaly rate and cost.
    #[must_use]
    pub fn simulate(
        &self,
        transactions: &[Transaction],
        projection_months: u16,
        paths: u32,
    ) -> EmergencyFundResult {
        let anomalies = self.detector.detect(transactions);

        #[allow(clippy::useless_let_if_seq)]
        let mut avg_historical_anomaly_cost_cents = 0;
        if !anomalies.is_empty() {
            let total: i64 = anomalies.iter().map(|a| a.amount_cents).sum();
            #[allow(clippy::cast_possible_wrap)]
            let count = anomalies.len() as i64;
            avg_historical_anomaly_cost_cents = total / count;
        }

        // Monthly probability of an anomaly occurring
        let monthly_anomaly_prob = if self.months_of_history > 0 {
            #[allow(clippy::cast_precision_loss)]
            let count = anomalies.len() as f64;
            count / f64::from(self.months_of_history)
        } else {
            0.0
        };

        if paths == 0 {
            return EmergencyFundResult {
                survival_probability: 1.0,
                avg_anomalies_encountered: 0.0,
                avg_historical_anomaly_cost_cents,
            };
        }

        let mut lcg = Lcg::new(self.seed);
        let mut survived_paths = 0;
        let mut total_anomalies_encountered = 0;

        for _ in 0..paths {
            let mut current_fund = self.emergency_fund_cents;
            let mut path_anomalies = 0;
            let mut survived = true;

            for _month_idx in 0..projection_months {
                // Baseline cashflow (replenish or burn)
                current_fund = current_fund.saturating_add(self.monthly_net_cashflow_cents);

                // Random anomaly shock
                let chance = lcg.next_f64();

                if chance < monthly_anomaly_prob && !anomalies.is_empty() {
                    path_anomalies += 1;

                    #[allow(
                        clippy::cast_precision_loss,
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss
                    )]
                    let idx = (lcg.next_f64() * (anomalies.len() as f64)) as usize;

                    // Clamp just in case
                    let safe_idx = idx.min(anomalies.len().saturating_sub(1));
                    let cost = anomalies[safe_idx].amount_cents;

                    current_fund = current_fund.saturating_sub(cost);
                } else if !anomalies.is_empty() {
                    // Match the exactly two PRNG calls if it had been an anomaly
                    let _ = lcg.next_f64();
                }

                if current_fund <= 0 {
                    survived = false;
                }
            }

            if survived {
                survived_paths += 1;
            }
            total_anomalies_encountered += path_anomalies;
        }

        EmergencyFundResult {
            survival_probability: f64::from(survived_paths) / f64::from(paths),
            avg_anomalies_encountered: f64::from(total_anomalies_encountered) / f64::from(paths),
            avg_historical_anomaly_cost_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_emergency_fund_simulation() {
        let detector = AnomalyDetector::new(1.5);

        let mut transactions = Vec::new();

        // 20 normal transactions of $1,000 to set the IQR properly
        for _ in 0..20 {
            let tx = TransactionBuilder::new("Normal")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), 100_000).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 100_000).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 2 anomaly transactions of $10,000
        for _ in 0..2 {
            let tx = TransactionBuilder::new("Emergency")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), 1_000_000).unwrap(),
                )
                .posting(
                    Posting::debit(AccountId::new("expenses:food").unwrap(), 1_000_000).unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 2 anomalies over 12 months = 1/6 probability per month. Average cost = 1_000_000.
        let sim = EmergencyFundSimulator::new(
            detector, 12,      // 12 months of history
            500_000, // small emergency fund
            0,       // 0 net cashflow
            42,      // Seed
        );

        let result = sim.simulate(&transactions, 60, 1000); // Project 60 months, 1000 paths

        // Survival should be very low because fund is small
        assert!(result.survival_probability < 0.5);
        assert_eq!(result.avg_historical_anomaly_cost_cents, 1_000_000);
        assert!(result.avg_anomalies_encountered > 0.0);
    }

    #[test]
    fn test_emergency_fund_safe() {
        let detector = AnomalyDetector::new(1.5);
        let mut transactions = Vec::new();

        // 20 normal transactions of $1,000
        for _ in 0..20 {
            let tx = TransactionBuilder::new("Normal")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), 100_000).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 100_000).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        let tx = TransactionBuilder::new("Emergency")
            .posting(
                Posting::credit(AccountId::new("assets:checking").unwrap(), 1_000_000).unwrap(),
            )
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 1_000_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let sim = EmergencyFundSimulator::new(
            detector, 120,        // 120 months of history (very rare anomaly)
            10_000_000, // $100k emergency fund
            100_000,    // saving $1k/mo
            42,
        );

        let result = sim.simulate(&transactions, 12, 1000);

        // Highly likely to survive
        assert!(result.survival_probability > 0.9);
        assert_eq!(result.avg_historical_anomaly_cost_cents, 1_000_000);
    }

    #[test]
    fn test_zero_paths() {
        let detector = AnomalyDetector::new(1.5);
        let sim = EmergencyFundSimulator::new(detector, 12, 5000, 0, 42);
        let result = sim.simulate(&[], 12, 0);
        assert!((result.survival_probability - 1.0).abs() < f64::EPSILON);
        assert!((result.avg_anomalies_encountered - 0.0).abs() < f64::EPSILON);
    }
}
