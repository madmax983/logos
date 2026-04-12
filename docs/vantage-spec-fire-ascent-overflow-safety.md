# 🔭 Vantage: Spec for Fire Ascent Overflow Safety

## 👤 User Story
"As a planner aiming for Financial Independence, Retire Early (FIRE), I want the system to safely handle extremely large FIRE number projections without crashing, so that an unusually large target or input error doesn't break the entire simulation and ruin my planning session."

## 🤔 So What?
A financial planning tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive FIRE numbers, extreme withdrawal rates), the software should gracefully cap results at hardware limits or report an explicit error, rather than suffering a hard crash that destroys the entire user session. Resilient software builds trust; fragile software creates frustration.

## 🎯 Metric Definition
Success = 0 panics during FIRE ascent simulations, even when providing inputs that result in a `fire_number` near `i64::MAX`. The simulation must complete and return a capped value or explicitly fail with an impossible ascent state, rather than crashing the application.

## 🔎 Gap Analysis
Currently, `FireAscentSimulator::ascend` calculates intermediate milestone camps using unguarded multiplication: `let camp3 = (fire_number * 3) / 4;`. This assumes `fire_number * 3` will never exceed the 64-bit integer limit. Standard financial simulation engines handle edge cases safely by using saturating arithmetic (capping at the maximum limit), manual bounds checking, or reordering operations (e.g., `(fire_number / 4).saturating_mul(3)`) to avoid unexpected runtime panics on intermediate steps.

## ✅ Acceptance Criteria
- The `FireAscentSimulator::ascend` function must not panic when simulating ascents with extremely large FIRE numbers.
- The intermediate milestone calculation logic (like `camp3`) must be updated to handle potential arithmetic overflows safely (e.g., via safe mathematical refactoring or saturating math).
- All existing tests, including chaos/proptests that trigger this overflow (like `test_fire_ascent_panics_on_overflow`), must pass successfully after the fix (i.e. the `#[should_panic]` should be removed and the test updated to expect a graceful simulation completion).

## 🚫 Out of Scope
- Rewriting the entire projection engine to use arbitrary-precision numbers.
- Adding complex UI error dialogs for overflow states beyond existing CLI output.
