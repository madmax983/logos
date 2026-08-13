👤 **User Story:**
As a user performing long-term financial planning, I want the system to handle extreme inputs gracefully (like massive account balances, unrealistic income, or multi-century projections), so that the application returns a helpful error instead of completely crashing and losing my work.

🤔 **So What? (Business Problem):**
Financial planning inherently invites "what-if" scenarios. Users will input extremely large numbers just to see what happens, or might accidentally add too many zeros to a salary or projection horizon. Currently, these edge cases cause hard application crashes due to underlying system limits. A crash destroys user trust and disrupts their workflow. We need to validate inputs and return user-friendly errors when mathematical limits are exceeded.

📈 **Metric Definition:**
- Success = 0 application crashes when users input maximum possible values for any financial metric (income, expenses, time horizons, portfolio values, or simulation paths).
- Usage Metric = Decrease in error reports related to unexpected termination during simulation or planning operations.

🔎 **Gap Analysis:**
Our core financial planning modules currently assume users will always provide "reasonable" inputs. They do not safeguard against massive calculations or excessive memory requests, meaning edge-case inputs lead directly to system failure. The standard in professional financial software is to reject unreasonable inputs with clear validation messages before calculation begins.

✅ **Acceptance Criteria:**
- Must validate time horizons (e.g., projection months/years) and return a clear error if the duration is too long to compute.
- Must validate financial amounts (e.g., income, expenses, portfolio balances, vest amounts) and return a clear error if the math exceeds system limits.
- Must validate simulation complexity (e.g., number of paths) and return a clear error before attempting to allocate excessive memory.
- Must ensure all calculations involving percentages and large balances gracefully return an error state rather than crashing.

🚫 **Out of Scope:**
- Arbitrary-precision arithmetic (e.g., big integers) to actually compute these massive, unrealistic numbers.
- Changing the underlying data structures; we only need to add boundary validations.
