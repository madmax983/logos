#![cfg(feature = "nova")]

//! Time Billionaire Calculator
//!
//! Evaluates how much "time" (in seconds) someone has left, treating 1 billion
//! seconds (about 31.7 years) as the ultimate currency, far more valuable than dollars.

/// Represents the result of a Time Billionaire calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeBillionaireResult {
    /// The number of seconds remaining in the expected lifespan.
    pub seconds_remaining: i64,
    /// The number of "Time Billions" remaining (seconds / 1,000,000,000).
    pub time_billions: f64,
    /// True if the person has at least 1 billion seconds left.
    pub is_time_billionaire: bool,
}

/// A calculator to convert remaining lifespan into seconds.
#[derive(Debug, Clone)]
pub struct TimeBillionaireCalculator {
    current_age_years: u8,
    life_expectancy_years: u8,
}

impl TimeBillionaireCalculator {
    /// Creates a new `TimeBillionaireCalculator`.
    ///
    /// # Arguments
    /// * `current_age_years` - The person's current age.
    /// * `life_expectancy_years` - The expected lifespan (e.g., 80).
    #[must_use]
    pub const fn new(current_age_years: u8, life_expectancy_years: u8) -> Self {
        Self {
            current_age_years,
            life_expectancy_years,
        }
    }

    /// Calculates the remaining time billions.
    #[must_use]
    pub fn calculate(&self) -> TimeBillionaireResult {
        if self.current_age_years >= self.life_expectancy_years {
            return TimeBillionaireResult {
                seconds_remaining: 0,
                time_billions: 0.0,
                is_time_billionaire: false,
            };
        }

        let years_remaining = self.life_expectancy_years - self.current_age_years;
        // 365.25 days/year * 24 hours/day * 60 mins/hour * 60 secs/min = 31_557_600 seconds/year
        let seconds_per_year: i64 = 31_557_600;
        let seconds_remaining = i64::from(years_remaining) * seconds_per_year;

        #[allow(clippy::cast_precision_loss)]
        let time_billions = seconds_remaining as f64 / 1_000_000_000.0;

        TimeBillionaireResult {
            seconds_remaining,
            time_billions,
            is_time_billionaire: seconds_remaining >= 1_000_000_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_billionaire() {
        // 20 years old, 80 year life expectancy.
        // 60 years remaining.
        // 60 years * 365.25 days * 24 hours * 60 minutes * 60 seconds
        // = 1,893,456,000 seconds
        let calc = TimeBillionaireCalculator::new(20, 80);
        let result = calc.calculate();

        assert_eq!(result.seconds_remaining, 1_893_456_000);
        assert!((result.time_billions - 1.893).abs() < 0.001);
        assert!(result.is_time_billionaire);
    }

    #[test]
    fn test_not_time_billionaire() {
        // 60 years old, 80 year life expectancy.
        // 20 years remaining.
        // 20 years * 365.25 days * 24 hours * 60 minutes * 60 seconds
        // = 631,152,000 seconds
        let calc = TimeBillionaireCalculator::new(60, 80);
        let result = calc.calculate();

        assert_eq!(result.seconds_remaining, 631_152_000);
        assert!((result.time_billions - 0.631).abs() < 0.001);
        assert!(!result.is_time_billionaire);
    }

    #[test]
    fn test_past_expectancy() {
        // 85 years old, 80 year life expectancy.
        // 0 seconds remaining.
        let calc = TimeBillionaireCalculator::new(85, 80);
        let result = calc.calculate();

        assert_eq!(result.seconds_remaining, 0);
        assert!(result.time_billions.abs() < f64::EPSILON);
        assert!(!result.is_time_billionaire);
    }
}
