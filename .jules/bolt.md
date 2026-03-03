**[HashSet Allocations]**
**Learning:** `HashSet` can operate on borrowed references seamlessly avoiding intermediate allocations on values mapped from Iterators. This helps when constructing maps/sets of existing items simply for lookups during loops.
**Action:** When creating intermediate HashSets to filter `Vec`s of larger structured values, map to the references of their ids e.g. `.map(Type::id)` instead of `.map(|x| x.id().clone())` to save intermediate heap allocations on keys.
