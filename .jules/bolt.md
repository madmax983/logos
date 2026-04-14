## 2023-10-27 - Reduce unconditional `clone` in `HashMap` grouping
**Learning:** Using `HashMap::entry(key.clone()).or_default().push(val)` unconditionally clones the key, which is wasteful for `String` keys when appending to vectors, because the clone happens for *every* element rather than just once per group.
**Action:** Replace `entry` API with `get_mut` check and fallback to `insert` with `clone` when grouping elements by string keys to minimize heap allocations.
## 2024-05-19 - Removed Intermediate Allocations from Mermaid Exporter
**Learning:** Re-evaluating `Iterator::filter()` multiple times over a short slice is often dramatically faster and safer than eagerly allocating intermediate `Vec` collections using `Iterator::partition()`, especially on hot paths or loops where the number of elements is small.
**Action:** Replace `partition()` with chained `filter()` calls unless the condition is extremely expensive to compute.
**Vec capacity pre-allocation for lines**
**Learning:** For strings processed iteratively (e.g., CSV imports), `str::lines().count()` is an efficient way to count elements without reallocating since it acts as a fast iterator (often optimized). Passing this to `Vec::with_capacity()` prevents multiple allocations and `HashSet` rehashing when importing a large number of rows.
**Action:** When a function initializes `Vec::new()` or `HashSet::new()` before looping over `.lines()` or an iterator with a predictable length, calculate the length and use `with_capacity` to eliminate intermediate memory allocations.
