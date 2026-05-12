# 🔭 Vantage: Spec for Monte Carlo Memory Allocation Safety

## 👤 User Story
As a user running financial projections, I want the Monte Carlo simulator to gracefully reject excessively large simulation path requests, so that my application remains stable and provides a helpful error instead of completely crashing due to out-of-memory errors.

## 🤔 Business Problem
Currently, the Monte Carlo simulator accepts any number of simulation paths provided by the user. If an astronomical number is passed (like maximum 32-bit integer), the system blindly attempts to allocate memory for all paths at once. This causes a total process crash (OOM panic). A complete system failure provides a terrible user experience, destroys any unsaved work, and makes the system vulnerable to accidental or intentional resource exhaustion. We need the system to be resilient and fail gracefully.

## 📈 Metric Definition
- **Success:** The application returns a clear, structured error when requested to run an excessive number of simulation paths, avoiding any process panics or OOM crashes.
- **Usage Metric:** Zero out-of-memory crashes originating from the Monte Carlo simulation engine.

## 🔍 Gap Analysis
The core simulation engine currently lacks boundary validation for its resource-intensive inputs. Standard resilient systems either stream data, enforce hard caps, or implement backpressure. Our immediate gap is the lack of a protective hard cap on memory allocations.

## ✅ Acceptance Criteria
- Must define and enforce a reasonable upper boundary limit for the number of simulation paths.
- Must intercept requests exceeding this boundary before any memory allocation occurs.
- Must return a clear, structured error indicating that the requested capacity or limit was exceeded, rather than crashing.
- Must continue to successfully process simulation requests that fall within the safe limits.

## 🚫 Out of Scope
- Implementing streaming or distributed processing to support infinite simulation paths.
- Optimizing the memory footprint of the actual simulation logic for valid path counts.
