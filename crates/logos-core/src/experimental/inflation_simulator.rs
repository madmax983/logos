//! Inflation Simulator
//!
//! A module to project the eroding effect of inflation on cash reserves
//! or purchasing power over time. It helps users understand why simply saving
//! cash might not be enough due to the silent loss of value over years.

/// A simulator that projects purchasing power based on an initial amount and inflation rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InflationSimulator {
    initial_amount_cents: i64,
    annual_inflation_rate_pct: f64,
}

impl InflationSimulator {
    /// Creates a new `InflationSimulator` with a starting amount and an annual inflation rate percentage.
    #[must_use]
    pub const fn new(initial_amount_cents: i64, annual_inflation_rate_pct: f64) -> Self {
        Self {
            initial_amount_cents,
            annual_inflation_rate_pct,
        }
    }

    /// Projects the real purchasing power of the initial amount over `years`.
    ///
    /// Returns a vector where the index corresponds to the year (0 is the current year).
    #[must_use]
    pub fn project_purchasing_power(&self, years: u16) -> Vec<i64> {
        let mut power = Vec::with_capacity((years + 1) as usize);

        let inflation_factor = 1.0 + (self.annual_inflation_rate_pct / 100.0);

        for year in 0..=years {
            #[allow(clippy::cast_precision_loss)]
            let current_value_f64 = self.initial_amount_cents as f64;

            // Formula: Real Value = Initial / (1 + inflation)^year
            let deflator = inflation_factor.powi(year.into());
            let deflated_value = current_value_f64 / deflator;

            #[allow(clippy::cast_possible_truncation)]
            let deflated_cents = deflated_value.round() as i64;

            power.push(deflated_cents);
        }

        power
    }

    /// Converts the projected purchasing power into a "Coffee Index".
    ///
    /// This represents the number of cups of coffee you can buy in the future
    /// with the same initial cash, assuming the price of coffee inflates at the given rate.
    #[must_use]
    pub fn coffee_index(&self, years: u16, current_coffee_cost_cents: i64) -> Vec<i64> {
        if current_coffee_cost_cents <= 0 {
            return vec![0; (years + 1) as usize];
        }

        let power = self.project_purchasing_power(years);
        power.into_iter().map(|cents| cents / current_coffee_cost_cents).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchasing_power_projection() {
        // Start with $10,000, 3% inflation.
        let sim = InflationSimulator::new(1_000_000, 3.0);
        let power = sim.project_purchasing_power(5);

        // Year 0: $10,000
        assert_eq!(power[0], 1_000_000);

        // Year 1: $10,000 / 1.03 = $9,708.73 -> 970_873 cents
        assert_eq!(power[1], 970_874); // Note: 970873.78 rounded is 970874

        // Year 5: $10,000 / (1.03^5) = $8,626.08 -> 862_608 cents
        assert_eq!(power[5], 862_609); // Note: 862608.78 rounded is 862609
    }

    #[test]
    fn test_coffee_index() {
        let sim = InflationSimulator::new(100_000, 5.0); // $1000, 5% inflation
        let coffees = sim.coffee_index(3, 500); // 3 years, $5 per coffee

        // Year 0: $1000 / $5 = 200 coffees
        assert_eq!(coffees[0], 200);

        // Year 1: $1000 / 1.05 = $952.38. / $5 = 190 coffees
        assert_eq!(coffees[1], 190);

        // Year 3: $1000 / (1.05^3) = $863.83. / $5 = 172 coffees
        assert_eq!(coffees[3], 172);
    }

    #[test]
    fn test_coffee_index_zero_cost() {
        let sim = InflationSimulator::new(100_000, 5.0);
        let coffees = sim.coffee_index(3, 0);
        assert_eq!(coffees, vec![0, 0, 0, 0]);
    }
}
