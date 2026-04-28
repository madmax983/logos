Title: "👺 Havoc: `IncomeRouter` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large amount like `i64::MAX` to `route_income` causes an arithmetic multiplication overflow panic. The `*` operator in `let allocated = (amount_cents * i64::from(rule.percentage)) / 100;` is unguarded.

📉 **The Stack Trace:**
```
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core income_router_panics_on_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed people wouldn't route massive amounts of income. You were wrong. A single massive paycheck crashes the router."

---

Title: "👺 Havoc: `PortfolioRebalancer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large balance (like `i64::MAX`) to `rebalance` causes an arithmetic multiplication overflow panic. The `*` operator in `let allocated = (total_value * i64::from(target.percentage)) / 100;` is unguarded.

📉 **The Stack Trace:**
```
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core portfolio_rebalancer_panics_on_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed total portfolio value would never exceed RAM. You were wrong. Rebalancing crashes if the balance is too high."

---

Title: "👺 Havoc: `GoalFundProjector` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large `months` value (e.g., `> 2184`) to `project_timeline` causes an integer multiplication overflow panic. The `*` operator in `let month_start_days = (month_index - 1) * 30;` and `let month_end_days = month_index * 30;` is unguarded for `u16`.

📉 **The Stack Trace:**
```
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core project_timeline_panics_on_u16_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed people wouldn't project their goal fund for more than 182 years. You were wrong. A large projection horizon completely crashes the simulation instead of returning an error."

---

Title: "👺 Havoc: `FireAscentSimulator` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large value for `monthly_expenses` (e.g., `i64::MAX / 20`) causes a multiplication overflow panic in `fire_number_cents` when computing yearly expenses.

📉 **The Stack Trace:**
```
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core test_fire_ascent_panics_on_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed FIRE simulations wouldn't use massive expense inputs. You were wrong. Massive expenses overflow the withdrawal target."

---

Title: "👺 Havoc: `RsuAutoDistributor` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a massive gross vest amount to `distribute_rsu_vest` causes an arithmetic multiplication overflow. The `*` operator in `let smoothing_cents = (gross_vest_cents * i64::from(policy.smoothing_buffer_pct())) / 100;` is unguarded.

📉 **The Stack Trace:**
```
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core havoc_rsu_distribute_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed RSU vests wouldn't cause integer overflow during percentage allocations. You were wrong."
