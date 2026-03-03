**[HashSet Allocations]**
**Learning:** `HashSet` can operate on borrowed references seamlessly avoiding intermediate allocations on values mapped from Iterators. This helps when constructing maps/sets of existing items simply for lookups during loops.
**Action:** When creating intermediate HashSets to filter `Vec`s of larger structured values, map to the references of their ids e.g. `.map(Type::id)` instead of `.map(|x| x.id().clone())` to save intermediate heap allocations on keys.
## 2024-03-24 - [Avoid re-allocations and clones on hot paths]
**Learning:** `current_projection_without_superseded` was allocating an unbounded `Vec` and `HashSet`, and unnecessarily cloning strings.
**Action:** Use `Vec::with_capacity` and `HashSet::with_capacity` where max length is known. Collect references into the `HashSet` to avoid cloning `String` objects when filtering.
