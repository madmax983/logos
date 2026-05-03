#![cfg(feature = "nova")]

//! Black Swan FIRE Simulator
//!
//! A mashup between `FireSimulator` and `AnomalyDetector`.
//! It detects historical anomalous expenses and risk-adjusts your FIRE target
//! by assuming a certain number of these "Black Swans" will happen during your retirement.

use crate::domain::transaction::Transaction;
use crate::experimental::anomaly_detector::AnomalyDetector;
use crate::planning::fire::FireSimulator;

/// The result of a risk-adjusted FIRE calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlackSwanFireResult {
    /// The original FIRE target in cents.
    pub base_fire_number_cents: i64,
    /// The number of anomalies detected in the provided history.
    pub anomalies_detected: usize,
    /// The total cost of detected anomalies in cents.
    pub total_anomaly_cost_cents: i64,
    /// The adjusted FIRE target accounting for future anomalies.
    pub risk_adjusted_fire_number_cents: i64,
}

/// A simulator that risk-adjusts FIRE targets based on historical outliers.
#[derive(Debug, Clone)]
pub struct BlackSwanFireSimulator {
    fire_sim: FireSimulator,
    anomaly_detector: AnomalyDetector,
    /// How many years of historical transactions are provided.
    history_years: f64,
    /// How many years to project anomalies into the future (e.g., retirement length).
    retirement_years: u8,
}

impl BlackSwanFireSimulator {
    /// Creates a new `BlackSwanFireSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator.
    /// * `anomaly_detector` - The detector used to find historical outliers.
    /// * `history_years` - The timespan of the provided transactions in years.
    /// * `retirement_years` - How long to project anomalies into the future.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        anomaly_detector: AnomalyDetector,
        history_years: f64,
        retirement_years: u8,
    ) -> Self {
        Self {
            fire_sim,
            anomaly_detector,
            history_years,
            retirement_years,
        }
    }

    /// Calculates the risk-adjusted FIRE number based on historical transactions.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn calculate(&self, transactions: &[Transaction]) -> BlackSwanFireResult {
        let base_fire = self.fire_sim.fire_number_cents();

        if base_fire == 0 || base_fire == i64::MAX || self.history_years <= 0.0 {
            return BlackSwanFireResult {
                base_fire_number_cents: base_fire,
                anomalies_detected: 0,
                total_anomaly_cost_cents: 0,
                risk_adjusted_fire_number_cents: base_fire,
            };
        }

        let anomalies = self.anomaly_detector.detect(transactions);
        let mut total_anomaly_cost: i64 = 0;

        for anomaly in &anomalies {
            total_anomaly_cost = total_anomaly_cost.saturating_add(anomaly.amount_cents);
        }

        // Calculate average annual anomaly cost
        let annual_anomaly_cost = (total_anomaly_cost as f64 / self.history_years).round() as i64;

        // Multiply by retirement years to get the extra buffer needed
        let total_future_buffer =
            annual_anomaly_cost.saturating_mul(i64::from(self.retirement_years));

        BlackSwanFireResult {
            base_fire_number_cents: base_fire,
            anomalies_detected: anomalies.len(),
            total_anomaly_cost_cents: total_anomaly_cost,
            risk_adjusted_fire_number_cents: base_fire.saturating_add(total_future_buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};
    use crate::planning::fire::FireConfig;

    #[test]
    fn test_black_swan_fire() {
        let mut fire_sim = FireSimulator::new(500_000); // 5k/mo = 60k/yr -> 1.5M FIRE number
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });

        let detector = AnomalyDetector::new(1.5);

        // 2 years of history, 30 year retirement
        let sim = BlackSwanFireSimulator::new(fire_sim, detector, 2.0, 30);

        let mut transactions = Vec::new();

        // Normal amounts
        // Note: IQR calculation needs at least 4 items to be accurate, and since it calculates on all
        // passing transactions, we need to dilute the outliers to avoid shifting the IQR bounds too much.
        let amounts = vec![
            1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000, 2100, 2200, 2300,
            2400,
        ];

        for amount in amounts {
            let tx = TransactionBuilder::new("Groceries")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), amount).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // Black Swan 1: Medical Emergency ($10k)
        let tx = TransactionBuilder::new("Medical")
            .posting(
                Posting::credit(AccountId::new("assets:checking").unwrap(), 1_000_000).unwrap(),
            )
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 1_000_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        // Black Swan 2: Roof Replacement ($20k)
        let tx = TransactionBuilder::new("Roof")
            .posting(
                Posting::credit(AccountId::new("assets:checking").unwrap(), 2_000_000).unwrap(),
            )
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 2_000_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let result = sim.calculate(&transactions);

        assert_eq!(result.base_fire_number_cents, 150_000_000);
        assert_eq!(result.anomalies_detected, 2);
        assert_eq!(result.total_anomaly_cost_cents, 3_000_000);

        // Average annual anomaly = 3M / 2 years = 1.5M
        // Total buffer for 30 years = 1.5M * 30 = 45M
        // Risk adjusted FIRE = 150M + 45M = 195M
        assert_eq!(result.risk_adjusted_fire_number_cents, 195_000_000);
    }

    #[test]
    fn test_zero_base_fire() {
        let fire_sim = FireSimulator::new(0);
        let detector = AnomalyDetector::new(1.5);
        let sim = BlackSwanFireSimulator::new(fire_sim, detector, 2.0, 30);
        let result = sim.calculate(&[]);

        assert_eq!(result.base_fire_number_cents, 0);
        assert_eq!(result.risk_adjusted_fire_number_cents, 0);
    }

    #[test]
    fn test_max_base_fire() {
        let mut fire_sim = FireSimulator::new(500_000);
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 0,
        });
        let detector = AnomalyDetector::new(1.5);
        let sim = BlackSwanFireSimulator::new(fire_sim, detector, 2.0, 30);
        let result = sim.calculate(&[]);

        assert_eq!(result.base_fire_number_cents, i64::MAX);
        assert_eq!(result.risk_adjusted_fire_number_cents, i64::MAX);
    }

    #[test]
    fn test_zero_history_years() {
        let fire_sim = FireSimulator::new(500_000);
        let detector = AnomalyDetector::new(1.5);
        let sim = BlackSwanFireSimulator::new(fire_sim, detector, 0.0, 30);
        let result = sim.calculate(&[]);

        assert_eq!(result.base_fire_number_cents, 150_000_000);
        assert_eq!(result.risk_adjusted_fire_number_cents, 150_000_000);
    }
}
