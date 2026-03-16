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
