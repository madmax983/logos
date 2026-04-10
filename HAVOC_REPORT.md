# 👺 Havoc: Missing Arithmetic Guards in RSU Budget Projections

🧨 **The Trigger:**
When projecting RSU budget plans under varying market scenarios (`bear`, `base`, `bull`), the system calculates baseline remaining cents by subtracting fixed commitments directly from the conservative budget (`conservative_budget_cents - input.fixed_commitments_cents`) and calculates sweeps using `surplus_cents.saturating_mul(...) / 100`. Additionally, the `project_rsu_forecast_summary` assumes that the summation of projected RSU vesting events will not overflow the 64-bit integer limit, utilizing an unguarded addition loop.
If simulated market conditions result in boundary logic scenarios or edge cases, standard subtraction/addition triggers an arithmetic overflow in debug mode or wraps incorrectly in release mode. We injected specific `proptest` suites to expose these implicit vulnerabilities across `project_rsu_budget_plan` and `project_rsu_forecast_summary`.

📉 **The Stack Trace:**
```
thread 'project_rsu_budget_plan_panics_on_overflow' panicked at:
attempt to subtract with overflow
thread 'project_rsu_forecast_summary_panics_on_overflow' panicked at:
attempt to add with overflow
```

🧪 **Reproduction:**
Run the chaos testing suites specifically designed to bombard the API layers:
```bash
cargo test -p logos-reporting --test rsu_budget_plan_havoc
cargo test -p logos-reporting --test rsu_forecast_havoc
```

😈 **Comment:**
"You assumed the inputs to the RSU budget projections would always remain comfortably within 64-bit bounds and that `fixed_commitments_cents` could never trigger an underflow during standard arithmetic subtraction. You were wrong. A massive sequence of RSU vests coupled with strict sweeping policies just blew past the integer limits and crashed the core reporting engine."
