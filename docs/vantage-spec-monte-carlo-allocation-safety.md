# 🔭 Vantage: Spec for Monte Carlo Allocation Safety

👤 **User Story:**
As a long-term investor, I want the Monte Carlo simulator to gracefully reject an unreasonably large number of simulation paths, so that the application doesn't completely crash my system by attempting to allocate more memory than is available.

🤔 **So What?**
Financial simulation tools must be robust and fail gracefully when given extreme inputs. Currently, the system attempts to allocate massive amounts of memory based purely on an unchecked input value, leading to hard crashes. A crashing application destroys user trust. We must safely cap or validate inputs before resource allocation.

🎯 **Metric Definition:**
Success = 0 system crashes or out-of-memory panics when running a Monte Carlo simulation, even with extremely large inputs. The system returns a clear error instead.

🔎 **Gap Analysis:**
The Monte Carlo simulator currently uses the provided number of paths directly to pre-allocate a massive collection without any upper-bounds checking. Most enterprise financial tools enforce limits on simulation counts to prevent uncontrolled resource exhaustion.

✅ **Acceptance Criteria:**
- The simulator must safely reject unreasonably large values for simulation paths.
- The system must not crash or panic due to memory allocation overflow.
- A descriptive error must be returned to the user when the input exceeds safe boundaries.

🚫 **Out of Scope:**
- Attempting to support massive numbers of simulation paths via out-of-core processing or distributed computing.
- Altering the underlying deterministic randomness implementation.
