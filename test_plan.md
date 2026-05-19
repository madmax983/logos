**Nova feature idea:** A `FireDelayCalculator` which calculates exactly how many months an expense delays your FIRE (Financial Independence, Retire Early) date.
I noticed we have `OpportunityCostAnalyzer` and `FireGoalSeeker` / `NetWorthProjector` / `FireSimulator`. I can mash them up to calculate the *delay in FIRE* caused by a recurring expense or a one-time purchase.

This answers the fundamental question: *"If I buy this $50,000 car, how many more months do I have to work?"*

Or: *"If I keep my $100/mo subscription, does it push my FIRE date back by a year?"*

The spark: The `logos-core` library focuses on strict accounting, but the `planning` and `experimental` modules are about the *future*. Showing someone the *time* cost of an expense is often more powerful than the dollar cost.

I will create a `fire_delay_calculator.rs` in `crates/logos-core/src/experimental/`.
