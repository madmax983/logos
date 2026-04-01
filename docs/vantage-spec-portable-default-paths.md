# 🔭 Vantage: Spec for Portable Default Paths

👤 **User Story:**
As a new developer or user evaluating the software, I want the local server to start successfully using portable default configurations, so that I can try the system out-of-the-box without needing to troubleshoot developer-specific hardcoded paths.

🤔 **So What?**
Hardcoding a specific developer's local machine path into the default server configuration guarantees that the "Getting Started" experience will fail for everyone else. First impressions matter. If a user cannot run the most basic startup command without digging into source code or setting undocumented environment variables, they will abandon the product.

🎯 **Metric Definition:**
Success = A fresh installation of the application can successfully launch the local server without requiring manual environment variable configuration or manual path overrides.

🔎 **Gap Analysis:**
The system currently defaults to a local workspace path belonging to a specific developer's machine. Modern developer tools expect default paths to be either relative to the current project directory or gracefully fallback to a standard system temporary or home directory.

✅ **Acceptance Criteria:**
- The default manifest path used by the local server must be relative to the active workspace or a standard cross-platform directory.
- The server must not fail to start due to a missing file at a hardcoded, user-specific absolute path.
- The system should still allow users to override the path via an environment variable if they choose.

🚫 **Out of Scope:**
- Implementing an automatic file discovery mechanism that scans the entire filesystem.
- Building a comprehensive configuration wizard UI for server setup.
