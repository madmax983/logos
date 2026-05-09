# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
As a user running financial projections, I want the Monte Carlo simulator to safely handle or reject excessively large simulation path inputs, so that the application doesn't crash my system by exhausting available memory.

🤔 **So What? (Business Problem):**
The current Monte Carlo simulator attempts to allocate memory proportional to the requested number of simulation paths. If a user inputs a massive value (e.g., billions of paths), the application attempts to allocate more RAM than physically available, resulting in a hard crash. A reliable financial tool must defend against resource exhaustion and gracefully handle edge-case inputs without crashing, preserving user trust and application stability.

🎯 **Metric Definition:**
- Success = 0 memory allocation panics or system crashes when running the Monte Carlo simulator with massive path inputs.
- Success = The system safely caps inputs to a reasonable maximum or returns a clear error instead of attempting impossible allocations.

🔎 **Gap Analysis:**
The current projection engine does not bound the number of simulation paths allowed, attempting to store the entire state for all requested paths simultaneously in memory. This leads to out-of-memory errors on extreme inputs. A robust engine must anticipate memory limits and constrain inputs.

✅ **Acceptance Criteria:**
- The simulator must not panic with memory allocation errors when given extremely large path inputs.
- The simulator must implement a safe upper boundary for the maximum number of paths allowed in a single run.
- If the requested paths exceed the safe boundary, the system should either gracefully cap the paths to the maximum allowed or return a user-friendly error.
- All existing tests, including chaos tests that trigger this overflow, must pass or be handled correctly after the fix.

🚫 **Out of Scope:**
- Rewriting the simulator to stream path data to disk instead of using memory.
- Changing the underlying probabilistic projection model logic.
