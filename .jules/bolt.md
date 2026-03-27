**[HashSet Allocations]**
**Learning:** `HashSet` can operate on borrowed references seamlessly avoiding intermediate allocations on values mapped from Iterators. This helps when constructing maps/sets of existing items simply for lookups during loops.
**Action:** When creating intermediate HashSets to filter `Vec`s of larger structured values, map to the references of their ids e.g. `.map(Type::id)` instead of `.map(|x| x.id().clone())` to save intermediate heap allocations on keys.
## 2024-03-24 - [Avoid re-allocations and clones on hot paths]
**Learning:** `current_projection_without_superseded` was allocating an unbounded `Vec` and `HashSet`, and unnecessarily cloning strings.
**Action:** Use `Vec::with_capacity` and `HashSet::with_capacity` where max length is known. Collect references into the `HashSet` to avoid cloning `String` objects when filtering.
**[Vec Allocation for Polars Series]**
**Learning:** When preparing string columns for Polars `Series::new`, we don't need to construct an intermediate `Vec<String>` with `.clone()`. Polars can serialize directly from a `Vec<&str>`.
**Action:** Use `.as_str()` mapped directly to `Vec<&str>` to avoid thousands of intermediate heap allocations per serialized batch.
**[Persistence Reordering Risk]**
**Learning:** While refactoring persistence closures to avoid `.clone()`, do not rearrange the exact call order of `persist_X_graph` and `persist_X_memory` without an explicit architectural goal. Reordering persistence operations can lead to referential integrity bugs where child operations persist prior to parents.
**Action:** Only refactor the inner arguments (e.g. `.clone()`) on persistence calls, without moving lines of code.

**Optimize Retain to While Loop**
**Learning:** `Vec::retain()` inside a loop over $M$ iterations against an array of $N$ thresholds yields an $O(M \times N)$ time complexity because `retain()` sweeps the whole array and shifts elements.
**Action:** Clone and `sort_unstable()` the thresholds upfront, then maintain an external index across iterations to perform checks in $O(M + N)$ time, avoiding unnecessary shifting and traversal.

**Flatten filter_map allocation under-sizing**
**Learning:** Chaining `.into_iter().flatten().filter_map(...)` onto an `Option<Vec>` and calling `.collect::<Vec<_>>()` causes multiple intermediate vector allocations or under-sizing due to lost iterator `size_hint` boundaries.
**Action:** Use `map_or_else(Vec::new, |items| { let mut v = Vec::with_capacity(items.len()); v.extend(items.iter().filter_map(...)); v })` to ensure exactly one heap allocation that covers the upper bound of possible returned items.

**Pre-Sorted State for Projections**
**Learning:** In read-heavy projection or simulation methods (e.g. `project_timeline`), cloning a vector just to `.sort_unstable()` it before a hot loop introduces unnecessary heap allocations.
**Action:** Move the sorting logic (`.sort_unstable()`) to the data insertion methods (e.g. `add_milestone_cents`) to maintain a pre-sorted state, eliminating the need to `.clone()` the vector inside the projection method.

**Pre-allocate Vector for Outgoing Edges**
**Learning:** Initializing vectors with `Vec::new()` and then continuously pushing into them on a hot path causes multiple heap allocations, which degrades performance. Additionally, using `records.iter()` for a loop that can consume the records without borrowing is less efficient.
**Action:** Replace `Vec::new()` with `Vec::with_capacity(outgoing_edges.len())` when the capacity is known beforehand to avoid continuous heap allocations. Remove explicit `.iter()` where appropriate.
**[AppRuntime::month_report_for Redundant Iteration]**
**Learning:** `AppRuntime::month_report_for` was iterating over the entire transaction history three separate times to calculate `checking_balance_cents`, `income_cents`, and `expense_cents` via `.filter().flat_map().filter().sum()`. This caused excessive allocations and redundant work on hot paths. Additionally, `.sum()` and `i64::abs()` were vulnerable to panic on overflow with extremely large numbers.
**Action:** Replaced the three separate iterator chains with a single `for` loop over `self.store.transactions().filter(transaction_in_month)` that aggregates all three variables using `saturating_add` and `checked_abs().unwrap_or(i64::MAX)`. This eliminates redundant scans and protects against panic.

**[HashMap Preallocation]**
**Learning:** `HashMap::with_capacity()` avoids intermediate allocations and resizing when constructing maps from an iterator with a known size, compared to `.collect::<HashMap<_, _>>()`.
**Action:** Replace `.collect::<HashMap<_, _>>()` with `HashMap::with_capacity(len)` and a simple `.insert` loop when the item count is statically known (e.g. from `.len()`).

**[Sum Capacity for FlatMap Extends]**
**Learning:** When pushing multiple smaller vectors into a single output vector inside a loop, `Vec::new()` causes multiple reallocations. `Vec::with_capacity()` can eliminate this if the exact or upper-bound total size is calculated beforehand by summing the inner lengths.
**Action:** Replace `Vec::new()` with `Vec::with_capacity(capacity)` where `capacity` is pre-calculated by summing `.len()` over the sub-collections before `.extend()`.

**[Preallocate Vec from exact len]**
**Learning:** `Vec::new()` requires multiple allocations when the upper bound of the length is known from another collection being mapped or iterated over.
**Action:** Use `Vec::with_capacity(collection.len())` instead of `Vec::new()` when initializing a vector that will be populated by an iterator with a known length.
## 2026-03-26 - Optimize Vector and HashSet Allocation in Graph Deserialization
**Learning:** Rustdoc (`///`) generates documentation for items, not statements. Using `///` inside a function body triggers an `unused_doc_comments` warning, which fails the build under strict clippy settings (`-D warnings`). When iterating over graph edges, assigning the result to a variable allows for exact capacity checking and prevents unneeded dynamic resizing.
**Action:** Use standard comments (`//`) for internal logic and implementation details inside functions instead of rustdoc comments. Always pre-allocate `Vec` and `HashSet` when the exact size of the incoming iterator is known, such as when processing graph edges.

## 2024-05-18 - Pre-allocate HashMaps when Loading Graph Nodes
**Learning:** Found several `HashMap` instances initialized with `HashMap::new()` during database node loading (e.g. `load_transactions`, `load_statement_lines`), despite comments claiming they were pre-allocated. This causes unnecessary reallocation and rehashing while loading large datasets into memory.
**Action:** Use the iterator's `.size_hint()` method on the node ID iterators (e.g. `let (lower, upper) = node_ids.size_hint(); let capacity = upper.unwrap_or(lower);`) and initialize the maps using `HashMap::with_capacity(capacity)` to eliminate runtime rehashing and reallocation.
