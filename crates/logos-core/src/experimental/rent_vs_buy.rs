//! Rent vs. Buy Simulator
//!
//! A financial planning tool that compares the long-term wealth outcomes of renting a home
//! versus buying a home. It factors in sunk costs (rent, property taxes, maintenance, interest)
//! and opportunity costs (investing the down payment instead of buying).
//!
//! 🌟 Nova Feature: Connects standard cash flow concepts to complex compounding opportunity costs,
//! providing a clear breakeven analysis for major life decisions.

/// The parameters for a "Rent" scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct RentScenario {
    pub initial_monthly_rent_cents: i64,
    pub annual_rent_increase_pct: f64,
    pub renters_insurance_monthly_cents: i64,
}

/// The parameters for a "Buy" scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct BuyScenario {
    pub home_price_cents: i64,
    pub down_payment_cents: i64,
    pub mortgage_interest_rate_pct: f64,
    pub mortgage_term_years: u32,
    pub property_tax_annual_pct: f64,
    pub home_maintenance_annual_pct: f64,
    pub home_appreciation_annual_pct: f64,
    pub closing_costs_cents: i64,
    pub homeowners_insurance_monthly_cents: i64,
}

/// The result at a specific year of the simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YearResult {
    pub year: u32,
    pub renter_net_worth_cents: i64,
    pub buyer_net_worth_cents: i64,
    pub home_value_cents: i64,
    pub remaining_mortgage_cents: i64,
}

/// The overall outcome of the simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RentVsBuyOutcome {
    pub yearly_results: Vec<YearResult>,
    /// The first year where the buyer's net worth exceeds the renter's net worth.
    pub breakeven_year: Option<u32>,
}

/// A simulator to determine if renting or buying is better over time.
#[derive(Debug, Clone)]
pub struct RentVsBuySimulator {
    rent: RentScenario,
    buy: BuyScenario,
    market_return_annual_pct: f64,
    inflation_annual_pct: f64,
}

impl RentVsBuySimulator {
    #[must_use]
    pub const fn new(
        rent: RentScenario,
        buy: BuyScenario,
        market_return_annual_pct: f64,
        inflation_annual_pct: f64,
    ) -> Self {
        Self {
            rent,
            buy,
            market_return_annual_pct,
            inflation_annual_pct,
        }
    }

    /// Calculates the monthly mortgage payment (Principal + Interest)
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn calculate_monthly_mortgage(&self) -> i64 {
        let principal = (self.buy.home_price_cents - self.buy.down_payment_cents) as f64;
        if principal <= 0.0 {
            return 0;
        }

        let r = self.buy.mortgage_interest_rate_pct / 100.0 / 12.0;
        let n = f64::from(self.buy.mortgage_term_years * 12);

        if r == 0.0 {
            return (principal / n).round() as i64;
        }

        let factor = (1.0 + r).powf(n);
        let payment = principal * (r * factor) / (factor - 1.0);
        payment.round() as i64
    }

    /// Simulates the net worth of both renting and buying over a number of years.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn simulate(&self, years: u32) -> RentVsBuyOutcome {
        let mut yearly_results = Vec::with_capacity(years as usize);

        let initial_investment_cents = self.buy.down_payment_cents + self.buy.closing_costs_cents;
        let mut renter_portfolio_cents = initial_investment_cents as f64;

        let monthly_mortgage = self.calculate_monthly_mortgage() as f64;
        let mut remaining_principal =
            (self.buy.home_price_cents - self.buy.down_payment_cents) as f64;
        let mut home_value = self.buy.home_price_cents as f64;
        let mortgage_rate_monthly = self.buy.mortgage_interest_rate_pct / 100.0 / 12.0;

        let mut current_monthly_rent = self.rent.initial_monthly_rent_cents as f64;

        let market_return_monthly = self.market_return_annual_pct / 100.0 / 12.0;

        let mut breakeven_year = None;

        for year in 1..=years {
            let property_tax_monthly =
                (home_value * (self.buy.property_tax_annual_pct / 100.0)) / 12.0;
            let maintenance_monthly =
                (home_value * (self.buy.home_maintenance_annual_pct / 100.0)) / 12.0;

            for _month in 1..=12 {
                // Renter Math
                let rent_costs =
                    current_monthly_rent + self.rent.renters_insurance_monthly_cents as f64;

                // Buyer Math
                let mut interest_payment = 0.0;
                let mut principal_payment = 0.0;

                if remaining_principal > 0.0 {
                    interest_payment = remaining_principal * mortgage_rate_monthly;
                    principal_payment = monthly_mortgage - interest_payment;
                    if principal_payment > remaining_principal {
                        principal_payment = remaining_principal;
                    }
                    remaining_principal -= principal_payment;
                }

                let buy_costs = interest_payment
                    + principal_payment
                    + property_tax_monthly
                    + maintenance_monthly
                    + self.buy.homeowners_insurance_monthly_cents as f64;

                // Difference in cash flow
                let diff = buy_costs - rent_costs;

                // Renter invests the difference
                renter_portfolio_cents *= 1.0 + market_return_monthly;
                renter_portfolio_cents += diff;
            }

            // Apply annual appreciation/increases
            home_value *= 1.0 + (self.buy.home_appreciation_annual_pct / 100.0);
            current_monthly_rent *= 1.0 + (self.rent.annual_rent_increase_pct / 100.0);

            let buyer_net_worth = home_value - remaining_principal;
            let renter_net_worth = renter_portfolio_cents;

            yearly_results.push(YearResult {
                year,
                renter_net_worth_cents: renter_net_worth.round() as i64,
                buyer_net_worth_cents: buyer_net_worth.round() as i64,
                home_value_cents: home_value.round() as i64,
                remaining_mortgage_cents: remaining_principal.round() as i64,
            });

            if breakeven_year.is_none() && buyer_net_worth > renter_net_worth {
                breakeven_year = Some(year);
            }
        }

        let _ = self.inflation_annual_pct; // Not used directly in this simplified model, could be used for rent adjustments or real return

        RentVsBuyOutcome {
            yearly_results,
            breakeven_year,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_monthly_mortgage() {
        let rent = RentScenario {
            initial_monthly_rent_cents: 200_000,
            annual_rent_increase_pct: 3.0,
            renters_insurance_monthly_cents: 15_00,
        };
        let buy = BuyScenario {
            home_price_cents: 40_000_000,
            down_payment_cents: 8_000_000, // 20%
            mortgage_interest_rate_pct: 5.0,
            mortgage_term_years: 30,
            property_tax_annual_pct: 1.2,
            home_maintenance_annual_pct: 1.0,
            home_appreciation_annual_pct: 3.0,
            closing_costs_cents: 1_200_000,
            homeowners_insurance_monthly_cents: 10_000,
        };
        let simulator = RentVsBuySimulator::new(rent, buy, 7.0, 2.0);

        let mortgage = simulator.calculate_monthly_mortgage();
        // $320,000 at 5% for 30 years is ~$1717.83 -> 171783 cents
        assert!((mortgage - 171_783).abs() < 500); // within $5
    }

    #[test]
    fn test_rent_vs_buy_simulation() {
        let rent = RentScenario {
            initial_monthly_rent_cents: 200_000, // $2000
            annual_rent_increase_pct: 3.0,
            renters_insurance_monthly_cents: 20_00, // $20
        };
        // Buyer buys a $400k home
        let buy = BuyScenario {
            home_price_cents: 40_000_000,
            down_payment_cents: 8_000_000, // $80k
            mortgage_interest_rate_pct: 5.0,
            mortgage_term_years: 30,
            property_tax_annual_pct: 1.0,               // $4k/yr
            home_maintenance_annual_pct: 1.0,           // $4k/yr
            home_appreciation_annual_pct: 3.0,          // Appreciates at 3%
            closing_costs_cents: 1_000_000,             // $10k
            homeowners_insurance_monthly_cents: 10_000, // $100/mo
        };

        let simulator = RentVsBuySimulator::new(rent, buy, 8.0, 3.0); // 8% market return
        let outcome = simulator.simulate(30);

        assert_eq!(outcome.yearly_results.len(), 30);

        // Year 1: Renting usually wins because of closing costs & high early interest
        let yr1 = &outcome.yearly_results[0];
        assert!(yr1.renter_net_worth_cents > yr1.buyer_net_worth_cents);

        // Over a long time, assuming 3% appreciation vs 8% market, it's a race.
        // It might or might not break even. Let's just check the data structure is populated correctly.
        assert!(yr1.home_value_cents > 40_000_000);
        assert!(yr1.remaining_mortgage_cents < 32_000_000);
    }

    #[test]
    fn test_calculate_monthly_mortgage_zero_interest() {
        let rent = RentScenario {
            initial_monthly_rent_cents: 200_000,
            annual_rent_increase_pct: 3.0,
            renters_insurance_monthly_cents: 15_00,
        };
        let buy = BuyScenario {
            home_price_cents: 40_000_000,
            down_payment_cents: 4_000_000, // 10%
            mortgage_interest_rate_pct: 0.0,
            mortgage_term_years: 30,
            property_tax_annual_pct: 1.2,
            home_maintenance_annual_pct: 1.0,
            home_appreciation_annual_pct: 3.0,
            closing_costs_cents: 1_200_000,
            homeowners_insurance_monthly_cents: 10_000,
        };
        let simulator = RentVsBuySimulator::new(rent, buy, 7.0, 2.0);
        let mortgage = simulator.calculate_monthly_mortgage();
        assert_eq!(mortgage, 36_000_000 / (30 * 12));
    }
}
