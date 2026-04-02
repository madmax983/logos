# 🔭 Vantage: Spec for Generic Server Manifest Fallback

👤 **User Story:**
"As a new evaluator of the system, I want the server start commands to use sensible, generic default paths out-of-the-box, so that I can run the software immediately without debugging hardcoded developer machine paths."

💼 **Business Problem:**
The server launch command relies on a hardcoded, developer-specific absolute file path. This blocks new users from evaluating the software, destroying the onboarding experience and increasing time-to-first-value. Complexity is a cost; low-friction onboarding is revenue.

✅ **Acceptance Criteria:**
- **Success Metric:** A fresh installation can execute the server start command without crashing due to a missing hardcoded absolute path, using a reasonable generic relative path as a fallback.
- The default location for the server manifest must be a relative path that works across different operating systems.
- The system must preserve the ability for users to explicitly override the default path via standard environment variable configuration.

🚫 **Out of Scope:**
- Building a comprehensive centralized configuration file system for the server.
- Automatically downloading missing external components if they do not exist locally.
