**[Zero-Cost AccountId Cloning]**
**Learning:** The `AccountId` primitive was heavily cloned across the application (e.g. TUI, distributors, and memory store loops) using a heap-allocated `String` which created unnecessary memory pressure.
**Action:** Replaced the inner `String` of domain primitive `AccountId` with `Arc<str>` (a zero-cost abstraction), reducing heap allocations across thousands of operations without changing the struct's API boundaries.
**[Zero-Cost CategoryGroupId Cloning]**
**Learning:** The `CategoryGroupId` primitive was cloned frequently (e.g. within `Category` structs and budget structures) using a heap-allocated `String` which created unnecessary memory allocations.
**Action:** Replaced the inner `String` of domain primitive `CategoryGroupId` with `Arc<str>` (a zero-cost abstraction), reducing heap allocations across categorization and budgeting operations without changing the struct's API boundaries.
**[Eliminate Unconditional Key Clones in HashMap Insertions]**
**Learning:** Using `HashMap::entry(key.clone()).or_insert(...)` within a loop unconditionally clones the key string on every single iteration, even if the key already exists in the map. This causes significant, unnecessary heap allocations on hot paths (e.g., aggregating transactions by category or transaction ID).
**Action:** Replace `entry` calls with `get_mut` followed by an `insert` fallback when updating maps with owned keys like `String` or `CategoryGroupId`. This ensures the key is only cloned when genuinely inserting a new entry, entirely eliminating allocations for existing keys.
