#![cfg(feature = "nova")]

//! Rent vs Buy Simulator
//!
//! A simulator to compare the long-term financial outcomes of renting versus buying a home,
//! taking into account opportunity costs, sunk costs, property appreciation, and investment returns.

/// The financial outcome of a simulation scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    /// Total net wealth at the end of the simulation period in cents.
    pub final_net_wealth_cents: i64,
    /// Total sunk costs (rent, property taxes, maintenance, interest) in cents.
    pub total_sunk_costs_cents: i64,
}

/// The result of the rent vs buy comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RentVsBuyResult {
    /// Simulation results for the renting scenario.
    pub rent_scenario: ScenarioResult,
    /// Simulation results for the buying scenario.
    pub buy_scenario: ScenarioResult,
    /// The financial difference between buying and renting. Positive means buying is better.
    pub buy_advantage_cents: i64,
}

/// A simulator for the rent vs buy decision.
#[derive(Debug, Clone)]
pub struct RentVsBuySimulator {
    home_price_cents: i64,
    down_payment_cents: i64,
    mortgage_interest_rate_pct: f64,
    mortgage_term_years: u8,
    property_tax_rate_pct: f64,
    maintenance_rate_pct: f64,
    home_appreciation_pct: f64,
    monthly_rent_cents: i64,
    rent_inflation_pct: f64,
    investment_return_pct: f64,
}

impl RentVsBuySimulator {
    /// Creates a new `RentVsBuySimulator`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        home_price_cents: i64,
        down_payment_cents: i64,
        mortgage_interest_rate_pct: f64,
        mortgage_term_years: u8,
        property_tax_rate_pct: f64,
        maintenance_rate_pct: f64,
        home_appreciation_pct: f64,
        monthly_rent_cents: i64,
        rent_inflation_pct: f64,
        investment_return_pct: f64,
    ) -> Self {
        Self {
            home_price_cents,
            down_payment_cents,
            mortgage_interest_rate_pct,
            mortgage_term_years,
            property_tax_rate_pct,
            maintenance_rate_pct,
            home_appreciation_pct,
            monthly_rent_cents,
            rent_inflation_pct,
            investment_return_pct,
        }
    }

    /// Simulates the rent vs buy decision over a given number of years.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn simulate(&self, years: u8) -> RentVsBuyResult {
        let months = u32::from(years) * 12;
        let loan_principal_cents = self
            .home_price_cents
            .saturating_sub(self.down_payment_cents);
        let monthly_interest_rate = self.mortgage_interest_rate_pct / 100.0 / 12.0;
        let total_mortgage_months = u32::from(self.mortgage_term_years) * 12;

        let monthly_mortgage_payment_cents =
            if loan_principal_cents > 0 && total_mortgage_months > 0 {
                if monthly_interest_rate > 0.0 {
                    let p = loan_principal_cents as f64;
                    let i = monthly_interest_rate;
                    let n = f64::from(total_mortgage_months);
                    let compound = (1.0_f64 + i).powf(n);
                    let payment = p * (i * compound) / (compound - 1.0);
                    payment.round() as i64
                } else {
                    loan_principal_cents / i64::from(total_mortgage_months)
                }
            } else {
                0
            };

        let monthly_property_tax =
            (self.home_price_cents as f64 * (self.property_tax_rate_pct / 100.0) / 12.0).round()
                as i64;
        let monthly_maintenance =
            (self.home_price_cents as f64 * (self.maintenance_rate_pct / 100.0) / 12.0).round()
                as i64;

        let monthly_investment_rate = self.investment_return_pct / 100.0 / 12.0;
        let monthly_rent_inflation_rate = self.rent_inflation_pct / 100.0 / 12.0;

        let mut current_home_value = self.home_price_cents as f64;
        let monthly_home_appreciation_rate = self.home_appreciation_pct / 100.0 / 12.0;

        let mut remaining_loan = loan_principal_cents as f64;

        let mut buy_sunk_costs = 0_i64;
        let mut rent_sunk_costs = 0_i64;

        let mut current_rent = self.monthly_rent_cents as f64;

        // Buying start scenario
        let mut buy_investments = 0.0_f64;

        // Renting start scenario
        let mut rent_investments = self.down_payment_cents as f64;

        for _ in 0..months {
            // --- BUYING ---
            let interest_payment = remaining_loan * monthly_interest_rate;
            let principal_payment =
                (monthly_mortgage_payment_cents as f64 - interest_payment).max(0.0);

            remaining_loan = (remaining_loan - principal_payment).max(0.0);

            buy_sunk_costs += interest_payment.round() as i64;
            buy_sunk_costs += monthly_property_tax;
            buy_sunk_costs += monthly_maintenance;

            current_home_value *= 1.0 + monthly_home_appreciation_rate;

            let buy_total_monthly_cost =
                monthly_mortgage_payment_cents + monthly_property_tax + monthly_maintenance;

            // --- RENTING ---
            let current_rent_cents = current_rent.round() as i64;
            rent_sunk_costs += current_rent_cents;

            let rent_total_monthly_cost = current_rent_cents;

            current_rent *= 1.0 + monthly_rent_inflation_rate;

            // --- INVESTMENT DIFFERENCE ---
            if buy_total_monthly_cost > rent_total_monthly_cost {
                // Renting is cheaper this month, invest the difference
                rent_investments += (buy_total_monthly_cost - rent_total_monthly_cost) as f64;
            } else {
                // Buying is cheaper this month, invest the difference
                buy_investments += (rent_total_monthly_cost - buy_total_monthly_cost) as f64;
            }

            buy_investments *= 1.0 + monthly_investment_rate;
            rent_investments *= 1.0 + monthly_investment_rate;
        }

        let buy_home_equity = current_home_value - remaining_loan;

        let buy_net_wealth_cents = buy_home_equity.round() as i64 + buy_investments.round() as i64;
        let rent_net_wealth_cents = rent_investments.round() as i64;

        RentVsBuyResult {
            rent_scenario: ScenarioResult {
                final_net_wealth_cents: rent_net_wealth_cents,
                total_sunk_costs_cents: rent_sunk_costs,
            },
            buy_scenario: ScenarioResult {
                final_net_wealth_cents: buy_net_wealth_cents,
                total_sunk_costs_cents: buy_sunk_costs,
            },
            buy_advantage_cents: buy_net_wealth_cents - rent_net_wealth_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rent_vs_buy_10_years() {
        let sim = RentVsBuySimulator::new(
            50_000_000, // $500k home
            10_000_000, // $100k down payment (20%)
            6.0,        // 6% mortgage interest
            30,         // 30 year mortgage
            1.2,        // 1.2% property tax
            1.0,        // 1.0% maintenance
            3.0,        // 3% home appreciation
            250_000,    // $2.5k/mo rent (high rent makes buying better)
            3.0,        // 3% rent inflation
            7.0,        // 7% investment return
        );

        let result = sim.simulate(10);

        // We expect buying to build more wealth in this scenario
        assert!(result.buy_advantage_cents > 0);
        assert!(result.buy_scenario.final_net_wealth_cents > 0);
        assert!(result.rent_scenario.final_net_wealth_cents > 0);
    }

    #[test]
    fn test_zero_investment_return() {
        let sim = RentVsBuySimulator::new(
            50_000_000, // $500k home
            10_000_000, // $100k down payment (20%)
            6.0,        // 6% mortgage interest
            30,         // 30 year mortgage
            1.2,        // 1.2% property tax
            1.0,        // 1.0% maintenance
            3.0,        // 3% home appreciation
            200_000,    // $2k/mo rent
            3.0,        // 3% rent inflation
            0.0,        // 0% investment return
        );

        let result = sim.simulate(10);

        // Buying should be advantageous because rent investments don't grow, but home does.
        assert!(result.buy_advantage_cents > 0);
    }

    #[test]
    fn test_zero_interest_mortgage() {
        let sim = RentVsBuySimulator::new(
            50_000_000, // $500k home
            10_000_000, // $100k down payment (20%)
            0.0,        // 0% mortgage interest
            30,         // 30 year mortgage
            1.2,        // 1.2% property tax
            1.0,        // 1.0% maintenance
            3.0,        // 3% home appreciation
            250_000,    // $2.5k/mo rent
            3.0,        // 3% rent inflation
            7.0,        // 7% investment return
        );

        let result = sim.simulate(10);

        assert!(result.buy_advantage_cents > 0);
    }

    #[test]
    fn test_renting_is_better() {
        let sim = RentVsBuySimulator::new(
            100_000_000, // $1M home
            20_000_000,  // $200k down payment (20%)
            8.0,         // 8% mortgage interest
            30,          // 30 year mortgage
            2.0,         // 2% property tax
            2.0,         // 2% maintenance
            2.0,         // 2% home appreciation
            150_000,     // $1.5k/mo rent (cheap rent!)
            1.0,         // 1% rent inflation
            10.0,        // 10% investment return
        );

        let result = sim.simulate(10);

        // Renting should be vastly superior here
        assert!(result.buy_advantage_cents < 0);
    }
}
