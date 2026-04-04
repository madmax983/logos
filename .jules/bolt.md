**[Zero-Cost AccountId Cloning]**
**Learning:** The `AccountId` primitive was heavily cloned across the application (e.g. TUI, distributors, and memory store loops) using a heap-allocated `String` which created unnecessary memory pressure.
**Action:** Replaced the inner `String` of domain primitive `AccountId` with `Arc<str>` (a zero-cost abstraction), reducing heap allocations across thousands of operations without changing the struct's API boundaries.
