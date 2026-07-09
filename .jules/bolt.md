## 2023-10-27 - Reduce unconditional `clone` in `HashMap` grouping
**Learning:** Using `HashMap::entry(key.clone()).or_default().push(val)` unconditionally clones the key, which is wasteful for `String` keys when appending to vectors, because the clone happens for *every* element rather than just once per group.
**Action:** Replace `entry` API with `get_mut` check and fallback to `insert` with `clone` when grouping elements by string keys to minimize heap allocations.
## 2024-05-19 - Removed Intermediate Allocations from Mermaid Exporter
**Learning:** Re-evaluating `Iterator::filter()` multiple times over a short slice is often dramatically faster and safer than eagerly allocating intermediate `Vec` collections using `Iterator::partition()`, especially on hot paths or loops where the number of elements is small.
**Action:** Replace `partition()` with chained `filter()` calls unless the condition is extremely expensive to compute.
**Vec capacity pre-allocation for lines**
**Learning:** For strings processed iteratively (e.g., CSV imports), `str::lines().count()` is an efficient way to count elements without reallocating since it acts as a fast iterator (often optimized). Passing this to `Vec::with_capacity()` prevents multiple allocations and `HashSet` rehashing when importing a large number of rows.
**Action:** When a function initializes `Vec::new()` or `HashSet::new()` before looping over `.lines()` or an iterator with a predictable length, calculate the length and use `with_capacity` to eliminate intermediate memory allocations.
## Avoid fighting the borrow checker with HashSet string deduplication
**Learning:** When deduplicating items using `HashSet` and simultaneously collecting them into a `Vec` or passing them out of the current scope, using a `HashSet<&str>` to avoid `.clone()` on locally created `String`s often results in complex borrow checker errors (E0502), because the `String` must eventually be moved or pushed.
**Action:** Do not attempt to prematurely optimize `.insert(string.clone())` into `HashSet` on hot import paths if the string must also be collected, without carefully structuring the lifetime boundaries. The extra allocation from `.clone()` is often the correct trade-off for memory safety.
## YYYY-MM-DD - Iterator Cloned Optimization

**Learning:** Removing `.cloned()` and `.collect::<Vec<_>>()` from an iterator chain successfully avoids intermediate heap allocations and deep cloning of objects. However, doing so changes the iterator's yielded item type from owned values (e.g. `T`) to references (e.g. `&T`).
**Action:** When removing `.cloned()` from iterator chains to avoid intermediate allocations, ensure you update downstream variables in the loop body (e.g., changing `&item` to `item`) to resolve `clippy::needless_borrow` warnings, as the iterator will now yield references instead of owned values.
**[Title: O(n^2) String insertions on formatting path]
**Learning:** `String::insert(0, c)` causes all existing bytes in the string to be shifted right by one. Using this inside a loop over characters turns formatting into an O(n^2) operation with frequent implicit re-allocations.
**Action:** Always pre-allocate with `String::with_capacity` if the max length is bounded, and build the string sequentially from left to right using `push` or `push_str`.
## YYYY-MM-DD - Avoiding repeated string insertions inside formatting loops
**Learning:** `String::insert(0, c)` causes all existing bytes in the string to be shifted right by one. Using this inside a loop over characters turns formatting into an O(n^2) operation with frequent implicit re-allocations. Similar issues can occur with repeated `.push_str()` on loops without pre-allocating the underlying buffer.
**Action:** Always pre-allocate with `String::with_capacity` if the max length is bounded, and build the string sequentially from left to right using `push` or `push_str`. For loops building multiple strings (like X/Y axes), estimating capacity based on item count avoids continual reallocation.

## 2026-04-27 - Reduce Iteration Allocations
**Learning:** Found several places where `.iter().map(...).collect()` was being used on vectors that were owned and going to be discarded, which borrows the elements and creates unnecessary indirection/allocations. Changing them to `.into_iter().map(|row| ...(&row)).collect()` consumes the vector and avoids borrowing if the mapping function doesn't require it, or allows the `Vec` to be consumed. Note that for simple structs and references this is minor, but combining `.into_iter()` avoids re-borrowing.
**Action:** Use `.into_iter()` instead of `.iter()` whenever a vector is no longer needed, especially when building result collections.

## 2025-07-09 - Remove Intermediate Vectors for Database Queries
**Learning:** Diesel's `.eq_any()` method accepts any type that implements `IntoIterator`. Previously, we were allocating intermediate `Vec<&str>` collections purely to pass to `eq_any` (e.g., `let ids: Vec<&str> = transaction_ids.iter().map(TransactionId::as_str).collect()`).
**Action:** Pass the mapped iterators directly into `.eq_any()` instead of using `.collect::<Vec<_>>()` to avoid heap allocations when building SQL `IN` clauses.
