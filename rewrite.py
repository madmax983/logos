import re

with open("crates/logos-core/src/planning/fire.rs", "r") as f:
    content = f.read()

# 1. Module level
module_lore = """//! FIRE (Financial Independence, Retire Early) Simulation Module
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
//! vests (adjusted for risk via haircut tiers)."""

content = content.replace("//! FIRE (Financial Independence, Retire Early) Simulation Module\n//!\n//! Provides primitives to project progress toward financial independence\n//! by incorporating current expenses, base net worth, and upcoming RSU\n//! vests (adjusted for risk via haircut tiers).", module_lore)


# 2. FireSimulator struct
struct_sim = """/// A simulator to calculate FIRE (Financial Independence, Retire Early) metrics.
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
/// ```"""
content = content.replace("/// A simulator to calculate FIRE (Financial Independence, Retire Early) metrics.\n///\n/// The `FireSimulator` computes target retirement numbers based on a safe\n/// withdrawal rate and monthly expenses. It also factors in risk-adjusted\n/// upcoming RSU vests to determine a \"safe\" net worth and the resulting\n/// progress percentage.\n///\n/// # Examples\n/// ```\n/// use logos_core::planning::fire::{FireConfig, FireSimulator, UpcomingVest};\n///\n/// // Create a simulator for $5,000 monthly expenses ($60,000/yr).\n/// // The default Safe Withdrawal Rate is 4%, yielding a $1,500,000 FIRE number.\n/// let mut sim = FireSimulator::new(500_000);\n///\n/// // Add current assets and liabilities: $200k assets, $50k liabilities = $150k Base Net Worth\n/// sim.add_assets_liabilities(20_000_000, 5_000_000);\n///\n/// // Add an upcoming RSU vest: 500 units @ $1000 ($500k gross), vesting in 60 days.\n/// // At 60 days, the default haircut tier retains 60%, adding $300k safe value.\n/// // Total Safe Net Worth = $150k (base) + $300k (RSUs) = $450k.\n/// sim.add_upcoming_vest(UpcomingVest {\n///     avg_close_price_cents: 100_000,\n///     units: 500,\n///     days_to_vest: 60,\n/// });\n///\n/// assert_eq!(sim.fire_number_cents(), 150_000_000);\n/// assert_eq!(sim.safe_net_worth_cents(), 45_000_000);\n/// assert_eq!(sim.fire_progress_pct(), 30); // $450k / $1.5M = 30%\n/// ```", struct_sim)

# 3. FireConfig
fire_config = """/// Configuration for the FIRE Simulator.
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
/// ```"""
content = content.replace("/// Configuration for the FIRE Simulator.\n///\n/// # Examples\n/// ```\n/// use logos_core::planning::fire::FireConfig;\n///\n/// let config = FireConfig { safe_withdrawal_rate_pct: 3 };\n/// ```", fire_config)

# 4. new
new_meth = """    /// Ignites a new [`FireSimulator`] based on your monthly burn rate.
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
    /// ```"""
content = content.replace("    /// Creates a new `FireSimulator` with the given monthly expenses.\n    ///\n    /// By default, this uses a 4% safe withdrawal rate and standard haircut tiers.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::FireSimulator;\n    ///\n    /// // Simulator for $5,000 monthly expenses ($60,000/yr)\n    /// let sim = FireSimulator::new(500_000);\n    /// ```", new_meth)

# 5. set_config
set_config = """    /// Alters the core assumptions of the simulation.
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
    /// ```"""
content = content.replace("    /// Updates the configuration, such as changing the safe withdrawal rate.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::{FireConfig, FireSimulator};\n    ///\n    /// let mut sim = FireSimulator::new(500_000);\n    /// sim.set_config(FireConfig { safe_withdrawal_rate_pct: 3 });\n    /// ```", set_config)

# 6. add_assets
add_assets = """    /// Injects your current liquid reality into the simulation.
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
    /// ```"""
content = content.replace("    /// Adds base liquid assets and liabilities to the calculation.\n    /// This directly increases the base net worth by `assets_cents - liabilities_cents`.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::FireSimulator;\n    ///\n    /// let mut sim = FireSimulator::new(500_000);\n    /// // Add $200k in assets, $50k in liabilities\n    /// sim.add_assets_liabilities(20_000_000, 5_000_000);\n    /// ```", add_assets)

# 7. add_vest
add_vest = """    /// Registers an unhatched chicken (upcoming RSU vest) into your net worth.
    ///
    /// Because stock prices fluctuate and employees sometimes leave before vesting,
    /// future grants are inherently risky. We apply a temporal haircut based on how far out
    /// the vest date is, ensuring your projections aren't built on a house of cards.
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
    /// ```"""
content = content.replace("    /// Registers an upcoming RSU vest to be included in the safe net worth.\n    ///\n    /// Vests are risk-adjusted based on the number of days until the vest date\n    /// and the configured [`HaircutTierTable`].\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::{FireSimulator, UpcomingVest};\n    ///\n    /// let mut sim = FireSimulator::new(500_000);\n    /// \n    /// // Add a vest of 500 units @ $1000 each ($500k gross) in 60 days\n    /// sim.add_upcoming_vest(UpcomingVest {\n    ///     avg_close_price_cents: 100_000,\n    ///     units: 500,\n    ///     days_to_vest: 60,\n    /// });\n    /// ```", add_vest)

# 8. fire_number
fire_number = """    /// Reveals the mountain peak: your target FIRE number in cents.
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
    /// ```"""
content = content.replace("    /// Calculates the target FIRE number in cents.\n    ///\n    /// Returns `i64::MAX` if the safe withdrawal rate is configured to 0.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::FireSimulator;\n    ///\n    /// // $5000/month expenses = $60,000/yr.\n    /// // At 4% safe withdrawal rate, target is $1.5M.\n    /// let sim = FireSimulator::new(500_000);\n    /// assert_eq!(sim.fire_number_cents(), 150_000_000);\n    /// ```", fire_number)

# 9. safe_nw
safe_nw = """    /// Distills your total financial picture into a single, risk-adjusted "safe" net worth.
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
    /// ```"""
content = content.replace("    /// Computes the risk-adjusted \"safe\" net worth in cents.\n    ///\n    /// The safe net worth is the sum of liquid assets minus liabilities,\n    /// plus the sum of all upcoming vests scaled by their respective haircut tiers.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::FireSimulator;\n    ///\n    /// let mut sim = FireSimulator::new(500_000);\n    /// sim.add_assets_liabilities(20_000_000, 5_000_000); // $150k base NW\n    /// assert_eq!(sim.safe_net_worth_cents(), 15_000_000);\n    /// ```", safe_nw)

# 10. fire_progress
fire_progress = """    /// Translates the mountain climb into a simple progress bar percentage.
    ///
    /// By dividing your safe net worth by your target FIRE number, you get a clean
    /// 0 to 100 percentage of how close you are to financial independence.
    ///
    /// # Examples
    /// ```
    /// use logos_core::planning::fire::FireSimulator;
    ///
    /// let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number
    /// sim.add_assets_liabilities(30_000_000, 0); // $300k NW
    ///
    /// assert_eq!(sim.fire_progress_pct(), 20); // 300k / 1.5M = 20%
    /// ```"""
content = content.replace("    /// Returns the progress towards the FIRE number as an integer percentage from 0 to 100.\n    ///\n    /// # Examples\n    /// ```\n    /// use logos_core::planning::fire::FireSimulator;\n    ///\n    /// let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number\n    /// sim.add_assets_liabilities(30_000_000, 0); // $300k NW\n    /// \n    /// assert_eq!(sim.fire_progress_pct(), 20); // 300k / 1.5M = 20%\n    /// ```", fire_progress)


with open("crates/logos-core/src/planning/fire.rs", "w") as f:
    f.write(content)
