**[HashSet Allocations]**
**Learning:** `HashSet` can operate on borrowed references seamlessly avoiding intermediate allocations on values mapped from Iterators. This helps when constructing maps/sets of existing items simply for lookups during loops.
**Action:** When creating intermediate HashSets to filter `Vec`s of larger structured values, map to the references of their ids e.g. `.map(Type::id)` instead of `.map(|x| x.id().clone())` to save intermediate heap allocations on keys.
## 2024-03-24 - [Avoid re-allocations and clones on hot paths]
**Learning:** `current_projection_without_superseded` was allocating an unbounded `Vec` and `HashSet`, and unnecessarily cloning strings.
**Action:** Use `Vec::with_capacity` and `HashSet::with_capacity` where max length is known. Collect references into the `HashSet` to avoid cloning `String` objects when filtering.

**Optimize `format!` string allocations in loops**
**Learning:** Using `format!` inside a hot loop (like hashing hundreds or thousands of rows) repeatedly allocates new `String`s on the heap, causing significant overhead.
**Action:** Instead of `format!`, pre-allocate a `String::with_capacity(N)` buffer before the loop. Inside the loop, call `buf.clear()` and use `std::fmt::Write` with the `write!` macro to format data directly into the reused buffer without triggering new allocations.
