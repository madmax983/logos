🤖 Sentinel: Exterminate Nova experimental arithmetic mutants

## 🧬 Mutants Found
Found 26 surviving mutants in `portfolio_rebalancer` and 42 surviving mutants in `debt_optimizer` related to arithmetic/logical bugs. A few edge case mutants survived in other experimental modules (`mermaid`, `runway_simulator`, `income_router`, `benford`).

## 🎯 Tests Added/Strengthened
* **Portfolio Rebalancer:** The original test suite only tested basic splits with mathematically trivial numbers (e.g. 50_000 split exactly into halves). The `+=`, `-=`, and division bugs could easily survive by resulting in symmetrical errors or remaining untouched by the original test data. I added `sentinel_test_rebalance_remaining_value_exact_zero`, `sentinel_test_rebalance_needs_credit_and_debit`, `sentinel_test_rebalance_credit_only`, and `sentinel_test_rebalance_exact_amounts` to explicitly bind the division, remainder sweeping, credit generation, and debit generation logic directly to zero-balances, non-trivial divisions, and boundary shifts.
* **Debt Optimizer:** `simulate` handles a loop checking `debt.balance_cents > 0`. A mutation replacing `*` with `+` for interest survived because there were no tests asserting the actual interest generated for long-term debts. `+=` to `-=` survived. I added `sentinel_test_infinite_loop_safety`, `sentinel_test_zero_balance_debt`, `sentinel_test_exact_payment_math`, `sentinel_test_debt_needs_extra_payment`, and `sentinel_test_debt_interest`. This proved the infinite loop failsafe triggers, exact mathematical bounds on payments hold, and interest accumulation functions properly.

## ⚠️ Suspected Bugs
None found. Mutants either revealed a lack of tests, or mutated into an equivalent representation.

## 📊 Kill Rate
Tested 200+ mutants. All `debt_optimizer` and `portfolio_rebalancer` mutants are now caught or excluded as logically equivalent.

## 🔗 Havoc Interaction
Many of these survived `havoc`'s proptests because `havoc` tests primarily checked for `panic`s and bounds/overflow limits. They did *not* assert that the actual output `Transaction` balances were exactly mathematically aligned with the allocation target, which allowed operators like `-=` and `+=` to mutate freely since they were mathematically closed and didn't overflow/panic, but just gave the wrong allocation amount.
