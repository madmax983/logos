#![cfg(feature = "nova")]

//! Wealth Velocity Analyzer
//!
//! A module that calculates the first and second derivatives of net worth over time.
//! Provides a "speedometer" and "accelerometer" for wealth accumulation, making it
//! easy to determine if net worth growth is accelerating or decelerating.

/// A single snapshot of net worth at a given time point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetWorthSnapshot {
    /// The net worth in cents.
    pub net_worth_cents: i64,
    /// The number of days elapsed since the start of tracking or a reference epoch.
    pub days_elapsed: u32,
}

/// The result of a wealth velocity analysis over two or more snapshots.
#[derive(Debug, Clone, PartialEq)]
pub struct WealthVelocityResult {
    /// The average change in net worth per day (cents/day). This is the "Velocity".
    pub velocity_cents_per_day: f64,
    /// The rate of change of the velocity (cents/day^2). This is the "Acceleration".
    pub acceleration_cents_per_day_sq: f64,
}

/// Analyzes a series of net worth snapshots to determine velocity and acceleration.
#[derive(Debug, Clone, Default)]
pub struct WealthVelocityAnalyzer {}

impl WealthVelocityAnalyzer {
    /// Creates a new `WealthVelocityAnalyzer`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the wealth velocity and acceleration given an ordered series of snapshots.
    ///
    /// # Arguments
    ///
    /// * `snapshots` - A slice of `NetWorthSnapshot`s sorted chronologically by `days_elapsed`.
    ///
    /// # Returns
    ///
    /// `None` if fewer than 2 snapshots are provided, or if the time difference between the first
    /// and last snapshot is 0. Otherwise, returns a `WealthVelocityResult`.
    /// The `acceleration_cents_per_day_sq` will be 0.0 if fewer than 3 snapshots are provided.
    #[must_use]
    pub fn analyze(&self, snapshots: &[NetWorthSnapshot]) -> Option<WealthVelocityResult> {
        if snapshots.len() < 2 {
            return None;
        }

        let first = &snapshots[0];
        let last = &snapshots[snapshots.len() - 1];

        let total_days = last.days_elapsed.saturating_sub(first.days_elapsed);
        if total_days == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let total_change_cents = (last.net_worth_cents - first.net_worth_cents) as f64;
        let velocity = total_change_cents / f64::from(total_days);

        let acceleration = if snapshots.len() >= 3 {
            // To compute simple acceleration, we can calculate the velocity of the first half
            // and the velocity of the second half, then find the rate of change between them.
            // A more sophisticated approach would use linear regression or finite differences,
            // but a midpoint comparison provides a solid baseline.

            let mid_idx = snapshots.len() / 2;
            let mid = &snapshots[mid_idx];

            let first_half_days = mid.days_elapsed.saturating_sub(first.days_elapsed);
            let second_half_days = last.days_elapsed.saturating_sub(mid.days_elapsed);

            if first_half_days > 0 && second_half_days > 0 {
                #[allow(clippy::cast_precision_loss)]
                let v1 = (mid.net_worth_cents - first.net_worth_cents) as f64
                    / f64::from(first_half_days);
                #[allow(clippy::cast_precision_loss)]
                let v2 = (last.net_worth_cents - mid.net_worth_cents) as f64
                    / f64::from(second_half_days);

                // The change in velocity over the total time period
                (v2 - v1) / f64::from(total_days)
            } else {
                0.0
            }
        } else {
            0.0
        };

        Some(WealthVelocityResult {
            velocity_cents_per_day: velocity,
            acceleration_cents_per_day_sq: acceleration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_or_single_snapshot() {
        let analyzer = WealthVelocityAnalyzer::new();
        assert_eq!(analyzer.analyze(&[]), None);
        assert_eq!(
            analyzer.analyze(&[NetWorthSnapshot {
                net_worth_cents: 100,
                days_elapsed: 0
            }]),
            None
        );
    }

    #[test]
    fn test_zero_time_elapsed() {
        let analyzer = WealthVelocityAnalyzer::new();
        let snapshots = vec![
            NetWorthSnapshot {
                net_worth_cents: 100,
                days_elapsed: 10,
            },
            NetWorthSnapshot {
                net_worth_cents: 200,
                days_elapsed: 10,
            },
        ];
        assert_eq!(analyzer.analyze(&snapshots), None);
    }

    #[test]
    fn test_constant_velocity_no_acceleration() {
        let analyzer = WealthVelocityAnalyzer::new();

        // Gaining 100 cents per day
        let snapshots = vec![
            NetWorthSnapshot {
                net_worth_cents: 0,
                days_elapsed: 0,
            },
            NetWorthSnapshot {
                net_worth_cents: 1000,
                days_elapsed: 10,
            },
            NetWorthSnapshot {
                net_worth_cents: 2000,
                days_elapsed: 20,
            },
        ];

        let result = analyzer.analyze(&snapshots).unwrap();
        assert!((result.velocity_cents_per_day - 100.0).abs() < f64::EPSILON);
        assert!((result.acceleration_cents_per_day_sq - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_accelerating_wealth() {
        let analyzer = WealthVelocityAnalyzer::new();

        // V1: 0 to 10 days = +1000 -> 100/day
        // V2: 10 to 20 days = +3000 -> 300/day
        // Total change: 4000 over 20 days = 200/day
        // Acceleration: (300 - 100) / 20 = 10/day^2
        let snapshots = vec![
            NetWorthSnapshot {
                net_worth_cents: 0,
                days_elapsed: 0,
            },
            NetWorthSnapshot {
                net_worth_cents: 1000,
                days_elapsed: 10,
            },
            NetWorthSnapshot {
                net_worth_cents: 4000,
                days_elapsed: 20,
            },
        ];

        let result = analyzer.analyze(&snapshots).unwrap();
        assert!((result.velocity_cents_per_day - 200.0).abs() < f64::EPSILON);
        assert!((result.acceleration_cents_per_day_sq - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decelerating_wealth() {
        let analyzer = WealthVelocityAnalyzer::new();

        // V1: 0 to 10 days = +3000 -> 300/day
        // V2: 10 to 20 days = +1000 -> 100/day
        // Total change: 4000 over 20 days = 200/day
        // Acceleration: (100 - 300) / 20 = -10/day^2
        let snapshots = vec![
            NetWorthSnapshot {
                net_worth_cents: 0,
                days_elapsed: 0,
            },
            NetWorthSnapshot {
                net_worth_cents: 3000,
                days_elapsed: 10,
            },
            NetWorthSnapshot {
                net_worth_cents: 4000,
                days_elapsed: 20,
            },
        ];

        let result = analyzer.analyze(&snapshots).unwrap();
        assert!((result.velocity_cents_per_day - 200.0).abs() < f64::EPSILON);
        assert!((result.acceleration_cents_per_day_sq - -10.0).abs() < f64::EPSILON);
    }
}
