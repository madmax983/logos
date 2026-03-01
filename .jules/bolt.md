## 2024-03-24 - [Avoid re-allocations and clones on hot paths]
**Learning:** `current_projection_without_superseded` was allocating an unbounded `Vec` and `HashSet`, and unnecessarily cloning strings.
**Action:** Use `Vec::with_capacity` and `HashSet::with_capacity` where max length is known. Collect references into the `HashSet` to avoid cloning `String` objects when filtering.
