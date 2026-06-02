# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
As a system operator running Monte Carlo simulations, I want the system to reject impossibly large path counts gracefully so that the process doesn't crash from memory allocation overflow.

🤔 **So What?:**
Crashing the system due to unconstrained user input degrades trust and stability. Passing `u32::MAX` currently attempts to allocate over 34GB of RAM, causing a SIGABRT panic. Providing a clear error message instead allows the system to remain stable and informs the user to correct their input.

📈 **Metric Definition:**
Success = The simulation gracefully returns an error instead of panicking when `paths` exceeds a reasonable limit (e.g., trying to allocate beyond available memory bounds).

🔍 **Gap Analysis:**
Currently, unconstrained path counts cause an OS-level out-of-memory abort via `Vec::with_capacity(paths as usize)`.

✅ **Acceptance Criteria:**
- Must validate the `paths` parameter before allocation.
- Must return a clear domain error if `paths` is too large.
- Must not crash with SIGABRT or memory allocation failures.

🚫 **Out of Scope:**
- Dynamic memory limit calculation based on the host OS.
