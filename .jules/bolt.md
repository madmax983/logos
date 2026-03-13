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

**[Iterator Length Optimization Theater]**
**Learning:** `AletheiaDB` node scans (e.g., `scan_nodes_by_label`) return iterators that do not yield their length upfront. Attempting to use `.collect::<Vec<_>>()` simply to calculate the length for `Vec::with_capacity()` introduces an unnecessary intermediate memory allocation, which is a performance regression.
**Action:** When an iterator's length isn't cheaply known via `len()` or `size_hint()`, do not collect it just to pre-allocate capacity. Fall back to `Vec::new()` to allow the standard vector reallocation strategy to do its job.

**[Clone Obfuscation]**
**Learning:** Changing a function's parameter signature to accept a reference (e.g., `&StoredMonthClose`) instead of an owned value (`StoredMonthClose`) provides no performance benefit if the receiving function merely calls `.clone()` internally on that reference anyway to satisfy its own needs. It simply moves the allocation.
**Action:** Avoid 'optimization theater'. Only change function signatures to take references if you can eliminate the `.clone()` entirely throughout the entire call stack.
