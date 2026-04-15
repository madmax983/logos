# 🔭 Vantage: Spec for Net Worth Projector Overflow Safety

👤 **User Story:**
"As a user planning my financial future, I want the net worth projector to safely handle extremely long timelines, so that I can project multi-generational or very long-term wealth accumulation without the application crashing."

🤔 **So What?**
A financial simulation tool must be resilient to all logical boundaries. Currently, attempting to simulate timelines spanning hundreds of years or enormous durations causes the simulation to abruptly fail due to numeric limitations. When the application crashes instead of gracefully reporting bounds or completing the projection, it degrades user trust and makes the tool feel fragile. Handling edge-case long durations is critical for a robust user experience.

🎯 **Metric Definition:**
Success = 0 system crashes or unexpected terminations during net worth timeline projection, even when simulating the absolute maximum supported durations. The system should safely compute the timeline or return an appropriate bounds error.

🔎 **Gap Analysis:**
The current simulation engine calculates the exact days within the targeted projection window. Because it doesn't gracefully handle the arithmetic scaling required for massive timeline values, inputting a massive number of months exceeds the system's capacity to store the intermediate calculations. A robust financial engine needs to safely handle mathematical expansion for long-range planning without hitting internal hard limits that lead to system panics.

✅ **Acceptance Criteria:**
- The timeline projection engine must not crash or panic when provided with massive, edge-case time durations.
- The system must correctly complete the simulation for long ranges or return a clear error stating the maximum timeline boundary has been reached.

🚫 **Out of Scope:**
- Transitioning the underlying database or data model to handle theoretically infinite precision numbers.
- Altering the fundamental 30-day month abstraction logic used for forecasting.
